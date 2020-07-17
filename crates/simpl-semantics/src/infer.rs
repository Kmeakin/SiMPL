use super::{
    ty::{Polytype, TermVar, Type, TypeVar, TypeVarGen, Types},
    Union,
};
use simpl_syntax::ast::{AppExpr, Binding, Expr, LambdaExpr, LetExpr, LiteralExpr};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    ops::{Deref, DerefMut},
};

pub type InferResult<T> = Result<T, InferError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InferError {
    msg: String,
}

impl InferError {
    pub fn new(msg: String) -> InferError {
        InferError { msg }
    }
}

/// A substitution is a mapping from type variables to types.
#[derive(Clone, Debug)]
pub struct Subst(pub HashMap<TypeVar, Type>);

impl Deref for Subst {
    type Target = HashMap<TypeVar, Type>;
    fn deref(&self) -> &HashMap<TypeVar, Type> {
        &self.0
    }
}
impl DerefMut for Subst {
    fn deref_mut(&mut self) -> &mut HashMap<TypeVar, Type> {
        &mut self.0
    }
}

impl Subst {
    /// Construct an empty substitution.
    pub fn new() -> Subst {
        Subst(HashMap::new())
    }

    /// To compose two substitutions, we apply self to each type in other and
    /// union the resulting substitution with self.
    pub fn compose(&self, other: &Subst) -> Subst {
        Subst(
            self.union(
                &other
                    .iter()
                    .map(|(k, v)| (k.clone(), v.apply(self)))
                    .collect(),
            ),
        )
    }
}

/// A type environment is a mapping from Expr variables to polytypes.
#[derive(Clone, Debug)]
pub struct TypeEnv(pub HashMap<TermVar, Polytype>);

impl Deref for TypeEnv {
    type Target = HashMap<TermVar, Polytype>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for TypeEnv {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TypeEnv {
    /// Construct an empty type environment.
    pub fn new() -> TypeEnv {
        TypeEnv(HashMap::new())
    }

    /// Create a polytype
    fn generalize(&self, ty: &Type) -> Polytype {
        Polytype {
            vars: ty
                .free_type_vars()
                .difference(&self.free_type_vars())
                .cloned()
                .collect(),
            ty: ty.clone(),
        }
    }

    fn infer_inner(&self, expr: &Expr, gen: &mut TypeVarGen) -> InferResult<(Subst, Type)> {
        let (subst, ty) = match expr {
            // A literal is typed as it's primitive type.
            Expr::Lit(ref lit) => Ok((
                Subst::new(),
                match lit {
                    LiteralExpr::Int(_) => Type::Int,
                    LiteralExpr::Float(_) => Type::Float,
                    LiteralExpr::Bool(_) => Type::Bool,
                },
            )),

            // A variable is typed as an instantiation of the corresponding type in the
            // environment.
            Expr::Var(ref v) => match self.get(&v.0) {
                Some(s) => Ok((Subst::new(), s.instantiate(gen))),
                None => Err(InferError::new(format!("Unbound variable: {:?}", v))),
            },

            // Let (variable binding) is typed by:
            // - Removing any existing type with the same name as the binding variable to prevent
            // name clashes.
            // - Inferring the type of the binding.
            // - Applying the resulting substitution to the environment and generalizing to the
            // binding type.
            // - Inserting the generalized type to the binding variable in the new environment.
            // - Applying the substution for the binding to the environment and inferring the type
            // of the expression.
            Expr::Let(LetExpr { bindings, body }) => {
                let mut env = self.clone();

                let mut s1 = Subst::new();
                for Binding { var, .. } in bindings {
                    // Remove any existing type with the same name as the binding variable to
                    // prevent name clashes
                    env.remove(var);

                    // letrec
                    env.insert(
                        var.clone(),
                        Polytype {
                            ty: Type::Var(gen.next()),
                            vars: vec![],
                        },
                    );
                }

                for Binding { var, val } in bindings {
                    // Infer the type of the binding
                    let (s, t) = env.infer_inner(val, gen)?;
                    s1 = s1.compose(&s);
                    let tp = env.apply(&s1).generalize(&t);

                    // Insert the generalized type in the new environment
                    env.insert(var.clone(), tp);
                }

                let (s2, t2) = env.apply(&s1).infer_inner(body, gen)?;
                Ok((s2.compose(&s1), t2))
            }

            // An abstraction is typed by:
            // - Removing any existing type with the same name as the argument to prevent name
            // clashes.
            // - Inserting a new type variable for the argument.
            // - Inferring the type of the expression in the new environment to define the type of
            // the expression.
            // - Applying the resulting substitution to the argument to define the type of the
            // argument.
            Expr::Lambda(LambdaExpr { args, body }) => {
                let mut env = self.clone();
                let mut freshes = Vec::new();

                for arg in args {
                    // Remove any existing type with the same name as the argument to prevent name
                    // clashes
                    env.remove(arg);

                    // choose a fresh type variable for the argument
                    let fresh = Type::Var(gen.next());
                    freshes.push(fresh.clone());

                    let fresh_poly = Polytype {
                        vars: Vec::new(),
                        ty: fresh.clone(),
                    };
                    // add the argument's type variable to the environemnt
                    env.insert(arg.into(), fresh_poly);
                }

                // infer a type for the body
                let (s1, t1) = env.infer_inner(body, gen)?;

                // the fresh type variable may be unified with a concrete type if var is used in
                // the body
                Ok((
                    s1.clone(),
                    Type::Fn(
                        freshes.iter().map(|fresh| fresh.apply(&s1)).collect(),
                        box t1,
                    ),
                ))
            }

            // An application is typed by:
            // - Inferring the type of the callee.
            // - Applying the resulting substitution to the argument and inferring it's type.
            // - Finding the most general unifier for the callee type and a function from the
            // argument type to a new type variable. This combines the previously known type of the
            // function and the type as it is now being used.
            // - Applying the unifier to the new type variable.
            Expr::App(AppExpr { func, args }) => {
                let (s1, t1) = self.infer_inner(func, gen)?;
                match t1 {
                    Type::Fn(params, _) if params.len() != args.len() => {
                        return Err(InferError::new(format!(
                            "Arity mismatch: expected {} arguments, got {}",
                            params.len(),
                            args.len()
                        )));
                    }
                    _ => {}
                };

                let env = self.apply(&s1);

                let mut s2 = Subst::new();
                let mut arg_types = Vec::new();
                for arg in args {
                    let (s, t) = env.infer_inner(arg, gen)?;
                    s2 = s2.compose(&s);
                    arg_types.push(t);
                }

                let tv = Type::Var(gen.next());
                let s3 = t1
                    .apply(&s2)
                    .most_general_unifier(&Type::Fn(arg_types, box tv.clone()))?;

                Ok((s3.compose(&s2.compose(&s1)), tv.apply(&s3)))
            }
        }?;

        Ok((subst, ty))
    }

    /// Perform type inference on an expression and return the resulting type,
    /// if any.
    pub fn infer(&self, exp: &Expr, gen: &mut TypeVarGen) -> InferResult<Type> {
        let (subst, ty) = self.infer_inner(exp, gen)?;
        Ok(ty.apply(&subst))
    }
}
