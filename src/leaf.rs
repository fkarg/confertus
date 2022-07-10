use std::fmt;
use crate::traits;

/// Primitive type used as bit container in [`Leaf`]. Probably [`u64`] or [`u128`].
pub type LeafValue = u64;

/// Leaf element of [`crate::DynamicBitVector`]. Next to its value ([`u128`]) and bits used inside
/// (`nums`), it contains a reference to its parent [`crate::Node`].
///
/// bit size: 17~25 bytes
#[derive(PartialEq, Clone, Default)]
pub struct Leaf {
    /// reference to parent [`crate::Node`]
    pub parent: usize, // 8 bytes
    /// container for actual bit values
    pub value: LeafValue, // 8~16 bytes
    /// number of bits used in `value`-container. Below `u128::BITS == 128`, so `u8::MAX = 255` is
    /// sufficient
    pub nums: u8, // realistically below u128::BITS, so u8::MAX = 255 is sufficient. // 1 byte
}

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
    pub fn access(&self, index: usize) -> bool {
        (self.value >> index) & 1 == 1
    }

    /// Appends bit to the end of `value`, as long as there is free capacity.
    ///
    /// # Errors
    /// If the capacity `nums` exceeds `LeafValue::BITS` bits.
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
    /// # Panics
    /// If the capacity `nums` exceeds `LeafValue::BITS` bits.
    pub unsafe fn push_unchecked(&mut self, bit: bool) {
        self.value |= (bit as LeafValue) << self.nums;
        self.nums += 1;
    }

    /// Insert `bit` at position `index` in [`Leaf`].
    pub fn insert(&mut self, index: usize, bit: bool) {
        unsafe { self.insert_unchecked(index, bit) }
        todo!()
    }

    /// Unchecked version of [`Leaf::insert`]
    // TODO: update ones
    pub unsafe fn insert_unchecked(&mut self, index: usize, bit: bool) {
        let lmask = LeafValue::MAX.rotate_left(LeafValue::BITS - index as u32);
        let rmask = LeafValue::MAX.rotate_right(index as u32);
        self.value = (self.value & lmask) | (1 << index) | ((self.value & rmask) >> 1);
        self.nums += 1;
    }

    // TODO: update ones
    /// # Panics
    /// If index is larger than [`LeafValue::BITS`]
    pub fn delete(&mut self, index: usize) {
        let lmask = LeafValue::MAX.rotate_left(LeafValue::BITS - 1 - index as u32);
        let rmask = LeafValue::MAX.rotate_right(index as u32);
        self.value = (self.value & lmask) | ((self.value & rmask) << 1);
        self.nums -= 1;
    }

    pub fn ones(&self) -> usize {
        self.value.count_ones() as usize
    }

    pub fn rank(&self, bit: bool, index: usize) -> usize {
        // TODO: without including `bit` for ones
        (self.value & LeafValue::MAX.rotate_left(LeafValue::BITS - index as u32)).count_ones()
            as usize;
        todo!();
    }

    pub fn select(&self, bit: bool, index: usize) -> usize {
        todo!()
    }

    pub fn nums(&self) -> usize {
        self.nums.into()
    }

    fn balance(&self) -> i8 {
        1
    }
}

impl traits::Dot for Leaf {
    fn dotviz(&self, self_id: isize) -> String {
        format!("L{self_id} [label=\"L{self_id}\\n{:#066b}\\nnums={}\" shape=record];\n", self.value, self.nums)
        // format!("L{self_id} [label=\"L{self_id}\\n{:#066b}\\nnums={}\" shape=record];\n\
        //         L{self_id} -> N{} [label=<Parent>];\n", self.value, self.nums, self.parent)
    }
}
