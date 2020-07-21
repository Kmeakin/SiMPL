pub use crate::grammar;
pub mod ast;

#[cfg(test)]
mod test;

pub type ParseError<'a> =
    lalrpop_util::ParseError<usize, lalrpop_util::lexer::Token<'a>, &'static str>;

pub fn parse(src: &str) -> Result<ast::Expr, ParseError> {
    let parser = grammar::ExprParser::new();
    parser.parse(src)
}