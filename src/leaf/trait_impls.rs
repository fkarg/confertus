use super::*;
use crate::traits::*;

impl<T: fmt::Binary> Dot for Leaf<T> {
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
impl<T: fmt::Binary> fmt::Debug for Leaf<T>
where
    T: fmt::Debug + fmt::Binary,
{
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
