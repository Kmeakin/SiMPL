use crate::{subst::Subst, ty::Type};
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
        params: Vec<(Symbol, Type)>,
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

impl Expr {
    pub fn apply(&self, subst: &Subst) -> Self {
        match self {
            Self::Lit { ty, val } => Self::Lit {
                ty: subst.apply_ty(ty),
                val: val.clone(),
            },
            Self::Var { ty, name } => Self::Var {
                ty: subst.apply_ty(ty),
                name: name.clone(),
            },
            Self::If {
                ty,
                test,
                then_branch,
                else_branch,
            } => Self::If {
                ty: subst.apply_ty(ty),
                test: box test.apply(subst),
                then_branch: box then_branch.apply(subst),
                else_branch: box else_branch.apply(subst),
            },
            Self::Let { ty, bindings, body } => Self::Let {
                ty: subst.apply_ty(ty),
                bindings: bindings
                    .iter()
                    .map(|(name, ty, val)| (name.clone(), ty.apply(subst), val.clone()))
                    .collect(),
                body: box body.apply(subst),
            },
            Self::Lambda { ty, params, body } => Self::Lambda {
                ty: subst.apply_ty(ty),
                params: params
                    .iter()
                    .map(|(name, ty)| (name.clone(), ty.apply(subst)))
                    .collect(),
                body: box body.apply(subst),
            },
            Self::App { ty, func, args } => Self::App {
                ty: subst.apply_ty(ty),
                func: box func.apply(subst),
                args: args.iter().map(|arg| arg.apply(subst)).collect(),
            },
        }
    }
}
