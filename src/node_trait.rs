use core::panic;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;

#[cfg(feature = "uuid")]
use uuid::Uuid;

/// A trait used to mark which types can be used as a NodeID or EdgeID, currently only use `u8` through `u128` and `Uuid` for `KVGraph`.
pub trait HgNode:
    Debug + Eq + PartialEq + Hash + PartialOrd + Ord + Clone + Copy + Serialize + FromStr + ToString
{
    fn max_number() -> Self;
    fn zero() -> Self;
    fn plus_one(&mut self);
}

#[cfg(feature = "uuid")]
impl HgNode for Uuid {
    fn max_number() -> Self {
        Uuid::from_u128(u128::MAX)
    }

    fn zero() -> Self {
        Uuid::nil()
    }

    fn plus_one(&mut self) {
        panic!("You should never use this.")
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
