use super::{closure::convert, llvm::Compiler};
use crate::{hir::Expr, types::infer_and_apply};
use inkwell::{context::Context, OptimizationLevel};
use insta::assert_snapshot;
use std::str::FromStr;

#[track_caller]
fn test_compile(src: &str) {
    let expr = Expr::from_str(src).unwrap();
    let expr = infer_and_apply(&expr);
    let cexpr = convert(expr);

    let ctx = Context::create();
    let builder = ctx.create_builder();
    let module = ctx.create_module("test_compile");

    let compiler = Compiler {
        llvm: &ctx,
        module,
        builder,
    };

    let module = compiler.compile_toplevel(&cexpr);

    match module.verify() {
        Ok(()) => {}
        Err(s) => {
            println!("{}\n", module.print_to_string().to_string());
            eprintln!("{}", s.to_string());
            panic!()
        }
    }

    assert_snapshot!(module.print_to_string().to_string());
}

#[track_caller]
fn test_compile_and_execute<T: std::fmt::Debug + PartialEq>(src: &str, expected: T) {
    let expr = Expr::from_str(src).unwrap();
    let expr = infer_and_apply(&expr);
    let cexpr = convert(expr);

    let ctx = Context::create();
    let builder = ctx.create_builder();
    let module = ctx.create_module("test_compile");

    let compiler = Compiler {
        llvm: &ctx,
        module,
        builder,
    };

    let module = compiler.compile_toplevel(&cexpr);

    match module.verify() {
        Ok(()) => {}
        Err(s) => {
            println!("{}\n", module.print_to_string().to_string());
            eprintln!("{}", s.to_string());
            panic!()
        }
    }

    // assert_snapshot!(module.print_to_string().to_string());

    let exec_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();
    let f = unsafe { exec_engine.get_function::<unsafe extern "C" fn() -> T>("toplevel") }.unwrap();
    assert_eq!(unsafe { f.call() }, expected)
}

#[test]
fn compile_lit() {
    test_compile_and_execute("0", 0);
    test_compile_and_execute("1", 1);
    test_compile_and_execute("true", true);
    test_compile_and_execute("false", false);
    test_compile_and_execute("0.0", 0.0);
    test_compile_and_execute("4.5", 4.5);
}

#[test]
fn compile_vars() {
    test_compile_and_execute("let x = 5 in x", 5);
}

#[test]
fn compile_if() {
    test_compile_and_execute("if true then 5 else 10", 5);
    test_compile_and_execute("if false then 5 else 10", 10);
    test_compile_and_execute(r"let f = \b -> if b then 5 else 10 in f true", 5);
    test_compile_and_execute(r"let f = \b -> if b then 5 else 10 in f false", 10);
}

#[test]
fn compile_lambda() {
    test_compile(r"let x = 5 in \y: Int -> x");
    test_compile(r"\x: Int -> x");
}

#[test]
fn compile_app() {
    test_compile_and_execute(
        r"
let x = 5,
    f = \y -> x
in f 555
",
        5,
    );
    test_compile_and_execute(
        r"
    let ifThenElse = \b, x, y -> if b then x else y
    in  ifThenElse false 5 10
    ",
        10,
    );

    test_compile_and_execute(
        r"
let const5 = \x: Int -> 5,
    apply = \f, x -> f x,
in apply const5 0
",
        5,
    );

    test_compile_and_execute(
        r"
let x = 5,
    const5 = \y: Int -> x,
    apply = \f, x -> f x,
in apply const5 0
",
        5,
    );

    test_compile_and_execute(
        r"
let plus2 = \x -> x + 2,
    mul3  = \x -> x * 3,
    compose = \f, g, x -> f (g x),
    myFn  = compose mul3 plus2,
in myFn 5
",
        21,
    );
}

#[test]
fn compile_ops() {
    test_compile_and_execute("1 * 2 + 3 / 4", 2); // (1 * 2) + (3 / 4)
    test_compile_and_execute("1 + 2 * 3 - 4", 3); // 1 + (2 * 3) - 4
    test_compile_and_execute("1 == 1", true);
    test_compile_and_execute("1 != 1", false);
    test_compile_and_execute("let x = 6, y = 7 in x * y", 42);
}
