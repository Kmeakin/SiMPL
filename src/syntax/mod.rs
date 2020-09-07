pub mod ast;

#[cfg(test)]
mod test;

pub type ParseError<'a> =
    lalrpop_util::ParseError<usize, lalrpop_util::lexer::Token<'a>, &'static str>;

pub fn parse(src: &str) -> Result<ast::Expr, ParseError> {
    let parser = grammar::ExprParser::new();
    parser.parse(src)
}

lalrpop_mod!(
    #[allow(dead_code, clippy::all, clippy::pedantic, clippy::nursery)]
    pub grammar, "/syntax/grammar.rs"
);
