use super::{
    infer::TypeEnv,
    ty::{Polytype, Type, TypeVar, TypeVarGen},
};
use simpl_syntax::grammar::ExprParser;

fn default_env() -> TypeEnv {
    let mut env = TypeEnv::new();

    // not: Bool -> Bool
    env.insert(
        "not".into(),
        Polytype {
            vars: vec![],
            ty: Type::Fn(vec![Type::Bool], box Type::Bool),
        },
    );

    // is_zero: Int -> Bool
    env.insert(
        "is_zero".into(),
        Polytype {
            vars: vec![],
            ty: Type::Fn(vec![Type::Int], box Type::Bool),
        },
    );

    // add: Int, Int -> Int
    env.insert(
        "add".into(),
        Polytype {
            vars: vec![],
            ty: Type::Fn(vec![Type::Int, Type::Int], box Type::Int),
        },
    );

    // sub: Int, Int -> Int
    env.insert(
        "sub".into(),
        Polytype {
            vars: vec![],
            ty: Type::Fn(vec![Type::Int, Type::Int], box Type::Int),
        },
    );

    // mul: Int, Int -> Int
    env.insert(
        "mul".into(),
        Polytype {
            vars: vec![],
            ty: Type::Fn(vec![Type::Int, Type::Int], box Type::Int),
        },
    );

    // if_then_else: Bool, 't0, 't0 -> 't0
    env.insert(
        "if_then_else".into(),
        Polytype {
            vars: vec![TypeVar(0)],
            ty: Type::Fn(
                vec![Type::Bool, Type::Var(TypeVar(0)), Type::Var(TypeVar(0))],
                box Type::Var(TypeVar(0)),
            ),
        },
    );

    env
}

#[track_caller]
fn test_infer_ok(src: &str, expected: Type) {
    let mut env = default_env();
    let mut gen = TypeVarGen::new();

    let parser = ExprParser::new();
    let ast = match parser.parse(src) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("{}", e);
            panic!("PARSE FAILED");
        }
    };

    let ty = env.infer(&ast, &mut gen).unwrap();
    assert_eq!(ty, expected);
}

#[test]
fn literals() {
    test_infer_ok("123", Type::Int);
    test_infer_ok("123.456", Type::Float);
    test_infer_ok("true", Type::Bool);
    test_infer_ok("false", Type::Bool);
}

#[test]
fn vars() {
    test_infer_ok("let abc = 1 in abc;", Type::Int);
    test_infer_ok("let abc = 1 in let xyz = abc in xyz;;", Type::Int);
    test_infer_ok("let abc = 1, xyz = abc in xyz;", Type::Int);
}

#[test]
fn lambda() {
    test_infer_ok(
        r"\(x) -> x;",
        Type::Fn(vec![Type::Var(TypeVar(0))], box Type::Var(TypeVar(0))),
    );

    test_infer_ok(
        r"\(x) -> 1;",
        Type::Fn(vec![Type::Var(TypeVar(0))], box Type::Int),
    );
}

#[test]
fn app() {
    test_infer_ok(r"\() -> 1;()", Type::Int);
    test_infer_ok(r"\(x) -> x;(5)", Type::Int);
    test_infer_ok(r"\(x, y) -> y;(5, true)", Type::Bool);
    test_infer_ok(r"add(1, 1)", Type::Int);
    test_infer_ok(r"if_then_else(true, 1, 0)", Type::Int);
}

#[test]
fn letrec() {
    // loop: () -> 't1
    test_infer_ok(
        r"let loop = \() -> loop(); in loop;",
        Type::Fn(vec![], box Type::Var(TypeVar(1))),
    );

    // loop: () -> 't1
    // loop() is the "bottom type": it can be used anywhere any type is expected
    // in this instance, loop() is instantiated as a Bool and an Int
    test_infer_ok(
        r"
let loop = \() -> loop();
in
    if_then_else(loop(), add(loop(), 1), loop());",
        Type::Int,
    );

    // fact: Int -> Int
    test_infer_ok(
        r"let fact = \(x) -> if_then_else(is_zero(x), 0, mul(x, fact(sub(x,
    1)))); in fact;",
        Type::Fn(vec![Type::Int], box Type::Int),
    );

    // is_even: Int -> bool
    test_infer_ok(
        r"
let is_even = \(x) -> if_then_else(is_zero(x), true, is_odd(sub(x, 1)));,
    is_odd = \(x) -> if_then_else(is_zero(x), false, is_even(sub(x, 1)));
in
    is_even;
",
        Type::Fn(vec![Type::Int], box Type::Bool),
    );

    test_infer_ok("let a = b, b = a in a;", Type::Var(TypeVar(1)));
}
