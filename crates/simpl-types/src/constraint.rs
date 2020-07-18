use crate::{
    ty::Type,
    typed_ast::{Expr, Lit},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraint(pub(crate) Type, pub(crate) Type);
pub type Constraints = Vec<Constraint>;

fn lit_type(lit: Lit) -> Type {
    match lit {
        Lit::Int(_) => Type::Int,
        Lit::Bool(_) => Type::Bool,
        Lit::Float(_) => Type::Float,
    }
}

pub fn collect(expr: Expr) -> Constraints {
    match expr {
        Expr::Lit { ty, val } => vec![Constraint(ty, lit_type(val))],

        Expr::Var { ty, name } => vec![],

        Expr::If {
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

            cons.extend(collect(*test.clone()));
            cons.extend(collect(*then_branch.clone()));
            cons.extend(collect(*else_branch.clone()));

            cons
        }

        Expr::Lambda { ty, args, body } => {
            let mut cons = vec![Constraint(
                ty.clone(),
                Type::Fn(
                    args.iter().map(|(_, t)| t).cloned().collect(),
                    box body.ty(),
                ),
            )];

            cons.extend(collect(*body));

            cons
        }

        Expr::App { ty, func, args } => {
            let mut cons = vec![Constraint(
                func.ty(),
                Type::Fn(args.iter().map(|arg| arg.ty()).collect(), box ty),
            )];

            for arg in args {
                cons.extend(collect(arg));
            }

            cons.extend(collect(*func));

            cons
        }

        Expr::Let { ty, bindings, body } => {
            let mut cons = vec![Constraint(ty, body.ty())];

            cons.extend(
                bindings
                    .iter()
                    .map(|(_name, t, val)| Constraint(t.clone(), val.ty()))
                    .collect::<Constraints>(),
            );

            for (_, _, val) in bindings {
                cons.extend(collect(val));
            }

            cons.extend(collect(*body));

            cons
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ty::TypeVarGen;

    #[test]
    fn constrain_int() {
        let mut gen = TypeVarGen::new();

        let t1 = gen.fresh();
        let expr = Expr::Lit {
            ty: t1.clone(),
            val: Lit::Int(1),
        };
        assert_eq!(collect(expr), vec![Constraint(t1, Type::Int)]);
    }

    #[test]
    fn constrain_bool() {
        let mut gen = TypeVarGen::new();

        let t1 = gen.fresh();
        let expr = Expr::Lit {
            ty: t1.clone(),
            val: Lit::Bool(true),
        };
        assert_eq!(collect(expr), vec![Constraint(t1, Type::Bool)]);
    }

    #[test]
    fn constrain_float() {
        let mut gen = TypeVarGen::new();

        let t1 = gen.fresh();
        let expr = Expr::Lit {
            ty: t1.clone(),
            val: Lit::Float(1.23),
        };
        assert_eq!(collect(expr), vec![Constraint(t1, Type::Float)]);
    }

    #[test]
    fn constrain_lambda() {
        let mut gen = TypeVarGen::new();

        let t1 = gen.fresh();
        let t2 = gen.fresh();
        let t3 = gen.fresh();

        let expr = Expr::Lambda {
            ty: t1.clone(),
            args: vec![("param".into(), t2.clone())],
            body: box Expr::Var {
                ty: t3.clone(),
                name: "param".into(),
            },
        };
        assert_eq!(
            collect(expr),
            vec![Constraint(t1, Type::Fn(vec![t2.clone()], box t3))]
        );
    }

    #[test]
    fn constrain_var() {
        let mut gen = TypeVarGen::new();

        let t1 = gen.fresh();

        let expr = Expr::Var {
            ty: t1.clone(),
            name: "name".into(),
        };

        assert_eq!(collect(expr), vec![]);
    }

    #[test]
    fn constrain_app() {
        let mut gen = TypeVarGen::new();

        let t1 = gen.fresh();
        let t2 = gen.fresh();
        let t3 = gen.fresh();

        let expr = Expr::App {
            ty: t1.clone(),
            func: box Expr::Var {
                ty: t2.clone(),
                name: "fn".into(),
            },
            args: vec![Expr::Var {
                ty: t3.clone(),
                name: "arg".into(),
            }],
        };

        assert_eq!(
            collect(expr),
            vec![Constraint(t2, Type::Fn(vec![t3], box t1))]
        );
    }

    #[test]
    fn constrain_let() {
        let mut gen = TypeVarGen::new();

        let t1 = gen.fresh();
        let t2 = gen.fresh();
        let t3 = gen.fresh();
        let t4 = gen.fresh();

        let expr = Expr::Let {
            ty: t1.clone(),
            bindings: vec![(
                "name".into(),
                t2.clone(),
                Expr::Var {
                    ty: t3.clone(),
                    name: "value".into(),
                },
            )],
            body: box Expr::Var {
                ty: t4.clone(),
                name: "body".into(),
            },
        };

        assert_eq!(collect(expr), vec![Constraint(t1, t4), Constraint(t2, t3)]);
    }

    #[test]
    fn constrain_identity() {
        let mut gen = TypeVarGen::new();

        let t1 = gen.fresh();
        let t2 = gen.fresh();
        let t3 = gen.fresh();

        let expr = Expr::Lambda {
            ty: t1.clone(),
            args: vec![("a".into(), t2.clone())],
            body: box Expr::Var {
                ty: t3.clone(),
                name: "a".into(),
            },
        };

        assert_eq!(
            collect(expr),
            vec![Constraint(t1, Type::Fn(vec![t2], box t3))]
        );
    }

    #[test]
    fn constrain_const() {
        let mut gen = TypeVarGen::new();
        let t1 = gen.fresh();
        let t2 = gen.fresh();
        let t3 = gen.fresh();
        let t4 = gen.fresh();
        let t5 = gen.fresh();

        let expr = Expr::Lambda {
            ty: t1.clone(),
            args: vec![("a".into(), t2.clone())],
            body: box Expr::Lambda {
                ty: t3.clone(),
                args: vec![("b".into(), t4.clone())],
                body: box Expr::Var {
                    ty: t5.clone(),
                    name: "a".into(),
                },
            },
        };

        assert_eq!(
            collect(expr),
            vec![
                Constraint(t1, Type::Fn(vec![t2], box t3.clone())),
                Constraint(t3, Type::Fn(vec![t4], box t5))
            ]
        )
    }

    #[test]
    fn constrain_compose() {
        let mut gen = TypeVarGen::new();
        let t1 = gen.fresh();
        let t2 = gen.fresh();
        let t3 = gen.fresh();
        let t4 = gen.fresh();
        let t5 = gen.fresh();
        let t6 = gen.fresh();
        let t7 = gen.fresh();
        let t8 = gen.fresh();
        let t9 = gen.fresh();
        let t10 = gen.fresh();
        let t11 = gen.fresh();

        let expr = Expr::Lambda {
            ty: t1.clone(),
            args: vec![("f".into(), t2.clone())],
            body: box Expr::Lambda {
                ty: t3.clone(),
                args: vec![("g".into(), t4.clone())],
                body: box Expr::Lambda {
                    ty: t5.clone(),
                    args: vec![("x".into(), t6.clone())],
                    body: box Expr::App {
                        ty: t7.clone(),
                        func: box Expr::Var {
                            ty: t8.clone(),
                            name: "f".into(),
                        },
                        args: vec![Expr::App {
                            ty: t9.clone(),
                            func: box Expr::Var {
                                ty: t10.clone(),
                                name: "g".into(),
                            },
                            args: vec![Expr::Var {
                                ty: t11.clone(),
                                name: "x".into(),
                            }],
                        }],
                    },
                },
            },
        };

        assert_eq!(
            collect(expr),
            vec![
                Constraint(t1.clone(), Type::Fn(vec![t2.clone()], box t3.clone())),
                Constraint(t3.clone(), Type::Fn(vec![t4.clone()], box t5.clone())),
                Constraint(t5.clone(), Type::Fn(vec![t6.clone()], box t7.clone())),
                Constraint(t8.clone(), Type::Fn(vec![t9.clone()], box t7.clone())),
                Constraint(t10.clone(), Type::Fn(vec![t11.clone()], box t9.clone())),
            ]
        );
    }
}
