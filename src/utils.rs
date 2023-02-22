use core::num;
use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
    ops::Add,
    u8,
};
use uuid::Uuid;

use crate::{traits::*, structs::bit_nodes::BitNodes};

/// Returns subset of power set of given id's with the condition that each set returned has dims number
/// of elements.
pub fn power_set<N: HgNode>(v: Vec<N>, dims: usize) -> HashSet<Vec<N>> {
    if dims == 0 {
        return HashSet::new();
    }
    if dims == 1 {
        return HashSet::from([Vec::new(), v]);
    }
    if dims > v.len() {
        let l = v.len();
        return power_set(v, l);
    }
    let copy = v.clone();
    let smallers = power_set(copy, dims - 1);
    let mut ret = HashSet::new();
    for node in v {
        for mut smaller in smallers.clone() {
            if smaller.contains(&node) == false {
                smaller.push(node.clone());
                smaller.sort();
                ret.insert(smaller);
            }
        }
    }
    ret
}

#[derive(Debug, Clone)]
pub struct PowerSetBits<const M: usize> {
    pub bits: [u8; M],
}

impl<const M: usize> PowerSetBits<M> {
    pub fn num_ones(&self) -> u32 {
        self.bits.iter().fold(0, |tot, b| tot + b.count_ones())
    }
    pub fn rotate_left_one(&mut self) {
        let leading_bit = self.bits[0] & 0b1000_0000_u8;
        for ix in 0..M {
            if ix < M - 1 {
                self.bits[ix] = self.bits[ix] << 1;
                let next_leading_bit = self.bits[ix + 1] & 0b1000_0000_u8;
                self.bits[ix] = self.bits[ix] | next_leading_bit.reverse_bits();
            } else {
                self.bits[ix] = self.bits[ix] << 1;
                self.bits[ix] = self.bits[ix] | leading_bit.reverse_bits();
            }
        }
    }
    pub fn rotate_right_one(&mut self) {
        let trailing_bit = self.bits[M - 1] & 0b0000_0001_u8;
        for ix in 0..M {
            if ix < M - 1 {
                self.bits[M - 1 - ix] = self.bits[M - 1 - ix] >> 1;
                let next_trailing_bit = self.bits[M - 1 - ix - 1] & 0b0000_0001_u8;
                self.bits[M - 1 - ix] = self.bits[M - 1 - ix] | next_trailing_bit.reverse_bits();
            } else {
                self.bits[0] = self.bits[0] >> 1;
                self.bits[0] = self.bits[0] | trailing_bit.reverse_bits();
            }
        }
    }

    // TODO: Implement up to 8 shift, then use that as the base instead of one
    pub fn rotate_left(&mut self, l: u32) {
        for _ in 0..l {
            self.rotate_left_one();
        }
    }

    // TODO: Implement up to 8 shift, then use that as the base instead of one
    pub fn rotate_right(&mut self, r: u32) {
        for _ in 0..r {
            self.rotate_right_one();
        }
    }

    pub fn leading_ones(&self) -> u32 {
        let mut ret = 0;
        for ix in 0..M {
            if self.bits[ix].leading_ones() < 8 {
                ret += self.bits[ix].leading_ones();
                return ret;
            } else {
                ret += 8;
            }
        }
        ret
    }

    pub fn leading_zeros(&self) -> u32 {
        let mut ret = 0;
        for ix in 0..M {
            if self.bits[ix].leading_zeros() < 8 {
                ret += self.bits[ix].leading_zeros();
                return ret;
            } else {
                ret += 8;
            }
        }
        ret
    }

    pub fn flip_kth_one(&mut self, k: u32) {
        if k == 0 {
            return;
        }
        let mut total_rotations_left = 0;
        let mut ones_passed = 0;
        let mut num_zeros = self.leading_zeros();
        self.rotate_left(num_zeros);
        total_rotations_left += num_zeros;
        while ones_passed < k - 1 {
            if self.leading_zeros() == 0 {
                let num_ones = u32::min(k - ones_passed - 1, self.leading_ones());
                self.rotate_left(num_ones);
                ones_passed += num_ones;
                total_rotations_left += num_ones;
            } else {
                num_zeros = self.leading_zeros();
                self.rotate_left(num_zeros);
                total_rotations_left += num_zeros;
            }
        }
        num_zeros = self.leading_zeros();
        self.rotate_left(num_zeros);
        total_rotations_left += num_zeros;
        self.bits[0] = self.bits[0] ^ 0b_1000_0000_u8;
        self.rotate_right(total_rotations_left);
    }

    pub fn get_nodes_set(&self) -> HashSet<[u8; M]> {
        let mut ret = HashSet::new();
        let mut tmp = self.clone();
        for ix in 0..self.num_ones() {
            let mut pb = PowerSetBits {bits: [0; M]};
            pb.flip_kth_bit(tmp.leading_zeros());
            tmp.flip_kth_one(1);
            ret.insert(pb.bits);
        }
        ret
    }

