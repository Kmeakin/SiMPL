use crate::{
    ty::{Type, TypeEnv},
    FromId, IdGen,
};
use simpl_syntax::ast::Expr;
pub use simpl_syntax::ast::{Ident, Lit};

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

pub type TypeVarGen = IdGen<Type>;
impl FromId for Type {
    fn from_id(id: u32) -> Type {
        Type::Var(id)
    }
}

impl TypedExpr {
    pub fn from_str(src: &str) -> Result<Self, String> {
        // TODO: return a trait object instead of unwrapping
        let ast = simpl_syntax::parse(src).unwrap();
        Self::from_ast(ast)
    }

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
                assert!(bindings.len() >= 1);
                let ty = gen.next();

                // TODO: desugar `let v1 = e1, v2 = e2 in b` into
                // `let v1 = e1 in let v2 = e2 in b`
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
                    body: box Self::from_ast_inner(*body, &extended_tenv, gen)?,
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
                assert!(params.len() >= 1);
                let ty = gen.next();

                // TODO: desugar `\x, y -> e` into `\x -> \y -> e`
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
                    body: box Self::from_ast_inner(*body, &extended_tenv, gen)?,
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
