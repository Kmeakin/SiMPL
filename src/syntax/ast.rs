pub use crate::types::ty::Type;
use derive_more::Display;
pub use simple_symbol::Symbol;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Lit {
        val: Lit,
    },
    Var {
        name: Symbol,
    },
    Binop {
        lhs: Box<Self>,
        rhs: Box<Self>,
        op: Binop,
    },
    If {
        test: Box<Self>,
        then: Box<Self>,
        els: Box<Self>,
    },
    Let {
        bindings: Vec<LetBinding>,
        body: Box<Self>,
    },
    Letrec {
        bindings: Vec<LetBinding>,
        body: Box<Self>,
    },
    Lambda {
        params: Vec<Param>,
        body: Box<Self>,
    },
    App {
        func: Box<Self>,
        arg: Box<Self>,
    },
}

#[derive(Debug, Clone, PartialEq, Display)]
pub enum Lit {
    #[display(fmt = "{}", _0)]
    Bool(bool),
    #[display(fmt = "{}", _0)]
    Int(i64),
    #[display(fmt = "{}", _0)]
    Float(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LetBinding {
    pub name: Symbol,
    pub ann: Option<Type>,
    pub val: Box<Expr>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Binop {
    IntAdd,
    IntSub,
    IntMul,
    IntDiv,
    IntLt,
    IntLeq,
    IntGt,
    IntGeq,
    FloatAdd,
    FloatSub,
    FloatMul,
    FloatDiv,
    FloatLt,
    FloatLeq,
    FloatGt,
    FloatGeq,
    Eq,
    Neq,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: Symbol,
    pub ann: Option<Type>,
}

impl From<bool> for Lit {
    fn from(other: bool) -> Self {
        Self::Bool(other)
    }
}

impl From<i64> for Lit {
    fn from(other: i64) -> Self {
        Self::Int(other)
    }
}

impl From<f64> for Lit {
    fn from(other: f64) -> Self {
        Self::Float(other)
    }
}
