pub use crate::syntax::ast::{Ident, Lit};
use crate::{
    syntax::ast::Expr,
    types::ty::{Type, TypeEnv},
    util::counter::{Counter, FromId},
};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum TypedExpr {
    Lit {
        ty: Type,
        val: Lit,
    },
    Var {
        ty: Type,
        name: String,
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
    pub name: Ident,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LetBinding {
    pub ty: Type,
    pub name: Ident,
    pub val: Box<TypedExpr>,
}

pub type TypeVarGen = Counter<Type>;
impl FromId for Type {
    fn from_id(id: u32) -> Self {
        Self::Var(id)
    }
}

impl FromStr for TypedExpr {
    type Err = String;
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        // TODO: return a trait object instead of unwrapping
        let ast = crate::syntax::parse(src).unwrap();
        Self::from_ast(ast)
    }
}

impl TypedExpr {
    pub fn from_ast(ast: Expr) -> Result<Self, String> {
        let mut gen = TypeVarGen::new();
        let tenv = TypeEnv::default();
        Self::from_ast_inner(ast, &tenv, &mut gen)
    }

    fn from_ast_inner(ast: Expr, tenv: &TypeEnv, gen: &mut TypeVarGen) -> Result<Self, String> {
        let expr = match ast {
            Expr::Lit { val } => Self::Lit {
                val,
                ty: gen.next(),
            },
            Expr::Var { name } => match tenv.get(&name) {
                None => return Err(format!("Unbound variable: {}", name)),
                Some(ty) => Self::Var {
                    ty: ty.clone(),
                    name,
                },
            },
            Expr::If {
                test,
                then_branch,
                else_branch,
            } => Self::If {
                ty: gen.next(),
                test: box Self::from_ast_inner(*test, tenv, gen)?,
                then_branch: box Self::from_ast_inner(*then_branch, tenv, gen)?,
                else_branch: box Self::from_ast_inner(*else_branch, tenv, gen)?,
            },
            Expr::Let { bindings, body } => {
                let (bindings, body) = expand_let(&bindings, *body);

                assert!(bindings.len() == 1);
                let ty = gen.next();

                let (binding_name, binding_val) = &bindings[0];
                let binding_ty = gen.next();
                let mut extended_tenv = tenv.clone();
                extended_tenv.insert(binding_name.into(), binding_ty.clone());

                Self::Let {
                    ty,
                    binding: LetBinding {
                        ty: binding_ty,
                        name: binding_name.clone(),
                        val: box Self::from_ast_inner(binding_val.clone(), tenv, gen)?,
                    },
                    body: box Self::from_ast_inner(body, &extended_tenv, gen)?,
                }
            }
            Expr::Letrec { bindings, body } => {
                let ty = gen.next();
                let mut extended_tenv = tenv.clone();

                let mut tvars = vec![];
                for (name, _) in &bindings {
                    let tvar = gen.next();
                    extended_tenv.insert(name.clone(), tvar.clone());
                    tvars.push(tvar);
                }

                let mut new_bindings = vec![];

                for ((name, val), tvar) in bindings.into_iter().zip(tvars) {
                    new_bindings.push(LetBinding {
                        name: name.clone(),
                        ty: tvar,
                        val: box Self::from_ast_inner(val.clone(), &extended_tenv, gen)?,
                    });
                }

                Self::Letrec {
                    ty,
                    bindings: new_bindings,
                    body: box Self::from_ast_inner(*body, &extended_tenv, gen)?,
                }
            }
            Expr::Lambda { params, body } => {
                let (params, body) = expand_lambda(&params, *body);

                assert!(params.len() == 1);
                let ty = gen.next();

                let param_name = &params[0];
                let param_ty = gen.next();
                let mut extended_tenv = tenv.clone();
                extended_tenv.insert(param_name.clone(), param_ty.clone());
                Self::Lambda {
                    ty,
                    param: Param {
                        ty: param_ty,
                        name: param_name.clone(),
                    },
                    body: box Self::from_ast_inner(body, &extended_tenv, gen)?,
                }
            }
            Expr::App { func, arg } => Self::App {
                ty: gen.next(),
                func: box Self::from_ast_inner(*func, tenv, gen)?,
                arg: box Self::from_ast_inner(*arg, tenv, gen)?,
            },
        };
        Ok(expr)
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

/// Expand an `ast::Expr::Let` with many bound variables into a nested
/// sequences of lets each binding a single variable
/// eg `let x = 1, y = 2 in add x y` expands to
///    `let x = 1 in let y = 2 in add x y`
fn expand_let(bindings: &[(Ident, Expr)], body: Expr) -> (Vec<(Ident, Expr)>, Expr) {
    assert!(!bindings.is_empty());
    if bindings.len() == 1 {
        (bindings.to_vec(), body)
    } else {
        let binding = &bindings[0];

        let (rest_bindings, rest_body) = expand_let(&bindings[1..], body);

        (
            vec![binding.clone()],
            Expr::Let {
                bindings: rest_bindings,
                body: box rest_body,
            },
        )
    }
}

/// Expand an `ast::Expr::Lambda` with many parameters into a nested
/// sequences of lambdas each binding a single parameter
/// eg `\x, y -> add x y` expands to
///    `\x -> \y -> add x y`
fn expand_lambda(params: &[Ident], body: Expr) -> (Vec<Ident>, Expr) {
    assert!(!params.is_empty());
    if params.len() == 1 {
        (params.to_vec(), body)
    } else {
        let param = &params[0];

        let (rest_params, rest_body) = expand_lambda(&params[1..], body);

        (
            vec![param.clone()],
            Expr::Lambda {
                params: rest_params,
                body: box rest_body,
            },
        )
    }
}

#[test]
fn test_expand_let() {
    let bindings = vec![
        ("x".into(), Expr::Lit { val: Lit::Int(1) }),
        ("y".into(), Expr::Lit { val: Lit::Int(2) }),
    ];

    let body = Expr::Lit { val: Lit::Int(0) };

    assert_eq!(
        expand_let(&bindings, body),
        (
            vec![("x".into(), Expr::Lit { val: Lit::Int(1) }),],
            Expr::Let {
                bindings: vec![("y".into(), Expr::Lit { val: Lit::Int(2) })],
                body: box Expr::Lit { val: Lit::Int(0) }
            }
        ),
    )
}

#[test]
fn test_expand_lambda() {
    let params = vec!["x".into(), "y".into()];
    let body = Expr::Lit { val: Lit::Int(0) };

    assert_eq!(
        expand_lambda(&params, body),
        (
            vec!["x".into()],
            Expr::Lambda {
                params: vec!["y".into()],
                body: box Expr::Lit { val: Lit::Int(0) }
            }
        ),
    )
}
