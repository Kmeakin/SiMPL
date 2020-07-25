//! `Exp` -> `Expr` pass
//! Alpha-renames exprs, so that no variable masks another in its enclosing
//! scope

use super::gensym::Gensym;
use crate::hir::{Expr, LetBinding, Param};
use simple_symbol::{intern, Symbol};
use std::{collections::HashMap, default::Default};

impl Expr {
    fn is_alpha_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Lit { val: x, .. }, Self::Lit { val: y, .. }) => x == y,
            (Self::Var { name: x, .. }, Self::Var { name: y, .. }) => x == y,
            (
                Self::If {
                    test: test1,
                    then_branch: then1,
                    else_branch: else1,
                    ..
                },
                Self::If {
                    test: test2,
                    then_branch: then2,
                    else_branch: else2,
                    ..
                },
            ) => test1.is_alpha_eq(test2) && then1.is_alpha_eq(then2) && else1.is_alpha_eq(else2),
            (Self::Let { .. }, Self::Let { .. }) => todo!(),
            (Self::Letrec { .. }, Self::Letrec { .. }) => todo!(),
            (Self::Lambda { .. }, Self::Lambda { .. }) => todo!(),
            (Self::App { .. }, Self::App { .. }) => todo!(),
            _ => false,
        }
    }
}

fn lookup(env: &HashMap<Symbol, Symbol>, name: Symbol) -> Symbol {
    *env.get(&name).unwrap_or(&name)
}

fn rename(expr: Expr) -> Expr {
    rename_inner(expr, &HashMap::default(), &mut Gensym::new("$"))
}

fn rename_var(name: Symbol, gen: &mut Gensym) -> Symbol {
    let gensym = gen.next();
    intern(format!("{}{}", name, gensym))
}

fn rename_inner(expr: Expr, env: &HashMap<Symbol, Symbol>, gen: &mut Gensym) -> Expr {
    match expr {
        Expr::Lit { .. } => expr,
        Expr::Var { ty, name } => Expr::Var {
            name: lookup(env, name),
            ty,
        },
        Expr::If {
            ty,
            test,
            then_branch,
            else_branch,
        } => Expr::If {
            ty,
            test: box rename_inner(*test, env, gen),
            then_branch: box rename_inner(*then_branch, env, gen),
            else_branch: box rename_inner(*else_branch, env, gen),
        },
        Expr::Let { ty, binding, body } => {
            let new_name = rename_var(binding.name, gen);
            let mut ext_env = env.clone();
            ext_env.insert(binding.name, new_name);
            Expr::Let {
                ty,
                binding: LetBinding {
                    name: new_name,
                    val: box rename_inner(*binding.val, env, gen),
                    ..binding
                },
                body: box rename_inner(*body, &ext_env, gen),
            }
        }
        Expr::Letrec { ty, bindings, body } => {
            let mut ext_env = env.clone();
            let mut new_names = vec![];
            let mut new_bindings = vec![];
            for binding in &bindings {
                let new_name = rename_var(binding.name, gen);
                new_names.push(new_name);
                ext_env.insert(binding.name, new_name);
            }

            for (binding, new_name) in bindings.iter().zip(new_names) {
                new_bindings.push(LetBinding {
                    ty: binding.ty.clone(),
                    name: new_name,
                    val: box rename_inner(*binding.val.clone(), &ext_env.clone(), gen),
                })
            }

            Expr::Letrec {
                ty,
                bindings: new_bindings,
                body: box rename_inner(*body, &ext_env, gen),
            }
        }
        Expr::Lambda { ty, param, body } => {
            let new_name = rename_var(param.name, gen);
            let mut ext_env = env.clone();
            ext_env.insert(param.name, new_name);
            Expr::Lambda {
                ty,
                param: Param {
                    name: new_name,
                    ty: param.ty,
                },
                body: box rename_inner(*body, &ext_env, gen),
            }
        }
        Expr::App { ty, func, arg } => Expr::App {
            ty,
            func: box rename_inner(*func, env, gen),
            arg: box rename_inner(*arg, env, gen),
        },
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use insta::assert_snapshot;
    use std::str::FromStr;

    #[track_caller]
    fn test_rename(src: &str) {
        let expr = Expr::from_str(src).unwrap();
        let renamed = rename(expr);
        assert_snapshot!(renamed.pretty());
    }

    #[test]
    fn nested_let() {
        test_rename(r"let x = 5 in let x = false in x");
    }

    #[test]
    fn letrec() {
        test_rename(r"letrec f = \x -> f x in f");
        test_rename(
            r"
letrec f = \x -> f x,
       g = \y -> g y
 in g f",
        );
    }

    #[test]
    fn lambda() {
        test_rename(r"\x -> \x -> x");
    }
}
