use derive_more::Display;

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

#[macro_export]
macro_rules! ty {
    [{$e:expr}] => {{
        Type::Var($e)
    }};

    [Int] => {Type::Int};
    [Float] => {Type::Float};
    [Bool] => {Type::Bool};


    [($($tts:tt)=>+)] => {{
        let tys = vec!($( ty!($tts)),*);
        fold_tys(&tys)
    }};

    [$($tts:tt)=>+] => {{
        let tys = vec!($( ty!($tts)),*);
        fold_tys(&tys)
    }};


}

fn fold_tys(tys: &[Type]) -> Type {
    assert!(tys.len() >= 2);
    let rev: Vec<_> = tys.into_iter().rev().collect();
    let head = rev[0].clone();
    let tail = &rev[1..];
    tail.iter().fold(head, |acc, x| {
        Type::Fn(box x.clone().clone(), box acc.clone())
    })
}

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