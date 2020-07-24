use crate::hir::Expr;
use maplit::hashset as hset;
use simple_symbol::Symbol;
use std::collections::HashSet;

pub fn free_vars(expr: &Expr) -> HashSet<Symbol> {
    match expr {
        Expr::Lit { .. } => HashSet::new(),
        Expr::Var { name, .. } => hset![*name],
        Expr::If {
            test,
            then_branch,
            else_branch,
            ..
        } => &(&free_vars(test) | &free_vars(then_branch)) | &free_vars(else_branch),
        Expr::Let { binding, body, .. } => {
            &free_vars(&*binding.val) | &(&free_vars(body) - &hset!(binding.name))
        }
        Expr::Letrec { bindings, body, .. } => {
            &bindings
                .iter()
                .fold(free_vars(body), |acc, b| &acc | &free_vars(&*b.val))
                - &bindings.iter().map(|b| b.name).collect::<HashSet<_>>()
        }
        Expr::Lambda { param, body, .. } => &free_vars(body) - &hset!(param.name),
        Expr::App { func, arg, .. } => &free_vars(func) | &free_vars(arg),
    }
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
        let free = free_vars(&ast);
        assert_eq!(free, expected);
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
