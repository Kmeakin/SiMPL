use crate::types::{
    ast::{LetBinding, Param, TypedExpr},
    constraint::{Constraint, Constraints},
    ty::{Type, TypeVar},
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Subst(HashMap<TypeVar, Type>);

impl Constraint {
    pub fn apply(&self, subst: &Subst) -> Self {
        let Self(ty1, ty2) = self;
        Self(ty1.apply(subst), ty2.apply(subst))
    }
}

impl Type {
    pub fn apply(&self, subst: &Subst) -> Self {
        subst.0.iter().fold(self.clone(), |acc, solution| {
            let (tvar, solution_ty) = solution;
            subst.replace(acc, *tvar, solution_ty.clone())
        })
    }
}

impl TypedExpr {
    pub fn apply(&self, subst: &Subst) -> Self {
        match self {
            Self::Lit { ty, val } => Self::Lit {
                ty: ty.apply(subst),
                val: val.clone(),
            },
            Self::Var { ty, name } => Self::Var {
                ty: ty.apply(subst),
                name: name.clone(),
            },
            Self::If {
                ty,
                test,
                then_branch,
                else_branch,
            } => Self::If {
                ty: ty.apply(subst),
                test: box test.apply(subst),
                then_branch: box then_branch.apply(subst),
                else_branch: box else_branch.apply(subst),
            },
            Self::Let { ty, binding, body } => Self::Let {
                ty: ty.apply(subst),
                binding: LetBinding {
                    ty: binding.ty.apply(subst),
                    ..binding.clone()
                },
                body: box body.apply(subst),
            },
            Self::Letrec { ty, bindings, body } => Self::Letrec {
                ty: ty.apply(subst),
                bindings: bindings
                    .iter()
                    .map(|binding| LetBinding {
                        ty: binding.ty.apply(subst),
                        ..binding.clone()
                    })
                    .collect(),
                body: box body.apply(subst),
            },
            Self::Lambda { ty, param, body } => Self::Lambda {
                ty: ty.apply(subst),
                param: Param {
                    ty: param.ty.apply(subst),
                    ..param.clone()
                },
                body: box body.apply(subst),
            },
            Self::App { ty, func, arg } => Self::App {
                ty: ty.apply(subst),
                func: box func.apply(subst),
                arg: box arg.apply(subst),
            },
        }
    }
}

impl Subst {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, tvar: TypeVar, ty: Type) {
        self.0.insert(tvar, ty);
    }

    pub fn apply_ty(&self, ty: &Type) -> Type {
        ty.apply(self)
    }

    pub fn apply_con(&self, con: &Constraint) -> Constraint {
        con.apply(self)
    }

    pub fn apply_cons(&self, cons: &[Constraint]) -> Constraints {
        cons.iter().map(|con| self.apply_con(con)).collect()
    }

    pub fn apply_expr(&self, expr: &TypedExpr) -> TypedExpr {
        expr.apply(self)
    }

    // Replace all occurances of `tvar` in `ty` with `ty`
    pub fn replace(&self, ty: Type, tvar: TypeVar, replacement: Type) -> Type {
        match ty {
            Type::Int | Type::Bool | Type::Float => ty,
            Type::Var(tvar2) if tvar == tvar2 => replacement,
            Type::Var(_) => ty,
            Type::Fn(box arg, box ret) => Type::Fn(
                box self.replace(arg, tvar, replacement.clone()),
                box self.replace(ret, tvar, replacement),
            ),
        }
    }

    pub fn compose(&self, other: &Self) -> Self {
        let substituted_this: HashMap<TypeVar, Type> = self
            .0
            .iter()
            .map(|(tvar, ty)| (*tvar, ty.apply(other)))
            .collect();

        let mut new_subst = HashMap::new();
        new_subst.extend(substituted_this);
        new_subst.extend(other.0.clone());
        Self(new_subst)
    }

    pub fn from_pair(tvar: TypeVar, ty: Type) -> Self {
        let mut hm = HashMap::new();
        hm.insert(tvar, ty);
        Self(hm)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn subst_var() {
        let subst = Subst::from_pair(1, Type::Int);
        assert_eq!(subst.apply_ty(&Type::Var(1)), Type::Int);
    }

    #[test]
    fn subst_type_var_in_fn() {
        let mut subst = Subst::new();
        subst.insert(1, Type::Int);
        subst.insert(2, Type::Bool);
        assert_eq!(
            subst.apply_ty(&Type::Fn(box Type::Var(1), box Type::Var(2))),
            Type::Fn(box Type::Int, box Type::Bool)
        )
    }

    #[test]
    fn subst_constraint() {
        let mut subst = Subst::new();
        subst.insert(1, Type::Int);
        subst.insert(2, Type::Bool);
        assert_eq!(
            subst.apply_con(&Constraint(Type::Var(1), Type::Var(2))),
            Constraint(Type::Int, Type::Bool)
        )
    }

    #[test]
    fn subst_constraints() {
        let mut subst = Subst::new();
        subst.insert(1, Type::Int);
        subst.insert(2, Type::Bool);
        subst.insert(3, Type::Fn(box Type::Int, box Type::Bool));

        let cons = vec![
            Constraint(Type::Var(1), Type::Var(2)),
            Constraint(Type::Var(2), Type::Var(3)),
        ];

        assert_eq!(
            subst.apply_cons(&cons),
            vec![
                Constraint(Type::Int, Type::Bool),
                Constraint(Type::Bool, Type::Fn(box Type::Int, box Type::Bool))
            ]
        );
    }

    #[test]
    fn composes() {
        let mut subst1 = Subst::new();
        subst1.insert(3, Type::Var(1));
        subst1.insert(2, Type::Int);
        subst1.insert(4, Type::Fn(box Type::Var(1), box Type::Var(2)));

        let mut subst2 = Subst::new();
        subst2.insert(1, Type::Int);
        subst2.insert(2, Type::Bool);

        let mut expected = Subst::new();
        expected.insert(1, Type::Int);
        expected.insert(2, Type::Bool); // variable in subst2 overrides that in subst1
        expected.insert(3, Type::Int);
        expected.insert(4, Type::Fn(box Type::Int, box Type::Bool));

        assert_eq!(subst1.compose(&subst2), expected);
    }
}
