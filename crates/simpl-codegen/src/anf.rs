use extend::ext;
pub use simpl_types::ast::{Ident, LetBinding, Lit};
use simpl_types::{
    ast::TypedExpr,
    ty::{LitExt, Type},
};

type Expr = TypedExpr;

#[ext(pub)]
impl Expr {
    fn is_anf(&self) -> bool {
        match self {
            Expr::Let {
                binding: LetBinding { val, .. },
                body,
                ..
            } => val.is_cexpr() || val.is_imm() && body.is_anf(),
            _ => self.is_cexpr() || self.is_aexpr(),
        }
    }

    fn is_cexpr(&self) -> bool {
        match self {
            Expr::App { func, arg, .. } => func.is_aexpr() && arg.is_aexpr(),
            Expr::If {
                test,
                then_branch,
                else_branch,
                ..
            } => test.is_aexpr() && then_branch.is_anf() && else_branch.is_anf(),
            _ => false,
        }
    }

    fn is_aexpr(&self) -> bool {
        match self {
            Expr::Lambda { body, .. } => body.is_anf(),
            _ => self.is_imm(),
        }
    }

    fn is_imm(&self) -> bool {
        match self {
            Self::Lit { .. } | Self::Var { .. } => true,
            _ => false,
        }
    }
}

fn normalize_expr(expr: Expr) -> Expr {
    normalize(expr, box |x| x)
}

fn normalize(expr: Expr, k: Box<dyn FnOnce(Expr) -> Expr>) -> Expr {
    match expr {
        Expr::If {
            ty,
            test,
            then_branch,
            else_branch,
        } => normalize_name(*test, box |t| {
            k(Expr::If {
                ty,
                test: box t,
                then_branch: box normalize_expr(*then_branch),
                else_branch: box normalize_expr(*else_branch),
            })
        }),

        Expr::Let { ty, binding, body } => normalize(*binding.clone().val, box |n1| Expr::Let {
            ty,
            binding: LetBinding {
                val: box n1,
                ..binding
            },
            body: box normalize(*body, k),
        }),

        Expr::Letrec { .. } => todo!(),

        Expr::Lambda { ty, param, body } => k(Expr::Lambda {
            ty,
            param,
            body: box normalize_expr(*body),
        }),

        Expr::App { ty, func, arg } => normalize_name(*func, box |t| {
            normalize_name(*arg, box |t2| {
                k(Expr::App {
                    ty,
                    func: box t,
                    arg: box t2,
                })
            })
        }),

        _ if expr.is_imm() => k(expr),

        _ => unreachable!(),
    }
}

fn normalize_name(expr: Expr, k: Box<dyn FnOnce(Expr) -> Expr>) -> Expr {
    fn is_imm(expr: &Expr) -> bool {
        expr.is_imm()
    }

    normalize(expr, box |n| {
        if is_imm(&n) {
            k(n)
        } else {
            let ty = n.ty();
            let t = Expr::Var {
                ty: ty.clone(),
                name: "gensym".into(),
            };
            Expr::Let {
                ty: ty.clone(),
                binding: LetBinding {
                    ty: ty.clone(),
                    name: "gensym".into(),
                    val: box n,
                },
                body: box k(t),
            }
        }
    })
}

mod test {
    use super::*;
    use insta::assert_snapshot;
    use simpl_types::{self, parse_and_type};

    #[track_caller]
    fn test_normalize(src: &str) {
        let expr = parse_and_type(src);
        let norm = normalize_expr(expr);
        assert!(norm.is_anf());
        assert_snapshot!(norm.pretty());
    }

    #[test]
    fn normalize_lit() {
        test_normalize("123");
    }

    #[test]
    fn normalize_if() {
        test_normalize("if true then 1 else 0");
        test_normalize("if (if false then true else false) then 1 else 0");
    }

    // TODO: should immediate values be allowed in the value of a binding?
    #[test]
    fn normalize_let() {
        test_normalize("let abc = 1 in abc");
    }

    #[test]
    fn normalize_lambda() {
        test_normalize(r"\x -> x");
        test_normalize(r"\x -> 1");
    }

    #[test]
    fn normalize_app() {
        test_normalize(r"add (mul 2 4) (mul 2 8)");
    }
}
