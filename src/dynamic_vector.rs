// use super::traits;
pub use super::leaf::*;
pub use super::node::*;
use std::ops::{Index, IndexMut};

/// Implementation of Dynamic Bit Vector using AVL/RB tree.
///
/// Current size: 56 bytes
#[derive(Debug, PartialEq, Clone)]
pub struct DynamicBitVector {
    /// index to root [`Node`]
    pub root: usize, // 8 bytes
    // positively indexed, usize
    /// Vector containing [`Node`]
    pub nodes: Vec<Node>, // 24 bytes
    // negatively indexed, isize
    /// Vector containing [`Leaf`]
    pub leafs: Vec<Leaf>, // 24 bytes
}

impl Index<usize> for DynamicBitVector {
    type Output = Node;

    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}

impl IndexMut<usize> for DynamicBitVector {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}

impl Index<isize> for DynamicBitVector {
    type Output = Leaf;

    fn index(&self, index: isize) -> &Self::Output {
        let uidx: usize;
        if index < 0 {
            uidx = -index as usize;
        } else {
            uidx = index as usize;
        }
        &self.leafs[uidx]
    }
}

impl IndexMut<isize> for DynamicBitVector {
    fn index_mut(&mut self, index: isize) -> &mut Self::Output {
        let uidx: usize;
        if index < 0 {
            uidx = -index as usize;
        } else {
            uidx = index as usize;
        }
        &mut self.leafs[uidx]
    }
}

impl DynamicBitVector {
    pub fn new() -> Self {
        DynamicBitVector {
            root: 0,
            nodes: vec![Node::new()], // create root node, but no children yet
            leafs: vec![],
        }
    }

    /// Insert intermediary Node `int_node_id` between current `child_id` and parent [`Node`], and
    /// update references accordingly
    fn insert_node(&mut self, child_id: isize, int_node_id: usize) {
        let parent_id = self[child_id].parent;
        if let Some(l) = self[parent_id].left {
            if l == child_id {
                self[parent_id].left = Some(int_node_id as isize);
                self[child_id].parent = int_node_id;
                self[int_node_id].parent = Some(parent_id);
                self[int_node_id].left = Some(child_id);
                return;
            }
        }
        if let Some(r) = self[parent_id].right {
            if r == child_id {
                self[parent_id].right = Some(int_node_id as isize);
                self[child_id].parent = int_node_id;
                self[int_node_id].parent = Some(parent_id);
                self[int_node_id].right = Some(child_id);
                return;
            }
        }
        panic!(
            "{} not subtree of current Node (parent {:?}).",
            child_id, parent_id
        );
    }

    /// Inserts new [`Node`] to position of current leaf, making it the left child of the newly
    /// created node. TODO: check for rotation necessity
    ///
    /// Returns id of newly created Node.
    #[inline]
    fn insert_node_at_leaf(&mut self, leaf: isize) -> usize {
        self.nodes.push(Node::new());
        let new_node_id = self.nodes.len() - 1; // offset by one between index and length
        self.insert_node(leaf, new_node_id);
        new_node_id
    }

    pub fn push(&mut self, bit: bool) {
        // let root = self.root;
        self.push_to(self.root, bit);

        // return;
        // let mut rightmost = root.clone();
        // while let Some(r) = self[rightmost].right {
        //     if r > 0 {
        //         rightmost = r as usize;
        //     } else {
        //         let leaf = &mut self[r];
        //         if u32::from(leaf.nums) < LeafValue::BITS {
        //             // check happened here
        //             unsafe {
        //                 leaf.push_unchecked(bit);
        //             }
        //         } else {
        //             // no space left in leaf. So, insert node at position of leaf, and make this
        //             // (and a newly created leaf) children, and push to right child.
        //             let new_node_id = self.insert_node_at_leaf(r);
        //             self.leafs.push(Leaf::new(new_node_id));
        //             let new_leaf_id = self.leafs.len() as isize;
        //             self[new_node_id].right = Some(new_leaf_id);
        //         }
        //     }
        // }
    }

    /// Append `bit` to the end, which is to say to the rightmost subtree.
    ///
    /// Invariances:
    /// - `nums` has number of bits in left subtree
    ///     -> won't change
    /// - `ones` has count of 1-bits in left subtree
    ///     -> won't change
    /// - `balance`-difference between two subtrees must not exceed 2, otherwise rotate
    ///     -> check when creating/inserting a new node/leaf
    fn push_to(&mut self, node: usize, bit: bool) {
        if let Some(r) = self[node].right {
            if r > 0 {
                self.push_to(r as usize, bit);
            } else {
                // r is Leaf
                let leaf = &self[r];
                match self[r].push(bit) {
                    Ok(_) => (),
                    Err(_) => {
                        let new_node_id = self.insert_node_at_leaf(r);
                        self.leafs.push(Leaf::new(new_node_id));
                        let new_leaf_id = self.leafs.len() as isize;
                        self[new_node_id].right = Some(new_leaf_id);
                        self.balance();
                    }
                }
            }
        }
    }

    /// right becomes parent
    fn rotate_left(&mut self, parent: isize, right: isize) {}

    /// left becomes parent
    fn rotate_right(&mut self, left: isize, parent: isize) {}

    /// balance tree
    pub fn balance(&mut self) {
        // first, recursively update all height values
        // when on a node where both subtrees have a differing height of more than two, rotate.
        todo!()
    }

    pub fn insert(&mut self, index: usize, bit: bool) {
        todo!()
    }

    pub fn delete(&mut self, index: usize) {
        todo!()
    }

    pub fn flip(&mut self, index: usize) {
        todo!()
    }

    pub fn rank(&mut self, bit: bool, index: usize) {
        todo!()
    }

    pub fn select(&mut self, bit: bool, index: usize) {
        todo!()
    }

    pub fn nums(self) -> usize {
        self[self.root].nums
    }

    pub fn ones(self) -> usize {
        self[self.root].ones
    }

    pub fn capacity(self) -> usize {
        self[self.root].size
    }
}
