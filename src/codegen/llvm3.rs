use super::closure::{CExpr, FreeVars, LetBinding, Lit, Param, Type};
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicType, BasicTypeEnum},
    values::{AnyValue, BasicValue, BasicValueEnum, FunctionValue, PointerValue},
    AddressSpace, IntPredicate,
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
    parent: Function<'a>,
    name: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct Function<'a> {
    value: FunctionValue<'a>,
    closure: Option<Closure<'a>>,
}

#[derive(Debug, Clone)]
pub struct Closure<'a> {
    free_vars: FreeVars,
    val: PointerValue<'a>,
}

impl<'a> Ctx<'a> {
    pub fn new(parent: FunctionValue<'a>) -> Self {
        Self {
            env: Env::new(),
            name: None,
            parent: Function {
                value: parent,
                closure: None,
            },
        }
    }
}

impl Type {
    fn llvm_type<'a>(&self, compiler: &Compiler<'a>) -> BasicTypeEnum<'a> {
        match self {
            Type::Bool => compiler.llvm.bool_type().into(),
            Type::Int => compiler.llvm.i64_type().into(),
            Type::Float => compiler.llvm.f64_type().into(),
            Type::Fn(..) => compiler.closure_ty(),
            Type::Var(_) => panic!("Cannot instantiate type {}", self),
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
        match self.module.get_struct_type("Closure") {
            Some(ty) => ty.into(),
            None => {
                let ty = self.llvm.opaque_struct_type("Closure");
                ty.set_body(&[self.void_ptr_ty(), self.void_ptr_ty()], false);
                ty.into()
            }
        }
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
            CExpr::Var { name, .. } | CExpr::EnvRef { name, .. } => self.compile_var(ctx, *name),
            CExpr::Lit { val, .. } => self.compile_lit(val),
            CExpr::Let { binding, body, .. } => self.compile_let(ctx, binding, body),
            CExpr::MkClosure {
                param,
                free_vars,
                body,
                ..
            } => self.compile_lambda(ctx, param, free_vars, body),
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

    fn compile_let(&self, ctx: &Ctx<'ctx>, binding: &LetBinding, body: &CExpr) -> BasicValueEnum {
        let binding_name = &binding.name.to_string();
        let mut ctx = ctx.clone();
        let alloca = self
            .builder
            .build_alloca(binding.ty.llvm_type(self), binding_name);
        let value = self.compile_expr(&ctx, &binding.val);
        self.builder.build_store(alloca, value);
        ctx.env.insert(binding.name, alloca);

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
            .position_at_end(ctx.parent.value.get_last_basic_block().unwrap());

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

        for (idx, (name, ty)) in free_vars.iter().enumerate() {
            let sname = &format!("env.{}", resolve(*name));
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
            .fn_type(&[env_ty, param.ty.llvm_type(self)], false);
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
        let env_alloca = self.builder.build_alloca(env_ty, "env");
        self.builder
            .build_store(env_alloca, fn_val.get_nth_param(0).unwrap());

        for (idx, (name, ty)) in free_vars.iter().enumerate() {
            let sname = &format!("env.{}", resolve(*name));
            let field_alloca = self.builder.build_alloca(ty.llvm_type(self), sname);
            let field_gep = self
                .builder
                .build_struct_gep(env_alloca, idx as u32, sname)
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

        let body = self.compile_expr(&ctx, body);
        self.builder.build_return(Some(&body));

        fn_val
    }
}
