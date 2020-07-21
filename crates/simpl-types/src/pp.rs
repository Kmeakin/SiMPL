use crate::ast::TypedExpr as Expr;
use pretty::{Doc, RcDoc};

const INDENT: isize = 4;
const WIDTH: usize = 40;

impl Expr {
    pub fn to_doc(&self) -> RcDoc<()> {
        match self {
            Self::Lit { ty, val } => RcDoc::as_string(val),
            Self::Var { ty, name } => RcDoc::as_string(name),
            Self::If {
                ty,
                test,
                then_branch,
                else_branch,
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
            Self::App { ty, func, arg } => RcDoc::text("(")
                .append(func.to_doc())
                .append(RcDoc::space())
                .append(arg.to_doc())
                .append(RcDoc::text(")"))
                .group(),
            Self::Lambda { ty, param, body } => RcDoc::text(r"\")
                .append(param.name.clone())
                .append(RcDoc::space())
                .append(RcDoc::text("->"))
                .append(RcDoc::space())
                .append(body.to_doc())
                .group(),
            Self::Let { ty, binding, body } => RcDoc::text("let")
                .append(RcDoc::space())
                .append(binding.name.clone())
                .append(RcDoc::space())
                .append(RcDoc::text("="))
                .append(RcDoc::space())
                .append(binding.val.to_doc())
                .append(RcDoc::space())
                .append(RcDoc::text("in"))
                .append(RcDoc::line())
                .append(body.to_doc())
                .nest(INDENT)
                .group(),
            Self::Letrec { ty, bindings, body } => todo!(),
        }
    }

    pub fn pretty(&self) -> String {
        let mut v = Vec::new();
        self.to_doc().render(WIDTH, &mut v).unwrap();
        String::from_utf8(v).unwrap()
    }
}

mod test {
    use super::*;
    use crate::parse_and_type;
    use insta::assert_snapshot;

    #[track_caller]
    fn test_pp(src: &str) {
        let expr = crate::parse_and_type(src);
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
}
