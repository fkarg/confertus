use crate::traits::*;
use either::{Left, Right};
use std::fmt;

type Side<T> = either::Either<T, T>;
// type NumSize = u8;

/// Container type used to contain bits in [`Leaf`]. Sensible options are [`u64`] and [`u128`].
/// Might be replaced with custom implementation featuring higher bit container size later (e.g.
/// 4096, or dynamically dependent on total BitVector capacity).
pub type LeafValue = u64;

pub const HALF: u32 = LeafValue::BITS / 2;

/// Leaf element of [`crate::DynamicBitVector`], particularly implementing the traits
/// [`StaticBitVec`] and [`DynBitVec`].
/// Next to its value ([`LeafValue`]) and field for capacity used inside (`nums`), it contains a
/// reference to its parent [`crate::Node`].
///
/// Instance bit size: 17~25 bytes, depending on `LeafValue`
#[derive(PartialEq, Clone, Default)]
pub struct Leaf {
    /// reference to parent [`crate::Node`] (8 byte)
    pub parent: usize, // 8 bytes
    /// container for actual bit values (8-16 byte)
    pub value: LeafValue, // 8~16 bytes
    /// number of bits used in `value`-container. Up to `u128::BITS == 128`, so `u8::MAX = 255` is
    /// sufficient. (1 byte)
    pub nums: u8, // realistically below u128::BITS, so u8::MAX = 255 is sufficient. // 1 byte
}

impl Leaf {
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

    /// Cunstructs a new `Leaf` with parent `parent`, container [`LeafValue`] and size `nums`.
    #[inline]
    pub fn create(parent: usize, value: LeafValue, nums: u8) -> Self {
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
    /// If used capacity `nums` equals `LeafValue::BITS` bits before push (Leaf is full).
    pub fn push(&mut self, bit: bool) -> Result<(), &str> {
        if u32::from(self.nums) < LeafValue::BITS {
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
    /// Unchecked invariant:
    /// - `self.nums < LeafValue::BITS`
    #[inline]
    pub unsafe fn push_unchecked(&mut self, bit: bool) {
        self.value |= (bit as LeafValue) << self.nums;
        self.nums += 1;
    }

    // INSERT

    /// Unchecked version of [`Leaf::insert`]
    ///
    /// # Safety
    /// Unchecked invariants:
    /// - `index <= self.nums`
    ///     (and, by extension)
    /// - `index < LeafValue::BITS`
    pub unsafe fn insert_unchecked(&mut self, index: usize, bit: bool) {
        // results in "attempt to shift left with overflow" in line+4. TODO: debug sometime
        // probably in left shift with index, but then index is 'broken'?
        //
        // shift 'higher' indexed values on the left one to the left, and keep 'lower' indexed
        // values to the right there.
        let lmask = LeafValue::MAX.overflowing_shl(index as u32).0; // in- or excluding index here?
        let rmask = !lmask; // right side mask is just left shift mask with bits flipped
        self.value =
            ((self.value & lmask) << 1) | ((bit as LeafValue) << index) | (self.value & rmask);
        self.nums += 1;
    }

    // DELETE

    /// Unchecked version of [`Leaf::delete`]
    ///
    /// # Safety
    /// List of unchecked invariants:
    /// - `self.nums > 0`
    /// - `index < self.nums`
    ///     (and, by extension)
    /// - `index < LeafValue::BITS`
    pub unsafe fn delete_unchecked(&mut self, index: usize) {
        let lmask = LeafValue::MAX.overflowing_shl(index as u32).0;
        let rmask = !lmask;
        // move left mask one more position to the left (to exclude bit to delete), and then move
        // one position to the right (to overwrite bit to delete).
        self.value = ((self.value & (lmask << 1)) >> 1) | (self.value & rmask);
        self.nums -= 1;
    }

    // SPLIT

    /// Return full second/left half of `Leaf`-values, and remove them from `self`, to be inserted
    /// to a Leaf right of `self`.
    pub fn split_to_right(&mut self) -> LeafValue {
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
    pub fn split_to_left(&mut self) -> LeafValue {
        // save the first/right half of self.value temporarily. zero out the rest.
        let ret = (self.value << HALF) >> HALF;
        // keep second half of self.value, zero out the others.
        self.value >>= HALF;
        // Size is now reduced by half size.
        self.nums -= HALF as u8;
        // return first half
        ret
    }

    // MERGE / EXTEND

    /// Extend LeafValue container with given values on given side by `num`.
    ///
    /// `Left` side means that values are originally of lower index than current leaf, thus
    /// inserting them to the beginning.
    ///
    /// `Right` side means that values are originally of higher index than current leaf, thus
    /// inserting them at the end.
    #[inline]
    pub fn extend(&mut self, values: Side<LeafValue>, nums: u8) {
        match values {
            Right(v) => self.extend_from(&Leaf::create(0, v, nums)),
            Left(v) => self.prepend(&Leaf::create(0, v, nums)),
        }
    }

    /// Extend LeafValue container with values from other Leaf with originally higher index.
    /// Appends new values to end.
    #[inline]
    pub fn extend_from(&mut self, leaf: &Leaf) {
        self.value |= leaf.values() << leaf.nums();
        self.nums += leaf.nums() as u8;
    }

    /// Prepend other values to existing values in LeafValue container. Current values are moved
    /// later.
    #[inline]
    pub fn prepend(&mut self, leaf: &Leaf) {
        self.value <<= leaf.nums();
        self.value |= leaf.values();
        self.nums += leaf.nums() as u8;
    }
}

mod trait_impls;

#[cfg(test)]
mod tests;
