use super::llvm::Compiler;
use crate::{
    hir::{Expr, Lit, Symbol},
    types::infer_and_apply,
};
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

    let res = compiler.compile_toplevel(&expr);
    assert_snapshot!(res.print_to_string().to_string());
}

#[test]
fn compile_lit() {
    test_compile("5");
    test_compile("true");
    test_compile("false");
}
