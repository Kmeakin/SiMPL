use crate::ty::Type;
pub use simpl_syntax::ast::{Lit, Symbol};

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
        bindings: Vec<(Symbol, Type, Expr)>,
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

impl Expr {
    pub fn ty(&self) -> Type {
        match self {
            Self::Lit { ty, .. }
            | Self::Var { ty, .. }
            | Self::If { ty, .. }
            | Self::Let { ty, .. }
            | Self::Lambda { ty, .. }
            | Self::App { ty, .. } => ty.clone(),
        }
    }
}
