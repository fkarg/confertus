/// Node element of [`crate::DynamicBitVector`]. Contains references (indices) to parent `Node`,
/// left and right subtrees, as well as `nums`, the number of used bits in the left subtree, `ones`
/// the number of ones in the left subtree, and `size`, the total capacity of the current subtree.
///
/// instance size: 72 bytes
#[derive(Debug, PartialEq, Clone)]
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
    pub size: usize, // 8 bytes
    /// number of 'filled' bits on the left  subtree
    pub nums: usize, // 8 bytes
    /// number of ones on the left subtree
    pub ones: usize, // 8 bytes
    /// height of current subtree
    pub balance: i8, // 1 byte
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
            balance: 0,
        }
    }

    /// Used when inserting a Node in place of a [`crate::Leaf`] or rotating to keep balance
    pub fn replace_child_with(&mut self, child: isize, new_child: isize) {
        if let Some(l) = self.left {
            if l == child {
                self.left = Some(l);
                return ();
            }
        }
        if let Some(r) = self.right {
            if r == child {
                self.right = Some(r);
                return ();
            }
        }
        panic!(
            "{} not subtree of current Node (parent {:?}).",
            child, self.parent
        );
    }
}
