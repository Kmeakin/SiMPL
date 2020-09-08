use crate::{
    hir::{Binop, Expr},
    types::ty::{Type, TypeEnv},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraint(pub(crate) Type, pub(crate) Type);
pub type Constraints = Vec<Constraint>;

/// Collect constraints, and check for unbound variables
pub fn collect(expr: Expr) -> Constraints {
    let tenv = TypeEnv::default();
    collect_inner(expr, &tenv)
}

fn collect_inner(expr: Expr, tenv: &TypeEnv) -> Constraints {
    match expr {
        Expr::Lit { ty, val } => vec![Constraint(ty, val.ty())],
        Expr::Var { ty, name } => match tenv.get(name) {
            Some(ty2) => vec![Constraint(ty, ty2.clone())],
            None => panic!("Unbound variable: {}", name),
        },
        Expr::Binop { ty, lhs, rhs, op } => {
            use Binop::*;
            use Type::*;

            let (lhs_ty, rhs_ty, out_ty) = match op {
                IntAdd | IntSub | IntMul | IntDiv => (Int, Int, Int),
                IntLt | IntLeq | IntGt | IntGeq => (Int, Int, Bool),

                FloatAdd | FloatSub | FloatMul | FloatDiv => (Float, Float, Float),
                FloatLt | FloatLeq | FloatGt | FloatGeq => (Float, Float, Bool),

                Eq | Neq => (rhs.ty(), lhs.ty(), Bool),
            };

            let mut cons = vec![
                Constraint(lhs.ty(), lhs_ty),
                Constraint(rhs.ty(), rhs_ty),
                Constraint(ty, out_ty),
            ];

            cons.extend(collect_inner(*lhs, tenv));
            cons.extend(collect_inner(*rhs, tenv));
            cons
        }
        Expr::If {
            ty,
            test,
            then,
            els,
        } => {
            let mut cons = vec![
                Constraint(test.ty(), Type::Bool),
                Constraint(then.ty(), ty.clone()),
                Constraint(els.ty(), ty),
            ];
            cons.extend(collect_inner(*test, tenv));
            cons.extend(collect_inner(*then, tenv));
            cons.extend(collect_inner(*els, tenv));
            cons
        }

        Expr::Let { ty, binding, body } => {
            let mut ext_tenv = tenv.clone();
            ext_tenv.insert(binding.name, binding.ty.clone());

            let mut cons = vec![
                Constraint(ty, body.ty()),
                Constraint(binding.ty, binding.val.ty()),
            ];

            if let Some(ty) = binding.ann {
                cons.push(Constraint(ty, binding.val.ty()));
            }

            cons.extend(collect_inner(*binding.val, tenv));
            cons.extend(collect_inner(*body, &ext_tenv));
            cons
        }
        Expr::Letrec { ty, bindings, body } => {
            assert!(!bindings.is_empty());

            let mut ext_tenv = tenv.clone();
            let mut cons = vec![Constraint(ty, body.ty())];

            for binding in &bindings {
                cons.push(Constraint(binding.ty.clone(), binding.val.ty()));
                ext_tenv.insert(binding.name, binding.ty.clone())
            }

            for binding in bindings {
                cons.extend(collect_inner(*binding.val.clone(), &ext_tenv));
            }

            cons.extend(collect_inner(*body, &ext_tenv));
            cons
        }
        Expr::Lambda { ty, param, body } => {
            let mut ext_tenv = tenv.clone();
            ext_tenv.insert(param.name, param.ty.clone());

            let mut cons = vec![Constraint(
                ty,
                Type::Fn(box param.ty.clone(), box body.ty()),
            )];

            if let Some(ty) = param.ann {
                cons.push(Constraint(ty, param.ty))
            }

            cons.extend(collect_inner(*body, &ext_tenv));
            cons
        }
        Expr::App { ty, func, arg } => {
            let mut cons = vec![Constraint(func.ty(), Type::Fn(box arg.ty(), box ty))];
            cons.extend(collect_inner(*func, tenv));
            cons.extend(collect_inner(*arg, tenv));
            cons
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        hir::{LetBinding, Lit, Param},
        ty,
        types::ty::TypeVarGen,
    };
    use simple_symbol::intern;

    #[test]
    fn constrain_int() {
        let mut gen = TypeVarGen::new();

        let t1 = gen.next();
        let expr = Expr::Lit {
            ty: t1.clone(),
            val: Lit::Int(1),
        };
        assert_eq!(collect(expr), vec![Constraint(t1, Type::Int)]);
    }

    #[test]
    fn constrain_bool() {
        let mut gen = TypeVarGen::new();

        let t1 = gen.next();
        let expr = Expr::Lit {
            ty: t1.clone(),
            val: Lit::Bool(true),
        };
        assert_eq!(collect(expr), vec![Constraint(t1, Type::Bool)]);
    }

    #[test]
    fn constrain_float() {
        let mut gen = TypeVarGen::new();

        let t1 = gen.next();
        let expr = Expr::Lit {
            ty: t1.clone(),
            val: Lit::Float(1.23),
        };
        assert_eq!(collect(expr), vec![Constraint(t1, Type::Float)]);
    }

    #[test]
    fn constrain_lambda() {
        let mut gen = TypeVarGen::new();

        let t0 = gen.next();
        let t1 = gen.next();
        let t2 = gen.next();

        let expr = Expr::Lambda {
            ty: t0.clone(),
            param: Param {
                ty: t1.clone(),
                name: intern("param").into(),
                ann: None,
            },
            body: box Expr::Var {
                ty: t2.clone(),
                name: intern("param").into(),
            },
        };
        assert_eq!(
            collect(expr),
            vec![Constraint(t0, ty![{1} => {2}]), Constraint(t2, t1)],
        );
    }

    #[test]
    fn constrain_var() {
        let mut gen = TypeVarGen::new();

        let t0 = gen.next();

        let expr = Expr::Var {
            ty: t0.clone(),
            name: intern("not").into(),
        };

        assert_eq!(collect(expr), vec![Constraint(t0, ty![Bool => Bool])]);
    }

    #[test]
    fn constrain_app() {
        let mut gen = TypeVarGen::new();

        let t0 = gen.next();
        let t1 = gen.next();
        let t2 = gen.next();

        let expr = Expr::App {
            ty: t0.clone(),
            func: box Expr::Var {
                ty: t1.clone(),
                name: intern("add").into(),
            },
            arg: box Expr::Lit {
                ty: t2.clone(),
                val: Lit::Int(0),
            },
        };

        assert_eq!(
            collect(expr),
            vec![
                Constraint(t1.clone(), ty![{2} => {0}]),
                Constraint(t1.clone(), ty![Int => Int => Int]),
                Constraint(t2.clone(), ty![Int])
            ]
        );
    }

    #[test]
    fn constrain_let() {
        let mut gen = TypeVarGen::new();

        let t0 = gen.next();
        let t1 = gen.next();
        let t2 = gen.next();
        let t3 = gen.next();

        let expr = Expr::Let {
            ty: t0.clone(),
            binding: LetBinding {
                ty: t1.clone(),
                name: intern("name").into(),
                val: box Expr::Lit {
                    ty: t2.clone(),
                    val: Lit::Bool(false),
                },
                ann: None,
            },
            body: box Expr::Var {
                ty: t3.clone(),
                name: intern("name").into(),
            },
        };

        assert_eq!(
            collect(expr),
            vec![
                Constraint(t0, t3.clone()),
                Constraint(t1.clone(), t2.clone()),
                Constraint(t2, Type::Bool),
                Constraint(t3, t1),
            ]
        );
    }

    #[test]
    fn constrain_identity() {
        let mut gen = TypeVarGen::new();

        let t0 = gen.next();
        let t1 = gen.next();
        let t2 = gen.next();

        let expr = Expr::Lambda {
            ty: t0.clone(),
            param: Param {
                ty: t1.clone(),
                name: intern("a").into(),
                ann: None,
            },
            body: box Expr::Var {
                ty: t2.clone(),
                name: intern("a").into(),
            },
        };

        assert_eq!(
            collect(expr),
            vec![Constraint(t0, ty![{1} => {2}]), Constraint(t2, t1)]
        );
    }

    #[test]
    fn constrain_const() {
        let mut gen = TypeVarGen::new();
        let t0 = gen.next();
        let t1 = gen.next();
        let t2 = gen.next();
        let t3 = gen.next();
        let t4 = gen.next();

        let expr = Expr::Lambda {
            ty: t0.clone(),
            param: Param {
                ty: t1.clone(),
                name: intern("a").into(),
                ann: None,
            },
            body: box Expr::Lambda {
                ty: t2.clone(),
                param: Param {
                    ty: t3.clone(),
                    name: intern("b").into(),
                    ann: None,
                },
                body: box Expr::Var {
                    ty: t4.clone(),
                    name: intern("a").into(),
                },
            },
        };

        assert_eq!(
            collect(expr),
            vec![
                Constraint(t0, ty![{1} => {2}]),
                Constraint(t2, ty![{3} => {4}]),
                Constraint(t4, t1)
            ]
        )
    }

    #[test]
    fn constrain_compose() {
        let mut gen = TypeVarGen::new();
        let t0 = gen.next();
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

        let expr = Expr::Lambda {
            ty: t0.clone(),
            param: Param {
                ty: t1.clone(),
                name: intern("f").into(),
                ann: None,
            },
            body: box Expr::Lambda {
                ty: t2.clone(),
                param: Param {
                    ty: t3.clone(),
                    name: intern("g").into(),
                    ann: None,
                },
                body: box Expr::Lambda {
                    ty: t4.clone(),
                    param: Param {
                        ty: t5.clone(),
                        name: intern("x").into(),
                        ann: None,
                    },
                    body: box Expr::App {
                        ty: t6.clone(),
                        func: box Expr::Var {
                            ty: t7.clone(),
                            name: intern("f").into(),
                        },
                        arg: box Expr::App {
                            ty: t8.clone(),
                            func: box Expr::Var {
                                ty: t9.clone(),
                                name: intern("g").into(),
                            },
                            arg: box Expr::Var {
                                ty: t10.clone(),
                                name: intern("x").into(),
                            },
                        },
                    },
                },
            },
        };

        assert_eq!(
            collect(expr),
            vec![
                Constraint(t0.clone(), Type::Fn(box t1.clone(), box t2.clone())),
                Constraint(t2.clone(), Type::Fn(box t3.clone(), box t4.clone())),
                Constraint(t4.clone(), Type::Fn(box t5.clone(), box t6.clone())),
                Constraint(t7.clone(), Type::Fn(box t8.clone(), box t6.clone())),
                Constraint(t7.clone(), t1.clone()),
                Constraint(t9.clone(), Type::Fn(box t10.clone(), box t8.clone())),
                Constraint(t9.clone(), t3.clone()),
                Constraint(t10.clone(), t5.clone()),
            ]
        );
    }
}
