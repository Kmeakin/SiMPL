use crate::{
    ty::{Type, TypeEnv},
    FromId, IdGen,
};
use simpl_syntax2::ast::Expr;
pub use simpl_syntax2::ast::{Ident, Lit};

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
        binding: (Type, Ident, Box<Self>),
        body: Box<Self>,
    },
    Letrec {
        ty: Type,
        bindings: Vec<(Type, Ident, Self)>,
        body: Box<Self>,
    },
    Lambda {
        ty: Type,
        param: (Type, Ident),
        body: Box<Self>,
    },
    App {
        ty: Type,
        func: Box<Self>,
        arg: Box<Self>,
    },
}

pub type TypeVarGen = IdGen<Type>;
impl FromId for Type {
    fn from_id(id: u32) -> Type {
        Type::Var(id)
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
            Expr::Let { bindings, body } => todo!(),
            Expr::Letrec { bindings, body } => todo!(),
            Expr::Lambda { params, body } => todo!(),
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
