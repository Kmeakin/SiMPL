use id_arena::{Arena, Id};
use simpl_syntax::ast;

pub type ExprId = Id<Expr>;
pub type Symbol = String;

pub type ExprArena = Arena<Expr>;

#[derive(Debug, Clone, Default)]
struct ArenaBuilder {
    arena: ExprArena,
}

impl ArenaBuilder {
    fn store_inner(&mut self, expr: ast::Expr) -> ExprId {
        match expr {
            ast::Expr::Lit(lit) => self.arena.alloc(Expr::Lit(lit.into())),
            ast::Expr::Var(ast::VarExpr(var)) => self.arena.alloc(Expr::Var(VarExpr(var))),
            ast::Expr::If(ast::IfExpr {
                test,
                then_branch,
                else_branch,
            }) => {
                let test_id = self.store_inner(*test);
                let then_id = self.store_inner(*then_branch);
                let else_id = self.store_inner(*else_branch);

                self.arena.alloc(Expr::If(IfExpr {
                    test: test_id,
                    then_branch: then_id,
                    else_branch: else_id,
                }))
            }
            ast::Expr::Let(ast::LetExpr { bindings, body }) => {
                let binding_ids = bindings
                    .iter()
                    .map(|ast::Binding { var, val }| Binding {
                        var: var.into(),
                        val: self.store_inner(*val.clone()),
                    })
                    .collect();
                let body_id = self.store_inner(*body);

                self.arena.alloc(Expr::Let(LetExpr {
                    bindings: binding_ids,
                    body: body_id,
                }))
            }
            ast::Expr::Lambda(ast::LambdaExpr { args, body }) => {
                let body_id = self.store_inner(*body);

                self.arena.alloc(Expr::Lambda(LambdaExpr {
                    args,
                    body: body_id,
                }))
            }
            ast::Expr::App(ast::AppExpr { func, args }) => {
                let func_id = self.store_inner(*func);

                let arg_ids = args
                    .iter()
                    .map(|arg| self.store_inner(arg.clone()))
                    .collect();

                self.arena.alloc(Expr::App(AppExpr {
                    func: func_id,
                    args: arg_ids,
                }))
            }
        }
    }
}

pub fn store(expr: ast::Expr) -> (ExprArena, ExprId) {
    let mut builder = ArenaBuilder::default();
    let id = builder.store_inner(expr);
    (builder.arena, id)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Lit(LitExpr),
    Var(VarExpr),
    If(IfExpr),
    Let(LetExpr),
    Lambda(LambdaExpr),
    App(AppExpr),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LitExpr {
    Bool(bool),
    Int(i64),
    Float(f64),
}

impl From<ast::LiteralExpr> for LitExpr {
    fn from(other: ast::LiteralExpr) -> LitExpr {
        match other {
            ast::LiteralExpr::Bool(x) => LitExpr::Bool(x),
            ast::LiteralExpr::Int(x) => LitExpr::Int(x),
            ast::LiteralExpr::Float(x) => LitExpr::Float(x),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VarExpr(pub Symbol);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct IfExpr {
    pub test: ExprId,
    pub then_branch: ExprId,
    pub else_branch: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LetExpr {
    pub bindings: Vec<Binding>,
    pub body: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    pub var: Symbol,
    pub val: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LambdaExpr {
    pub args: Vec<Symbol>,
    pub body: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppExpr {
    pub func: ExprId,
    pub args: Vec<ExprId>,
}
