use crate::util::counter::{Counter, FromId};

#[derive(Debug, Copy, Clone)]
pub struct Gensym {
    counter: Counter<u32>,
    prefix: &'static str,
}

impl Gensym {
    pub fn new(prefix: &'static str) -> Self {
        Self {
            counter: Counter::new(),
            prefix,
        }
    }

    pub fn current(&self) -> String {
        format!("{}{}", self.prefix, self.counter.current())
    }

    pub fn next(&mut self) -> String {
        let x = self.counter.next();
        format!("{}{}", self.prefix, x)
    }

    pub fn reset(&mut self) {
        self.counter.reset()
    }
}
