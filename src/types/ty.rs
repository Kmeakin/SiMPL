use crate::{
    hir::{Lit, Symbol},
    util::counter::{Counter, FromId},
};
use derive_more::Display;
use simple_symbol::intern;
use std::collections::HashMap;

pub type TypeVar = u32;

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum Type {
    #[display(fmt = "Int")]
    Int,
    #[display(fmt = "Float")]
    Float,
    #[display(fmt = "Bool")]
    Bool,
    #[display(fmt = "t{}", _0)]
    Var(TypeVar),
    #[display(fmt = "{}", "display_fn_type(_0, _1)")]
    Fn(Box<Type>, Box<Type>),
}

fn display_fn_type(t1: &Type, t2: &Type) -> String {
    if let Type::Fn(..) = t1 {
        format!("({}) -> {}", t1, t2)
    } else {
        format!("{} -> {}", t1, t2)
    }
}

impl Lit {
    pub const fn ty(&self) -> Type {
        match self {
            Self::Int(_) => Type::Int,
            Self::Bool(_) => Type::Bool,
            Self::Float(_) => Type::Float,
        }
    }
}

pub type TypeVarGen = Counter<Type>;

impl FromId for Type {
    fn from_id(id: u32) -> Self {
        Self::Var(id)
    }
}

#[macro_export]
macro_rules! ty {
    [{$e:expr}] => {{
        Type::Var($e)
    }};

    [Int] => {Type::Int};
    [Float] => {Type::Float};
    [Bool] => {Type::Bool};


    [($($tts:tt)=>+)] => {{
        let mut tys = vec!($( ty!($tts)),*);
        $crate::types::ty::fold_tys(&mut tys)
    }};

    [$($tts:tt)=>+] => {{
        let mut tys = vec!($( ty!($tts)),*);
        $crate::types::ty::fold_tys(&mut tys)
    }};


}

pub fn fold_tys(tys: &mut [Type]) -> Type {
    assert!(tys.len() >= 2);
    tys.reverse();
    let head = tys[0].clone();
    let tail = &tys[1..];
    tail.iter()
        .fold(head, |acc, ty| Type::Fn(box ty.clone(), box acc.clone()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A mapping from `Ident`s (that is, variables) to `Type`s.
/// Used when looking up type of an `Expr::Var`
pub struct TypeEnv(HashMap<Symbol, Type>);

impl Default for TypeEnv {
    fn default() -> Self {
        let mut hm = HashMap::new();
        hm.insert(intern("add"), ty![Int => Int => Int]);
        hm.insert(intern("sub"), ty![Int => Int => Int]);
        hm.insert(intern("mul"), ty![Int => Int => Int]);
        hm.insert(intern("is_zero"), ty![Int => Bool]);
        hm.insert(intern("not"), ty![Bool => Bool]);

        Self(hm)
    }
}

impl TypeEnv {
    pub fn empty() -> Self {
        Self(HashMap::new())
    }

    pub fn get(&self, var: Symbol) -> Option<&Type> {
        self.0.get(&var)
    }

    pub fn insert(&mut self, name: Symbol, ty: Type) {
        self.0.insert(name, ty);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use Type::*;

    #[test]
    fn test_ty_macro() {
        assert_eq!(ty![Int].to_string(), "Int");
        assert_eq!(ty![Float].to_string(), "Float");
        assert_eq!(ty![Bool].to_string(), "Bool");
        assert_eq!(ty![Int => Bool].to_string(), "Int -> Bool");
        assert_eq!(
            ty![Int => Bool => Float].to_string(),
            "Int -> Bool -> Float"
        );
        assert_eq!(
            ty![(Int => Bool) => Float].to_string(),
            "(Int -> Bool) -> Float"
        );
        assert_eq!(
            ty![Int => (Bool => Float)].to_string(),
            "Int -> Bool -> Float"
        );
        assert_eq!(ty![{ 0 }].to_string(), "t0");
        assert_eq!(ty![{0} => {0}].to_string(), "t0 -> t0");
        assert_eq!(ty![({0} => {1}) => {1}].to_string(), "(t0 -> t1) -> t1");
    }

    #[test]
    fn test_type_display() {
        assert_eq!(Int.to_string(), "Int");
        assert_eq!(Float.to_string(), "Float");
        assert_eq!(Bool.to_string(), "Bool");
        assert_eq!(Var(0).to_string(), "t0");
        assert_eq!(Fn(box Int, box Int).to_string(), "Int -> Int");
        assert_eq!(
            Fn(box Int, box Fn(box Bool, box Int)).to_string(),
            "Int -> Bool -> Int"
        );
        assert_eq!(
            Fn(box Fn(box Int, box Bool), box Int).to_string(),
            "(Int -> Bool) -> Int"
        );
        assert_eq!(
            Fn(box Fn(box Int, box Bool), box Int).to_string(),
            "(Int -> Bool) -> Int"
        );
        assert_eq!(
            Fn(
                box Fn(box Int, box Fn(box Bool, box Int)),
                box Fn(box Int, box Bool)
            )
            .to_string(),
            "(Int -> Bool -> Int) -> Int -> Bool"
        );
    }
}
