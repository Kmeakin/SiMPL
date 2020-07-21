use crate::{
    constraint::Constraint,
    subst::Subst,
    ty::{Type, TypeVar},
};

pub fn unify(cons: &[Constraint]) -> Subst {
    match &cons[..] {
        [] => Subst::new(),
        [head, tail @ ..] => {
            let subst = unify1(head);
            let substituted_tail = subst.apply_cons(&tail.to_vec());
            let subst_tail = unify(&substituted_tail);
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
        (Type::Var(tvar), ty) | (ty, Type::Var(tvar)) => unify_var(*tvar, ty),
        (Type::Fn(box arg1, box ret1), Type::Fn(box arg2, box ret2)) => unify(&[
            Constraint(arg1.clone(), arg2.clone()),
            Constraint(ret1.clone(), ret2.clone()),
        ]),
        _ => panic!("Cannot unify {} with {}", t1, t2),
    }
}

fn unify_var(tvar: TypeVar, ty: &Type) -> Subst {
    match ty {
        Type::Var(tvar2) if tvar == *tvar2 => Subst::new(),
        Type::Var(_) => Subst::from_pair(tvar, ty.clone()),
        ty if occurs(tvar, ty) => panic!("Circular use: {} occurs in {}", tvar, ty),
        ty => Subst::from_pair(tvar, ty.clone()),
    }
}

/// Check if `Type` contains `tvar`
fn occurs(tvar: TypeVar, ty: &Type) -> bool {
    match ty {
        Type::Var(tvar2) => tvar == *tvar2,
        Type::Fn(arg, ret) => occurs(tvar, arg) || occurs(tvar, ret),
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn unify_2_ints() {
        let subst = unify(&[Constraint(Type::Int, Type::Int)]);
        assert_eq!(subst, Subst::new());
    }

    #[test]
    fn unify_2_floats() {
        let subst = unify(&[Constraint(Type::Float, Type::Float)]);
        assert_eq!(subst, Subst::new());
    }

    #[test]
    fn unify_2_bools() {
        let subst = unify(&[Constraint(Type::Bool, Type::Bool)]);
        assert_eq!(subst, Subst::new());
    }

    #[test]
    fn unify_2_vars() {
        let subst = unify(&[Constraint(Type::Var(1), Type::Var(2))]);
        assert_eq!(subst, Subst::from_pair(1, Type::Var(2)));
    }

    #[test]
    fn unify_2_fns() {
        let subst = unify(&[Constraint(
            Type::Fn(box Type::Bool, box Type::Bool),
            Type::Fn(box Type::Bool, box Type::Bool),
        )]);
        assert_eq!(subst, Subst::new());
    }

    #[test]
    fn unify_var_with_non_var() {
        let subst = unify(&[Constraint(Type::Var(1), Type::Int)]);
        assert_eq!(subst, Subst::from_pair(1, Type::Int));
    }

    #[test]
    fn unify_vars_in_fns() {
        let subst = unify(&[Constraint(
            Type::Fn(box Type::Var(1), box Type::Bool),
            Type::Fn(box Type::Int, box Type::Var(2)),
        )]);

        let mut expected = Subst::new();
        expected.insert(1, Type::Int);
        expected.insert(2, Type::Bool);
        assert_eq!(subst, expected);
    }
}
