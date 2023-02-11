use serde::Serialize;
use std::hash::Hash;
use uuid::Uuid;

pub trait HgNode: Eq + PartialEq + Hash + PartialOrd + Ord + Clone + Copy + Serialize {}

impl HgNode for Uuid {}
impl HgNode for u128 {}
impl HgNode for u64 {}
impl HgNode for u32 {}
impl HgNode for u16 {}
impl HgNode for u8 {}
