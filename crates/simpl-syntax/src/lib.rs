#![feature(box_syntax)]

#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub grammar);

pub mod ast;

#[cfg(test)]
mod test;
