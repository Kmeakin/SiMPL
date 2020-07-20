use crate::{ty, ty::Type, *};

#[test]
fn infer_identity_fn() {
    let expr = TypedExpr::from_str(r"\x -> x").unwrap();
    let ty = type_of(expr);
    assert_eq!(ty, ty![{1} => {1}]);
}

#[test]
fn infer_const_fn() {
    let expr = TypedExpr::from_str(r"\a -> \b -> a").unwrap();
    let ty = type_of(expr);
    assert_eq!(ty, ty![{1} => {3} => {1}])
}

#[test]
fn infer_compose_fn() {
    let expr = TypedExpr::from_str(r"\f -> \g -> \x -> f (g x)").unwrap();
    let ty = type_of(expr);
    assert_eq!(ty, ty![({7} => {6}) => ({5} => {7}) => {5} => {6}]);
}

#[test]
fn infer_pred_fn() {
    let expr = TypedExpr::from_str(r"\pred -> if pred 1 then 2 else 3").unwrap();
    let ty = type_of(expr);
    assert_eq!(ty, ty![(Int => Bool) => Int]);
}

#[test]
fn infer_inc_fn() {
    let expr = TypedExpr::from_str(r"let inc = \x -> add x 1 in inc (inc 1)").unwrap();
    let ty = type_of(expr);
    assert_eq!(ty, ty![Int])
}
