use crate::hir::{Expr, Lit, Param, Symbol, Type};
use maplit::hashmap as hmap;
use std::collections::HashMap;

pub type FreeVars = HashMap<Symbol, Type>;

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
        ty: Type,
        param: Param,
        free_vars: FreeVars,
        body: Box<Self>,
    },
    App {
        ty: Type,
        func: Box<Self>,
        arg: Box<Self>,
    },
}

#[derive(Debug, Clone)]
pub struct LetBinding {
    pub ty: Type,
    pub name: Symbol,
    pub val: Box<CExpr>,
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
            | Self::App { ty, .. } => ty.clone(),
        }
    }
}

pub fn convert(expr: Expr) -> CExpr {
    match expr {
        Expr::Lit { ty, val } => CExpr::Lit { ty, val },
        Expr::Var { ty, name } => CExpr::Var { ty, name },
        Expr::If {
            ty,
            test,
            then,
            els,
        } => CExpr::If {
            ty,
            test: box convert(*test),
            then: box convert(*then),
            els: box convert(*els),
        },
        Expr::Let { ty, binding, body } => CExpr::Let {
            ty,
            binding: LetBinding {
                ty: binding.ty,
                name: binding.name,
                val: box convert(*binding.val),
            },
            body: box convert(*body),
        },
        Expr::Letrec { ty, bindings, body } => CExpr::Letrec {
            ty,
            bindings: bindings
                .into_iter()
                .map(|binding| LetBinding {
                    ty: binding.ty,
                    name: binding.name,
                    val: box convert(*binding.val),
                })
                .collect(),
            body: box convert(*body),
        },
        Expr::Lambda {
            ref ty,
            ref param,
            ref body,
        } => {
            let fv = free_vars(&expr);
            let subst = &fv
                .iter()
                .map(|(name, ty)| {
                    (
                        *name,
                        CExpr::EnvRef {
                            name: *name,
                            ty: ty.clone(),
                        },
                    )
                })
                .collect();

            CExpr::MkClosure {
                ty: ty.clone(),
                param: param.clone(),
                free_vars: fv,
                body: box substitute(convert(*body.clone()), subst),
            }
        }
        Expr::App { ty, func, arg } => CExpr::App {
            ty,
            func: box convert(*func),
            arg: box convert(*arg),
        },
    }
}

fn substitute(expr: CExpr, subst: &HashMap<Symbol, CExpr>) -> CExpr {
    match expr {
        CExpr::Lit { .. } | CExpr::EnvRef { .. } => expr,
        CExpr::Var { name, .. } => subst.get(&name).unwrap_or(&expr).clone(),
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
        CExpr::Letrec { ty, bindings, body } => CExpr::Letrec {
            ty,
            bindings: bindings
                .into_iter()
                .map(|binding| LetBinding {
                    val: box substitute(*binding.val, subst),
                    ..binding
                })
                .collect(),
            body: box substitute(*body, subst),
        },
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
        CExpr::App { ty, func, arg } => CExpr::App {
            ty,
            func: box substitute(*func, subst),
            arg: box substitute(*arg, subst),
        },
    }
}

pub fn free_vars(expr: &Expr) -> FreeVars {
    match expr {
        Expr::Lit { .. } => hmap![],
        Expr::Var { name, ty } => hmap![*name => ty.clone()],
        Expr::If {
            test, then, els, ..
        } => hashmap_union(
            hashmap_union(free_vars(test), free_vars(then)),
            free_vars(els),
        ),
        Expr::Let { binding, body, .. } => hashmap_diff(
            hashmap_union(free_vars(&*binding.val), free_vars(body)),
            &hmap![binding.name => binding.ty.clone()],
        ),

        Expr::Letrec { bindings, body, .. } => hashmap_diff(
            bindings.iter().fold(free_vars(body), |acc, b| {
                hashmap_union(acc, free_vars(&*b.val))
            }),
            &bindings.iter().map(|b| (b.name, b.ty.clone())).collect(),
        ),
        Expr::Lambda { param, body, .. } => {
            hashmap_diff(free_vars(body), &hmap![param.name => param.ty.clone()])
        }

        Expr::App { func, arg, .. } => hashmap_union(free_vars(func), free_vars(arg)),
    }
}

fn hashmap_union<K, V>(hm1: HashMap<K, V>, hm2: HashMap<K, V>) -> HashMap<K, V>
where
    K: std::hash::Hash + Eq,
    V: std::hash::Hash + Eq,
{
    let mut ret = HashMap::new();
    ret.extend(hm1);
    ret.extend(hm2);
    ret
}

fn hashmap_diff<K, V>(hm1: HashMap<K, V>, hm2: &HashMap<K, V>) -> HashMap<K, V>
where
    K: std::hash::Hash + Eq + Clone + std::fmt::Debug,
    V: std::hash::Hash + Eq + Clone + std::fmt::Debug,
{
    hm1.into_iter()
        .filter_map(|(k, v)| {
            if hm2.contains_key(&k) {
                Some((k, v))
            } else {
                None
            }
        })
        .collect::<HashMap<K, V>>()
}
