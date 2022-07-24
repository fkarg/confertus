use super::*;
use crate::traits::*;

impl Dot for Leaf {
    fn dotviz(&self, self_id: isize) -> String {
        format!(
            "L{self_id} [label=\"Leaf[{self_id}]\\n{:#0width$b}\\nnums={}\" shape=record];\n",
            self.value,
            self.nums,
            width = (LeafValue::BITS + 2) as usize //         L{self_id} -> N{} [label=<Parent>];\n", self.value, self.nums, self.parent)
        )
    }
}

/// Debug formatting is of format `Leaf[P: <{self.parent}>, nums {self.nums}, value {self.value in
/// binary representation}]`
impl fmt::Debug for Leaf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Leaf[P: <{:3}>, nums {:3}, value {:#0width$b}]",
            self.parent,
            self.nums,
            self.value,
            width = (LeafValue::BITS + 2) as usize
        )
    }
}

/// Forward Static Bit Vector functionality from [`LeafValue`]-container to [`Leaf`]
impl StaticBitVec for Leaf {
    type Intern = LeafValue;

    #[inline]
    fn ones(&self) -> usize {
        self.value.ones()
    }

    #[inline]
    fn access(&self, index: usize) -> bool {
        self.value.access(index)
    }

    #[inline]
    fn rank(&self, bit: bool, index: usize) -> usize {
        self.value.rank(bit, index)
    }

    #[inline]
    fn select(&self, bit: bool, n: usize) -> usize {
        self.value.select(bit, n)
    }

    #[inline]
    fn values(&self) -> Self::Intern {
        self.value
    }
}

/// Provide Dynamic Bit Vector functionality for [`Leaf`] via underlying container and forwarded
/// [`StaticBitVec`] functionality.
impl DynBitVec for Leaf {
    #[inline]
    fn insert(&mut self, index: usize, bit: bool) -> Result<(), &'static str> {
        if (self.nums as u32) < LeafValue::BITS && index <= self.nums as usize {
            unsafe { self.insert_unchecked(index, bit) };
            Ok(())
        } else if index > self.nums as usize {
            println!("index {index} out of bounds for {}", self.nums);
            Err("Leaf.insert: Index out of bounds `index > self.nums`")
        } else if (self.nums as u32) >= LeafValue::BITS {
            Err("Leaf.insert: No free capacity left")
        } else {
            unreachable!()
        }
    }

    #[inline]
    fn delete(&mut self, index: usize) -> Result<(), &'static str> {
        if !self.is_empty() && index < self.nums as usize {
            unsafe { self.delete_unchecked(index) };
            Ok(())
        } else if self.is_empty() {
            Err("Tried to delete in empty leaf")
        } else {
            println!(
                "deletion: attempted deletion of {index}, but size is {}",
                self.nums
            );
            Err("deletion of non-allocated position: `index >= self.nums`")
        }
    }

    #[inline]
    fn flip(&mut self, index: usize) {
        // unchecked:
        // - index < self.nums
        // (and, by extension)
        // - index < LeafValue::BITS
        self.value ^= 1 << index;
    }

    #[inline]
    fn nums(&self) -> usize {
        self.nums.into()
    }
}
