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
fn infer_identity_fn() {
    let expr = Expr::from_str(r"\x -> x").unwrap();
    let ty = type_of(&expr);
    assert_eq!(ty, ty![{1} => {1}]);
}

#[test]
fn infer_const_fn() {
    let expr = Expr::from_str(r"\a -> \b -> a").unwrap();
    let ty = type_of(&expr);
    assert_eq!(ty, ty![{1} => {3} => {1}])
}

#[test]
fn infer_compose_fn() {
    let expr = Expr::from_str(r"\f -> \g -> \x -> f (g x)").unwrap();
    let ty = type_of(&expr);
    assert_eq!(ty, ty![({8} => {6}) => ({5} => {8}) => {5} => {6}]);
}

#[test]
fn infer_pred_fn() {
    let expr = Expr::from_str(r"\pred -> if pred 1 then 2 else 3").unwrap();
    let ty = type_of(&expr);
    assert_eq!(ty, ty![(Int => Bool) => Int]);
}

#[test]
fn infer_inc_fn() {
    let expr = Expr::from_str(r"let inc = \x -> x + 1 in inc (inc 1)").unwrap();
    let ty = type_of(&expr);
    assert_eq!(ty, ty![Int])
}

#[test]
fn infer_letrec() {
    let expr = Expr::from_str(
        r"
letrec
    countdown = \x -> if is_zero x
                      then 0
                      else countdown (sub x 1)
in
    countdown",
    )
    .unwrap();
    let ty = type_of(&expr);
    assert_eq!(ty, ty![Int => Int])
}

#[test]
fn infer_letrec_mutually_recursive() {
    let expr = Expr::from_str(
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
    )
    .unwrap();
    let ty = type_of(&expr);
    assert_eq!(ty, ty![Int => Bool])
}

#[test]
fn infer_let_many() {
    let expr = Expr::from_str(
        r"
let
    x = 1,
    y = add x 1,
    z = \a -> add a y
in
    z",
    )
    .unwrap();
    let ty = type_of(&expr);
    assert_eq!(ty, ty![Int => Int])
}

#[test]
fn infer_lambda_many() {
    let expr = Expr::from_str(
        r"
let
    f = \a, b -> a b
in
    f",
    )
    .unwrap();
    let ty = type_of(&expr);
    assert_eq!(ty, ty![({5} => {6}) => {5} => {6}])
}

#[test]
fn infer_annotations() {
    let expr = Expr::from_str(
        r"
let
    idInt = \a: Int -> a
in
    idInt",
    )
    .unwrap();
    let ty = type_of(&expr);
    assert_eq!(ty, ty![Int => Int]);

    let expr = Expr::from_str(
        r"
let
    idInt: Int -> Int = \a -> a
in
    idInt",
    )
    .unwrap();
    let ty = type_of(&expr);
    assert_eq!(ty, ty![Int => Int])
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
