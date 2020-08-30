use super::{
    closure::{CExpr, LetBinding, Param, Symbol, Type},
    gensym::Gensym,
    util::FreeVars,
};

#[derive(Debug, Clone)]
pub struct Compiler {
    preamble: String,
    gen: Gensym,
}

const C_RESERVED_IDENTS: &[&'static str] = &[
    "auto",
    "_Bool",
    "break",
    "case",
    "char",
    "_Complex",
    "const",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extern",
    "float",
    "for",
    "goto",
    "if",
    "_Imaginary",
    "inline",
    "int",
    "long",
    "register",
    "restrict",
    "return",
    "short",
    "signed",
    "sizeof",
    "static",
    "struct",
    "switch",
    "typedef",
    "union",
    "unsigned",
    "void",
    "volatile",
    "while",
];

fn is_c_ident_char(c: char) -> bool {
    match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => true,
        _ => false,
    }
}

fn c_ident(ident: &str) -> String {
    let ident = ident.replace(|c| !is_c_ident_char(c), "_");
    if C_RESERVED_IDENTS.contains(&ident.as_str()) {
        format!("{}_", ident)
    } else {
        ident
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            preamble: "".into(),
            gen: Gensym::new(""),
        }
    }

    fn compile_mk_closure(
        &mut self,
        closure_id: u32,
        ty: Type,
        free_vars: FreeVars,
        param: Param,
        body: CExpr,
    ) -> String {
        let body_ty = body.ty();
        let body = self.compile_expr(body);

        let closure_ty = format!("closure{}", closure_id);

        let closure_fields = free_vars
            .iter()
            .map(|(name, ty)| (c_ident(&name.to_string()), ty.to_string()))
            .collect::<Vec<_>>();

        let struct_closure = &format!(
            "\
typedef struct {{
    {fields}
}} {closure_ty};",
            fields = closure_fields
                .iter()
                .map(|(name, ty)| format!("{} {};", ty, name))
                .collect::<Vec<_>>()
                .join("\n"),
        );

        let mk_closure = format!(
            "\
{closure_ty} mk_{closure_ty}({args}) {{
    return ({closure_ty}) {{{inits}}};
}}",
            args = closure_fields
                .iter()
                .map(|(name, ty)| format!("{} {}", ty, name))
                .collect::<Vec<_>>()
                .join(","),
            inits = closure_fields
                .iter()
                .map(|(name, ty)| format!(".{name} = {name}"))
                .collect::<Vec<_>>()
                .join(",")
        );

        let call_closure = format!(
            "\
{body_ty} call_{closure_ty}({closure_ty} c) {{
    return {body};
}}"
        );

        self.preamble.push_str(&format!(
            "{struct_closure}\n\n{mk_closure}\n\n{call_closure}"
        ));

        format!(
            "mk_{closure_ty}({args})",
            args = closure_fields
                .iter()
                .map(|(name, _)| format!("{name}"))
                .collect::<Vec<_>>()
                .join(","),
        )
    }

    fn compile_expr(&mut self, expr: CExpr) -> String {
        match expr {
            CExpr::Lit { val, .. } => val.to_string(),
            CExpr::Var { name, .. } => c_ident(&name.to_string()),
            CExpr::If {
                test, then, els, ..
            } => format!(
                "({}) ? ({}) : ({})",
                self.compile_expr(*test),
                self.compile_expr(*then),
                self.compile_expr(*els)
            ),
            CExpr::Let { binding, body, .. } => {
                let binding_ty = match *binding.val {
                    CExpr::MkClosure { closure_id, .. } => format!("closure{}", closure_id),
                    _ => format!("{}", binding.ty),
                };
                format!(
                    "{} {} = {};\n{}",
                    binding_ty,
                    c_ident(&binding.name.to_string()),
                    self.compile_expr(*binding.val),
                    self.compile_expr(*body)
                )
            }
            CExpr::Letrec { .. } => todo!(),
            CExpr::MkClosure {
                closure_id,
                ty,
                free_vars,
                param,
                body,
            } => self.compile_mk_closure(closure_id, ty, free_vars, param, *body),
            CExpr::AppClosure { .. } => format!("call_closure()"),
            CExpr::EnvRef { name, .. } => format!("c.{name}"),
        }
    }
}

pub fn compile_prog(expr: CExpr) -> String {
    let mut compiler = Compiler::new();
    let body = compiler.compile_expr(expr);
    let preamble = compiler.preamble;

    format!(
        "\
#include \"prelude.h\"
{}
int main (int argc, char** argv) {{
    {};
    return 0;
}}",
        preamble, body,
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        codegen::{anf::normalize_expr, closure::closure_convert},
        hir::Expr,
        types::{self, parse_and_type},
    };
    use std::str::FromStr;

    #[track_caller]
    fn test_compile(src: &str) {
        let expr = parse_and_type(src);
        let anf = normalize_expr(expr);
        let clo_conv = closure_convert(&anf);
        let compiled = compile_prog(clo_conv);
        insta::assert_snapshot!(compiled)
    }

    #[test]
    fn test_compile_lit() {
        test_compile(r"5");
        test_compile(r"1.5");
        test_compile(r"true");
    }

    #[test]
    fn test_compile_if() {
        test_compile(r"if true then 1 else 0");
        test_compile(
            r"
if (if false then true else false)
    then (if true then 1 else 0)
    else (if true then 0 else 1)
",
        );
    }

    #[test]
    fn test_compile_mk_closure() {
        test_compile(
            r"
let x = 5,
    capture_x = \ignored -> x
in capture_x 0
",
        );
    }
}
