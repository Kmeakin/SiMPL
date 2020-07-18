use crate::{
    constraint::{Constraint, Constraints},
    ty::{Type, TypeVar},
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Subst(HashMap<TypeVar, Type>);

impl Constraint {
    pub fn apply(&self, subst: &Subst) -> Self {
        let Constraint(ty1, ty2) = self;
        Constraint(ty1.apply(subst), ty2.apply(subst))
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

    pub fn apply_cons(&self, cons: Constraints) -> Constraints {
        cons.iter().map(|con| self.apply_con(con)).collect()
    }

    // Replace all occurances of `tvar` in `ty` with `ty`
    pub fn replace(&self, ty: Type, tvar: TypeVar, replacement: Type) -> Type {
        match ty {
            Type::Int | Type::Bool | Type::Float => ty,
            Type::Var(tvar2) if tvar == tvar2 => replacement,
            Type::Var(_) => ty,
            Type::Fn(args, box ret) => Type::Fn(
                args.iter()
                    .map(|arg| self.replace(arg.clone(), tvar, replacement.clone()))
                    .collect(),
                box self.replace(ret, tvar, replacement),
            ),
        }
    }

    pub fn compose(&self, other: &Subst) -> Self {
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
            subst.apply_ty(&Type::Fn(vec![Type::Var(1)], box Type::Var(2))),
            Type::Fn(vec![Type::Int], box Type::Bool)
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
        subst.insert(3, Type::Fn(vec![Type::Int], box Type::Bool));

        let cons = vec![
            Constraint(Type::Var(1), Type::Var(2)),
            Constraint(Type::Var(2), Type::Var(3)),
        ];

        assert_eq!(
            subst.apply_cons(cons),
            vec![
                Constraint(Type::Int, Type::Bool),
                Constraint(Type::Bool, Type::Fn(vec![Type::Int], box Type::Bool))
            ]
        );
    }

    #[test]
    fn composes() {
        let mut subst1 = Subst::new();
        subst1.insert(3, Type::Var(1));
        subst1.insert(2, Type::Int);
        subst1.insert(4, Type::Fn(vec![Type::Var(1)], box Type::Var(2)));

        let mut subst2 = Subst::new();
        subst2.insert(1, Type::Int);
        subst2.insert(2, Type::Bool);

        let mut expected = Subst::new();
        expected.insert(1, Type::Int);
        expected.insert(2, Type::Bool); // variable in subst2 overrides that in subst1
        expected.insert(3, Type::Int);
        expected.insert(4, Type::Fn(vec![Type::Int], box Type::Bool));

        assert_eq!(subst1.compose(&subst2), expected);
    }
}
