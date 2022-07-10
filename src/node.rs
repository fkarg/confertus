use std::fmt;
use crate::traits::Dot;

/// Node element of [`super::DynamicBitVector`]. Contains references (indices) to parent `Node`,
/// left and right subtrees, as well as `nums`, the number of used bits in the left subtree, `ones`
/// the number of ones in the left subtree, and `size`, the total capacity of the current subtree.
///
/// maximum ideal instance size: 48 bytes + 5 bit
#[derive(PartialEq, Clone, Default)]
pub struct Node {
    // TODO: remove option from values to reduce used bit sizes
    /// reference to parent Node
    pub parent: Option<usize>, // 8 bytes + 1bit
    /// left side subtree where `isize` is the index to child element
    pub left: Option<isize>, // 8 bytes + 1bit
    /// right side subtree where `isize` is the index to child element
    pub right: Option<isize>, // 8 bytes + 1bit
    //// total number of filled bits, across both subtrees
    /// total bit capacity, across both subtrees
    pub size: usize, // 8 bytes // not necessary?
    /// number of 'filled' bits on the left  subtree
    pub nums: usize, // 8 bytes
    /// number of ones on the left subtree
    pub ones: usize, // 8 bytes
    /// difference of height between left and right subtree
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

impl Node {
    pub fn new() -> Self {
        Node {
            parent: None,
            left: None,
            right: None,
            size: 0,
            nums: 0,
            ones: 0,
            rank: 0,
        }
    }

    fn rank(self) -> i8 {
        self.rank
    }

    /// Used when inserting a Node in place of a [`super::Leaf`] or rotating to keep rank
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

        let parent = format!("N{self_id} -> N{} [label=<Parent>,color=green];\n", self.parent.unwrap_or(self_id as usize));

        format!(
            "N{self_id} [label=\"N{self_id}\\nnums={} ones={} rank={}\"];\n\
            {}\
            {}\
            {}\
            ",
            self.nums,
            self.ones,
            self.rank,
            left,
            right,
            parent,
            )
    }
}
