use crate::{ast::Expr, grammar, parse};
use insta::assert_debug_snapshot;

#[track_caller]
fn test_parse_ok(src: &str) {
    match parse(src) {
        Ok(ast) => assert_debug_snapshot!(ast),
        Err(e) => {
            eprintln!("{}", e);
            panic!("FAILED TO PARSE")
        }
    }
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
    test_parse_ok("if true then 1 else 0;");
}

#[test]
fn let_binding() {
    test_parse_ok("let x = 5 in x;");
    test_parse_ok("let x = 5, y = false in x;");
}

#[test]
fn lambda_abstraction() {
    test_parse_ok(r"\() -> 1;");
    test_parse_ok(r"\(x) -> x;");
    test_parse_ok(r"\(x, y) -> y;");
}

#[test]
fn function_application() {
    test_parse_ok(r"f()");
    test_parse_ok(r"f(1, 2)");
    test_parse_ok(r"let id = \(x) -> x; in id(id);")
}