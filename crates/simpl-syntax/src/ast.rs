pub type Ident = String;

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

#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    Bool(bool),
    Int(i64),
    Float(f64),
}
