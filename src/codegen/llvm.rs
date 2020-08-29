use crate::hir::{Expr, Lit, Param, Type};
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::BasicType,
    values::{AnyValue, BasicValue, BasicValueEnum, FunctionValue, PointerValue},
    IntPredicate,
};
use simple_symbol::Symbol;
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
        let parent = self.module.add_function("toplevel", ty, None);
        let entry = self.ctx.append_basic_block(parent, "entry");
        self.builder.position_at_end(entry);

        let env = Env::new();
        let body_val = self.compile_expr(&env, parent, expr);
        self.builder.build_return(Some(&body_val));

        match self.module.verify() {
            Ok(()) => {}
            Err(s) => {
                println!("{}\n", self.module.print_to_string().to_string());
                eprintln!("{}", s.to_string());
                panic!()
            }
        }

        &self.module
    }

    fn compile_expr(&self, env: &Env<'ctx>, parent: FunctionValue, expr: &Expr) -> BasicValueEnum {
        match expr {
            Expr::Lit { val, .. } => self.compile_lit(*val),
            Expr::Var { name, .. } => self.compile_var(env, *name),
            Expr::If {
                test,
                then_branch,
                else_branch,
                ..
            } => self.compile_if(env, parent, test, then_branch, else_branch),
            Expr::Lambda { ty, param, body } => {
                self.compile_lambda(env, parent, ty.clone(), param.clone(), body)
            }
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

    fn compile_if(
        &self,
        env: &Env<'ctx>,
        parent: FunctionValue,
        test: &Expr,
        then: &Expr,
        els: &Expr,
    ) -> BasicValueEnum {
        let test_val = self.compile_expr(env, parent, test);

        let then_ty = then.ty().llvm_type(self.ctx);
        let else_ty = els.ty().llvm_type(self.ctx);
        assert_eq!(then_ty, else_ty);

        let const_true = self.ctx.bool_type().const_int(1, false);
        let cmp = self.builder.build_int_compare(
            IntPredicate::EQ,
            test_val.into_int_value(),
            const_true,
            "cmp",
        );

        let then_bb = self.ctx.append_basic_block(parent, "then");
        let else_bb = self.ctx.append_basic_block(parent, "else");
        let cont_bb = self.ctx.append_basic_block(parent, "cont");

        self.builder.build_conditional_branch(cmp, then_bb, else_bb);

        self.builder.position_at_end(then_bb);
        let then_val = self.compile_expr(env, parent, then);
        self.builder.build_unconditional_branch(cont_bb);
        let then_bb = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(else_bb);
        let else_val = self.compile_expr(env, parent, els);
        self.builder.build_unconditional_branch(cont_bb);
        let else_bb = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(cont_bb);
        let phi = self.builder.build_phi(then_ty, "phi");
        phi.add_incoming(&[(&then_val, then_bb), (&else_val, else_bb)]);

        phi.as_basic_value()
    }

    fn compile_lambda(
        &self,
        env: &Env<'ctx>,
        parent: FunctionValue,
        ty: Type,
        param: Param,
        body: &Expr,
    ) -> BasicValueEnum {
        let param_name = &param.name.to_string();
        let mut env = env.clone();

        let fn_ty = ty.llvm_fn_type(self.ctx).unwrap();
        let fn_val = self.module.add_function("lambda", fn_ty, None);
        fn_val
            .get_first_param()
            .unwrap()
            .set_name(&param.name.to_string());

        let entry = self.ctx.append_basic_block(fn_val, "lambda_entry");
        self.builder.position_at_end(entry);
        let alloca = self
            .builder
            .build_alloca(param.ty.llvm_type(self.ctx), param_name);
        self.builder
            .build_store(alloca, fn_val.get_first_param().unwrap());
        env.insert(param.name, alloca);

        let body = self.compile_expr(&env, fn_val, body);

        self.builder.build_return(Some(&body));

        self.builder
            .position_at_end(parent.get_last_basic_block().unwrap());

        fn_val.as_any_value_enum().into_pointer_value().into()
    }
}
