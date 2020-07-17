use super::{
    infer::TypeEnv,
    ty::{Polytype, Type, TypeVar, TypeVarGen},
};
use simpl_syntax::grammar::ExprParser;

fn default_env() -> TypeEnv {
    let mut env = TypeEnv::new();

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

    // if_then_else: Bool, 't1, 't1 -> 't1
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

// #[test]
fn bottom() {
    // loop: () -> 't1
    test_infer_ok(
        r"let loop = \() -> loop(); in loop;",
        Type::Fn(vec![], box Type::Var(TypeVar(0))),
    );
}
