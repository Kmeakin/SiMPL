use super::{CExpr, LetBinding};
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

impl CExpr {
    pub fn to_doc(&self) -> RcDoc<()> {
        match self {
            Self::Lit { val, .. } => RcDoc::as_string(val),
            Self::Var { name, .. } => RcDoc::as_string(name),
            Self::If {
                test, then, els, ..
            } => RcDoc::text("if")
                .append(RcDoc::space())
                .append(test.to_doc())
                .append(RcDoc::line())
                .append(RcDoc::text("then"))
                .append(RcDoc::space())
                .append(then.to_doc())
                .append(RcDoc::line())
                .append(RcDoc::text("else"))
                .append(RcDoc::space())
                .append(els.to_doc())
                .nest(INDENT)
                .group(),
            Self::AppClosure { func, arg, .. } => RcDoc::text("(")
                .append(func.to_doc())
                .append(RcDoc::space())
                .append(arg.to_doc())
                .append(RcDoc::text(")"))
                .group(),
            Self::MkClosure { param, body, .. } => RcDoc::text(r"\")
                .append(param.name.to_string())
                .append(RcDoc::space())
                .append(RcDoc::text("->"))
                .append(RcDoc::space())
                .append(body.to_doc())
                .group(),
            Self::EnvRef { name, .. } => RcDoc::text("(")
                .append("envRef")
                .append(RcDoc::space())
                .append(name.to_string())
                .append(RcDoc::text(")"))
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
