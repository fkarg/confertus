pub use super::leaf::*;
pub use super::node::*;
use crate::commands;
use crate::traits::{BitContainer, Dot, StaticBitVec};
use either;
use either::{Left, Right};
use std::fmt;
use std::ops::{Add, Index, IndexMut};

type Side<T> = either::Either<T, T>;

/// Implementation of Dynamic Bit Vector using self-balancing [AVL
/// tree](https://en.wikipedia.org/wiki/AVL_tree).
///
/// Instance bit size: 56 bytes = 448
/// (not included: bit sizes of instances in Vector structures)
#[derive(Debug, PartialEq, Clone, Default)]
pub struct DynamicBitVector<T>
where
    T: StaticBitVec + fmt::Debug + BitContainer,
{
    /// index to root [`Node`], 8 bytes
    pub root: usize, // 8 bytes
    // positively indexed, usize
    /// Vector containing [`Node`], 24 bytes
    pub nodes: Vec<Node>, // 24 bytes
    // negatively indexed, isize
    /// Vector containing [`Leaf`], 24 bytes
    pub leafs: Vec<Leaf<T>>, // 24 bytes
                             // last: isize, // 8 bytes, index to right-most leaf
                             // prev: isize, // 8 bytes, index to previously accessed leaf
}

impl<T> DynamicBitVector<T>
where
    T: StaticBitVec + fmt::Debug + BitContainer,
{
    // CONSTRUCTOR

    /// Constructs new `DynamicBitVector` with empty root [`Node`].
    pub fn new() -> Self {
        DynamicBitVector {
            root: 0,
            nodes: vec![Node::new()], // create root node, but no children yet
            leafs: vec![Leaf::new(0)],
        }
    }

    // ACCESS

    /// Return value at position `index` of `DynamicBitVector`.
    ///
    /// # Panics
    /// If `index` is out of bounds.
    #[inline]
    pub fn access(&mut self, index: usize) -> bool {
        self.apply(Self::get_leaf, index)
    }

    /// Recursive descension to position `index`, based on `node`.
    fn get_node(&mut self, node: usize, index: usize) -> bool {
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

    fn get_leaf(&mut self, leaf: isize, index: usize) -> bool {
        self[leaf].access(index)
    }

    // APPLY

    /// Descend tree to position `index` and apply function `f` with `f(self, leaf, index) -> T`.
    ///
    /// Used to implement traversal for [`DynamicBitVector::access`], [`DynamicBitVector::flip`],
    /// [`DynamicBitVector::delete`], [`DynamicBitVector::insert`]
    ///
    /// # Panics
    /// If tree invariances are violated
    pub fn apply(
        &mut self,
        mut f: impl FnMut(&mut DynamicBitVector<T>, isize, usize) -> T,
        index: usize,
    ) -> T {
        self.apply_node(self.root, f, index)
    }

    fn apply_node(
        &mut self,
        node: usize,
        mut f: impl FnMut(&mut DynamicBitVector<T>, isize, usize) -> T,
        index: usize,
    ) -> T {
        // index 128 is at right side when `nums == 128`, include right side/equal sign
        if self[node].nums <= index {
            // enter right side
            let right_id = self[node].right.unwrap();
            if right_id >= 0 {
                self.apply_node(right_id as usize, f, index - self[node].nums)
            } else {
                // leaf
                f(self, right_id, index - self[node].nums)
            }
        } else {
            // enter left side
            let left_id = self[node].left.unwrap();
            if left_id >= 0 {
                self.apply_node(left_id as usize, f, index)
            } else {
                // leaf
                f(self, left_id, index)
            }
        }
    }

    /// Descend tree to position `index` and apply function `f` with `f(self, leaf, index, bit) ->
    /// T`. Function `g` with `g(self, node, left_descent) -> T` is used to modify the return value of `f`
    /// dependent on `node`. Its value is added to the result of recursion.
    ///
    /// Used to implement traversal for [`DynamicBitVector::rank`]
    ///
    /// # Panics
    /// If tree invariances are violated
    pub fn apply_bitop(
        &mut self,
        mut f: impl FnMut(&mut DynamicBitVector<T>, isize, usize, bool) -> T,
        g: impl Fn(&DynamicBitVector<T>, usize, bool) -> T,
        index: usize,
        bit: bool,
    ) -> T
    where
        T: Add<Output = T>,
    {
        self.apply_bitop_node(self.root, f, g, index, bit)
    }

    fn apply_bitop_node(
        &mut self,
        node: usize,
        mut f: impl FnMut(&mut DynamicBitVector<T>, isize, usize, bool) -> T,
        g: impl Fn(&DynamicBitVector<T>, usize, bool) -> T,
        index: usize,
        bit: bool,
    ) -> T
    where
        T: Add<Output = T>,
    {
        // index 128 is at right side when `nums == 128`, include right side/equal sign
        if self[node].nums <= index {
            // enter right side
            let right_id = self[node].right.unwrap();
            if right_id >= 0 {
                g(self, node, false)
                    + self.apply_bitop_node(right_id as usize, f, g, index - self[node].nums, bit)
            } else {
                // leaf
                f(self, right_id, index, bit)
            }
        } else {
            // enter left side
            let left_id = self[node].left.unwrap();
            if left_id >= 0 {
                g(self, node, true) + self.apply_bitop_node(left_id as usize, f, g, index, bit)
            } else {
                // leaf
                f(self, left_id, index, bit)
            }
        }
    }

    // NODE INSERTION/CREATION

    /// Insert intermediary [`Node`] `int_node_id` between [`Leaf`] `child_id` and parent Node.
    /// `child_id` will always be inserted as the left child of the new intermediary node.
    /// Updates references accordingly.
    ///
    /// Assumes that intermediary node does not have children (overwrites `left` child otherwise)
    /// or otherwise relevant information (`nums` and `ones` get overwritten too).
    fn insert_intermediary_node(&mut self, child_id: isize, int_node_id: usize) {
        println!("Insert Node {} for {}", int_node_id, child_id);
        let parent_id = self[child_id].parent;
        if let Some(l) = self[parent_id].left {
            if l == child_id {
                self[parent_id].left = Some(int_node_id as isize);
                self.insert_node_common(child_id, parent_id, int_node_id);
                return;
            }
        }
        if let Some(r) = self[parent_id].right {
            if r == child_id {
                self[parent_id].right = Some(int_node_id as isize);
                self.insert_node_common(child_id, parent_id, int_node_id);
                return;
            }
        }
        println!(".insert_intermediary_node {}", self);
        panic!(
            "{} not subtree of current Node (parent {:?}).",
            child_id, parent_id
        );
    }

    #[inline]
    fn insert_node_common(&mut self, child_id: isize, parent_id: usize, int_id: usize) {
        self[child_id].parent = int_id;
        self[int_id].parent = Some(parent_id);
        self[int_id].left = Some(child_id);
        self[int_id].nums = self[child_id].nums();
        self[int_id].ones = self[child_id].ones();
        self[int_id].rank = -1; // 'left-leaning'
    }

    /// Inserts new [`Node`] to position of current leaf, making it the left child of the newly
    /// created node.
    ///
    /// Returns id of newly created Node.
    #[inline]
    fn insert_node_at_leaf(&mut self, leaf: isize) -> usize {
        // offset by one between index and length
        let new_node_id = self.nodes.len();
        self.nodes.push(Node::new());
        self.insert_intermediary_node(leaf, new_node_id);
        new_node_id
    }

    /// Move the right subtree of `node` to the left side. Expects the left subtree to be empty
    /// (will be overwritten otherwise) and the right to be nonempty (panics otherwise).
    ///
    /// # Panics
    /// If right child is [`None`]
    fn move_right_child_left(&mut self, node: usize) {
        println!("Moving R to L in {:?}", self[node]);
        self[node].left = self[node].right;
        self[node].right = None;

        let left_id = self[node]
            .left
            .expect("cannot move right to left without right subtree");

        // update `nums` and `ones` accordingly
        self[node].nums = self[left_id].nums();
        self[node].ones = self[left_id].ones();
        self[node].rank -= 2;
    }

    // PUSH

    /// Append `bit` to the rightmost position in the rightmost [`Leaf`].
    #[inline]
    pub fn push(&mut self, bit: bool) {
        // let root = self.root;
        self.push_node(self.root, bit);
    }

    /// Append `bit` to the rightmost position in the rightmost [`Leaf`], descending from `node`.
    ///
    /// Invariances:
    /// - `nums` has number of bits in left subtree
    ///     -> won't change
    /// - `ones` has count of 1-bits in left subtree
    ///     -> won't change
    /// - `balance`-difference between two subtrees must not exceed 2, otherwise rotate
    ///     -> check when creating/inserting a new node/leaf
    /// - `size` should have information about total capacity
    ///     -> update when creating new [`Leaf`]
    fn push_node(&mut self, node: usize, bit: bool) {
        // First, find rightmost Leaf. Descend tree right-based.
        if let Some(r) = self[node].right {
            // if the id `r` is positive, it's a node, if it's negative, it's a leaf
            if r >= 0 {
                // node found. push there
                self.push_node(r as usize, bit);
            } else {
                // rightmost Leaf found. `push_leaf` walks through all possible cases
                self.push_leaf(r, bit);
            }
        } else {
            // no right-side child. create leaf and insert as right child
            let new_leaf = self.create_right_leaf(node);

            // now, we can push into the right side leaf we just created
            self.push_node(node, bit);
        }
    }

    /// Given a [`Node`] `node` and its right child [`Leaf`] `leaf`, attempt to append `bit` to `leaf`.
    ///
    /// # Cases:
    /// - OK: insertion was possible. No need to update anything
    /// - Err: Capacity of right leaf full. Check if left child exists.
    ///     - if no, move right to left, create new right one, push there.
    ///     - if yes, insert new node at position of right leaf, move leaf to left of newly created
    ///     node, create new right leaf, and push there.
    fn push_leaf(&mut self, leaf: isize, bit: bool) {
        match self[leaf].push(bit) {
            // Leaf.push
            Ok(_) => (),
            Err(_) => {
                // Capacity on leaf full.
                // check if left child exists and different from self
                let node = self[leaf].parent;
                // we assume to be the right child of parent
                if self[node].left.is_some() {
                    // Something exists on left side too. So we need to insert a new node at the
                    // right side at the current position of `leaf`.
                    let new_node_id = self.insert_node_at_leaf(leaf);

                    // update `rank` up to the root
                    self.retrace(new_node_id, 1);

                    // next, we need to insert a new Leaf and push into it.
                    // Luckily, we already have code for that
                    self.push_node(new_node_id, bit);
                } else {
                    // No child on left side. Move right leaf over to left side.
                    self.move_right_child_left(node);
                    // then we need to insert a new leaf and push into it.
                    // Luckily, we already have code for that
                    self.push_node(node, bit);
                }
            }
        }
    }

    // TRACING

    /// Retrace and update rank of parent until root (or until they cancel out, or until
    /// a rebalancing happens). `node` self is expected to be updated already.
    ///
    /// # Arguments
    /// * `node: usize` - begin retracing here, first to get updated is parent of `node`
    /// * `depth_change: i8` - if depth change was positive or negative. Addition/Subtraction to
    /// rank depends on which child side it came from.
    pub fn retrace(&mut self, node: usize, depth_change: i8) {
        if self[node].rank == 0 {
            // node/tree is balanced here, no propagation necessary
            return;
        }

        // find out side and parent, update parent accordingly, ascend
        match self.get_node_side(node) {
            Some(Right(p)) => {
                self[p].rank += depth_change;
                self.check_rebalance(node, p, depth_change);
            }
            Some(Left(p)) => {
                self[p].rank -= depth_change;
                self.check_rebalance(node, p, depth_change);
            }
            None => {} // found root, we're done
        }
    }

    #[inline]
    fn check_rebalance(&mut self, node: usize, parent: usize, depth_change: i8) {
        if i8::abs(self[parent].rank) == 2 {
            // we can now rebalance, and don't need to continue tracing
            self.rebalance(node, parent);
            return;
        }
        self.retrace(parent, depth_change);
    }

    fn remove_retrace(&mut self, parent: usize, removed_child: usize) {
        if let Some(l) = self[parent].left {
            if l == removed_child as isize {
                self[parent].left = None;
                self[parent].rank += 1;
                if i8::abs(self[parent].rank) == 2 {
                    self.rebalance_no_child(parent);
                } else {
                    self.retrace(parent, 1);
                }
            }
        }
        if let Some(r) = self[parent].left {
            if r == removed_child as isize {
                self[parent].right = None;
                self[parent].rank -= 1;
                if i8::abs(self[parent].rank) == 2 {
                    self.rebalance_no_child(parent);
                } else {
                    self.retrace(parent, -1);
                }
            }
        }
    }

    // ROTATION

    /// Left rotation of [`Node`]s `x` and `z`.
    ///
    /// Assumes that `z` is right child of `x`, `x.rank == 2` and `z.rank == 1|0`.
    /// (0 only happens for deletion)
    pub fn rotate_left(&mut self, z: usize, x: usize) {
        let mut trace = false;
        let grand_parent = self[x].parent;
        self[z].parent = grand_parent;

        self[x].parent = Some(z);

        self[x].right = self[z].left;
        self[z].left = Some(x as isize);

        if x == self.root {
            // grand_parent == None
            self.root = z;
        } else {
            self[grand_parent.unwrap()].replace_child_with(x as isize, z as isize);
        }

        // only possible in case of deletion
        if self[z].rank == 0 {
            self[x].rank = 1;
            self[z].rank = -1;
            trace = true;
        } else {
            self[z].rank = 0;
            self[x].rank = 0;
        }

        self[z].nums += self[x].nums;
        self[z].ones += self[x].ones;

        // move right subtree of X
        let r = self[x].right.unwrap();
        if r >= 0 {
            // node
            self[r as usize].parent = Some(x);
        } else {
            // leaf
            self[r].parent = x;
        }
        if trace && grand_parent.is_some() {
            self.retrace(grand_parent.unwrap(), -1);
        }
    }

    /// Right rotation of [`Node`]s `x` and `z`.
    ///
    /// Assumes that `z` is left child of `x`, `x.rank == 2` and `z.rank == 1|0`.
    /// (0 only happens for deletion)
    pub fn rotate_right(&mut self, z: usize, x: usize) -> usize {
        let mut trace = false;
        let grand_parent = self[x].parent;
        self[z].parent = grand_parent;

        self[x].parent = Some(z);

        self[x].left = self[z].right;
        self[z].right = Some(x as isize);

        if x == self.root {
            // grand_parent == None
            self.root = z;
        } else {
            self[grand_parent.unwrap()].replace_child_with(x as isize, z as isize);
        }
        // only possible in case of deletion
        if self[z].rank == 0 {
            self[x].rank = 1; // ?
            self[z].rank = -1; // ?
            trace = true;
        } else {
            self[z].rank = 0;
            self[x].rank = 0;
        }

        self[z].nums += self[x].nums;
        self[z].ones += self[x].ones;

        // move left subtree of X
        let r = self[x].left.unwrap();
        if r >= 0 {
            // node
            self[r as usize].parent = Some(x);
        } else {
            // leaf
            self[r].parent = x;
        }
        if trace && grand_parent.is_some() {
            self.retrace(grand_parent.unwrap(), -1);
        }
        todo!(".rotate_right {}", self) // validate
    }

    // BALANCING

    /// Rebalance tree to reestablish the rank difference invariance (valid values -1, 0, 1).
    /// This is done via combinations of left and right rotations. For insertions, at most two
    /// rotations are necessary, deletions might require up until `log(depth)` rotations to
    /// reestablish balance. (rebalancing after deletion requires additional retracing which is not
    /// yet implemented)
    ///
    /// - `parent` is [`Node`] with temporary rank / balance factor violation
    /// - `node` is higher child of `parent`
    pub fn rebalance(&mut self, node: usize, parent: usize) {
        println!(
            "rebalance ranks: parent.r {} node.r {}",
            self[parent].rank, self[node].rank
        );
        println!("rebalance node ids: parent {} node {}", parent, node);
        // invariance has been broken at `parent`, while `node` is the 'higher' child. (unclear
        // which side)
        // match self.get_node_side // TODO: update
        if let Some(r) = self[parent].right {
            if r == node as isize {
                // node is right child
                if self[node].rank >= 0 {
                    println!(" Right Right violation");
                    self.rotate_left(node, parent);
                } else {
                    println!(" Right Left violation");
                    let y = self[node].left.unwrap() as usize;
                    self.rotate_right(y, node);
                    self.rotate_left(y, parent);
                }
            }
        }
        if let Some(l) = self[parent].left {
            if l == node as isize {
                // node is left child
                if self[node].rank <= 0 {
                    println!(" Left Left violation");
                    self.rotate_right(node, parent);
                } else {
                    println!(" Left Right violation");
                    let y = self[node].right.unwrap() as usize;
                    self.rotate_left(y, node);
                    self.rotate_right(y, parent);
                }
            }
        }
    }

    /// Rebalance tree on `parent`, where highest node might not be known. One child has to be of
    /// `|rank| == 1` while the other is `rank == 0`. Safe to assume, given that parent has `|rank|
    /// == 2` (would be zero otherwise).
    pub fn rebalance_no_child(&mut self, parent: usize) {
        if let Some(l) = self[parent].left {
            if l >= 0 {
                if i8::abs(self[l as usize].rank) == 1 {
                    self.rebalance(l as usize, parent);
                }
            }
        }
        if let Some(r) = self[parent].right {
            if r >= 0 {
                if i8::abs(self[r as usize].rank) == 1 {
                    self.rebalance(r as usize, parent);
                }
            }
        }
        panic!("Node has no child with |rank| == 1 but achieved |rank| == 2 somehow")
    }

    // INSERT

    /// Insert `bit` at position `index`
    #[inline]
    pub fn insert(&mut self, index: usize, bit: bool) {
        self.insert_node(self.root, index, bit);
    }

    /// Handle inserting `bit` at position `index` in given `leaf`
    fn insert_leaf(&mut self, leaf: isize, index: usize, bit: bool) {
        // check for leaf full, split, traverse, rebalance, insert if true.
        if self[leaf].nums as u32 >= LeafValue::BITS {
            let node = self.split_leaf(leaf);
            self.insert_node(node, index, bit);
        } else {
            // since we already checked size of `nums`
            unsafe { self[leaf].insert_unchecked(index, bit) };
            println!("unchecked insert of {bit} at {index}");
        }
    }

    /// Handle inserting `bit` at position `index` in given `node`
    fn insert_node(&mut self, node: usize, index: usize, bit: bool) {
        // update `nums` and `ones` values during descent
        if self[node].nums <= index {
            // enter right side
            if let Some(right_id) = self[node].right {
                if right_id >= 0 {
                    self.insert_node(right_id as usize, index - self[node].nums, bit)
                } else {
                    // leaf
                    self.insert_leaf(right_id, index - self[node].nums, bit)
                }
            } else {
                // create right side leaf and insert
                let leaf_id = self.create_right_leaf(node);
                // even retracing won't disturb order
                self.insert_leaf(leaf_id, index - self[node].nums, bit);
            }
        } else {
            // enter left side
            let left_id = self[node].left.unwrap();
            self[node].nums += 1;
            if bit {
                self[node].ones += 1;
            }
            if left_id >= 0 {
                self.insert_node(left_id as usize, index, bit)
            } else {
                // leaf
                self.insert_leaf(left_id, index, bit)
            }
        }
    }

    /// Create [`Leaf`] as right child of `node`, returns id of newly created Leaf.
    pub fn create_right_leaf(&mut self, node: usize) -> isize {
        // get id for new leaf
        let leaf_id = -(self.leafs.len() as isize);

        // create and push new Leaf with `node` as parent
        self.leafs.push(Leaf::new(node));

        // insert newly created leaf to right side
        self[node].right = Some(leaf_id);

        // add +1 to rank for creating leaf on right side
        self[node].rank += 1;

        // update `rank` up to the root
        self.retrace(node, 1);

        leaf_id
    }

    // DELETE

    /// Delete bit at position `index`.
    ///
    /// Steps:
    /// - find Leaf responsible for value with `index` for deletion
    /// - Delete bit from Leaf
    /// - Check if Leaf value smaller than threshold (1/4th `LeafValue::BITS`)
    ///     - if not, nothing happens
    ///     - else: find closest neighbor. Depending on its size:
    ///         - <= 3/4 `LeafValue::BITS`: merge
    ///             - delete Leaf, propagate, rotate, delete Node
    ///         - > 3/4 `LeafValue::BITS`: steal bits
    ///             - how many? ~1/4th
    #[inline]
    pub fn delete(&mut self, index: usize) {
        let leaf = self.apply(Self::delete_leaf, index);
        self.update_left_values(self[leaf].parent, leaf);
        // self.delete_node(self.root, index);
    }

    /// Delete bit at position `index` in `leaf`, handle all cases.
    /// Returns `leaf` where bit got deleted.
    #[inline]
    fn delete_leaf(&mut self, leaf: isize, index: usize) -> isize {
        self[leaf].delete(index).ok().unwrap();
        // check for leaf empty, merge, traverse, rebalance if true
        if self[leaf].nums as u32 <= LeafValue::BITS / 4 {
            self.merge_away(leaf);
        }
        leaf
    }

    fn delete_node(&mut self, node: usize, index: usize) {
        // TODO: update `nums` and `ones` during descent
        // update `nums` and `ones` values during descent
        if self[node].nums <= index {
            // enter right side
            let right_id = self[node].right.unwrap();
            if right_id >= 0 {
                self.delete_node(right_id as usize, index - self[node].nums);
            } else {
                // leaf
                self.delete_leaf(right_id, index - self[node].nums);
            }
        } else {
            // enter left side
            let left_id = self[node].left.unwrap();
            // self[node].nums += 1;
            // if bit {
            //     self[node].ones += 1;
            // }
            // // TODO: welp, information to update nums and bits not really available here.
            if left_id >= 0 {
                self.delete_node(left_id as usize, index);
            } else {
                // leaf
                self.delete_leaf(left_id, index);
            }
        }
        self.viz_stop();
        todo!(".delete_node {}", self);
    }

    // CLOSEST_NEIGHBOR_*

    /// Return closest immediately sequential neighbor to given [`Leaf`] `leaf`, should it exist.
    /// `Either` additionally tells if it was a right or left child.
    pub fn closest_neighbor_leaf(&self, leaf: isize) -> Option<Side<isize>> {
        // first, check other child of immediate parent
        let parent = self[leaf].parent;
        if let Some(l) = self[parent].left {
            if l != leaf {
                // child is on right side of parent
                return Some(Right(l));
            }
        }
        if let Some(r) = self[parent].right {
            if r != leaf {
                // child is on left side of parent
                return Some(Left(r));
            }
        }

        // ascend to parent, try again
        self.closest_neighbor_child(parent)
    }

    /// Try to return a Leaf that is the closest neighbor (left or right) to the given Node
    /// `child` by ascending, and descending the respectively 'other' side of `child`. Fails if no
    /// such neighbor exists.
    pub fn closest_neighbor_child(&self, child: usize) -> Option<Side<isize>> {
        if let Some(p) = self[child].parent {
            if let Some(l) = self[p].left {
                if l != (child as isize) {
                    // child is on right side of parent
                    return self.descend_rightmost(p);
                }
            }
            if let Some(r) = self[p].right {
                if r != (child as isize) {
                    // child is on left side of parent
                    return self.descend_leftmost(p);
                }
            }
            // ascend to parent, try again
            return self.closest_neighbor_child(p);
        }
        // does not exist.
        None
    }

    // DESCEND

    /// Try to return the leftmost Leaf to be found by descending from `node`
    fn descend_leftmost(&self, node: usize) -> Option<Side<isize>> {
        if let Some(l) = self[node].left {
            if l >= 0 {
                return self.descend_leftmost(node as usize);
            } else {
                return Some(Left(l));
            }
        }
        if let Some(r) = self[node].right {
            if r >= 0 {
                return self.descend_leftmost(node as usize);
            } else {
                return Some(Left(r));
            }
        }
        panic!(".descend_leftmost: Node does not have children")
    }

    /// Try to return the rightmost Leaf to be found by descending from `node`
    fn descend_rightmost(&self, node: usize) -> Option<Side<isize>> {
        if let Some(r) = self[node].right {
            if r >= 0 {
                return self.descend_rightmost(node as usize);
            } else {
                return Some(Right(r));
            }
        }
        if let Some(l) = self[node].left {
            if l >= 0 {
                return self.descend_rightmost(node as usize);
            } else {
                return Some(Right(l));
            }
        }
        panic!(".descend_rightmost: Node does not have children")
    }

    // MERGE

    /// Try to find neighboring Leaf and merge into, or steal values if neighbor is too full.
    ///
    /// Assumption: `leaf` has a used size of `<= 3/4 LeafValue::BITS`.
    ///
    /// Merge, when found neighbor has at least `1/4 LeafValue::BITS` to spare. Otherwise, steal.
    pub fn merge_away(&mut self, leaf: isize) {
        // first, find neighboring child.
        if let Some(neighbor) = self.closest_neighbor_leaf(leaf) {
            let n = neighbor.either_into::<isize>();
            // neighbor is leaf. check if we can merge into
            if (self[n].nums as u32) <= { 3 * LeafValue::BITS / 4 } {
                // neighbor has enough room to spare, merge
                let parent = self[leaf].parent;
                self.merge_leafs(leaf, neighbor);
                self.update_left_values_node(parent);
            } else {
                let stolen_bits = self[n].nums - HALF as u8;
                let extension = match neighbor {
                    Right(n) => Right(self[n].split_to_left()),
                    Left(n) => Left(self[n].split_to_right()),
                };
                self[leaf].extend(extension, stolen_bits);
                self.update_left_values(self[leaf].parent, leaf);
            }
            // update parent `nums` and `ones` for neighbor with new bits
            self.update_left_values(self[n].parent, n);
        }
        // no neighbor exists. Cannot merge, but that's ok too
    }

    /// It's expected that `small_leaf` has size `<= 1/4 LeafValue::BITS`, and
    /// size of `merge_or_steal_into` is `<= 3/4 LeafValue::BITS`. Might panic otherwise.
    ///
    /// This operation will remove the Leaf `small_leaf` from `self.leafs`.
    fn merge_leafs(&mut self, small_leaf: isize, merge_or_steal_into: Side<isize>) {
        let leaf = self[small_leaf].clone();
        let parent = self[small_leaf].parent;
        match merge_or_steal_into {
            Left(l) => {
                self[l].extend_from(leaf);
                // left child gets removed, increase rank balance towards right
                self[parent].rank += 1;
            }
            Right(r) => {
                self[r].prepend(leaf);
                // right child gets removed, decrease rank balance towards right
                self[parent].rank -= 1;
            }
        };

        // remove leaf from memory. Parent rank is updated already
        self.swap_remove_leaf(small_leaf);

        // check parent for necessity of rebalancing
        if i8::abs(self[parent].rank) == 2 {
            self.rebalance_no_child(parent);
            // check if `parent` is now empty Node.
            if self[parent].left.is_none() && self[parent].right.is_none() {
                if let Some(gparent) = self[parent].parent {
                    // remove Node.
                    self.swap_remove_node(parent);
                    // delete removed child and retrace
                    self.remove_retrace(gparent, parent);
                }
            }
        } else {
            self.retrace(parent, -1);
        }
    }

    // SWAP_REMOVE

    /// Remove Leaf with given index `leaf`. Swaps with currently last in `self.leafs` and updates
    /// the child index of the parent of the swapped Leaf.
    pub fn swap_remove_leaf(&mut self, leaf: isize) {
        match self.get_leaf_side((self.leafs.len() - 1) as isize) {
            Left(p) => {
                self[p].left = Some(leaf);
                self.leafs.swap_remove((-leaf) as usize);
            }
            Right(p) => {
                self[p].right = Some(leaf);
                self.leafs.swap_remove((-leaf) as usize);
            }
        }
    }

    /// Remove Node with given index `node`. Swaps with currently last Node and updates its parent
    /// index for the swapped child.
    pub fn swap_remove_node(&mut self, node: usize) {
        // figure out situation of node to swap with.
        match self.get_node_side(self.nodes.len() - 1) {
            Some(Left(p)) => {
                // last node is left child of `p`. update parent reference and delete
                self[p].left = Some(node as isize);
                self.nodes.swap_remove(node);
            }
            Some(Right(p)) => {
                // last node is right child of `p`. update parent reference and delete
                self[p].right = Some(node as isize);
                self.nodes.swap_remove(node);
            }
            None => {
                // node doesn't have parent => it's the root!
                self.root = node;
                self.nodes.swap_remove(node);
            }
        }
    }

    // FLIP

    /// Flip bit at position `index`
    #[inline]
    pub fn flip(&mut self, index: usize) {
        self.apply(Self::flip_leaf, index);
    }

    #[inline]
    fn flip_leaf(&mut self, leaf: isize, index: usize) {
        self[leaf].flip(index)
    }

    #[inline]
    fn flip_node(&mut self, node: usize, index: usize) {
        self.apply_node(node, Self::flip_leaf, index)
    }

    // RANK

    /// Return number of `bit`-values before position `index`.
    ///
    /// `&mut self` is only needed due to implementation via [`DynamicBitVector::apply`].
    #[inline]
    pub fn rank(&mut self, index: usize, bit: bool) -> usize {
        self.apply_bitop(Self::rank_leaf, Self::rank_add, index, bit)
    }

    #[inline]
    fn rank_leaf(&mut self, leaf: isize, index: usize, bit: bool) -> usize {
        self[leaf].rank(bit, index)
    }

    fn rank_add(&self, node: usize, left_descent: bool) -> usize {
        if left_descent {
            0
        } else {
            self[node].ones
        }
    }

    // SELECT

    /// Return position of `n`-th `bit`-value
    #[inline]
    pub fn select(&self, n: usize, bit: bool) -> usize {
        self.select_node(self.root, n, bit)
    }

    #[inline]
    fn select_leaf(&self, leaf: isize, n: usize, bit: bool) -> usize {
        self[leaf].select(bit, n)
    }

    fn select_node(&self, node: usize, n: usize, bit: bool) -> usize {
        if self[node].nums - self[node].ones <= n {
            // descend right side
            let right_id = self[node].right.unwrap();
            if right_id >= 0 {
                self[node].nums + self.select_node(right_id as usize, n - self[node].nums, bit)
            } else {
                // leaf
                self[node].nums + self.select_leaf(right_id, n - self[node].nums, bit)
            }
        } else {
            // descend left side
            let left_id = self[node].left.unwrap();
            if left_id >= 0 {
                self.select_node(left_id as usize, n, bit)
            } else {
                // leaf
                self.select_leaf(left_id, n, bit)
            }
        }
    }

    // GET_SIDE

    /// Given some Child `child`, return side on parent and parent index
    #[inline]
    pub fn get_side(&self, child: isize) -> Option<Side<usize>> {
        if child >= 0 {
            self.get_node_side(child as usize)
        } else {
            Some(self.get_leaf_side(child))
        }
    }

    /// Given Leaf `child`, return side on parent and parent index
    pub fn get_leaf_side(&self, child: isize) -> Side<usize> {
        let parent = self[child].parent;
        if let Some(l) = self[parent].left {
            if l == child {
                return Left(parent);
            }
        }
        if let Some(r) = self[parent].right {
            if r == child {
                return Right(parent);
            }
        }
        panic!("leaf L{child} is not child of supposed parent N{parent}")
    }

    /// Given Node `child`, return side on parent and parent index
    pub fn get_node_side(&self, child: usize) -> Option<Side<usize>> {
        if let Some(parent) = self[child as usize].parent {
            if let Some(l) = self[parent].left {
                if l == child as isize {
                    return Some(Left(parent));
                }
            }
            if let Some(r) = self[parent].right {
                if r == child as isize {
                    return Some(Right(parent));
                }
            }
        }
        None
    }

    // NUMS

    /// Capacity used in left subtree
    #[inline]
    pub fn nums(self) -> usize {
        self[self.root].nums
    }

    // ONES

    /// Count of 1-bits in left subtree
    #[inline]
    pub fn ones(self) -> usize {
        self[self.root].ones
    }

    // MISC

    /// Output current tree state to file for visualization and pause execution until input is
    /// given
    #[inline]
    fn viz_stop(&self) {
        println!("stopped for visualization. to continue: [Enter]");
        commands::write_file("tmp.txt", &self.dotviz(0)).unwrap();
        commands::wait_continue();
        println!();
    }

    /// Recursively update parent values in case of left-child modification of `nums` or `ones`,
    /// coming from `child`
    pub fn update_left_values(&mut self, node: usize, child: isize) {
        // check if child is left child
        if let Some(l) = self[node].left {
            if l == child {
                // was left child
                if l >= 0 {
                    // left child is node
                    self[node].nums = self[l as usize].nums;
                    self[node].ones = self[l as usize].ones;
                    // add values from the right child of `l`
                    self.right_child_values(node, self[l as usize].right);
                } else {
                    // left child is leaf
                    self[node].nums = self[l].nums as usize;
                    self[node].ones = self[l].ones();
                }
                if let Some(p) = self[node].parent {
                    self.update_left_values(p, node as isize)
                }
            }
        }
        // everything else (was right child) doesn't matter
    }

    #[inline]
    fn update_left_values_node(&mut self, node: usize) {
        if let Some(l) = self[node].left {
            self.update_left_values(node, l);
        } else {
            self[node].nums = 0;
            self[node].ones = 0;
        }
    }

    /// Graphically, update values `nums` and `ones` from `N1` (`to_update`) by adding values from
    /// `N2` (which happened in `update_left_values`) and `N3` (`left_right_child`), which is
    /// happening here.
    /// ```text
    ///                           ┌───┐
    ///                           │   │
    ///                        ┌──┤N1 │
    ///                        │  │num│
    ///                        │  └───┘
    ///                      ┌─▼─┐
    ///                      │   │
    ///                      │N2 ├───┐
    ///                      │num│   │
    ///                      └───┘ ┌─▼─┐
    ///                            │   │
    ///                            │N3 │
    ///                            │num│
    ///                            └───┘
    /// ```
    #[inline]
    fn right_child_values(&mut self, to_update: usize, left_right_child: Option<isize>) {
        if let Some(r) = left_right_child {
            if r >= 0 {
                // node
                self[to_update].nums += self[r as usize].nums;
                self[to_update].ones += self[r as usize].ones;
            } else {
                // leaf
                self[to_update].nums += self[r].nums as usize;
                self[to_update].ones += self[r].ones();
            }
        }
    }

    /// Split content of `leaf` in two, and replace location with new Node. Afterward, apply
    /// [`DynamicBitVector::retrace`]. Returns id of newly created [`Node`]. Potentially
    /// rebalances when tracing ranks.
    pub fn split_leaf(&mut self, leaf: isize) -> usize {
        // creating new node and making current leaf left child
        let new_node = self.insert_node_at_leaf(leaf);
        self.retrace(new_node, 1);
        // moving left half of leaf to newly created leaf to the right.
        let values = self[leaf].split_to_right();
        let leaf_id = self.create_right_leaf(new_node);
        self[leaf_id].value = values;
        self[leaf_id].nums = (LeafValue::BITS / 2) as u8;

        // making leaf right child of new Node
        new_node
    }
}

// further modules with implementations
mod impls;

#[cfg(test)]
mod tests;
