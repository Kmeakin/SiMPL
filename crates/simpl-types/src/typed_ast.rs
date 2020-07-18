use crate::ty::Type;
use simpl_syntax::{
    ast,
    ast::{Lit, Symbol},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
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
        test: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    Let {
        ty: Type,
        bindings: Vec<(Symbol, Expr)>,
        body: Box<Expr>,
    },
    Lambda {
        ty: Type,
        args: Vec<(Symbol, Type)>,
        body: Box<Expr>,
    },
    App {
        ty: Type,
        func: Box<Expr>,
        args: Vec<Expr>,
    },
}
