use crate::types::ty::Type;
use inkwell::{
    context::Context,
    types::{BasicType, BasicTypeEnum, FunctionType},
    AddressSpace,
};

impl Type {
    pub fn llvm_type<'ctx>(&self, ctx: &'ctx Context) -> BasicTypeEnum<'ctx> {
        match self {
            Self::Bool => ctx.bool_type().into(),
            Self::Int => ctx.i32_type().into(),
            Self::Float => ctx.f64_type().into(),
            Self::Fn(arg, ret) => ret
                .llvm_type(ctx)
                .fn_type(&[arg.llvm_type(ctx)], false)
                .ptr_type(AddressSpace::Global)
                .into(),
            Self::Var(_) => panic!("Cannot instantiate type {}", self),
        }
    }

    pub fn llvm_fn_type<'ctx>(&self, ctx: &'ctx Context) -> Option<FunctionType<'ctx>> {
        match self {
            Self::Fn(arg, ret) => ret
                .llvm_type(ctx)
                .fn_type(&[arg.llvm_type(ctx)], false)
                .into(),
            _ => None,
        }
    }
}
