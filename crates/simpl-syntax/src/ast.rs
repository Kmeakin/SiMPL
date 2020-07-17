pub type Symbol = String;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Lit(LiteralExpr),
    Var(VarExpr),
    Let(LetExpr),
    Lambda(LambdaExpr),
    App(AppExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralExpr {
    Bool(bool),
    Int(i64),
    Float(f64),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VarExpr(pub Symbol);

#[derive(Debug, Clone, PartialEq)]
pub struct LetExpr {
    pub bindings: Vec<Binding>,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    pub var: Symbol,
    pub val: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LambdaExpr {
    pub args: Vec<Symbol>,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppExpr {
    pub func: Box<Expr>,
    pub args: Vec<Expr>,
}
