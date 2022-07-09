// use super::traits;
pub use super::diff::*;
pub use super::leaf::*;
pub use super::node::*;
use std::ops::{Index, IndexMut};

/// Implementation of Dynamic Bit Vector using AVL/RB tree.
///
/// Current size: 56 bytes
#[derive(Debug, PartialEq, Clone, Default)]
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

    /// Return value at position `index`
    pub fn get(&self, index: usize) -> bool {
        self.get_node(self.root, index)
    }

    /// Recursive function call
    #[inline(always)]
    fn get_node(&self, node: usize, index: usize) -> bool {
        if self[node].nums <= index {
            // enter right side
            let right_id = self[node].right.unwrap();
            if right_id > 0 {
                self.get_node(right_id as usize, index - self[node].nums)
            } else {
                // leaf
                self[right_id].access(index - self[node].nums)
            }
        } else {
            // enter left side
            let left_id = self[node].left.unwrap();
            if left_id > 0 {
                self.get_node(left_id as usize, index)
            } else {
                // leaf
                self[left_id].access(index)
            }
        }
    }

    /// Insert intermediary [`Node`] `int_node_id` between [`Leaf`] `child_id` and parent Node.
    /// `child_id` will always be inserted as the left child of the new intermediary node.
    /// Updates references accordingly.
    ///
    /// Assumes that intermediary node does not have children (overwrites `left` child otherwise)
    /// or otherwise relevant information (`nums` and `ones` get overwritten too).
    fn insert_node(&mut self, child_id: isize, int_node_id: usize) -> Diff {
        let parent_id = self[child_id].parent;
        if let Some(l) = self[parent_id].left {
            if l == child_id {
                self[parent_id].left = Some(int_node_id as isize);
                return self.insert_node_common(child_id, parent_id, int_node_id);
            }
        }
        if let Some(r) = self[parent_id].right {
            if r == child_id {
                self[parent_id].right = Some(int_node_id as isize);
                return self.insert_node_common(child_id, parent_id, int_node_id);
            }
        }
        panic!(
            "{} not subtree of current Node (parent {:?}).",
            child_id, parent_id
        );
    }

    #[inline]
    fn insert_node_common(&mut self, child_id: isize, parent_id: usize, int_id: usize) -> Diff {
        self[child_id].parent = int_id;
        self[int_id].parent = Some(parent_id);
        self[int_id].left = Some(child_id);
        // setting nums and ones later when consuming diff
        self[int_id].nums = self[child_id].nums();
        self[int_id].ones = self[child_id].ones();
        self[int_id].balance = -1; // 'left-leaning'
        Diff::default()
    }

    /// Inserts new [`Node`] to position of current leaf, making it the left child of the newly
    /// created node.
    ///
    /// Returns id of newly created Node.
    #[inline]
    fn insert_node_at_leaf(&mut self, leaf: isize) -> (usize, Diff) {
        // offset by one between index and length
        let new_node_id = self.nodes.len();
        self.nodes.push(Node::new());
        let diff = self.insert_node(leaf, new_node_id);
        (new_node_id, diff)
    }

    /// Move the right subtree to the left side. Expects the left subtree to be empty (will be
    /// overwritten otherwise) and the right to be nonempty (Panics otherwise).
    ///
    /// # Panics
    /// If right child is [`None`]
    fn move_right_child_left(&mut self, node: usize) -> Diff {
        self[node].left = self[node].right;
        self[node].right = None;

        let left_id = self[node]
            .left
            .expect("cannot move right to left without right subtree");

        // update `nums` and `ones` accordingly
        self[node].nums = self[left_id].nums();
        self[node].ones = self[left_id].ones();
        Diff::move_child_right_to_left()
    }

    pub fn push(&mut self, bit: bool) {
        // let root = self.root;
        let diff = self.push_to(self.root, bit);
        self.consume_diff(self.root, diff);
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
    ///
    /// # Returns [`Diff`]
    /// to propagate changes back after descending
    fn push_to(&mut self, node: usize, bit: bool) -> Diff {
        // node is guaranteed to be a node. push to rightmost place. First: Find leaf
        if let Some(r) = self[node].right {
            // if the id `r` is positive, it's a node, if it's negative, it's a leaf
            if r > 0 {
                // node found. push there instead
                let diff = self.push_to(r as usize, bit);
                self.consume_diff(r as usize, diff)
            } else {
                // r is Leaf
                let diff = self.push_right_leaf(node, r, bit);
                self.consume_diff(node, diff)
            }
        } else {
            // no right-side child. create leaf and insert
            let new_leaf_id = -(self.leafs.len() as isize);
            self.leafs.push(Leaf::new(node));
            // inseart newly created leaf to right side
            self[node].right = Some(new_leaf_id);

            // update diff to includes changes from creating the leaf
            let diff = self.push_right_leaf(node, new_leaf_id, bit) + Diff::create_right_leaf();
            self.consume_diff(node, diff)
        }
    }

    /// Given a [`Node`] `node` and its child [`Leaf`] `leaf`, append `bit` to the end.
    ///
    /// # Cases:
    /// - OK: insertion was possible
    /// - Err: Capacity of right leaf full. Check if left child exists.
    ///     - if no, move right to left, create new right one, push there.
    ///     - if yes, insert new node at position of right leaf, move leaf to left of newly created
    ///     node, create new right leaf, and push there.
    ///
    /// # Returns: [`Diff`]
    fn push_right_leaf(&mut self, node: usize, leaf: isize, bit: bool) -> Diff {
        match self[leaf].push(bit) {
            // Leaf.push
            Ok(_) => Diff::insert_right(),
            Err(_) => {
                // first, check if left child exists.
                if let Some(_) = self[node].left {
                    // Something exists on left side. So we need to insert a new node at the right
                    // side at the current position of `leaf`.
                    let (new_node_id, diff_insert) = self.insert_node_at_leaf(leaf);

                    // check if tree needs to rebalance now
                    if i8::abs(self[node].balance + diff_insert.balance + 1) >= 2 {
                        self[node].balance += diff_insert.balance;
                        // tree needs rebalancing.
                        self.balance(node);
                        // afterwards, insert at top again
                        self.push(bit);
                        return Diff::default();
                    }
                    // We don't need to rebalance, so we can again use `push_to` instead.
                    self.push_to(new_node_id, bit);
                    diff_insert

                    // let new_leaf_id = -(self.leafs.len() as isize);
                    // self.leafs.push(Leaf::new(new_node_id));

                    // // leaf is now left child of `new_node_id`
                    // self[new_node_id].right = Some(new_leaf_id);

                    // // finally push to new right leaf
                    // let diff = self.push_right_leaf(new_node_id, new_leaf_id, bit);

                    // // update diff: balance is -1 from previous
                    // diff_insert + diff + Diff::create_right_node() + Diff::create_right_leaf()
                    // total balance: diff + (-1) + (-1) + (+1) = diff - 1
                } else {
                    // No child on left side. Move right leaf over and create new leaf
                    // no right-side child. Recurse with `push_to` on same node.
                    let diff_move = self.move_right_child_left(node);
                    let diff_push = self.push_to(node, bit);
                    // TODO: diff_push might get consumed twice?
                    // (`push_to` with consumation is twice in call chain ... don't add here?
                    // don't consume here, as it will be consumed higher up the call chain
                    // diff_move + diff_push
                    diff_move
                }
            }
        }
    }

    /// Consume given diff and update node values accordingly.
    /// Returns (modified) diff to propagate upwards
    ///
    /// `from_right`: if the values come from a right descent
    fn consume_diff(&mut self, node: usize, diff: Diff) -> Diff {
        //, from_right: bool) -> Diff {
        // nums:  // might not be needed?
        // ones:  // might not be needed?

        // size: changes when leaf got created or removed
        // update difference in size first
        // (if a leaf got created or removed)
        if diff.size != 0 {
            if diff.size > 0 {
                self[node].size += diff.size as usize;
            } else {
                self[node].size -= -diff.size as usize;
            }
        }

        // balance: changes when node got inserted
        if i8::abs(self[node].balance + diff.balance) >= 2 {
            // TODO: update diff based on new balancing
            self.balance(node);
        }
        diff
    }

    /// right becomes parent (?)
    fn rotate_left(&mut self, parent: isize, right: isize) {
        todo!("{:?}", self)
    }

    /// left becomes parent (?)
    fn rotate_right(&mut self, left: isize, parent: isize) {
        todo!("{:?}", self)
    }

    /// check if the tree needs to rebalance after the `balance`-value of `node` has been updated
    pub fn balance(&mut self, node: usize) {
        // invariance has been broken at node.
        todo!("{:?}", self)
    }

    pub fn insert(&mut self, index: usize, bit: bool) {
        todo!("{:?}", self)
    }

    pub fn delete(&mut self, index: usize) {
        todo!("{:?}", self)
    }

    pub fn flip(&mut self, index: usize) {
        todo!("{:?}", self)
    }

    pub fn rank(&mut self, bit: bool, index: usize) {
        todo!("{:?}", self)
    }

    pub fn select(&mut self, bit: bool, index: usize) {
        todo!("{:?}", self)
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
