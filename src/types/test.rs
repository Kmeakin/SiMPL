use crate::{
    ty,
    types::{
        ty::{Type, Type::*},
        *,
    },
};

#[track_caller]
fn test_infer(src: &str, expected: Type) {
    let expr = Expr::from_str(src).unwrap();
    let ty = type_of(&expr);
    assert_eq!(ty, expected);
}

#[test]
fn infer_lit() {
    test_infer("1", Int);
    test_infer("1.0", Float);
    test_infer("true", Bool);
    test_infer("false", Bool);
}

#[test]
fn infer_identity_fn() {
    test_infer(r"\x -> x", ty![{1} => {1}]);
}

#[test]
fn infer_const_fn() {
    test_infer(r"\a -> \b -> a", ty![{1} => {3} => {1}]);
}

#[test]
fn infer_compose_fn() {
    test_infer(
        r"\f -> \g -> \x -> f (g x)",
        ty![({8} => {6}) => ({5} => {8}) => {5} => {6}],
    );
}

#[test]
fn infer_pred_fn() {
    test_infer(
        r"\pred -> if pred 1 then 2 else 3",
        ty![(Int => Bool) => Int],
    );
}

#[test]
fn infer_inc_fn() {
    test_infer(r"let inc = \x -> x + 1 in inc (inc 1)", ty![Int]);
}

#[test]
fn infer_letrec() {
    test_infer(
        r"
letrec
    countdown = \x -> if is_zero x
                      then 0
                      else countdown (sub x 1)
in
    countdown",
        ty![Int => Int],
    );
}

#[test]
fn infer_letrec_mutually_recursive() {
    test_infer(
        r"
letrec
    is_even = \x -> if is_zero x
                      then true
                      else is_odd (sub x 1),
    is_odd  = \x -> if is_zero x
                      then false
                      else is_even (sub x 1)
in
    is_even",
        ty![Int => Bool],
    );
}

#[test]
fn infer_let_many() {
    test_infer(
        r"let x = 1, y = add x 1, z = \a -> add a y in z",
        ty![Int => Int],
    );
}

#[test]
fn infer_lambda_many() {
    test_infer(
        r"let f = \a, b -> a b in f",
        ty![({5} => {6}) => {5} => {6}],
    );
}

#[test]
fn infer_annotations() {
    test_infer(r"let idInt: Int -> Int = \a -> a in idInt", ty![Int => Int]);
}

#[test]
fn infer_operators() {
    test_infer("1 + 2", Int);
    test_infer("1.0 +. 2.0", Float);

    test_infer("1 == 2", Bool);
    test_infer("1.0 == 2.0", Bool);
    test_infer("true == false", Bool);
    test_infer(r"(\x -> x) == (\y -> y)", Bool);
}
