use crate::hir::{Expr, Lit};
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::BasicType,
    values::{BasicValueEnum, FunctionValue, PointerValue},
    IntPredicate,
};
use simple_symbol::{intern, resolve, Symbol};
use std::collections::HashMap;

type Env<'a> = HashMap<Symbol, PointerValue<'a>>;

#[derive(Debug)]
pub struct Compiler<'ctx> {
    pub ctx: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn compile_toplevel(&self, expr: &Expr) -> &Module<'ctx> {
        let ty = expr.ty().llvm_type(self.ctx).fn_type(&[], false);
        let function = self.module.add_function("toplevel", ty, None);
        let entry = self.ctx.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);
        let env = Env::new();
        let body_val = self.compile_expr(&env, expr);
        self.builder.build_return(Some(&body_val));

        &self.module
    }

    fn compile_expr(&self, env: &Env<'ctx>, expr: &Expr) -> BasicValueEnum {
        match expr {
            Expr::Lit { val, .. } => self.compile_lit(*val),
            Expr::Var { name, .. } => self.compile_var(env, *name),
            _ => todo!(),
        }
    }

    fn compile_lit(&self, val: Lit) -> BasicValueEnum {
        match val {
            Lit::Bool(b) => self.ctx.bool_type().const_int(b as u64, false).into(),
            Lit::Int(i) => self.ctx.i32_type().const_int(i as u64, false).into(),
            Lit::Float(f) => self.ctx.f64_type().const_float(f).into(),
        }
    }

    fn compile_var(&self, env: &Env<'ctx>, name: Symbol) -> BasicValueEnum<'ctx> {
        let ptr = env.get(&name).unwrap();
        self.builder.build_load(*ptr, &name.to_string())
    }
}
