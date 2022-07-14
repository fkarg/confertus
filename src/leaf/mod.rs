use crate::traits::*;
use core::arch::x86_64::{_pdep_u64, _tzcnt_u64};
use either::{Left, Right};
use num::Integer;
use std::fmt;

use core::ops::*; // {BitAnd, BitOr, Shl, Shr, Sub, Not};

type Side<T> = either::Either<T, T>;
// type NumSize = u8;

/// Container type used to contain bits in [`Leaf`]. Sensible options are [`u64`] and [`u128`].
/// Might be replaced with custom implementation featuring higher bit container size later (e.g.
/// 4096, or dynamically dependent on total BitVector capacity).
///
///
/// Implementation of [`Leaf::select_pdep`] is dependent on actual type, as the `pdep` command does
/// not automatically extend to `u128`. I researched conditional compilation for a bit, but
/// couldn't figure out how to do that.
pub type LeafValue = u64;

pub const HALF: u32 = LeafValue::BITS / 2;

/// Leaf element of [`crate::DynamicBitVector`]. Next to its value ([`LeafValue`]) and bits used
/// inside (`nums`), it contains a reference to its parent [`crate::Node`].
///
/// Instance bit size: 17~25 bytes, depending on `LeafValue`
#[derive(PartialEq, Clone, Default)]
pub struct Leaf<T> {
    /// reference to parent [`crate::Node`] (8 byte)
    pub parent: usize, // 8 bytes
    /// container for actual bit values (8-16 byte)
    pub value: T, // 8~16 bytes
    /// number of bits used in `value`-container. Below `u128::BITS == 128`, so `u8::MAX = 255` is
    /// sufficient. (1 byte)
    pub nums: u8, // realistically below u128::BITS, so u8::MAX = 255 is sufficient. // 1 byte
}

