#![feature(box_syntax)]
#![feature(box_patterns)]

mod annotate;
mod constraint;
mod subst;
mod ty;
pub mod typed_ast;
mod unify;

use crate::{ty::Type, typed_ast::Expr};

pub fn type_of(expr: Expr) -> Type {
    let cons = constraint::collect(expr.clone());
    let subst = unify::unify(cons);
    subst.apply_ty(&expr.ty())
}

pub fn parse_and_annotate(src: &str) -> Expr {
    // TODO: Return a Result instead of unwraping

    let ast = simpl_syntax::parse(src).unwrap();
    annotate::annotate(ast).unwrap()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn infer_identity_fn() {
        let expr = parse_and_annotate(r"\(a) -> a;");
        let ty = type_of(expr);
        assert_eq!(ty, Type::Fn(vec![Type::Var(2)], box Type::Var(2)))
    }

    #[test]
    fn infer_const_fn() {
        let expr = parse_and_annotate(r"\(a) -> \(b) -> a;;");
        let ty = type_of(expr);
        assert_eq!(
            ty,
            Type::Fn(
                vec![Type::Var(2)],
                box Type::Fn(vec![Type::Var(4)], box Type::Var(2))
            )
        )
    }

    #[test]
    fn infer_compose_fn() {
        let expr = parse_and_annotate(r"\(f) -> \(g) -> \(x) -> f(g(x));;;");
        let ty = type_of(expr);

        let t_1 = Type::Var(9);
        let t_2 = Type::Var(7);
        let t_3 = Type::Var(6);

        assert_eq!(
            ty,
            Type::Fn(
                vec![Type::Fn(vec![t_1.clone()], box t_2.clone())],
                box Type::Fn(
                    vec![Type::Fn(vec![t_3.clone()], box t_1)],
                    box Type::Fn(vec![t_3], box t_2)
                )
            )
        )
    }

    #[test]
    fn infer_pred_fn() {
        let expr = parse_and_annotate(r"\(pred) -> if pred(1) then 2 else 3;;");
        let ty = type_of(expr);
        assert_eq!(
            ty,
            Type::Fn(
                vec![Type::Fn(vec![Type::Int], box Type::Bool)],
                box Type::Int
            )
        )
    }

    #[test]
    fn infer_inc_fn() {
        let expr = parse_and_annotate(
            r"
let inc = \(x) -> add(x, 1);
in inc(inc(42));
",
        );
        let ty = type_of(expr);
        assert_eq!(ty, Type::Int)
    }

    #[test]
    fn infer_bottom_type() {
        let expr = parse_and_annotate(
            r"
let bot = \() -> bot();
in bot;
",
        );
        let ty = type_of(expr);
        assert_eq!(ty, Type::Fn(vec![], box Type::Var(4)));

        let expr = parse_and_annotate(
            r"
let bot = \() -> bot();
in if true then 1 else bot();;
",
        );
        let ty = type_of(expr);
        assert_eq!(ty, Type::Int)
    }

    #[test]
    fn letrec() {
        let expr = parse_and_annotate(
            r"
let fact = \(x) -> if is_zero(x) then 1 else mul(x, fact(sub(x, 1)));;
in fact(5);
",
        );
        let ty = type_of(expr);
        assert_eq!(ty, Type::Int);

        let expr = parse_and_annotate(
            r"
let is_odd  = \(x) -> if is_zero(x) then true else is_odd(sub(x, 1));;,
    is_even = \(x) -> if is_zero(x) then false else is_even(sub(x, 1));;
in is_even(4);
",
        );
        let ty = type_of(expr);
        assert_eq!(ty, Type::Bool)
    }

    #[test]
    fn let_polymorphism() {
        let expr = parse_and_annotate(
            r"
let first = \(x, y) -> x;,
    id    = \(x) -> x;
in first(id(0), id(false));
",
        );

        let ty = type_of(expr);
        assert_eq!(ty, Type::Int);

        // TODO: does Rust support let-polymorphism? The following doesnt
        // compile let id = |x| x;
        // let pair = (id(0), id(false));
        // dbg!(pair);
    }
}
