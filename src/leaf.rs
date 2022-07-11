use crate::traits;
use core::arch::x86_64::_pdep_u64;
use std;
use std::fmt;

/// Primitive type used as bit container in [`Leaf`]. Sensible options are [`u64`] and [`u128`].
/// (also `u256`, should it get implemented eventually)
pub type LeafValue = u64;

/// Leaf element of [`crate::DynamicBitVector`]. Next to its value ([`LeafValue`]) and bits used
/// inside (`nums`), it contains a reference to its parent [`crate::Node`].
///
/// Instance bit size: 17~25 bytes, depending on `LeafValue`
#[derive(PartialEq, Clone, Default)]
pub struct Leaf {
    /// reference to parent [`crate::Node`] (8 byte)
    pub parent: usize, // 8 bytes
    /// container for actual bit values (8-16 byte)
    pub value: LeafValue, // 8~16 bytes
    /// number of bits used in `value`-container. Below `u128::BITS == 128`, so `u8::MAX = 255` is
    /// sufficient. (1 byte)
    pub nums: u8, // realistically below u128::BITS, so u8::MAX = 255 is sufficient. // 1 byte
}

/// Debug formatting is of format `Leaf[P: <{self.parent}>, nums {self.nums}, value {self.value in
/// binary representation}]`
impl fmt::Debug for Leaf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if LeafValue::BITS == 64 {
            write!(
                f,
                "Leaf[P: <{:3}>, nums {:2}, value {:#066b}]",
                self.parent, self.nums, self.value
            )
        } else {
            write!(
                f,
                "Leaf[P: <{:3}>, nums {:3}, value {:#0130b}]",
                self.parent, self.nums, self.value
            )
        }
    }
}

impl Leaf {
    /// Constructs a new, empty `Leaf` with parent `parent`.
    #[inline]
    pub fn new(parent: usize) -> Self {
        Leaf {
            parent,
            value: 0,
            nums: 0,
        }
    }

    /// Access bit value at position `index`
    ///
    /// # Panics
    /// If `index` > [`LeafValue::BITS`]
    #[inline]
    pub fn access(&self, index: usize) -> bool {
        (self.value >> index) & 1 == 1
    }

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
    /// Unchecked invariant: used capacity `nums` is less than the total capacity of
    /// `LeafValue::BITS` bits before push.
    #[inline]
    pub unsafe fn push_unchecked(&mut self, bit: bool) {
        self.value |= (bit as LeafValue) << self.nums;
        self.nums += 1;
    }

    /// Insert `bit` at position `index` in [`Leaf`].
    ///
    /// # Errors
    /// When Leaf full or `index` out of bounds (`> self.nums` or `>= LeafValue::Bits`).
    pub fn insert(&mut self, index: usize, bit: bool) -> Result<(), &str> {
        if self.nums as u32 >= LeafValue::BITS {
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
    /// - `index < LeafValue::BITS`
    /// - `index <= self.nums`
    pub unsafe fn insert_unchecked(&mut self, index: usize, bit: bool) {
        let lmask = LeafValue::MAX.rotate_left(LeafValue::BITS - index as u32); // in- or excluding index here?
        let rmask = LeafValue::MAX.rotate_right(index as u32);
        self.value =
            (self.value & lmask) | ((bit as LeafValue) << index) | ((self.value & rmask) >> 1);
        self.nums += 1;
    }

    /// Remove bit value at position `index`
    ///
    /// # Errors
    /// When Leaf empty or `index` out of bounds (`> self.nums` or `> LeafValue::BITS`).
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
    /// - `index < LeafValue::BITS`
    /// - `index < self.nums`
    /// - `self.nums > 0`
    pub unsafe fn delete_unchecked(&mut self, index: usize) {
        let lmask = LeafValue::MAX.rotate_left(LeafValue::BITS - index as u32);
        let rmask = LeafValue::MAX.rotate_right(index as u32);
        self.value = (self.value & lmask) | ((self.value & rmask) << 1);
        self.nums -= 1;
    }

    /// Returns number on-bits in `self.values`
    ///
    /// runtime complexity: O(1)
    #[inline]
    pub fn ones(&self) -> usize {
        self.value.count_ones() as usize
    }

    /// Returns number of `bit`-values up to `index` in `self.value`
    ///
    /// runtime complexity: O(1)
    pub fn rank(&self, bit: bool, index: usize) -> usize {
        if bit {
            self.value.rotate_right(index as u32).count_ones() as usize
        } else {
            (!self.value).rotate_right(index as u32).count_ones() as usize
        }
    }

    /// ```text
    /// Algorithm for determining the position of the jth 1 in a machine word.
    /// ---
    /// 1: function PTSELECT(x, j)
    /// 2:     i ← SHIFTLEFT(1, j)
    /// 3:     p ← PDEP(i, x)
    /// 4:     return TZCNT(p)
    /// ```
    ///
    /// taken from <https://arxiv.org/pdf/1706.00990.pdf>
    pub unsafe fn select_pdep(&self, bit: bool, n: usize) -> usize {
        let array = if bit { self.value } else { !self.value };
        // self.value is u64
        _pdep_u64(1 << n, array as u64) as usize

        // // self.value is u128
        // if n < 64 {
        //     _pdep_u64(1 << n, array as u64) as usize
        // } else {
        //     _pdep_u64(1 << n, (array >> 64) as u64) as usize
        // }

        // yes, comment / uncomment ... no conditional compilation possible
    }

    /// Return index of `n`-th `bit`-value in `self.value`
    pub fn select(&self, bit: bool, n: usize) -> usize {
        if std::is_x86_feature_detected!("bmi2") {
            unsafe { self.select_pdep(bit, n) }
        } else {
            // fallback for non-bmi2-x86 architectures

            // Scan the leaf from left to right and look for the bit of
            // respective rank.
            let mut pos = 0;
            let mut i = n;
            for shift in (0..LeafValue::BITS).rev() {
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
        // todo!(".select {:?}", self);
    }

    /// Flip bit at position `index`
    ///
    /// runtime complexity: O(1)
    #[inline]
    pub fn flip(&mut self, index: usize) {
        self.value ^= 1 << index;
    }

    /// Return used capacity `self.nums`
    #[inline]
    pub fn nums(&self) -> usize {
        self.nums.into()
    }

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
}

impl traits::Dot for Leaf {
    fn dotviz(&self, self_id: isize) -> String {
        format!(
            "L{self_id} [label=\"L{self_id}\\n{:#066b}\\nnums={}\" shape=record];\n",
            self.value, self.nums
        )
        // format!("L{self_id} [label=\"L{self_id}\\n{:#066b}\\nnums={}\" shape=record];\n\
        //         L{self_id} -> N{} [label=<Parent>];\n", self.value, self.nums, self.parent)
    }
}
