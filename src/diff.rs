#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use super::leaf::LeafValue;
use std::ops::Add;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Diff {
    pub balance: i8,
    pub size: isize,
    pub nums: isize, // might not be needed?
    pub ones: isize, // might not be needed?
}

impl Add for Diff {
    type Output = Diff;

    fn add(self, other: Diff) -> Self::Output {
        Diff {
            balance: self.balance + other.balance,
            size: self.size + other.size,
            nums: self.nums + other.nums,
            ones: self.ones + other.ones,
        }
    }
}

impl Diff {
    #[inline]
    pub fn new() -> Self {
        Diff {
            balance: 0,
            size: 0,
            nums: 0,
            ones: 0,
        }
    }

    /// insertion in right subtree
    #[inline]
    pub fn insert_right() -> Self {
        Diff::default()
    }

    /// insertion in left subtree
    pub fn insert_left(bit: bool) -> Self {
        let ones = if bit { 1 } else { 0 };
        Diff {
            nums: 1,
            ones,
            ..Diff::default()
        }
    }

    /// Node creation in right subtree: balance is shifted +1
    #[inline]
    pub fn create_right_node() -> Self {
        Diff {
            balance: 1,
            ..Diff::default()
        }
    }

    /// Leaf creation in right subtree: balance is shifted +1
    #[inline]
    pub fn create_right_leaf() -> Self {
        Diff {
            balance: 1,
            size: LeafValue::BITS as isize,
            ..Diff::default()
        }
    }

    /// Balance-diff of moving a child from right to left is always 1
    /// (if it weren't true, ....
    #[inline]
    pub fn move_child_right_to_left() -> Self {
        Diff {
            balance: -1,
            ..Diff::default()
        }
    }

    pub fn insert_node_right() -> Self {
        Diff {
            balance: 1,
            ..Diff::default()
        }
    }
}