impl<T> Leaf<T>
where
    T: StaticBitVec + fmt::Debug + BitContainer,
{
    // ACCESS

    /// Access bit value at position `index`
    ///
    /// # Panics
    /// If `index` > [`T::BITS`]
    #[inline]
    pub fn access(&self, index: usize) -> bool {
        (self.value >> index) & 1 == 1
    }

    /// Return full internal bit container
    #[inline]
    pub fn values(&self) -> T {
        self.value
    }
    // }

    // impl Leaf {
    // CONSTRUCTORS

    /// Constructs a new, empty `Leaf` with parent `parent`.
    #[inline]
    pub fn new(parent: usize) -> Self {
        Leaf {
            parent,
            value: 0,
            nums: 0,
        }
    }

    /// Cunstructs a new `Leaf` with parent `parent` and
    #[inline]
    pub fn create(parent: usize, value: T, nums: u8) -> Self {
        Leaf {
            parent,
            value,
            nums,
        }
    }

    // PUSH

    /// Appends bit to the end of `self.value`.
    ///
    /// # Errors
    /// If used capacity `nums` equals `T::BITS` bits before push (Leaf is full).
    pub fn push(&mut self, bit: bool) -> Result<(), &str> {
        if u32::from(self.nums) < T::BITS {
            unsafe {
                self.push_unchecked(bit);
            }
            Ok(())
        } else {
            Err("tried to push value to full Leaf")
        }
    }

    /// Unchecked version of [`Leaf::push`]
    ///
    /// # Safety
    /// Unchecked invariant: used capacity `nums` is less than the total capacity of
    /// `T::BITS` bits before push.
    #[inline]
    pub unsafe fn push_unchecked(&mut self, bit: bool) {
        self.value |= (bit as T) << self.nums;
        self.nums += 1;
    }

    // INSERT

    /// Insert `bit` at position `index` in [`Leaf`].
    ///
    /// # Errors
    /// When Leaf full or `index` out of bounds (`> self.nums` or `>= T::Bits`).
    pub fn insert(&mut self, index: usize, bit: bool) -> Result<(), &str> {
        if self.nums as u32 >= T::BITS {
            Err("No free capacity left")
        } else {
            unsafe { self.insert_unchecked(index, bit) };
            Ok(())
        }
    }

    /// Unchecked version of [`Leaf::insert`]
    ///
    /// # Safety
    /// Unchecked invariants:
    /// - `index < T::BITS`
    /// - `index <= self.nums`
    pub unsafe fn insert_unchecked(&mut self, index: usize, bit: bool) {
        // results in "attempt to subtract with overflow". TODO: debug sometime
        // results in "attempt to shift left with overflow". on insert 0 0 TOOD: debug sometime
        let lmask = T::MAX << (T::BITS - index as u32); // in- or excluding index here?
        let rmask = T::MAX >> (index as u32);
        self.value = (self.value & lmask) | ((bit as T) << index) | ((self.value & rmask) >> 1);
        self.nums += 1;
    }

    // DELETE

    /// Remove bit value at position `index`
    ///
    /// # Errors
    /// When Leaf empty or `index` out of bounds (`> self.nums` or `> T::BITS`).
    pub fn delete(&mut self, index: usize) -> Result<(), &str> {
        if self.is_empty() {
            Err("Tried to delete in empty leaf")
        } else {
            unsafe { self.delete_unchecked(index) };
            Ok(())
        }
    }

    /// Unchecked version of [`Leaf::delete`]
    ///
    /// # Safety
    /// List of unchecked invariants:
    /// - `index < T::BITS`
    /// - `index < self.nums`
    /// - `self.nums > 0`
    pub unsafe fn delete_unchecked(&mut self, index: usize) {
        let lmask = T::MAX << (T::BITS - index as u32);
        let rmask = T::MAX >> index as u32;
        self.value = ((self.value & lmask) >> 1) | (self.value & rmask);
        self.nums -= 1;
    }

    // ONES

    /// Returns number on-bits in `self.values`
    ///
    /// runtime complexity: O(1)
    #[inline]
    pub fn ones(&self) -> usize {
        self.value.count_ones() as usize
    }

    // NUMS

    /// Return used capacity `self.nums`
    #[inline]
    pub fn nums(&self) -> usize {
        self.nums.into()
    }

    // RANK

    /// Returns number of `bit`-values up to `index` in `self.value`
    ///
    /// runtime complexity: O(1)
    pub fn rank(&self, bit: bool, index: usize) -> usize {
        if bit {
            (self.value >> index as u32).count_ones() as usize
        } else {
            ((!self.value) >> index as u32).count_ones() as usize
        }
    }

    // SELECT

    /// ```text
    /// Algorithm for determining the position of the jth 1 in a machine word.
    /// ---
    /// 1: function PTSELECT(x, j)
    /// 2:     i ← SHIFTLEFT(1, j)
    /// 3:     p ← PDEP(i, x)
    /// 4:     return TZCNT(p)
    /// ```
    ///
    /// taken from <https://arxiv.org/pdf/1706.00990.pdf>.
    ///
    /// # Safety
    /// Only available for `x86_64`-based architecuters supporting feature sets `bmi1` and `bmi2`,
    /// which were both introduced by the fourth-generation intel
    /// [haswell](https://en.wikipedia.org/wiki/Haswell_(microarchitecture)) architecture nine
    /// years ago.
    // #[target_feature(enable = "bmi1")]
    // #[target_feature(enable = "bmi2")]
    pub unsafe fn select_pdep(&self, bit: bool, n: usize) -> usize {
        let array = if bit { self.value } else { !self.value };
        // self.value is u64
        _tzcnt_u64(_pdep_u64(1 << n, array as u64)) as usize

        // // self.value is u128
        // if n < 64 {
        //     _tzcnt_u64(_pdep_u64(1 << n, array as u64)) as usize
        // } else {
        //     _tzcnt_u64(_pdep_u64(1 << n, (array >> 64) as u64)) as usize + 64
        // }

        // yes, comment / uncomment ... no idea how to do conditional compilation
    }

    /// Return index of `n`-th `bit`-value in `self.value`
    ///
    /// Automatically uses [`Leaf::select_pdep`] if supported by architecture, but has fallback
    /// implementation if not.
    pub fn select(&self, bit: bool, n: usize) -> usize {
        if std::is_x86_feature_detected!("bmi2") && std::is_x86_feature_detected!("bmi1") {
            unsafe { self.select_pdep(bit, n) }
        } else {
            // fallback for non-bmi2-x86 architectures

            // Scan the leaf from left to right and look for the bit of
            // respective rank.
            let mut pos = 0;
            let mut i = n;
            for shift in (0..T::BITS).rev() {
                if (((self.value >> shift) & 1) != 0) == bit {
                    if i == 0 {
                        return pos;
                    }
                    i -= 1;
                    pos += 1;
                }
            }
            panic!("`n`-th `bit`-value not found in this Leaf.")
        }
    }

    // FLIP

    /// Flip bit at position `index`
    ///
    /// runtime complexity: O(1)
    #[inline]
    pub fn flip(&mut self, index: usize) {
        self.value ^= 1 << index;
    }

    // LENGTH

    /// Return used length of `self.value` (== `self.nums`)
    #[inline]
    pub fn len(&self) -> usize {
        self.nums.into()
    }

    /// If the Leaf has active values
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.nums == 0
    }

    // SPLIT

    /// Return full second/left half of `Leaf`-values, and remove them from `self`, to be inserted
    /// to a Leaf right of `self`.
    pub fn split_to_right(&mut self) -> T {
        // save the second/left half of self.value temporarily. zero out the rest.
        let ret = self.value.rotate_right(HALF) << HALF;
        // keep first half of self.value, zero out the others.
        self.value = (self.value << HALF) >> HALF;
        // Size is now reduced to exactly half size.
        self.nums = HALF as u8;
        // return second half shifted to the right.
        ret >> HALF
    }

    /// Return full first/right half of `Leaf`-values, and remove them from `self`, to be inserted
    /// to a Leaf left of `self`.
    pub fn split_to_left(&mut self) -> T {
        // save the first/right half of self.value temporarily. zero out the rest.
        let ret = (self.value << HALF) >> HALF;
        // keep second half of self.value, zero out the others.
        self.value = self.value >> HALF;
        // Size is now reduced by half size.
        self.nums -= HALF as u8;
        // return first half
        ret
    }

    // MERGE / EXTEND

    /// Extend T container with given values on given side by `num`.
    ///
    /// `Left` side means that values are originally of lower index than current leaf, thus
    /// inserting them to the beginning.
    ///
    /// `Right` side means that values are originally of higher index than current leaf, thus
    /// inserting them at the end.
    #[inline]
    pub fn extend(&mut self, values: Side<T>, nums: u8) {
        match values {
            Right(v) => self.extend_from(Leaf::create(0, v, nums)),
            Left(v) => self.prepend(Leaf::create(0, v, nums)),
        }
    }

    /// Extend T container with values from other Leaf with originally higher index.
    /// Appends new values to end.
    #[inline]
    pub fn extend_from(&mut self, ref leaf: Leaf<T>) {
        self.value |= leaf.values() << leaf.nums();
        self.nums += leaf.nums() as u8;
    }

    /// Prepend other values to existing values in T container. Current values are moved
    /// later.
    #[inline]
    pub fn prepend(&mut self, ref leaf: Leaf<T>) {
        self.value = leaf.value << leaf.nums();
        self.value |= leaf.values();
        self.nums += leaf.nums() as u8;
    }
}

mod trait_impls;

#[cfg(test)]
mod tests;
