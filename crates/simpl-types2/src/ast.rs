pub use simpl_syntax2::ast::{Ident, Lit};

#[derive(Debug, Clone, PartialEq)]
pub enum TypedExpr {
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
