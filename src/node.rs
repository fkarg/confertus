use crate::traits::{Dot, StaticBitVec};
use crate::{Leaf, LeafValue};
use std::fmt;

/// Node element of [`super::DynamicBitVector`]. Contains references (indices) to parent `Node`,
/// left and right subtrees, as well as `nums`, the number of used bits in the left subtree, `ones`
/// the number of ones in the left subtree, and `size`, the total capacity of the current subtree.
///
/// Instance bit size: 40 bytes + 5 bit = 325 bit
///
/// Should `u32`/`i32` (4'294'967'295/2'147'483'647 values) suffice, size would be 20 bytes + 5 bit = 165 bit
#[derive(PartialEq, Clone, Default)]
pub struct Node {
    // TODO: remove option from values to reduce used bit sizes
    /// index of parent Node, 8 bytes + 1bit
    pub parent: Option<usize>, // 8 bytes + 1bit
    /// left side subtree where `isize` is the index to child element, 8 bytes + 1bit
    pub left: Option<isize>, // 8 bytes + 1bit
    /// right side subtree where `isize` is the index to child element, 8 bytes + 1bit
    pub right: Option<isize>, // 8 bytes + 1bit
    /// number of 'filled' bits on the left  subtree, 8 byte
    pub nums: usize, // 8 bytes
    /// number of ones on the left subtree, 8 byte
    pub ones: usize, // 8 bytes
    /// difference of height between left and right subtree. Valid values are (-1, 0, 1), 2bit
    pub rank: i8, // 2 bit (valid values: -1, 0, 1)
                  // diff of height: right - left
                  // insertion to right increases
                  // insertion to left decreases
                  // deletion from right decreases
                  // deletion from left increases
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Node[P: <{:3?}>, L: {:4?}, R: {:4?}, nums {}, ones {}, rank {}]",
            self.parent, self.left, self.right, self.nums, self.ones, self.rank
        )
    }
}

/// Since a lot of operations on Nodes require acessing others by an index, most functionality is
/// implemented in [`crate::DynamicBitVector`] directly.
impl Node {
    /// Constructs new, empty `Node`.
    pub fn new() -> Self {
        Node {
            parent: None,
            left: None,
            right: None,
            nums: 0,
            ones: 0,
            rank: 0,
        }
    }

    /// Constructs new `Node` with given values
    pub fn create(
        parent: Option<usize>,
        left: Option<isize>,
        right: Option<isize>,
        nums: usize,
        ones: usize,
        rank: i8,
    ) -> Self {
        Node {
            parent,
            left,
            right,
            nums,
            ones,
            rank,
        }
    }

    /// Used when inserting a Node in place of a [`crate::Leaf`] or rotation to keep rank
    pub fn replace_child_with(&mut self, child: isize, new_child: isize) {
        if let Some(l) = self.left {
            if l == child {
                self.left = Some(new_child);
                return;
            }
        }
        if let Some(r) = self.right {
            if r == child {
                self.right = Some(new_child);
                return;
            }
        }
        panic!(
            "{} not subtree of current Node (parent {:?}).",
            child, self.parent
        );
    }
}

impl Dot for Node {
    fn dotviz(&self, self_id: isize) -> String {
        let right = if let Some(r) = self.right {
            if r >= 0 {
                format!("N{self_id} -> N{r} [label=<Right>,color=red];\n")
                // node
            } else {
                format!("N{self_id} -> L{} [label=<Right>,color=red];\n", -r)
            }
        } else {
            "".to_string()
        };
        let left = if let Some(l) = self.left {
            if l >= 0 {
                format!("N{self_id} -> N{l} [label=<Left>,color=blue];\n")
                // node
            } else {
                format!("N{self_id} -> L{} [label=<Left>,color=blue];\n", -l)
            }
        } else {
            "".to_string()
        };

        let parent = format!(
            "N{self_id} -> N{} [label=<Parent>,color=green];\n",
            self.parent.unwrap_or(self_id as usize)
        );

        format!(
            "N{self_id} [label=\"N{self_id}\\nnums={} ones={} rank={}\"];\n\
            {}\
            {}\
            {}\
            ",
            self.nums, self.ones, self.rank, left, right, parent,
        )
    }
}

// Welp, so much for that attempt. Could have been really useful, but you'd need some form of
// backreference to [`DynamicBitVector`], and I just don't think that's gonna be a thing.
impl StaticBitVec for Node {
    type Intern = Vec<LeafValue>;

    fn ones(&self) -> usize {
        self.ones
    }

    fn access(&self, index: usize) -> bool {
        unimplemented!("not possible without access to other Nodes/Leafs")
    }

    fn rank(&self, bit: bool, index: usize) -> usize {
        unimplemented!("not possible without access to other Nodes/Leafs")
    }

    fn select(&self, bit: bool, n: usize) -> usize {
        unimplemented!("not possible without access to other Nodes/Leafs")
    }

    fn values(&self) -> Self::Intern {
        unimplemented!("not possible without access to other Nodes/Leafs")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn creation() {
        let n = Node::new();
        assert_eq!(
            n,
            Node {
                parent: None,
                left: None,
                right: None,
                nums: 0,
                ones: 0,
                rank: 0,
            }
        );
    }
}
