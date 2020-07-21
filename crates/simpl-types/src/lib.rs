#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate)]
#![feature(box_syntax, box_patterns)]

use crate::{ast::TypedExpr, ty::Type};
use std::{marker::PhantomData, str::FromStr};

pub mod ast;
mod constraint;
mod subst;
pub mod ty;
mod unify;

mod pp;

#[cfg(test)]
mod test;

#[derive(Debug, Copy, Clone, Default)]
pub struct IdGen<T>
where
    T: FromId,
{
    counter: u32,
    _phantom: PhantomData<T>,
}

impl<T: FromId> IdGen<T> {
    pub fn new() -> Self {
        Self {
            counter: 0,
            _phantom: PhantomData,
        }
    }

    fn next_id(&mut self) -> u32 {
        let x = self.counter;
        self.counter += 1;
        x
    }

    pub fn current_id(&self) -> u32 {
        self.counter
    }

    pub fn current(&self) -> T {
        T::from_id(self.current_id())
    }

    pub fn fresh(&mut self) -> T {
        self.next().unwrap()
    }
}

impl<T: FromId> Iterator for IdGen<T> {
    type Item = T;

    /// Always returns `Some`
    fn next(&mut self) -> Option<Self::Item> {
        Some(T::from_id(self.next_id()))
    }
}

pub trait FromId {
    fn from_id(id: u32) -> Self;
}

impl FromId for u32 {
    fn from_id(id: u32) -> Self {
        id
    }
}

/// Infer the type of the expr
pub fn type_of(expr: &TypedExpr) -> Type {
    let cons = constraint::collect(expr.clone());
    let subst = unify::unify(&cons);
    subst.apply_ty(&expr.ty())
}

/// Infer the type of the expr, and apply the resulting substitution to the
/// expression (so every expr has its inferred type attatched)
pub fn infer_and_apply(expr: &TypedExpr) -> TypedExpr {
    let cons = constraint::collect(expr.clone());
    let subst = unify::unify(&cons);
    expr.apply(&subst)
}

/// Convenience function. Parse source code, and give every expr its inferred
/// type
pub fn parse_and_type(src: &str) -> TypedExpr {
    // TODO: return a &dyn impl Error instead of unwrapping
    let expr = TypedExpr::from_str(src).unwrap();
    infer_and_apply(&expr)
}