    pub fn flip_kth_bit(&mut self, k: u32) {
        self.rotate_left(k);
        self.bits[0] = self.bits[0] ^ 0b_1000_0000_u8;
        self.rotate_right(k);
    }

    pub fn is_zero(&self) -> bool {
        let mut ret = true;
        for ix in 0..M {
            if self.bits[ix] > 0 {
                ret = false;
                break;
            }
        }
        ret
    }

    /// TODO: This is currently broken, needs fixing.
    pub fn base_rotation_left(&mut self, bit_shift: u32) {
        let bit_shift_to_bit_flag = HashMap::from([
            (1_u32, 0b1000_0000_u8),
            (2_u32, 0b1100_0000_u8),
            (3_u32, 0b1110_0000_u8),
            (4_u32, 0b1111_0000_u8),
            (5_u32, 0b1111_1000_u8),
            (6_u32, 0b1111_1100_u8),
            (7_u32, 0b1111_1110_u8),
            (8_u32, 0b1111_1111_u8),
        ]);
        if let Some(bit_flag) = bit_shift_to_bit_flag.get(&bit_shift) {
            let leading_bits = self.bits[0] & bit_flag;
            for ix in 0..M {
                if ix < M - 1 {
                    self.bits[ix] = self.bits[ix] << bit_shift;
                    let next_leading_bit = self.bits[ix + 1] & bit_flag;
                    self.bits[ix] = self.bits[ix] | next_leading_bit.reverse_bits();
                } else {
                    self.bits[ix] = self.bits[ix] << bit_shift;
                    self.bits[ix] = self.bits[ix] | leading_bits.reverse_bits();
                }
            }
        }
    }

    pub fn to_bit_nodes(self) -> BitNodes<M> {
        BitNodes { bits: self.bits }
    }

    pub fn clear(&mut self) {
        self.bits = [0; M];
    }

    pub fn flip_all_bits(&mut self) {
        for ix in 0..M {
            self.bits[ix] = self.bits[ix] ^ u8::MAX;
        }
    }

    pub fn print_formatted(&self) {
        let mut buf = String::new();
        for ix in 0..M {
            write!(&mut buf, "_{:#010b}", self.bits[ix]);
        }
        println!("{:}", buf);
    }
}

mod test {
    use uuid::Uuid;

    use super::{power_set, PowerSetBits};

    #[test]
    fn test_leading_ones() {
        let og = PowerSetBits {
            bits: [
                0b_0111_1111_u8,
                0b_1111_1111_u8,
                0b_1111_1111_u8,
                0b_1111_1111_u8,
                0b_1100_0000_u8,
            ],
        };
        println!("leading ones: {:}", og.leading_ones());
    }

    #[test]
    fn test_leading_zeros() {
        let og = PowerSetBits {
            bits: [
                0b_1010_0000_u8,
                0b_0000_0000_u8,
                0b_0000_0000_u8,
                0b_0000_0000_u8,
                0b_0001_0000_u8,
                0b_1111_1111_u8,
                0b_1111_1111_u8,
                0b_1111_1111_u8,
                0b_1100_0000_u8,
            ],
        };
        println!("leading zeros: {:}", og.leading_zeros());
    }

    #[test]
    fn test_pb_flipper() {
        let mut og = PowerSetBits {
            bits: [
                0b_0110_1001_u8,
                0b_1001_0110_u8,
                0b_0001_0001_u8,
                0b_1010_1010_u8,
            ],
        };
        for k in 0..7 {
            println!("{:}", "#".repeat(50));
            println!("k = {:}", k);
            og.print_formatted();
            let mut pb = og.clone();
            pb.flip_kth_one(k);
            pb.print_formatted();
        }
    }

    #[test]
    fn test_pb_flips_all_ones() {
        let mut pb = PowerSetBits {
            bits: [u8::MAX; 4]
        };
        pb.flip_kth_one(1);
        pb.flip_kth_one(2);
        pb.flip_kth_one(3);
        pb.print_formatted();
        pb.flip_all_bits();
        pb.print_formatted();
    }

    #[test]
    fn test_power_set_bits_rotation_simple() {
        let mut pb = PowerSetBits {
            bits: [
                0b_0110_1001_u8,
                0b_1001_0110_u8,
                0b_0001_0001_u8,
                0b_1010_1010_u8,
            ],
        };
        let og = pb.clone();
        pb.print_formatted();
        pb.rotate_left(4);
        pb.print_formatted();
        println!("now rotate right");
        pb.rotate_right(4);
        pb.print_formatted();
        og.print_formatted();
    }

    #[test]
    fn test_power_set_bits_base_rotation() {
        let mut pb = PowerSetBits {
            bits: [
                0b_0110_1001_u8,
                0b_1001_0110_u8,
                0b_0001_0001_u8,
                0b_1010_1010_u8,
            ],
        };
        pb.print_formatted();
        pb.base_rotation_left(4);
        pb.print_formatted();
    }

    #[test]
    fn test_power_set() {
        let ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        println!("power set:\n{:?}", power_set(ids, 2));
    }
}
