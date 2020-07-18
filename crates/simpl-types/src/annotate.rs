//! Takes an `ast::Expr` and annotates each Expr with a fresh `Type`

use crate::{
    ty::{TypeEnv, TypeVarGen},
    typed_ast::Expr,
};
use simpl_syntax::ast;

pub fn annotate(expr: ast::Expr) -> Result<Expr, String> {
    annotate_with_gen(expr, &mut TypeVarGen::new())
}

pub fn annotate_with_gen(expr: ast::Expr, gen: &mut TypeVarGen) -> Result<Expr, String> {
    ann(expr, &mut TypeEnv::new(), gen)
}

fn ann(expr: ast::Expr, tenv: &mut TypeEnv, gen: &mut TypeVarGen) -> Result<Expr, String> {
    let ty = gen.fresh();
    let expr = match expr {
        ast::Expr::Lit { val } => Expr::Lit { val, ty },
        ast::Expr::Var { name } => match tenv.get(&name) {
            None => return Err(format!("Unbound variable: {}", name)),
            Some(ty) => Expr::Var {
                ty: ty.clone(),
                name,
            },
        },
        ast::Expr::If {
            box test,
            box then_branch,
            box else_branch,
        } => Expr::If {
            ty,
            test: box ann(test, tenv, gen)?,
            then_branch: box ann(then_branch, tenv, gen)?,
            else_branch: box ann(else_branch, tenv, gen)?,
        },
        ast::Expr::Let { bindings, box body } => Expr::Let {
            ty,
            bindings: bindings
                .into_iter()
                .map(|(var, val)| ann(val, tenv, gen).map(|val| (var, val)))
                .collect::<Result<Vec<_>, _>>()?,
            body: box ann(body, tenv, gen)?,
        },
        ast::Expr::Lambda { args, box body } => {
            let mut extended_tenv = tenv.clone();
            let mut new_args = vec![];
            for arg in args {
                let arg_ty = gen.fresh();
                extended_tenv.insert(arg.clone(), arg_ty.clone());
                new_args.push((arg, arg_ty));
            }

            Expr::Lambda {
                ty,
                args: new_args,
                body: box ann(body, &mut extended_tenv, gen)?,
            }
        }

        ast::Expr::App { box func, args } => Expr::App {
            ty,
            func: box ann(func, tenv, gen)?,
            args: args
                .into_iter()
                .map(|arg| ann(arg, tenv, gen))
                .collect::<Result<Vec<_>, _>>()?,
        },
    };
    Ok(expr)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ty::Type;
    use insta::assert_debug_snapshot;
    use simpl_syntax::{ast::Lit, parse};

    macro_rules! vec_clone {
	[$($e:expr),*] => {
		vec![$($e.clone()),*]
	};
}

    fn parse_and_annotate(src: &str) -> Expr {
        let ast = parse(src).unwrap();
        annotate(ast).unwrap()
    }

    #[track_caller]
    fn test_annotate(src: &str) {
        let ast = parse(src).unwrap();
        let typed_ast = annotate(ast);
        assert_debug_snapshot!(typed_ast);
    }

    #[test]
    fn annotate_identity() {
        let ast = parse_and_annotate(r"\(x) -> x;");
        dbg!(&ast);
        match ast {
            Expr::Lambda {
                ty: t1,
                args,
                body:
                    box Expr::Var {
                        ty: t3,
                        name: var_name,
                    },
            } => {
                assert_eq!(args.len(), 1);
                let (arg_name, t2) = &args[0];
                assert_eq!(arg_name, "x");

                assert_ne!(t1, t2.clone());
                assert_ne!(t1, t3.clone());
                assert_eq!(t2.clone(), t3);
            }
            _ => panic!("Match failed: {:#?}", ast),
        }
    }

    #[test]
    fn annotate_const() {
        let ast = parse_and_annotate(r"\(a) -> \(b) -> a;;");
        dbg!(&ast);
        match ast {
            Expr::Lambda {
                ty: t1,
                args: args1,
                body:
                    box Expr::Lambda {
                        ty: t3,
                        args: args2,
                        body: box Expr::Var { ty: t5, name },
                    },
            } => {
                assert_eq!(args1.len(), 1);
                assert_eq!(args2.len(), 1);
                let (arg1, t2) = &args1[0];
                let (arg2, t4) = &args2[0];
                assert_eq!(arg1, "a");
                assert_eq!(arg2, "b");
                assert_eq!(name, "a");

                let t1: Type = t1.clone();
                let t2: Type = t2.clone();
                let t3: Type = t3;
                let t4: Type = t4.clone();
                let t5: Type = t5;

                assert!(!vec_clone![t2, t3, t4, t5].contains(&t1));
                assert!(!vec_clone![t1, t3, t4].contains(&t2));
                assert!(!vec_clone![t1, t2, t4, t5].contains(&t3));
                assert!(!vec_clone![t1, t2, t3, t5].contains(&t4));
                assert!(!vec_clone![t1, t3, t4].contains(&t5));
                assert_eq!(t2, t5);
            }
            _ => panic!("Match failed: {:#?}", ast),
        }
    }

    #[test]
    fn annotate_compose() {
        fn f(
            t1: Type,
            t2: Type,
            t3: Type,
            t4: Type,
            t5: Type,
            t6: Type,
            t7: Type,
            t8: Type,
            t9: Type,
            t10: Type,
            t11: Type,
        ) {
            assert!(!vec_clone![t2, t3, t4, t5, t6, t7, t8, t9, t10, t11].contains(&t1));
            assert!(!vec_clone![t1, t3, t4, t5, t6, t7, t9, t10, t11].contains(&t2));
            assert!(!vec_clone![t1, t2, t4, t5, t6, t7, t8, t9, t10, t11].contains(&t3));
            assert!(!vec_clone![t1, t2, t3, t5, t6, t7, t8, t9, t11].contains(&t4));
            assert!(!vec_clone![t1, t2, t3, t4, t6, t7, t8, t9, t10, t11].contains(&t5));
            assert!(!vec_clone![t1, t2, t3, t4, t5, t7, t8, t9, t10].contains(&t6));
            assert!(!vec_clone![t1, t2, t3, t4, t5, t6, t8, t9, t10, t11].contains(&t7));
            assert!(!vec_clone![t1, t3, t4, t5, t6, t7, t9, t10, t11].contains(&t8));
            assert!(!vec_clone![t1, t2, t3, t4, t5, t6, t7, t8, t10, t11].contains(&t9));
            assert!(!vec_clone![t1, t2, t3, t5, t6, t7, t8, t9, t11].contains(&t10));
            assert!(!vec_clone![t1, t2, t3, t4, t5, t7, t8, t9, t10].contains(&t11));
            assert_eq!(t2.clone(), t8.clone());
            assert_eq!(t4.clone(), t10.clone());
            assert_eq!(t6.clone(), t11.clone());
        }

        let ast = parse_and_annotate(r"\(f) -> \(g) -> \(x) -> f(g(x));;;");
        dbg!(&ast);
        match ast {
            Expr::Lambda {
                ty: t1,
                args: args1,
                body:
                    box Expr::Lambda {
                        ty: t3,
                        args: args2,
                        body:
                            box Expr::Lambda {
                                ty: t5,
                                args: args3,
                                body:
                                    box Expr::App {
                                        ty: t7,
                                        func: box Expr::Var { ty: t8, name: var1 },
                                        args: args4,
                                    },
                            },
                    },
            } => {
                assert_eq!(args1.len(), 1);
                let (name1, t2) = &args1[0];
                assert_eq!(name1, "f");

                assert_eq!(args2.len(), 1);
                let (name2, t4) = &args2[0];
                assert_eq!(name2, "g");

                assert_eq!(args3.len(), 1);
                let (name3, t6) = &args3[0];
                assert_eq!(name3, "x");

                assert_eq!(var1, "f");

                assert_eq!(args4.len(), 1);
                let arg = &args4[0];
                match arg {
                    Expr::App {
                        ty: t9,
                        func:
                            box Expr::Var {
                                ty: t10,
                                name: var2,
                            },
                        args: args5,
                    } => {
                        assert_eq!(var2, "g");

                        assert_eq!(args5.len(), 1);
                        let arg = &args5[0];

                        match arg {
                            Expr::Var {
                                ty: t11,
                                name: var3,
                            } => {
                                assert_eq!(var3, "x");

                                let t1: Type = t1.clone();
                                let t2: Type = t2.clone();
                                let t3: Type = t3.clone();
                                let t4: Type = t4.clone();
                                let t5: Type = t5.clone();
                                let t6: Type = t6.clone();
                                let t7: Type = t7.clone();
                                let t8: Type = t8.clone();
                                let t9: Type = t9.clone();
                                let t10: Type = t10.clone();
                                let t11: Type = t11.clone();

                                f(t1, t2, t3, t4, t5, t6, t7, t8, t9, t10, t11);
                            }
                            _ => panic!("Match failed: {:#?}", arg),
                        }
                    }
                    _ => panic!("Match failed: {:#?}", arg),
                }
            }
            _ => panic!("Match failed: {:#?}", ast),
        }
    }

    #[test]
    fn annotate_pred() {
        fn f(t1: Type, t2: Type, t3: Type, t4: Type, t5: Type, t6: Type, t7: Type, t8: Type) {
            assert!(!vec_clone!(t2, t3, t4, t5, t6, t7, t8).contains(&t1));
            assert!(!vec_clone!(t1, t3, t4, t6, t7, t8).contains(&t2));
            assert!(!vec_clone!(t1, t2, t4, t5, t6, t7, t8).contains(&t3));
            assert!(!vec_clone!(t1, t2, t3, t5, t6, t7, t8).contains(&t4));
            assert!(!vec_clone!(t1, t3, t4, t6, t7, t8).contains(&t5));
            assert!(!vec_clone!(t1, t2, t3, t4, t5, t7, t8).contains(&t6));
            assert!(!vec_clone!(t1, t2, t3, t4, t5, t6, t8).contains(&t7));
            assert!(!vec_clone!(t1, t2, t3, t4, t5, t6, t7).contains(&t8));
            assert_eq!(t2, t5);
        }

        let ast = parse_and_annotate(r"\(pred) -> if pred(1) then 2 else 3;;");
        dbg!(&ast);
        match ast {
            Expr::Lambda {
                ty: t1,
                args: args1,
                body:
                    box Expr::If {
                        ty: t3,
                        test:
                            box Expr::App {
                                ty: t4,
                                func: box Expr::Var { ty: t5, name: var1 },
                                args: args2,
                            },
                        then_branch:
                            box Expr::Lit {
                                ty: t7,
                                val: Lit::Int(2),
                            },
                        else_branch:
                            box Expr::Lit {
                                ty: t8,
                                val: Lit::Int(3),
                            },
                    },
            } => {
                assert_eq!(args1.len(), 1);
                let (name1, t2) = &args1[0];
                assert_eq!(name1, "pred");

                assert_eq!(var1, "pred");

                assert_eq!(args2.len(), 1);
                let arg = &args2[0];

                match arg {
                    Expr::Lit {
                        ty: t6,
                        val: Lit::Int(1),
                    } => {
                        let t1: Type = t1.clone();
                        let t2: Type = t2.clone();
                        let t3: Type = t3.clone();
                        let t4: Type = t4.clone();
                        let t5: Type = t5.clone();
                        let t6: Type = t6.clone();
                        let t7: Type = t7.clone();
                        let t8: Type = t8.clone();

                        f(t1, t2, t3, t4, t5, t6, t7, t8);
                    }
                    _ => panic!("Match failed: {:#?}", arg),
                }
            }
            _ => panic!("Match failed: {:#?}", ast),
        }
    }
}
