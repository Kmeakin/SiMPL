//! Closure-conversion
//! Requires the `ANF` pass to be run first

use super::util::{free_vars, FreeVars};
pub use crate::hir::{Lit, Param, Symbol, Type};
use crate::{hir::Expr, util::counter::Counter};
use std::collections::HashMap;

mod pp;

#[derive(Debug, Clone)]
pub struct LetBinding {
    pub ty: Type,
    pub name: Symbol,
    pub val: Box<CExpr>,
}

#[derive(Debug, Clone)]
pub enum CExpr {
    Lit {
        ty: Type,
        val: Lit,
    },
    Var {
        ty: Type,
        name: Symbol,
    },
    EnvRef {
        closure_id: u32,
        ty: Type,
        name: Symbol,
    },
    If {
        ty: Type,
        test: Box<Self>,
        then: Box<Self>,
        els: Box<Self>,
    },
    Let {
        ty: Type,
        binding: LetBinding,
        body: Box<Self>,
    },
    Letrec {
        ty: Type,
        bindings: Vec<LetBinding>,
        body: Box<Self>,
    },
    MkClosure {
        closure_id: u32,
        ty: Type,
        param: Param,
        free_vars: FreeVars,
        body: Box<Self>,
    },
    AppClosure {
        ty: Type,
        func: Box<Self>,
        arg: Box<Self>,
    },
}

impl CExpr {
    pub fn ty(&self) -> Type {
        match self {
            Self::Lit { ty, .. }
            | Self::Var { ty, .. }
            | Self::EnvRef { ty, .. }
            | Self::If { ty, .. }
            | Self::Let { ty, .. }
            | Self::Letrec { ty, .. }
            | Self::MkClosure { ty, .. }
            | Self::AppClosure { ty, .. } => ty.clone(),
        }
    }
}

fn substitute(cexpr: CExpr, subst: &HashMap<Symbol, CExpr>) -> CExpr {
    match cexpr {
        CExpr::Lit { .. } => cexpr,
        CExpr::Var { ty, name } => subst.get(&name).unwrap_or(&CExpr::Var { ty, name }).clone(),
        CExpr::If {
            ty,
            test,
            then,
            els,
        } => CExpr::If {
            ty,
            test: box substitute(*test, subst),
            then: box substitute(*then, subst),
            els: box substitute(*els, subst),
        },
        CExpr::Let { ty, binding, body } => CExpr::Let {
            ty,
            binding: LetBinding {
                val: box substitute(*binding.val, subst),
                ..binding
            },
            body: box substitute(*body, subst),
        },
        CExpr::Letrec { .. } => todo!(),
        CExpr::MkClosure {
            closure_id,
            ty,
            param,
            free_vars,
            body,
        } => CExpr::MkClosure {
            closure_id,
            ty,
            param,
            free_vars,
            body: box substitute(*body, subst),
        },
        CExpr::AppClosure { ty, func, arg } => CExpr::AppClosure {
            ty,
            func: box substitute(*func, subst),
            arg: box substitute(*arg, subst),
        },
        CExpr::EnvRef {
            closure_id,
            name,
            ty,
        } => CExpr::EnvRef {
            closure_id,
            name,
            ty,
        },
    }
}

pub fn closure_convert(expr: &Expr) -> CExpr {
    let mut gen = Counter::new();
    closure_convert_inner(expr, &mut gen)
}

fn closure_convert_inner(expr: &Expr, gen: &mut Counter<u32>) -> CExpr {
    match expr {
        Expr::Lit { ty, val } => CExpr::Lit {
            ty: ty.clone(),
            val: *val,
        },
        Expr::Var { ty, name } => CExpr::Var {
            ty: ty.clone(),
            name: *name,
        },
        Expr::If {
            ty,
            test,
            then,
            els,
        } => CExpr::If {
            ty: ty.clone(),
            test: box closure_convert_inner(&**test, gen),
            then: box closure_convert_inner(&**then, gen),
            els: box closure_convert_inner(&**els, gen),
        },
        Expr::Let { ty, binding, body } => CExpr::Let {
            ty: ty.clone(),
            binding: LetBinding {
                ty: binding.ty.clone(),
                name: binding.name,
                val: box closure_convert_inner(&*binding.val, gen),
            },
            body: box closure_convert_inner(&*body, gen),
        },
        Expr::Letrec { .. } => todo!(),
        Expr::Lambda {
            ty,
            param,
            ref body,
        } => {
            let closure_id = gen.next();
            let fv = free_vars(&expr.clone());
            CExpr::MkClosure {
                closure_id,
                ty: ty.clone(),
                param: param.clone(),
                free_vars: fv.clone(),
                body: box substitute(
                    closure_convert_inner(&**body, gen),
                    &fv.iter()
                        .map(|(name, ty)| {
                            (
                                *name,
                                CExpr::EnvRef {
                                    closure_id,
                                    name: *name,
                                    ty: ty.clone(),
                                },
                            )
                        })
                        .collect(),
                ),
            }
        }
        Expr::App { ty, func, arg } => CExpr::AppClosure {
            ty: ty.clone(),
            func: box closure_convert_inner(&**func, gen),
            arg: box closure_convert_inner(&**arg, gen),
        },
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        codegen::anf::normalize_expr,
        hir::Expr,
        types::{self, parse_and_type},
    };
    use insta::assert_snapshot;
    use std::str::FromStr;

    #[track_caller]
    fn test_closure_convert(src: &str) {
        let expr = Expr::from_str(src).unwrap();
        let anf = normalize_expr(expr);
        let clo_conv = closure_convert(&anf);
        dbg!(&clo_conv);
        assert_snapshot!(clo_conv.pretty());
    }

    #[test]
    fn test0() {
        let src = r"
let const = \x -> \ignored -> x
in const 5
";
        test_closure_convert(src);
    }

    #[test]
    fn test1() {
        let src = r"
let x = 5,
    f = \ignored -> x
in f 100
";
        test_closure_convert(src);
    }

    #[test]
    fn test2() {
        let src = r"
let x = 5,
    y = 10,
    f = \ignored -> add y x
in f 100
";
        test_closure_convert(src);
    }
}
