use crate::{
    constraint::{Constraint, Constraints},
    subst::Subst,
    ty::{Type, TypeVar},
};

fn unify(cons: Constraints) -> Subst {
    match &cons[..] {
        [] => Subst::new(),
        [head, tail @ ..] => {
            let subst = unify1(head);
            let substituted_tail = subst.apply_cons(tail.to_vec());
            let subst_tail = unify(substituted_tail);
            subst.compose(&subst_tail)
        }
    }
}

fn unify1(con: &Constraint) -> Subst {
    let Constraint(t1, t2) = con;
    match (t1, t2) {
        (Type::Int, Type::Int) | (Type::Bool, Type::Bool) | (Type::Float, Type::Float) => {
            Subst::new()
        }
        (Type::Var(tvar), ty) | (ty, Type::Var(tvar)) => unify_var(&tvar, &ty),
        (Type::Fn(args1, box ret1), Type::Fn(args2, box ret2)) => unify(
            args1
                .into_iter()
                .zip(args2)
                .map(|(arg1, arg2)| Constraint(arg1.clone(), arg2.clone()))
                .chain(vec![Constraint(ret1.clone(), ret2.clone())])
                .collect(),
        ),
        _ => panic!("Cannot unify {} with {}", t1, t2),
    }
}

fn unify_var(tvar: &TypeVar, ty: &Type) -> Subst {
    match ty {
        Type::Var(tvar2) if tvar == tvar2 => Subst::new(),
        Type::Var(_) => Subst::from_pair(*tvar, ty.clone()),
        ty if occurs(tvar, ty) => panic!("Circular use: {} occurs in {}", tvar, ty),
        ty => Subst::from_pair(*tvar, ty.clone()),
    }
}

/// Check if `Type` contains `tvar`
fn occurs(tvar: &TypeVar, ty: &Type) -> bool {
    match ty {
        Type::Var(tvar2) => tvar == tvar2,
        Type::Fn(args, ret) => args.iter().any(|arg| occurs(tvar, arg)) || occurs(tvar, ret),
        _ => false,
    }
}

mod test {
    use super::*;

    #[test]
    fn unify_2_ints() {
        let subst = unify(vec![Constraint(Type::Int, Type::Int)]);
        assert_eq!(subst, Subst::new());
    }

    #[test]
    fn unify_2_floats() {
        let subst = unify(vec![Constraint(Type::Float, Type::Float)]);
        assert_eq!(subst, Subst::new());
    }

    #[test]
    fn unify_2_bools() {
        let subst = unify(vec![Constraint(Type::Bool, Type::Bool)]);
        assert_eq!(subst, Subst::new());
    }

    #[test]
    fn unify_2_vars() {
        let subst = unify(vec![Constraint(Type::Var(1), Type::Var(2))]);
        assert_eq!(subst, Subst::from_pair(1, Type::Var(2)));
    }

    #[test]
    fn unify_2_fns() {
        let subst = unify(vec![Constraint(
            Type::Fn(vec![Type::Bool], box Type::Bool),
            Type::Fn(vec![Type::Bool], box Type::Bool),
        )]);
        assert_eq!(subst, Subst::new());
    }

    #[test]
    fn unify_var_with_non_var() {
        let subst = unify(vec![Constraint(Type::Var(1), Type::Int)]);
        assert_eq!(subst, Subst::from_pair(1, Type::Int));
    }

    #[test]
    fn unify_vars_in_fns() {
        let subst = unify(vec![Constraint(
            Type::Fn(vec![Type::Var(1)], box Type::Bool),
            Type::Fn(vec![Type::Int], box Type::Var(2)),
        )]);

        let mut expected = Subst::new();
        expected.insert(1, Type::Int);
        expected.insert(2, Type::Bool);
        assert_eq!(subst, expected);
    }
}
