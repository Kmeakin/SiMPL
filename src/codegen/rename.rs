//! `Exp` -> `Expr` pass
//! Alpha-renames exprs, so that no variable masks another in its enclosing
//! scope

use super::gensym::Gensym;
use crate::hir::Expr;
use lazy_static::lazy_static;
use maplit::hashset as hset;
use simple_symbol::Symbol;
use std::{collections::HashSet, sync::Mutex};

lazy_static! {
    static ref GENSYM: Mutex<Gensym> = Mutex::new(Gensym::new("$"));
}

impl Expr {
    fn is_alpha_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Lit { val: x, .. }, Self::Lit { val: y, .. }) => x == y,
            (Self::Var { name: x, .. }, Self::Var { name: y, .. }) => x == y,
            (
                Self::If {
                    test: test1,
                    then_branch: then1,
                    else_branch: else1,
                    ..
                },
                Self::If {
                    test: test2,
                    then_branch: then2,
                    else_branch: else2,
                    ..
                },
            ) => test1.is_alpha_eq(test2) && then1.is_alpha_eq(then2) && else1.is_alpha_eq(else2),
            (Self::Let { .. }, Self::Let { .. }) => todo!(),
            (Self::Letrec { .. }, Self::Letrec { .. }) => todo!(),
            (Self::Lambda { .. }, Self::Lambda { .. }) => todo!(),
            (Self::App { .. }, Self::App { .. }) => todo!(),
            _ => false,
        }
    }
}
