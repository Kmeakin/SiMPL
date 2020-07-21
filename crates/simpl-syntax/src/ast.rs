pub type Ident = String;
use derive_more::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Lit {
        val: Lit,
    },
    Var {
        name: String,
    },
    If {
        test: Box<Self>,
        then_branch: Box<Self>,
        else_branch: Box<Self>,
    },
    Let {
        bindings: Vec<(Ident, Self)>,
        body: Box<Self>,
    },
    Letrec {
        bindings: Vec<(Ident, Self)>,
        body: Box<Self>,
    },
    Lambda {
        params: Vec<Ident>,
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
