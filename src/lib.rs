#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![feature(box_syntax, box_patterns)]
#![feature(format_args_capture)]
#![feature(bindings_after_at)]
#![feature(move_ref_pattern)]
#![allow(clippy::must_use_candidate)]
#![allow(dead_code)]

pub mod codegen;
pub mod hir;
pub mod syntax;
pub mod types;
pub(crate) mod util;

#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(
    #[allow(dead_code, clippy::all, clippy::pedantic, clippy::nursery)]
    pub grammar
);
