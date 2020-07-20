use crate::{
    ast::{Lit, TypedExpr},
    ty::{LitExt, Type},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraint(pub(crate) Type, pub(crate) Type);
pub type Constraints = Vec<Constraint>;

pub fn collect(expr: TypedExpr) -> Constraints {
    match expr {
        TypedExpr::Lit { ty, val } => vec![Constraint(ty, val.ty())],
        TypedExpr::Var { .. } => vec![],
        TypedExpr::If {
            ty,
            test,
            then_branch,
            else_branch,
        } => {
            let mut cons = vec![
                Constraint(test.ty(), Type::Bool),
                Constraint(then_branch.ty(), ty.clone()),
                Constraint(else_branch.ty(), ty),
            ];
            cons.extend(collect(*test));
            cons.extend(collect(*then_branch));
            cons.extend(collect(*else_branch));
            cons
        }

        TypedExpr::Let { ty, binding, body } => {
            let mut cons = vec![
                Constraint(ty, body.ty()),
                Constraint(binding.ty, binding.val.ty()),
            ];
            cons.extend(collect(*binding.val));
            cons.extend(collect(*body));
            cons
        }
        TypedExpr::Letrec { ty, bindings, body } => {
            assert!(bindings.len() >= 1);
            let binding = &bindings[0];

            let mut cons = vec![
                Constraint(ty, body.ty()),
                Constraint(binding.ty.clone(), binding.val.ty()),
            ];

            cons.extend(collect(*binding.val.clone()));
            cons.extend(collect(*body));
            cons
        }
        TypedExpr::Lambda { ty, param, body } => {
            let mut cons = vec![Constraint(ty, Type::Fn(box param.ty, box body.ty()))];
            cons.extend(collect(*body));
            cons
        }
        TypedExpr::App { ty, func, arg } => {
            let mut cons = vec![Constraint(func.ty(), Type::Fn(box arg.ty(), box ty))];
            cons.extend(collect(*func));
            cons.extend(collect(*arg));
            cons
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ast::{LetBinding, Param, TypeVarGen};

    #[test]
    fn constrain_int() {
        let mut gen = TypeVarGen::new();

        let t1 = gen.next();
        let expr = TypedExpr::Lit {
            ty: t1.clone(),
            val: Lit::Int(1),
        };
        assert_eq!(collect(expr), vec![Constraint(t1, Type::Int)]);
    }

    #[test]
    fn constrain_bool() {
        let mut gen = TypeVarGen::new();

        let t1 = gen.next();
        let expr = TypedExpr::Lit {
            ty: t1.clone(),
            val: Lit::Bool(true),
        };
        assert_eq!(collect(expr), vec![Constraint(t1, Type::Bool)]);
    }

    #[test]
    fn constrain_float() {
        let mut gen = TypeVarGen::new();

        let t1 = gen.next();
        let expr = TypedExpr::Lit {
            ty: t1.clone(),
            val: Lit::Float(1.23),
        };
        assert_eq!(collect(expr), vec![Constraint(t1, Type::Float)]);
    }

    #[test]
    fn constrain_lambda() {
        let mut gen = TypeVarGen::new();

        let t1 = gen.next();
        let t2 = gen.next();
        let t3 = gen.next();

        let expr = TypedExpr::Lambda {
            ty: t1.clone(),
            param: Param {
                ty: t2.clone(),
                name: "param".into(),
            },
            body: box TypedExpr::Var {
                ty: t3.clone(),
                name: "param".into(),
            },
        };
        assert_eq!(
            collect(expr),
            vec![Constraint(t1, Type::Fn(box t2.clone(), box t3))]
        );
    }

    #[test]
    fn constrain_var() {
        let mut gen = TypeVarGen::new();

        let t1 = gen.next();

        let expr = TypedExpr::Var {
            ty: t1.clone(),
            name: "name".into(),
        };

        assert_eq!(collect(expr), vec![]);
    }

    #[test]
    fn constrain_app() {
        let mut gen = TypeVarGen::new();

        let t1 = gen.next();
        let t2 = gen.next();
        let t3 = gen.next();

        let expr = TypedExpr::App {
            ty: t1.clone(),
            func: box TypedExpr::Var {
                ty: t2.clone(),
                name: "fn".into(),
            },
            arg: box TypedExpr::Var {
                ty: t3.clone(),
                name: "arg".into(),
            },
        };

        assert_eq!(
            collect(expr),
            vec![Constraint(t2, Type::Fn(box t3, box t1))]
        );
    }

    #[test]
    fn constrain_let() {
        let mut gen = TypeVarGen::new();

        let t1 = gen.next();
        let t2 = gen.next();
        let t3 = gen.next();
        let t4 = gen.next();

        let expr = TypedExpr::Let {
            ty: t1.clone(),
            binding: LetBinding {
                ty: t2.clone(),
                name: "name".into(),
                val: box TypedExpr::Var {
                    ty: t3.clone(),
                    name: "value".into(),
                },
            },
            body: box TypedExpr::Var {
                ty: t4.clone(),
                name: "body".into(),
            },
        };

        assert_eq!(collect(expr), vec![Constraint(t1, t4), Constraint(t2, t3)]);
    }

    #[test]
    fn constrain_identity() {
        let mut gen = TypeVarGen::new();

        let t1 = gen.next();
        let t2 = gen.next();
        let t3 = gen.next();

        let expr = TypedExpr::Lambda {
            ty: t1.clone(),
            param: Param {
                ty: t2.clone(),
                name: "a".into(),
            },
            body: box TypedExpr::Var {
                ty: t3.clone(),
                name: "a".into(),
            },
        };

        assert_eq!(
            collect(expr),
            vec![Constraint(t1, Type::Fn(box t2, box t3))]
        );
    }

    #[test]
    fn constrain_const() {
        let mut gen = TypeVarGen::new();
        let t1 = gen.next();
        let t2 = gen.next();
        let t3 = gen.next();
        let t4 = gen.next();
        let t5 = gen.next();

        let expr = TypedExpr::Lambda {
            ty: t1.clone(),
            param: Param {
                ty: t2.clone(),
                name: "a".into(),
            },
            body: box TypedExpr::Lambda {
                ty: t3.clone(),
                param: Param {
                    ty: t4.clone(),
                    name: "b".into(),
                },
                body: box TypedExpr::Var {
                    ty: t5.clone(),
                    name: "a".into(),
                },
            },
        };

        assert_eq!(
            collect(expr),
            vec![
                Constraint(t1, Type::Fn(box t2, box t3.clone())),
                Constraint(t3, Type::Fn(box t4, box t5))
            ]
        )
    }

    #[test]
    fn constrain_compose() {
        let mut gen = TypeVarGen::new();
        let t1 = gen.next();
        let t2 = gen.next();
        let t3 = gen.next();
        let t4 = gen.next();
        let t5 = gen.next();
        let t6 = gen.next();
        let t7 = gen.next();
        let t8 = gen.next();
        let t9 = gen.next();
        let t10 = gen.next();
        let t11 = gen.next();

        let expr = TypedExpr::Lambda {
            ty: t1.clone(),
            param: Param {
                ty: t2.clone(),
                name: "f".into(),
            },
            body: box TypedExpr::Lambda {
                ty: t3.clone(),
                param: Param {
                    ty: t4.clone(),
                    name: "g".into(),
                },
                body: box TypedExpr::Lambda {
                    ty: t5.clone(),
                    param: Param {
                        ty: t6.clone(),
                        name: "x".into(),
                    },
                    body: box TypedExpr::App {
                        ty: t7.clone(),
                        func: box TypedExpr::Var {
                            ty: t8.clone(),
                            name: "f".into(),
                        },
                        arg: box TypedExpr::App {
                            ty: t9.clone(),
                            func: box TypedExpr::Var {
                                ty: t10.clone(),
                                name: "g".into(),
                            },
                            arg: box TypedExpr::Var {
                                ty: t11.clone(),
                                name: "x".into(),
                            },
                        },
                    },
                },
            },
        };

        assert_eq!(
            collect(expr),
            vec![
                Constraint(t1.clone(), Type::Fn(box t2.clone(), box t3.clone())),
                Constraint(t3.clone(), Type::Fn(box t4.clone(), box t5.clone())),
                Constraint(t5.clone(), Type::Fn(box t6.clone(), box t7.clone())),
                Constraint(t8.clone(), Type::Fn(box t9.clone(), box t7.clone())),
                Constraint(t10.clone(), Type::Fn(box t11.clone(), box t9.clone())),
            ]
        );
    }
}
