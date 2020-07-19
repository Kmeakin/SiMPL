pub use simpl_syntax2::ast::{Ident, Lit};
use simpl_syntax2::{ast, parse, ParseError};

pub type ExprId = u32;

#[derive(Debug, Copy, Clone, Default)]
struct IdGen {
    counter: u32,
}

impl IdGen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next(&mut self) -> u32 {
        let x = self.counter;
        self.counter += 1;
        x
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Lit {
        id: ExprId,
        val: Lit,
    },
    Var {
        id: ExprId,
        name: String,
    },
    If {
        id: ExprId,
        test: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    Let {
        id: ExprId,
        bindings: Vec<(Ident, Expr)>,
        body: Box<Expr>,
    },
    Letrec {
        id: ExprId,
        bindings: Vec<(Ident, Expr)>,
        body: Box<Expr>,
    },
    Lambda {
        id: ExprId,
        params: Vec<Ident>,
        body: Box<Expr>,
    },
    App {
        id: ExprId,
        func: Box<Expr>,
        arg: Box<Expr>,
    },
}

impl Expr {
    pub fn from_ast(ast: ast::Expr) -> Expr {
        let mut gen = IdGen::new();

        Expr::from_ast_inner(ast, &mut gen)
    }

    pub fn from_str(src: &str) -> Result<Expr, ParseError> {
        let ast = parse(src)?;
        Ok(Expr::from_ast(ast))
    }

    fn from_ast_inner(ast: ast::Expr, gen: &mut IdGen) -> Expr {
        let id = gen.next();

        match ast {
            ast::Expr::Lit { val } => Expr::Lit { id, val },
            ast::Expr::Var { name } => Expr::Var { id, name },
            ast::Expr::If {
                box test,
                box then_branch,
                box else_branch,
            } => Expr::If {
                id,
                test: box Expr::from_ast_inner(test, gen),
                then_branch: box Expr::from_ast_inner(then_branch, gen),
                else_branch: box Expr::from_ast_inner(else_branch, gen),
            },
            ast::Expr::Let { bindings, box body } => Expr::Let {
                id,
                bindings: bindings
                    .into_iter()
                    .map(|(var, val)| (var, Expr::from_ast_inner(val, gen)))
                    .collect(),

                body: box Expr::from_ast_inner(body, gen),
            },
            ast::Expr::Letrec { bindings, box body } => Expr::Letrec {
                id,
                bindings: bindings
                    .into_iter()
                    .map(|(var, val)| (var, Expr::from_ast_inner(val, gen)))
                    .collect(),
                body: box Expr::from_ast_inner(body, gen),
            },
            ast::Expr::Lambda { params, box body } => Expr::Lambda {
                id,
                params,
                body: box Expr::from_ast_inner(body, gen),
            },
            ast::Expr::App { box func, box arg } => Expr::App {
                id,
                func: box Expr::from_ast_inner(func, gen),
                arg: box Expr::from_ast_inner(arg, gen),
            },
        }
    }
}
