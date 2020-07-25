use crate::hir::{Expr, LetBinding};
use pretty::RcDoc;

const INDENT: isize = 4;
const WIDTH: usize = 40;

fn binding_to_doc<'a>(binding: &'a LetBinding) -> RcDoc<'a> {
    RcDoc::nil()
        .append(binding.name.to_string())
        .append(RcDoc::space())
        .append(RcDoc::text("="))
        .append(RcDoc::space())
        .append(binding.val.to_doc())
}

impl Expr {
    pub fn to_doc(&self) -> RcDoc<()> {
        match self {
            Self::Lit { val, .. } => RcDoc::as_string(val),
            Self::Var { name, .. } => RcDoc::as_string(name),
            Self::If {
                test,
                then_branch,
                else_branch,
                ..
            } => RcDoc::text("if")
                .append(RcDoc::space())
                .append(test.to_doc())
                .append(RcDoc::line())
                .append(RcDoc::text("then"))
                .append(RcDoc::space())
                .append(then_branch.to_doc())
                .append(RcDoc::line())
                .append(RcDoc::text("else"))
                .append(RcDoc::space())
                .append(else_branch.to_doc())
                .nest(INDENT)
                .group(),
            Self::App { func, arg, .. } => RcDoc::text("(")
                .append(func.to_doc())
                .append(RcDoc::space())
                .append(arg.to_doc())
                .append(RcDoc::text(")"))
                .group(),
            Self::Lambda { param, body, .. } => RcDoc::text(r"\")
                .append(param.name.to_string())
                .append(RcDoc::space())
                .append(RcDoc::text("->"))
                .append(RcDoc::space())
                .append(body.to_doc())
                .group(),
            Self::Let { binding, body, .. } => RcDoc::text("let")
                .append(RcDoc::space())
                .append(binding_to_doc(binding))
                .append(RcDoc::space())
                .append(RcDoc::text("in"))
                .append(RcDoc::line())
                .append(body.to_doc())
                .nest(INDENT)
                .group(),
            Self::Letrec { bindings, body, .. } => {
                RcDoc::text("letrec").append(RcDoc::space()).append(
                    RcDoc::concat(bindings.iter().enumerate().map(|(i, b)| {
                        binding_to_doc(b)
                            .append(if i < bindings.len() - 1 {
                                RcDoc::text(",").append(RcDoc::hardline())
                            } else {
                                RcDoc::nil()
                            })
                            .nest(("letrec".len() + 1) as isize)
                            .group()
                    }))
                    .append(RcDoc::line())
                    .nest(-2 * INDENT)
                    .append(RcDoc::text("in"))
                    .nest(INDENT)
                    .append(RcDoc::line())
                    .append(body.to_doc())
                    .nest(INDENT)
                    .group(),
                )
            }
        }
    }

    pub fn pretty(&self) -> String {
        let mut v = Vec::new();
        self.to_doc().render(WIDTH, &mut v).unwrap();
        String::from_utf8(v).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use insta::assert_snapshot;
    use std::str::FromStr;

    #[track_caller]
    fn test_pp(src: &str) {
        let expr = Expr::from_str(src).unwrap();
        assert_snapshot!(expr.pretty());
    }

    #[test]
    fn pp_lit() {
        test_pp("123");
    }

    #[test]
    fn pp_var() {
        test_pp("add");
    }

    #[test]
    fn pp_if() {
        test_pp("if true then 1 else 0");
        test_pp("if (if false then true else true) then (if false then 1 else 0) else (if false then 420 else 69)");
    }

    #[test]
    fn pp_app() {
        test_pp("if not false then 1 else 0");
        test_pp("if not (is_zero (add 1 1)) then 50 else 100");
    }

    #[test]
    fn pp_lambda() {
        test_pp(r"if not false then \x -> x else \x -> not x");
        test_pp(r"if not false then \a, b -> a else \x, y -> y");
    }

    #[test]
    fn pp_let() {
        test_pp(r"let x = 5 in x");
        test_pp(r"let id = \x -> x, first = \a, b -> a in id not (first true 1)");
    }

    #[test]
    fn pp_letrec() {
        test_pp(r"letrec f = \x -> f x in f 0");
        test_pp(r"letrec f = \x -> x, g = \y -> y in f g");
        test_pp(r"letrec f1 = \a -> a, f2 = \b -> b, f3 = \c -> c, f4 = \d -> d in f1 f2 f3 f4");
    }
}
