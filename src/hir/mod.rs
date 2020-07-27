pub use crate::types::ty::Type;
use crate::{syntax::ast, types::ty::TypeVarGen};
use derive_more::Display;
pub use simple_symbol::Symbol;
use std::str::FromStr;

mod pp;

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
        test: Box<Self>,
        then_branch: Box<Self>,
        else_branch: Box<Self>,
    },
    Let {
        ty: Type,
        binding: LetBinding,
        body: Box<Self>,
    },
    Letrec {
        ty: Type,
        bindings: Vec<LetBinding>,
        body: Box<Self>,
    },
    Lambda {
        ty: Type,
        param: Param,
        body: Box<Self>,
    },
    App {
        ty: Type,
        func: Box<Self>,
        arg: Box<Self>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub ty: Type,
    pub name: Symbol,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LetBinding {
    pub ty: Type,
    pub name: Symbol,
    pub val: Box<Expr>,
}

#[derive(Debug, Copy, Clone, PartialEq, Display)]
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

impl From<ast::Lit> for Lit {
    fn from(other: ast::Lit) -> Self {
        match other {
            ast::Lit::Int(x) => Self::Int(x),
            ast::Lit::Float(x) => Self::Float(x),
            ast::Lit::Bool(x) => Self::Bool(x),
        }
    }
}

impl FromStr for Expr {
    type Err = String;
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        // TODO: return a trait object instead of unwrapping
        let ast = crate::syntax::parse(src).unwrap();
        Ok(Self::from_ast(ast))
    }
}

impl Expr {
    /// `ast::Expr` -> `hir::Expr`
    /// Expands nested let/lambda, and attatches fresh type variables to every
    /// expression/binder
    pub fn from_ast(ast: ast::Expr) -> Self {
        let mut gen = TypeVarGen::new();
        Self::from_ast_inner(ast, &mut gen)
    }

    fn from_ast_inner(ast: ast::Expr, gen: &mut TypeVarGen) -> Self {
        match ast {
            ast::Expr::Lit { val } => Self::Lit {
                val: val.into(),
                ty: gen.next(),
            },
            ast::Expr::Var { name } => Self::Var {
                name,
                ty: gen.next(),
            },
            ast::Expr::If {
                test,
                then_branch,
                else_branch,
            } => Self::If {
                ty: gen.next(),
                test: box Self::from_ast_inner(*test, gen),
                then_branch: box Self::from_ast_inner(*then_branch, gen),
                else_branch: box Self::from_ast_inner(*else_branch, gen),
            },
            ast::Expr::Let { bindings, body } => {
                let ((name, val), body) = expand_let(&bindings, *body);
                Self::Let {
                    ty: gen.next(),
                    binding: LetBinding {
                        ty: gen.next(),
                        name,
                        val: box Self::from_ast_inner(val, gen),
                    },
                    body: box Self::from_ast_inner(body, gen),
                }
            }
            ast::Expr::Letrec { bindings, body } => Self::Letrec {
                ty: gen.next(),
                bindings: bindings
                    .into_iter()
                    .map(|(name, val)| LetBinding {
                        ty: gen.next(),
                        name,
                        val: box Self::from_ast_inner(val, gen),
                    })
                    .collect(),
                body: box Self::from_ast_inner(*body, gen),
            },
            ast::Expr::Lambda { params, body } => {
                let (param, body) = expand_lambda(&params, *body);
                Self::Lambda {
                    ty: gen.next(),
                    param: Param {
                        name: param,
                        ty: gen.next(),
                    },
                    body: box Self::from_ast_inner(body, gen),
                }
            }
            ast::Expr::App { func, arg } => Self::App {
                ty: gen.next(),
                func: box Self::from_ast_inner(*func, gen),
                arg: box Self::from_ast_inner(*arg, gen),
            },
        }
    }

    pub fn ty(&self) -> Type {
        match self {
            Self::Lit { ty, .. }
            | Self::Var { ty, .. }
            | Self::If { ty, .. }
            | Self::Let { ty, .. }
            | Self::Letrec { ty, .. }
            | Self::Lambda { ty, .. }
            | Self::App { ty, .. } => ty.clone(),
        }
    }
}

fn expand_lambda(params: &[Symbol], body: ast::Expr) -> (Symbol, ast::Expr) {
    assert!(!params.is_empty());
    if params.len() == 1 {
        (params[0], body)
    } else {
        let param = &params[0];
        let (rest_param, rest_body) = expand_lambda(&params[1..], body);
        (
            *param,
            ast::Expr::Lambda {
                params: vec![rest_param],
                body: box rest_body,
            },
        )
    }
}

fn expand_let(
    bindings: &[(Symbol, ast::Expr)],
    body: ast::Expr,
) -> ((Symbol, ast::Expr), ast::Expr) {
    assert!(!bindings.is_empty());
    if bindings.len() == 1 {
        (bindings[0].clone(), body)
    } else {
        let binding = &bindings[0];
        let (rest_bindings, rest_body) = expand_let(&bindings[1..], body);
        (
            binding.clone(),
            ast::Expr::Let {
                bindings: vec![rest_bindings],
                body: box rest_body,
            },
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::syntax::ast::{Expr, Lit};
    use simple_symbol::intern;

    #[test]
    fn test_expand_lambda() {
        let params = vec![intern("x"), intern("y")];
        let body = Expr::Lit { val: Lit::Int(0) };

        assert_eq!(
            expand_lambda(&params, body),
            (
                intern("x"),
                Expr::Lambda {
                    params: vec![intern("y")],
                    body: box Expr::Lit { val: Lit::Int(0) }
                }
            ),
        )
    }

    #[test]
    fn test_expand_let() {
        let bindings = vec![
            (intern("x"), Expr::Lit { val: Lit::Int(1) }),
            (intern("y"), Expr::Lit { val: Lit::Int(2) }),
        ];

        let body = Expr::Lit { val: Lit::Int(0) };

        assert_eq!(
            expand_let(&bindings, body),
            (
                (intern("x"), Expr::Lit { val: Lit::Int(1) }),
                Expr::Let {
                    bindings: vec![(intern("y"), Expr::Lit { val: Lit::Int(2) })],
                    body: box Expr::Lit { val: Lit::Int(0) }
                }
            ),
        )
    }
}
