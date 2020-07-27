//! Closure-conversion
//! Requires the `ANF` pass to be run first

use super::util::free_vars;
use crate::hir::{Expr, Lit, Param, Type};
use simple_symbol::Symbol;
use std::collections::{HashMap, HashSet};

mod pp;

type Env = HashMap<Symbol, CExpr>;

#[derive(Debug, Clone, PartialEq)]
pub struct LetBinding {
    pub ty: Type,
    pub name: Symbol,
    pub val: Box<CExpr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CExpr {
    Lit {
        ty: Type,
        val: Lit,
    },
    Var {
        ty: Type,
        name: Symbol,
    },
    If {
        ty: Type,
        test: Box<Self>,
        then_branch: Box<Self>,
        else_branch: Box<Self>,
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
        ty: Type,
        param: Param,
        free_vars: HashSet<Symbol>,
        body: Box<Self>,
    },
    AppClosure {
        ty: Type,
        func: Box<Self>,
        arg: Box<Self>,
    },
    EnvRef {
        name: Symbol,
    },
}

fn substitute(cexpr: CExpr, subst: &HashMap<Symbol, CExpr>) -> CExpr {
    match cexpr {
        CExpr::Lit { .. } => cexpr,
        CExpr::Var { ty, name } => subst.get(&name).unwrap_or(&CExpr::Var { ty, name }).clone(),
        CExpr::If {
            ty,
            test,
            then_branch,
            else_branch,
        } => CExpr::If {
            ty: ty.clone(),
            test: box substitute(*test, subst),
            then_branch: box substitute(*then_branch, subst),
            else_branch: box substitute(*else_branch, subst),
        },
        CExpr::Let { ty, binding, body } => CExpr::Let {
            ty: ty.clone(),
            binding: LetBinding {
                val: box substitute(*binding.val, subst),
                ..binding
            },
            body: box substitute(*body, subst),
        },
        CExpr::Letrec { .. } => todo!(),
        CExpr::MkClosure {
            ty,
            param,
            free_vars,
            body,
        } => CExpr::MkClosure {
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
        CExpr::EnvRef { name } => CExpr::EnvRef { name },
    }
}

fn closure_convert(expr: &Expr) -> CExpr {
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
            then_branch,
            else_branch,
        } => CExpr::If {
            ty: ty.clone(),
            test: box closure_convert(&**test),
            then_branch: box closure_convert(&**then_branch),
            else_branch: box closure_convert(&**else_branch),
        },
        Expr::Let { ty, binding, body } => CExpr::Let {
            ty: ty.clone(),
            binding: LetBinding {
                ty: binding.ty.clone(),
                name: binding.name,
                val: box closure_convert(&*binding.val),
            },
            body: box closure_convert(&*body),
        },
        Expr::Letrec { .. } => todo!(),
        Expr::Lambda {
            ty,
            param,
            ref body,
        } => {
            let fv = free_vars(&expr.clone());
            CExpr::MkClosure {
                ty: ty.clone(),
                param: param.clone(),
                free_vars: fv.clone(),
                body: box substitute(
                    closure_convert(&**body),
                    dbg!(&fv
                        .iter()
                        .map(|&name| (name, CExpr::EnvRef { name }))
                        .collect()),
                ),
            }
        }
        Expr::App { ty, func, arg } => CExpr::AppClosure {
            ty: ty.clone(),
            func: box closure_convert(&**func),
            arg: box closure_convert(&**arg),
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
