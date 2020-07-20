#![allow(dead_code)]
#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(never_type)]

use std::marker::PhantomData;

pub mod ast;
mod constraint;
mod subst;
pub mod ty;

#[derive(Debug, Copy, Clone, Default)]
pub struct IdGen<T>
where
    T: FromId,
{
    counter: u32,
    _phantom: PhantomData<T>,
}

impl<T: FromId> IdGen<T> {
    pub fn new() -> Self {
        Self {
            counter: 0,
            _phantom: PhantomData,
        }
    }

    fn next_id(&mut self) -> u32 {
        let x = self.counter;
        self.counter += 1;
        x
    }

    pub fn current_id(&self) -> u32 {
        self.counter
    }

    pub fn current(&self) -> T {
        T::from_id(self.current_id())
    }

    pub fn next(&mut self) -> T {
        T::from_id(self.next_id())
    }
}

pub trait FromId {
    fn from_id(id: u32) -> Self;
}

impl FromId for u32 {
    fn from_id(id: u32) -> Self {
        id
    }
}
