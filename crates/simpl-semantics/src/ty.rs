use super::infer::{InferError, InferResult, Subst, TypeEnv};
use std::collections::{HashMap, HashSet};

/// A type variable represents a type that is not constrained.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeVar(pub u32);

impl TypeVar {
    /// Attempt to bind a type variable to a type, returning an appropriate
    /// substitution.
    fn bind(&self, ty: &Type) -> InferResult<Subst> {
        // Check for binding a variable to itself
        if let &Type::Var(ref u) = ty {
            if u == self {
                return Ok(Subst::new());
            }
        }

        // The occurs check prevents illegal recursive types.
        if ty.free_type_vars().contains(self) {
            return Err(InferError::new(format!(
                "occur check fails: {:?} vs {:?}",
                self, ty
            )));
        }

        let mut s = Subst::new();
        s.insert(self.clone(), ty.clone());
        Ok(s)
    }
}

/// A term variable is a variable referenced in an expression
pub type TermVar = String;

/// A monotype representing a concrete type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    Bool,
    Var(TypeVar),
    Fn(Vec<Type>, Box<Type>),
}

impl Type {
    /// Most general unifier, a substitution S such that S(self) is congruent to
    /// S(other).
    pub fn most_general_unifier(&self, other: &Type) -> InferResult<Subst> {
        match (self, other) {
            // If they are both primitives, no substitution needs to be done.
            (Type::Int, Type::Int) | (Type::Bool, Type::Bool) => Ok(Subst::new()),

            // If one of the types is variable, we can bind the variable to the type.
            // This also handles the case where they are both variables.
            (Type::Var(ref v), t) => v.bind(&t),
            (t, Type::Var(ref v)) => v.bind(&t),

            // For functions, we find the most general unifier for the inputs, apply the resulting
            // substitution to the outputs, find the outputs' most general unifier, and finally
            // compose the two resulting substitutions.
            (Type::Fn(ref args1, ref out1), Type::Fn(ref args2, ref out2)) => {
                let mut sub1 = Subst::new();
                for (arg1, arg2) in args1.iter().zip(args2) {
                    sub1 = sub1.compose(&arg1.most_general_unifier(arg2)?);
                }
                let sub2 = out1.apply(&sub1).most_general_unifier(&out2.apply(&sub1))?;
                Ok(sub1.compose(&sub2))
            }

            // Otherwise, the types cannot be unified.
            (t1, t2) => Err(InferError::new(format!(
                "types do not unify: {:?} vs {:?}",
                t1, t2
            ))),
        }
    }
}

/// A polytype is a type in which there are a number of for-all quantifiers,
/// i.e. some parts of the type may not be concrete but instead correct for all
/// possible types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Polytype {
    pub vars: Vec<TypeVar>,
    pub ty: Type,
}

impl Polytype {
    /// Instantiates a polytype into a type. Replaces all bound type variables
    /// with fresh type variables and return the resulting type.
    pub fn instantiate(&self, gen: &mut TypeVarGen) -> Type {
        let newvars = self.vars.iter().map(|_| Type::Var(gen.next()));
        self.ty
            .apply(&Subst(self.vars.iter().cloned().zip(newvars).collect()))
    }
}

/// A source of unique type variables.
#[derive(Debug, Clone, Default)]
pub struct TypeVarGen {
    counter: u32,
}

impl TypeVarGen {
    pub fn new() -> TypeVarGen {
        TypeVarGen { counter: 0 }
    }
    pub fn next(&mut self) -> TypeVar {
        let v = TypeVar(self.counter);
        self.counter += 1;
        v
    }
}

/// A trait common to all things considered types.
pub trait Types {
    /// Find the set of free variables in a type.
    fn free_type_vars(&self) -> HashSet<TypeVar>;

    /// Apply a substitution to a type.
    fn apply(&self, subst: &Subst) -> Self;
}

impl<T> Types for Vec<T>
where
    T: Types,
{
    // The free type variables of a vector of types is the union of the free type
    // variables of each of the types in the vector.
    fn free_type_vars(&self) -> HashSet<TypeVar> {
        self.iter()
            .map(|x| x.free_type_vars())
            .fold(HashSet::new(), |set, x| set.union(&x).cloned().collect())
    }

    // To apply a substitution to a vector of types, just apply to each type in the
    // vector.
    fn apply(&self, s: &Subst) -> Vec<T> {
        self.iter().map(|x| x.apply(s)).collect()
    }
}

impl Types for Type {
    fn free_type_vars(&self) -> HashSet<TypeVar> {
        match *self {
            // For a type variable, there is one free variable: the variable itself.
            Type::Var(ref s) => [s.clone()].iter().cloned().collect(),

            // Primitive types have no free variables
            Type::Int | Type::Bool | Type::Float => HashSet::new(),

            // For functions, we take the union of the free type variables of the input and output.
            Type::Fn(ref i, ref o) => i
                .free_type_vars()
                .union(&o.free_type_vars())
                .cloned()
                .collect(),
        }
    }

    fn apply(&self, s: &Subst) -> Type {
        match *self {
            // If this type references a variable that is in the substitution, return it's
            // replacement type. Otherwise, return the existing type.
            Type::Var(ref n) => s.get(n).cloned().unwrap_or(self.clone()),

            // To apply to a function, we simply apply to each of the input and output.
            Type::Fn(ref t1, ref t2) => Type::Fn(t1.apply(s), box t2.apply(s)),

            // A primitive type is changed by a substitution.
            _ => self.clone(),
        }
    }
}

impl Types for Polytype {
    /// The free type variables in a polytype are those that are free in the
    /// internal type and not bound by the variable mapping.
    fn free_type_vars(&self) -> HashSet<TypeVar> {
        self.ty
            .free_type_vars()
            .difference(&self.vars.iter().cloned().collect())
            .cloned()
            .collect()
    }

    /// Substitutions are applied to free type variables only.
    fn apply(&self, s: &Subst) -> Polytype {
        Polytype {
            vars: self.vars.clone(),
            ty: {
                let mut sub = s.clone();
                for var in &self.vars {
                    sub.remove(var);
                }
                self.ty.apply(&sub)
            },
        }
    }
}

impl Types for TypeEnv {
    /// The free type variables of a type environment is the union of the free
    /// type variables of each polytype in the environment.
    fn free_type_vars(&self) -> HashSet<TypeVar> {
        self.values()
            .map(|x| x.clone())
            .collect::<Vec<Polytype>>()
            .free_type_vars()
    }

    /// To apply a substitution, we just apply it to each polytype in the type
    /// environment.
    fn apply(&self, s: &Subst) -> TypeEnv {
        TypeEnv(self.iter().map(|(k, v)| (k.clone(), v.apply(s))).collect())
    }
}
