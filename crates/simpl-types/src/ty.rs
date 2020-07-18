use derive_more::Display;
use std::collections::HashMap;

type TypeVar = u32;
pub type TypeEnv = HashMap<String, Type>;

#[derive(Debug, Copy, Clone, Default)]
pub struct TypeVarGen {
    counter: u32,
}

impl TypeVarGen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fresh(&mut self) -> Type {
        self.counter += 1;
        Type::Var(self.counter)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum Type {
    #[display(fmt = "Int")]
    Int,
    #[display(fmt = "Bool")]
    Bool,
    #[display(fmt = "Float")]
    Float,
    #[display(fmt = "t{}", _0)]
    Var(TypeVar),
    #[display(fmt = "{} -> {}", "display_fn_args(_0)", _1)]
    Fn(Vec<Type>, Box<Type>),
}

fn display_fn_args(args: &[Type]) -> String {
    format!(
        "({})",
        args.iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    )
}
