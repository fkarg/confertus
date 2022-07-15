use crate::traits::*;
use super::*;


impl Dot for Leaf {
    fn dotviz(&self, self_id: isize) -> String {
        format!(
            "L{self_id} [label=\"L{self_id}\\n{:#066b}\\nnums={}\" shape=record];\n",
            self.value, self.nums
        )
        // format!("L{self_id} [label=\"L{self_id}\\n{:#066b}\\nnums={}\" shape=record];\n\
        //         L{self_id} -> N{} [label=<Parent>];\n", self.value, self.nums, self.parent)
    }
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

/// Forward Static Bit Vector functionality from container to [`Leaf`]
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

/// Provide Dynamic Bit Vector functionality via [`Leaf`] on underlying container and forwarded
/// [`StaticBitVec`] functionality
impl DynBitVec for Leaf {
    #[inline]
    fn insert(&mut self, index: usize, bit: bool) -> Result<(), &'static str> {
        if self.nums as u32 >= LeafValue::BITS {
            Err("No free capacity left")
        } else {
            unsafe { self.insert_unchecked(index, bit) };
            Ok(())
        }
    }

    #[inline]
    fn delete(&mut self, index: usize) -> Result<(), &'static str> {
        if self.is_empty() {
            Err("Tried to delete in empty leaf")
        } else {
            unsafe { self.delete_unchecked(index) };
            Ok(())
        }
    }

    #[inline]
    fn flip(&mut self, index: usize) {
        self.value ^= 1 << index;
    }

    #[inline]
    fn nums(&self) -> usize {
        self.nums.into()
    }
}
