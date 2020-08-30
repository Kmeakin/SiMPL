use crate::hir::{Expr, Type};
use maplit::{hashmap as hmap, hashset as hset};
use simple_symbol::Symbol;
use std::collections::{HashMap, HashSet};

pub type FreeVars = HashMap<Symbol, Type>;

pub fn free_vars(expr: &Expr) -> FreeVars {
    match expr {
        Expr::Lit { .. } => hmap![],
        Expr::Var { name, ty } => hmap![*name => ty.clone()],
        Expr::If {
            test, then, els, ..
        } => hashmap_union(
            hashmap_union(free_vars(test), free_vars(then)),
            free_vars(els),
        ),
        Expr::Let { binding, body, .. } => hashmap_diff(
            hashmap_union(free_vars(&*binding.val), free_vars(body)),
            hmap![binding.name => binding.ty.clone()],
        ),

        Expr::Letrec { bindings, body, .. } => hashmap_diff(
            bindings.iter().fold(free_vars(body), |acc, b| {
                hashmap_union(acc, free_vars(&*b.val))
            }),
            bindings.iter().map(|b| (b.name, b.ty.clone())).collect(),
        ),
        Expr::Lambda { param, body, .. } => {
            hashmap_diff(free_vars(body), hmap![param.name => param.ty.clone()])
        }

        Expr::App { func, arg, .. } => hashmap_union(free_vars(func), free_vars(arg)),
    }
}

fn hashmap_union<K, V>(hm1: HashMap<K, V>, hm2: HashMap<K, V>) -> HashMap<K, V>
where
    K: std::hash::Hash + Eq,
    V: std::hash::Hash + Eq,
{
    let mut ret = HashMap::new();
    ret.extend(hm1);
    ret.extend(hm2);
    ret
}

fn hashmap_diff<K, V>(hm1: HashMap<K, V>, hm2: HashMap<K, V>) -> HashMap<K, V>
where
    K: std::hash::Hash + Eq + Clone + std::fmt::Debug,
    V: std::hash::Hash + Eq + Clone + std::fmt::Debug,
{
    hm1.into_iter()
        .filter(|(k, _)| !hm2.contains_key(&k))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<HashMap<K, V>>()
}

#[cfg(test)]
mod test {
    use super::*;
    use maplit::hashset as hset;
    use simple_symbol::intern;
    use std::str::FromStr;

    #[track_caller]
    fn test_free_vars(src: &str, expected: HashSet<Symbol>) {
        let ast = Expr::from_str(src).unwrap();
        let free_vars = free_vars(&ast);
        let free_vars: HashSet<Symbol> = free_vars.iter().map(|(name, ty)| *name).collect();
        assert_eq!(free_vars, expected);
    }

    #[test]
    fn free_vars_lit() {
        test_free_vars("1", hset![]);
    }

    #[test]
    fn free_vars_if() {
        test_free_vars(
            "if abc then def else ghi",
            hset![intern("abc"), intern("def"), intern("ghi")],
        );
    }

    #[test]
    fn free_vars_let() {
        test_free_vars("let x = 5 in x y", hset![intern("y")]);
    }

    #[test]
    fn free_vars_letrec() {
        // TODO
    }

    #[test]
    fn free_vars_lambda() {
        test_free_vars(r"\x -> x", hset![]);
        test_free_vars(r"\x -> y", hset![intern("y")]);
    }

    #[test]
    fn free_vars_app() {
        test_free_vars("f x", hset![intern("f"), intern("x")]);
    }
}
