use crate::typed_ast::Symbol;
use derive_more::Display;
use std::collections::HashMap;

pub type TypeVar = u32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeEnv(HashMap<Symbol, Type>);

impl Default for TypeEnv {
    fn default() -> Self {
        let mut hm = HashMap::new();
        hm.insert(
            "add".into(),
            Type::Fn(vec![Type::Int, Type::Int], box Type::Int),
        );

        Self(hm)
    }
}

impl TypeEnv {
    pub fn empty() -> Self {
        Self(HashMap::new())
    }

    pub fn get(&self, var: &Symbol) -> Option<&Type> {
        self.0.get(var)
    }

    pub fn insert(&mut self, var: Symbol, val: Type) {
        self.0.insert(var, val);
    }
}

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
