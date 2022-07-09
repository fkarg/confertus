/// Primitive type used as bit container in [`Leaf`]. Probably [`u64`] or [`u128`].
pub type LeafValue = u64;

/// Leaf element of [`crate::DynamicBitVector`]. Next to its value ([`u128`]) and bits used inside
/// (`nums`), it contains a reference to its parent [`crate::Node`].
///
/// bit size: 17~25 bytes
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Leaf {
    /// reference to parent [`crate::Node`]
    pub parent: usize, // 8 bytes
    /// container for actual bit values
    pub value: LeafValue, // 8~16 bytes
    /// number of bits used in `value`-container. Below `u128::BITS == 128`, so `u8::MAX = 255` is
    /// sufficient
    pub nums: u8, // realistically below u128::BITS, so u8::MAX = 255 is sufficient. // 1 byte
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
    ///
    /// # Safety
    pub unsafe fn push_unchecked(&mut self, bit: bool) {
        self.value |= (bit as LeafValue) << self.nums;
        self.nums += 1;
    }

    /// Unchecked version of [`Leaf::insert`]
    /// # Safety
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

    pub fn nums(&self) -> usize {
        self.nums.into()
    }

    fn balance(&self) -> i8 {
        1
    }
}
