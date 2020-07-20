use crate::parse;
use insta::assert_debug_snapshot;

#[track_caller]
fn test_parse_ok(src: &str) {
    assert_debug_snapshot!(parse(src).unwrap())
}

#[test]
fn literal() {
    test_parse_ok("123");
    test_parse_ok("123.456");
    test_parse_ok("true");
    test_parse_ok("false");
}

#[test]
fn var() {
    test_parse_ok("abc");
}

#[test]
fn if_then_else() {
    test_parse_ok("if true then 1 else 0");
    test_parse_ok(
        "if true then
                     if false then 0.5 else 1.5
                 else 0",
    );
    test_parse_ok(
        "if true then 0
                 else if false then 0.5
                 else 1.5",
    );
}

#[test]
fn let_binding() {
    test_parse_ok("let x = 5 in x");
    test_parse_ok("let x = 5, y = false in x");
}

#[test]
fn letrec_binding() {
    test_parse_ok(r"letrec f = \x -> f x in f");
    test_parse_ok(r"letrec f = \x -> f x, g = \y -> y in f");
}

#[test]
fn lambda_abstraction() {
    test_parse_ok(r"\x -> x");
    test_parse_ok(r"\x, y -> y");
}

#[test]
fn function_application() {
    test_parse_ok(r"f 1");
    test_parse_ok(r"f g x");
    test_parse_ok(r"f (g x)");
}
