use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;

use uuid::Uuid;

/// # Nodes
///
pub trait HgNode:
    Debug + Eq + PartialEq + Hash + PartialOrd + Ord + Clone + Copy + Serialize + FromStr
{
    fn max_number() -> Self;
    fn zero() -> Self;
    fn plus_one(&mut self);
}

impl HgNode for Uuid {
    fn max_number() -> Self {
        Uuid::from_u128(u128::MAX)
    }

    fn zero() -> Self {
        Uuid::nil()
    }

    fn plus_one(&mut self) {
        let as_u128 = self.as_u128();
        *self = Uuid::from_u128(as_u128 + 1);
    }
}

impl HgNode for u128 {
    fn max_number() -> Self {
        u128::MAX
    }

    fn zero() -> Self {
        0
    }

    fn plus_one(&mut self) {
        *self = *self + 1;
    }
}
impl HgNode for u64 {
    fn max_number() -> Self {
        u64::MAX
    }

    fn zero() -> Self {
        0
    }

    fn plus_one(&mut self) {
        *self = *self + 1;
    }
}
impl HgNode for u32 {
    fn max_number() -> Self {
        u32::MAX
    }

    fn zero() -> Self {
        0
    }

    fn plus_one(&mut self) {
        *self = *self + 1;
    }
}
impl HgNode for u16 {
    fn max_number() -> Self {
        u16::MAX
    }

    fn zero() -> Self {
        0
    }

    fn plus_one(&mut self) {
        *self = *self + 1;
    }
}
impl HgNode for u8 {
    fn max_number() -> Self {
        u8::MAX
    }

    fn zero() -> Self {
        0
    }

    fn plus_one(&mut self) {
        *self = *self + 1;
    }
}
