//! `TypedExpr` -> `TypedExpr` pass
//! Assumes the rename pass has already run
//!
//! Input language:
//! e := x
//!    | n
//!    | if e then e else e
//!    | let x = e in e
//!    | letrec (x = e)+ in e
//!    | \x -> e
//!    | e e
//!
//! x := identifer
//!
//! n := int
//!    | float
//!    | bool
//!
//! Output language:
//! e := ae
//!    | ae ae
//!    | if ae e e
//!    | let x = e in e
//!
//! ae := \x -> e
//!     | x
//!     | n

use super::gensym::Gensym;
use crate::hir::{Expr, LetBinding};
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref GENSYM: Mutex<Gensym> = Mutex::new(Gensym::new("$"));
}

impl Expr {
    fn is_anf(&self) -> bool {
        self.is_e()
    }

    fn is_e(&self) -> bool {
        match self {
            Self::If {
                test,
                then_branch,
                else_branch,
                ..
            } => test.is_ae() && then_branch.is_ae() && else_branch.is_ae(),
            Self::Let { binding, body, .. } => binding.val.is_e() && body.is_e(),
            Self::Letrec { .. } => todo!(),
            Self::App { func, arg, .. } => func.is_ae() && arg.is_ae(),
            _ => self.is_ae(),
        }
    }

    fn is_ae(&self) -> bool {
        match self {
            Self::Lambda { body, .. } => body.is_e(),
            Self::Lit { .. } | Self::Var { .. } => true,
            _ => false,
        }
    }
}

pub fn normalize_expr(expr: Expr) -> Expr {
    GENSYM.lock().unwrap().reset();
    normalize_expr_inner(expr)
}

fn normalize_expr_inner(expr: Expr) -> Expr {
    normalize(expr, box |x| x)
}

fn normalize(expr: Expr, k: Box<dyn FnOnce(Expr) -> Expr>) -> Expr {
    match expr {
        Expr::If {
            ty,
            test,
            then_branch,
            else_branch,
        } => normalize_name(*test, box |t| {
            k(Expr::If {
                ty,
                test: box t,
                then_branch: box normalize_expr_inner(*then_branch),
                else_branch: box normalize_expr_inner(*else_branch),
            })
        }),

        Expr::Let { ty, binding, body } => normalize(*binding.clone().val, box |n1| Expr::Let {
            ty,
            binding: LetBinding {
                val: box n1,
                ..binding
            },
            body: box normalize(*body, k),
        }),

        Expr::Letrec { .. } => todo!(),

        Expr::Lambda { ty, param, body } => k(Expr::Lambda {
            ty,
            param,
            body: box normalize_expr_inner(*body),
        }),

        Expr::App { ty, func, arg } => normalize_name(*func, box |t| {
            normalize_name(*arg, box |t2| {
                k(Expr::App {
                    ty,
                    func: box t,
                    arg: box t2,
                })
            })
        }),

        Expr::Lit { .. } | Expr::Var { .. } => k(expr),
    }
}

fn normalize_name(expr: Expr, k: Box<dyn FnOnce(Expr) -> Expr>) -> Expr {
    normalize(expr, box |n| match n {
        Expr::Lit { .. } | Expr::Var { .. } => k(n),
        _ => {
            let name = GENSYM.lock().unwrap().next();
            let ty = n.ty();
            let t = Expr::Var {
                ty: ty.clone(),
                name,
            };
            Expr::Let {
                ty: ty.clone(),
                binding: LetBinding {
                    ty,
                    name,
                    val: box n,
                },
                body: box k(t),
            }
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::{self, parse_and_type};
    use insta::assert_snapshot;

    #[track_caller]
    fn test_normalize(src: &str) {
        let expr = parse_and_type(src);
        let norm = normalize_expr(expr);
        assert!(norm.is_anf());
        assert_snapshot!(norm.pretty());
    }

    #[test]
    fn normalize_lit() {
        test_normalize("123");
    }

    #[test]
    fn normalize_if() {
        test_normalize("if true then 1 else 0");
        test_normalize("if (if false then true else false) then 1 else 0");
    }

    // TODO: should immediate values be allowed in the value of a binding?
    #[test]
    fn normalize_let() {
        test_normalize("let abc = 1 in abc");
    }

    #[test]
    fn normalize_lambda() {
        test_normalize(r"\x -> x");
        test_normalize(r"\x -> 1");
    }

    #[test]
    fn normalize_app() {
        test_normalize(r"add (mul 2 4) (mul 2 8)");
    }
}
