pub type Symbol = String;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Lit {
        val: Lit,
    },
    Var {
        name: String,
    },
    If {
        test: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    Let {
        bindings: Vec<(Symbol, Expr)>,
        body: Box<Expr>,
    },
    Lambda {
        params: Vec<Symbol>,
        body: Box<Expr>,
    },
    App {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    Bool(bool),
    Int(i64),
    Float(f64),
}
