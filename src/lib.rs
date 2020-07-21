#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![feature(box_syntax, box_patterns)]

pub mod codegen;
pub mod syntax;
pub mod types;
pub(crate) mod util;

#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(
    #[allow(dead_code, clippy::all, clippy::pedantic, clippy::nursery)]
    pub grammar
);
