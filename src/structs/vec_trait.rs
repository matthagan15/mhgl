use std::ops::{Add, AddAssign, Mul, MulAssign};
use crate::structs::EdgeWeight;

trait HgVector:
    PartialEq + Eq + Clone + Add + AddAssign + Mul<EdgeWeight> + MulAssign<EdgeWeight>
{
    type Basis;
    fn zero() -> Self;
    
}