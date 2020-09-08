use super::closure::{Binop, CExpr, FreeVars, LetBinding, Lit, Param, Type};
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicType, BasicTypeEnum},
    values::{AnyValue, BasicValue, BasicValueEnum, FunctionValue, PointerValue},
    AddressSpace, FloatPredicate, IntPredicate,
};
use simple_symbol::{resolve, Symbol};
use std::collections::HashMap;

type Env<'a> = HashMap<Symbol, PointerValue<'a>>;

#[derive(Debug)]
pub struct Compiler<'ctx> {
    pub llvm: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
}

#[derive(Debug, Clone)]
pub struct Ctx<'a> {
    env: Env<'a>,
    parent: FunctionValue<'a>,
    name: Option<&'a str>,
}

impl<'a> Ctx<'a> {
    pub fn new(parent: FunctionValue<'a>) -> Self {
        Self {
            env: Env::new(),
            name: None,
            parent,
        }
    }
}

impl Type {
    fn llvm_type<'a>(&self, compiler: &Compiler<'a>) -> BasicTypeEnum<'a> {
        match self {
            Self::Bool => compiler.llvm.bool_type().into(),
            Self::Int => compiler.llvm.i64_type().into(),
            Self::Float => compiler.llvm.f64_type().into(),
            Self::Fn(..) => compiler.closure_ty(),
            Self::Var(_) => panic!("Cannot instantiate type {}", self),
        }
    }
}

