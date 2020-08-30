use super::llvm::Compiler;
use crate::{hir::Expr, types::infer_and_apply};
use inkwell::context::Context;
use insta::assert_snapshot;
use std::str::FromStr;

#[track_caller]
fn test_compile(src: &str) {
    let expr = Expr::from_str(src).unwrap();
    let expr = infer_and_apply(&expr);

    let ctx = Context::create();
    let builder = ctx.create_builder();
    let module = ctx.create_module("test_compile");

    let compiler = Compiler {
        ctx: &ctx,
        module,
        builder,
    };

    let module = compiler.compile_toplevel(&expr);

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

#[test]
fn compile_lit() {
    test_compile("5");
    test_compile("true");
    test_compile("false");
}

#[test]
fn compile_if() {
    test_compile("if true then 100 else 200");
}

#[test]
fn compile_fn() {
    test_compile(r"\b -> if b then 100 else 200");
}

#[test]
fn compile_app() {
    test_compile(r"(\b -> if b then 100 else 200) true");
    test_compile(r"(\b -> if b then 100 else 200) ((\b -> if b then false else true) true)");
}

#[test]
fn compile_let() {
    test_compile("let x = 5 in x");
    test_compile(
        r"
let not = \b -> if b then false else true
in not true
",
    );
    test_compile(
        r"
let not = \b -> if b then false else true,
    idBool = \b -> if b then true else false
in not true
",
    );
}
