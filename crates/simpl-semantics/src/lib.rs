#![feature(box_syntax)]
#![feature(box_patterns)]

use std::{collections::HashMap, hash::Hash};

pub mod infer;
mod ty;

#[cfg(test)]
mod test;

trait Union {
    fn union(&self, other: &Self) -> Self;
}

/// Implement union for HashMap such that the value in `self` is used over the
/// value in `other` in the event of a collision.
impl<K, V> Union for HashMap<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    fn union(&self, other: &Self) -> Self {
        let mut res = self.clone();
        for (key, value) in other {
            res.entry(key.clone()).or_insert_with(|| value.clone());
        }
        res
    }
}