impl<'ctx> Compiler<'ctx> {
    fn void_ptr_ty(&self) -> BasicTypeEnum<'ctx> {
        self.llvm.i8_type().ptr_type(AddressSpace::Generic).into()
    }

    fn closure_ty(&self) -> BasicTypeEnum<'ctx> {
        // struct Closure {
        //     void* code,
        //     void* env,
        // }

        self.module.get_struct_type("Closure").map_or_else(
            || {
                let ty = self.llvm.opaque_struct_type("Closure");
                ty.set_body(&[self.void_ptr_ty(), self.void_ptr_ty()], false);
                ty.into()
            },
            |ty| ty.into(),
        )
    }

    fn env_ty(&self, free_vars: &FreeVars) -> BasicTypeEnum<'ctx> {
        let env_ty = self.llvm.opaque_struct_type("Env");
        env_ty.set_body(
            &free_vars
                .iter()
                .map(|(_, ty)| ty.llvm_type(self))
                .collect::<Vec<_>>(),
            false,
        );
        env_ty.into()
    }

    pub fn compile_toplevel(&self, expr: &CExpr) -> &Module<'ctx> {
        let val_ty = expr.ty().llvm_type(self);
        let fn_ty = val_ty.fn_type(&[], false);

        let parent = self.module.add_function("toplevel", fn_ty, None);
        let entry = self.llvm.append_basic_block(parent, "toplevel_entry");
        self.builder.position_at_end(entry);

        let ctx = Ctx::new(parent);
        let body_val = self.compile_expr(&ctx, expr);
        self.builder.build_return(Some(&body_val));

        &self.module
    }

    fn compile_expr(&self, ctx: &Ctx<'ctx>, expr: &CExpr) -> BasicValueEnum {
        match expr {
            CExpr::Lit { val, .. } => self.compile_lit(val),
            CExpr::Var { name, .. } | CExpr::EnvRef { name, .. } => self.compile_var(ctx, *name),
            CExpr::Binop { lhs, rhs, op, .. } => self.compile_binop(ctx, lhs, rhs, *op),
            CExpr::If {
                test, then, els, ..
            } => self.compile_if(ctx, test, then, els),
            CExpr::Let { binding, body, .. } => self.compile_let(ctx, binding, body),
            CExpr::MkClosure {
                param,
                free_vars,
                body,
                ..
            } => self.compile_lambda(ctx, param, free_vars, body),
            CExpr::App { func, arg, ty } => self.compile_app(ctx, ty, func, arg),
            _ => todo!(),
        }
    }

    fn compile_lit(&self, val: &Lit) -> BasicValueEnum {
        match *val {
            Lit::Bool(b) => self
                .llvm
                .bool_type()
                .const_int(if b { 1 } else { 0 }, false)
                .into(),
            Lit::Int(i) => self
                .llvm
                .i64_type()
                .const_int(unsafe { std::mem::transmute(i) }, false)
                .into(),
            Lit::Float(f) => self.llvm.f64_type().const_float(f).into(),
        }
    }

    fn compile_var(&self, ctx: &Ctx<'ctx>, name: Symbol) -> BasicValueEnum {
        let ptr = ctx.env.get(&name).unwrap();
        self.builder.build_load(*ptr, &name.to_string())
    }

    fn compile_binop(
        &self,
        ctx: &Ctx<'ctx>,
        lhs: &CExpr,
        rhs: &CExpr,
        op: Binop,
    ) -> BasicValueEnum {
        #![allow(clippy::enum_glob_use)]
        use Binop::*;

        let lhs_val = self.compile_expr(ctx, lhs);
        let rhs_val = self.compile_expr(ctx, rhs);

        #[rustfmt::skip]
        macro_rules! int_op {($op:ident, $name:expr) => {self.builder.$op(lhs_val.into_int_value(), rhs_val.into_int_value(), $name) .into()};}

        #[rustfmt::skip]
        macro_rules! int_cmp {($cmp:expr, $name:expr) => {self.builder.build_int_compare($cmp, lhs_val.into_int_value(), rhs_val.into_int_value(), $name,) .into()};}

        #[rustfmt::skip]
        macro_rules! float_op {($op:ident, $name:expr) => {self.builder.$op(lhs_val.into_float_value(), rhs_val.into_float_value(), $name,) .into()};}

        #[rustfmt::skip]
        macro_rules! float_cmp {($cmp:expr, $name:expr) => {self.builder.build_float_compare($cmp, lhs_val.into_float_value(), rhs_val.into_float_value(), $name,) .into()};}

        match op {
            IntAdd => int_op!(build_int_add, "add"),
            IntSub => int_op!(build_int_sub, "sub"),
            IntMul => int_op!(build_int_mul, "mul"),
            IntDiv => int_op!(build_int_exact_signed_div, "div"),

            IntLt => int_cmp!(IntPredicate::SLT, "cmp"),
            IntLeq => int_cmp!(IntPredicate::SLE, "cmp"),
            IntGt => int_cmp!(IntPredicate::SGT, "cmp"),
            IntGeq => int_cmp!(IntPredicate::SGE, "cmp"),

            FloatAdd => float_op!(build_float_add, "add"),
            FloatSub => float_op!(build_float_sub, "sub"),
            FloatMul => float_op!(build_float_mul, "mul"),
            FloatDiv => float_op!(build_float_div, "div"),

            FloatLt => float_cmp!(FloatPredicate::OLT, "cmp"),
            FloatLeq => float_cmp!(FloatPredicate::OLE, "cmp"),
            FloatGt => float_cmp!(FloatPredicate::OGT, "cmp"),
            FloatGeq => float_cmp!(FloatPredicate::OGE, "cmp"),

            Eq => match lhs.ty() {
                Type::Bool | Type::Int => int_cmp!(IntPredicate::EQ, "cmp"),
                Type::Float => float_cmp!(FloatPredicate::OEQ, "cmp"),
                _ => todo!(),
            },

            Neq => match lhs.ty() {
                Type::Bool | Type::Int => int_cmp!(IntPredicate::NE, "cmp"),
                Type::Float => float_cmp!(FloatPredicate::UNE, "cmp"),
                _ => todo!(),
            },
        }
    }

    fn compile_if(
        &self,
        ctx: &Ctx<'ctx>,
        test: &CExpr,
        then: &CExpr,
        els: &CExpr,
    ) -> BasicValueEnum {
        assert_eq!(test.ty(), Type::Bool);
        let test_val = self.compile_expr(ctx, test);

        let then_ty = then.ty().llvm_type(self);
        let else_ty = els.ty().llvm_type(self);
        assert_eq!(then_ty, else_ty);

        let const_true = self.llvm.bool_type().const_int(1, false);
        let cmp = self.builder.build_int_compare(
            IntPredicate::EQ,
            test_val.into_int_value(),
            const_true,
            "cmp",
        );

        let then_bb = self.llvm.append_basic_block(ctx.parent, "then");
        let else_bb = self.llvm.append_basic_block(ctx.parent, "else");
        let cont_bb = self.llvm.append_basic_block(ctx.parent, "cont");
        self.builder.build_conditional_branch(cmp, then_bb, else_bb);

        // then branch
        self.builder.position_at_end(then_bb);
        let then_val = self.compile_expr(ctx, then);
        self.builder.build_unconditional_branch(cont_bb);
        let then_bb = self.builder.get_insert_block().unwrap();

        // else branch
        self.builder.position_at_end(else_bb);
        let else_val = self.compile_expr(ctx, els);
        self.builder.build_unconditional_branch(cont_bb);
        let else_bb = self.builder.get_insert_block().unwrap();

        // merge the branches
        self.builder.position_at_end(cont_bb);
        let phi = self.builder.build_phi(then_ty, "phi");
        phi.add_incoming(&[(&then_val, then_bb), (&else_val, else_bb)]);
        phi.as_basic_value()
    }

    fn compile_let(&self, ctx: &Ctx<'ctx>, binding: &LetBinding, body: &CExpr) -> BasicValueEnum {
        let binding_name = resolve(binding.name);
        let mut ctx = ctx.clone();
        let alloca = self
            .builder
            .build_alloca(binding.ty.llvm_type(self), binding_name);

        let old_name = ctx.name;
        ctx.name = Some(binding_name);
        let value = self.compile_expr(&ctx, &binding.val);
        self.builder.build_store(alloca, value);
        ctx.env.insert(binding.name, alloca);

        ctx.name = old_name;
        self.compile_expr(&ctx, body)
    }

    fn compile_lambda(
        &self,
        ctx: &Ctx<'ctx>,
        param: &Param,
        free_vars: &FreeVars,
        body: &CExpr,
    ) -> BasicValueEnum {
        let env_ty = self.env_ty(free_vars);
        let fn_val = self.compile_function(ctx, free_vars, env_ty, param, body);
        self.builder
            .position_at_end(ctx.parent.get_last_basic_block().unwrap());

        let closure = self.builder.build_alloca(self.closure_ty(), "closure");

        let code_gep = self
            .builder
            .build_struct_gep(closure, 0, "closure.code")
            .unwrap();
        let fn_val = fn_val.as_any_value_enum().into_pointer_value();
        let fn_val = self
            .builder
            .build_bitcast(fn_val, self.void_ptr_ty(), "closure.code");
        self.builder
            .build_store(code_gep, fn_val.as_any_value_enum().into_pointer_value());

        let env_gep = self
            .builder
            .build_struct_gep(closure, 1, "closure.env")
            .unwrap();
        let env_val = self.builder.build_malloc(env_ty, "closure.env").unwrap();

        for (idx, (name, _)) in free_vars.iter().enumerate() {
            let sname = &format!("env.{}", resolve(*name));

            #[allow(clippy::cast_possible_truncation)]
            let field_gep = self
                .builder
                .build_struct_gep(env_val, idx as u32, sname)
                .unwrap();
            let field_val = self.compile_var(ctx, *name);
            self.builder.build_store(field_gep, field_val);
        }

        let env_val = self
            .builder
            .build_bitcast(env_val, self.void_ptr_ty(), "closure.env");
        self.builder.build_store(env_gep, env_val);

        self.builder.build_load(closure, "closure")
    }

    fn compile_function(
        &self,
        ctx: &Ctx<'ctx>,
        free_vars: &FreeVars,
        env_ty: BasicTypeEnum<'ctx>,
        param: &Param,
        body: &CExpr,
    ) -> FunctionValue {
        let fn_name = ctx.name.unwrap_or("lambda");
        let fn_ty = body
            .ty()
            .llvm_type(self)
            .fn_type(&[self.void_ptr_ty(), param.ty.llvm_type(self)], false);
        let fn_val = self.module.add_function(fn_name, fn_ty, None);
        fn_val.get_nth_param(0).unwrap().set_name("env");
        fn_val
            .get_nth_param(1)
            .unwrap()
            .set_name(resolve(param.name));

        let entry = self
            .llvm
            .append_basic_block(fn_val, &format!("{fn_name}_entry"));
        self.builder.position_at_end(entry);

        // load captured env
        let mut ctx = ctx.clone();
        let env_alloca = self.builder.build_alloca(self.void_ptr_ty(), "env");
        self.builder
            .build_store(env_alloca, fn_val.get_nth_param(0).unwrap());
        let env_val = self
            .builder
            .build_load(env_alloca, "env")
            .into_pointer_value();
        let env_val = self
            .builder
            .build_bitcast(env_val, env_ty.ptr_type(AddressSpace::Generic), "env")
            .into_pointer_value();

        for (idx, (name, ty)) in free_vars.iter().enumerate() {
            let sname = &format!("env.{}", resolve(*name));
            let field_alloca = self.builder.build_alloca(ty.llvm_type(self), sname);

            #[allow(clippy::cast_possible_truncation)]
            let field_gep = self
                .builder
                .build_struct_gep(env_val, idx as u32, sname)
                .unwrap();
            let field_val = self.builder.build_load(field_gep, sname);
            self.builder.build_store(field_alloca, field_val);
            ctx.env.insert(*name, field_alloca);
        }

        // load param
        let param_name = resolve(param.name);
        let param_alloca = self
            .builder
            .build_alloca(param.ty.llvm_type(self), param_name);
        self.builder
            .build_store(param_alloca, fn_val.get_nth_param(1).unwrap());
        ctx.env.insert(param.name, param_alloca);

        ctx.parent = fn_val;
        let body = self.compile_expr(&ctx, body);
        self.builder.build_return(Some(&body));

        fn_val
    }

    fn compile_app(
        &self,
        ctx: &Ctx<'ctx>,
        result_ty: &Type,
        func: &CExpr,
        arg: &CExpr,
    ) -> BasicValueEnum {
        let closure = self.compile_expr(ctx, func);
        let closure_alloca = self.builder.build_alloca(self.closure_ty(), "closure");
        self.builder.build_store(closure_alloca, closure);
        let fn_gep = self
            .builder
            .build_struct_gep(closure_alloca, 0, "closure.fn")
            .unwrap();
        let fn_val = self.builder.build_load(fn_gep, "closure.fn");
        let fn_val = self
            .builder
            .build_bitcast(
                fn_val,
                result_ty
                    .llvm_type(self)
                    .fn_type(&[self.void_ptr_ty(), arg.ty().llvm_type(self)], false)
                    .ptr_type(AddressSpace::Generic),
                "closure.fn",
            )
            .into_pointer_value();

        let env_gep = self
            .builder
            .build_struct_gep(closure_alloca, 1, "closure.env")
            .unwrap();
        let env_val = self.builder.build_load(env_gep, "closure.env");
        let arg_val = self.compile_expr(ctx, arg);

        self.builder
            .build_call(fn_val, &[env_val, arg_val], "call")
            .try_as_basic_value()
            .left()
            .unwrap()
    }
}
