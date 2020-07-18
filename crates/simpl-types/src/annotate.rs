use crate::arena::{
    AppExpr, Binding, Expr, ExprArena, ExprId, IfExpr, LambdaExpr, LetExpr, LitExpr, Symbol,
    VarExpr,
};
use derive_more::Display;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
pub struct TypeVarGen {
    counter: u32,
}

impl Default for TypeVarGen {
    fn default() -> Self {
        Self { counter: 1 }
    }
}

impl TypeVarGen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fresh(&mut self) -> TypeVar {
        let t = self.counter;
        self.counter += 1;
        TypeVar(t)
    }
}

pub type Env = HashMap<Symbol, TypeVar>;
pub type Annotations = HashMap<ExprId, TypeVar>;
pub type Constraints = Vec<Constraint>;

#[derive(Debug, Clone, Display)]
#[display(fmt = "{} = {}", _0, _1)]
pub struct Constraint(TypeVar, Type);

#[derive(Debug, Clone, Display)]
enum Type {
    #[display(fmt = "Int")]
    Int,
    #[display(fmt = "Bool")]
    Bool,
    #[display(fmt = "Float")]
    Float,
    #[display(fmt = "{}", _0)]
    Var(TypeVar),

    #[display(fmt = "{} -> {}", "display_fn_args(_0)", _1)]
    Fn(Vec<Type>, Box<Type>),
}

fn display_fn_args(args: &[Type]) -> String {
    format!(
        "({})",
        args.iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[derive(Debug, Clone, Default)]
pub struct Annotator {
    arena: ExprArena,
    gen: TypeVarGen,
    annotations: HashMap<ExprId, TypeVar>,
    constraints: Constraints,
}

#[derive(Debug, Copy, Clone, Display)]
#[display(fmt = "t{}", _0)]
pub struct TypeVar(u32);

pub fn annotate(arena: ExprArena, id: ExprId) -> (Annotations, Constraints) {
    let annotator = Annotator::new(arena);
    annotator.annotate(id)
}

impl Annotator {
    pub fn new(arena: ExprArena) -> Self {
        Self {
            arena,
            ..Self::default()
        }
    }

    pub fn annotate(mut self, id: ExprId) -> (Annotations, Constraints) {
        self.annotate_inner(id, Env::new());
        (self.annotations, self.constraints)
    }

    fn annotate_inner(&mut self, expr_id: ExprId, env: Env) -> TypeVar {
        let arena = self.arena.clone();
        let expr = &arena[expr_id];

        match expr {
            Expr::Lit(lit) => {
                let tvar = self.gen.fresh();
                self.annotations.insert(expr_id, tvar);

                match lit {
                    LitExpr::Int(_) => self.constraints.push(Constraint(tvar, Type::Int)),
                    LitExpr::Bool(_) => self.constraints.push(Constraint(tvar, Type::Bool)),
                    LitExpr::Float(_) => self.constraints.push(Constraint(tvar, Type::Float)),
                }

                tvar
            }

            Expr::Var(VarExpr(var)) => match env.get(&var.to_owned()) {
                Some(&tvar) => {
                    self.annotations.insert(expr_id, tvar);
                    tvar
                }
                None => panic!(format!("Unbound variable: {}", var)),
            },

            Expr::If(IfExpr {
                test,
                then_branch,
                else_branch,
            }) => {
                // Annotate the parent expr
                let tvar = self.gen.fresh();
                self.annotations.insert(expr_id, tvar);

                let test_tvar = self.annotate_inner(*test, env.clone());
                let then_tvar = self.annotate_inner(*then_branch, env.clone());
                let else_tvar = self.annotate_inner(*else_branch, env);

                self.constraints.push(Constraint(test_tvar, Type::Bool));
                self.constraints
                    .push(Constraint(then_tvar, Type::Var(tvar)));
                self.constraints
                    .push(Constraint(else_tvar, Type::Var(tvar)));

                tvar
            }

            Expr::Lambda(LambdaExpr { args, body }) => {
                let tvar = self.gen.fresh();
                self.annotations.insert(expr_id, tvar);

                // Add the args to the environment
                let mut env = env.clone();
                let mut arg_tvars = vec![];
                args.iter().for_each(|arg| {
                    let arg_tvar = self.gen.fresh();
                    arg_tvars.push(Type::Var(arg_tvar));
                    env.insert(arg.into(), arg_tvar);
                });
                let body_tvar = self.annotate_inner(*body, env);

                self.constraints.push(Constraint(
                    tvar,
                    Type::Fn(arg_tvars, box Type::Var(body_tvar)),
                ));

                tvar
            }

            Expr::App(AppExpr { func, args }) => {
                let tvar = self.gen.fresh();
                self.annotations.insert(expr_id, tvar);

                let func_tvar = self.annotate_inner(*func, env.clone());

                args.iter().for_each(|arg| {
                    self.annotate_inner(*arg, env.clone());
                });

                let arg_tvars = (0..args.len())
                    .into_iter()
                    .map(|_| Type::Var(self.gen.fresh()))
                    .collect();

                self.constraints.push(Constraint(
                    func_tvar,
                    Type::Fn(arg_tvars, box Type::Var(tvar)),
                ));

                tvar
            }

            Expr::Let(LetExpr { bindings, body }) => {
                let tvar = self.gen.fresh();
                self.annotations.insert(expr_id, tvar);

                let mut extended_env = env.clone();
                for Binding { var, val } in bindings {
                    let var_tvar = self.gen.fresh();
                    extended_env.insert(var.into(), var_tvar);

                    let val_tvar = self.annotate_inner(*val, extended_env.clone());

                    self.constraints
                        .push(Constraint(var_tvar, Type::Var(val_tvar)));
                }

                let body_tvar = self.annotate_inner(*body, extended_env);

                self.constraints
                    .push(Constraint(tvar, Type::Var(body_tvar)));
                tvar
            }
        }
    }
}
