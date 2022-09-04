pub use super::leaf::*;
pub use super::node::*;
use crate::commands;
use crate::traits::{Dot, DynBitVec, StaticBitVec};
use either;
use either::{Left, Right};
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::ops::{Add, Index, IndexMut};

type Side<T> = either::Either<T, T>;

/// Implementation of Dynamic Bit Vector using self-balancing [AVL
/// tree](https://en.wikipedia.org/wiki/AVL_tree).
///
/// Instance bit size: 56 bytes = 448
/// (not included: bit sizes of instances in Vector structures)
#[derive(Debug, PartialEq, Clone, Default, Hash)]
pub struct DynamicBitVector {
    /// index to root [`Node`], 8 bytes
    pub root: usize, // 8 bytes
    // positively indexed, usize
    /// Vector containing [`Node`], 24 bytes
    pub nodes: Vec<Node>, // 24 bytes
    // negatively indexed, isize
    /// Vector containing [`Leaf`], 24 bytes
    pub leafs: Vec<Leaf>, // 24 bytes
                          // last: isize, // 8 bytes, index to right-most leaf
                          // prev: isize, // 8 bytes, index to previously accessed leaf
}

impl DynamicBitVector {
    // CONSTRUCTOR

    /// Constructs new `DynamicBitVector` with empty root [`Node`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            root: 0,
            nodes: vec![Node::new()], // create root node, but no children yet
            leafs: vec![Leaf::new(0)],
        }
    }

    // ACCESS

    /// Recursive descension to position `index`, based on `node`.
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

    #[inline]
    fn get_leaf(&self, leaf: isize, index: usize) -> bool {
        self[leaf].access(index)
    }

    // LENGTH

    /// Return current number of elements in bitvector.
    pub fn len(&self) -> usize {
        self.apply_bitop(Self::len_leaf, Self::len_add, usize::MAX, false)
    }

    #[inline]
    fn len_node(&self, node: usize, index: usize) -> usize {
        self.apply_bitop_node(node, Self::len_leaf, Self::len_add, index, false)
    }

    #[inline]
    fn len_leaf(&self, leaf: isize, index: usize, bit: bool) -> usize {
        self[leaf].nums()
    }

    #[inline]
    fn len_add(&self, node: usize, is_left_descendent: bool) -> usize {
        self[node].nums
    }

    // APPLY

    /// Descend tree to position `index` and apply function `f` with `f(self, leaf, index) -> T`.
    ///
    /// Used to implement traversal for [`DynamicBitVector::access`], [`DynamicBitVector::flip`],
    /// [`DynamicBitVector::delete`], [`DynamicBitVector::insert`]
    ///
    /// # Panics
    /// If tree invariances are violated
    #[inline]
    pub fn apply<T>(&mut self, mut f: impl FnMut(&mut Self, isize, usize) -> T, index: usize) -> T {
        self.apply_node(self.root, f, index)
    }

    fn apply_node<T>(
        &mut self,
        node: usize,
        mut f: impl FnMut(&mut Self, isize, usize) -> T,
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
    /// T`. Function `g` with `g(self, node, is_left_descent) -> T` is used to modify the return
    /// value of `f` dependent on `node`. Its value is added to the result of recursion.
    ///
    /// Used to implement traversal for [`DynamicBitVector::rank`]
    ///
    /// # Panics
    /// If tree invariances are violated
    #[inline]
    pub fn apply_bitop<T>(
        &self,
        mut f: impl FnMut(&Self, isize, usize, bool) -> T,
        g: impl Fn(&Self, usize, bool) -> T,
        index: usize,
        bit: bool,
    ) -> T
    where
        T: Add<Output = T>,
    {
        self.apply_bitop_node(self.root, f, g, index, bit)
    }

    fn apply_bitop_node<T>(
        &self,
        node: usize,
        mut f: impl FnMut(&Self, isize, usize, bool) -> T,
        g: impl Fn(&Self, usize, bool) -> T,
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
                g(self, node, false) + f(self, right_id, index - self[node].nums, bit)
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
        #[cfg(debug_assertions)]
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
        unreachable!(
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
        #[cfg(debug_assertions)]
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
        #[cfg(debug_assertions)]
        self.validate(&format!(".push of '{bit}'")).unwrap();
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
    /// Assumes that `z` is right child of `x`, `x.rank == 2` and `z.rank == -1|1|0`
    /// (0 can only happens after deletion, and correct ranks might not be zero in all situations
    /// after rotation). This is done via the following steps (in order):
    /// 1. Update parent of Z to parent of X
    /// 2. Update child of parent of X to Z
    /// 3. Update parent of X to Z
    /// 4. Update right child of X to left chiled of Z: T23
    /// 5. Update parent of T23
    /// 6. Update left child of Z to X
    /// 7. Update ranks of X and Z to 0
    /// 8. Update `nums` and `zeros` of Z
    /// ```text
    ///         │parent                                          │parent
    ///         │                                                │
    ///       ┌─▼─┐                                            ┌─▼─┐
    ///   left│ X │right                                   left│ Z │right
    ///   ┌───┤   ├───┐                                    ┌───┤   ├───┐
    ///   │   │r:2│   │                                    │   │r:0│   │
    /// ┌─▼─┐ └───┘ ┌─▼─┐                                ┌─▼─┐ └───┘ ┌─▼─┐
    /// │   │   left│ Z │right    left rotation      left│ X │right  │   │
    /// │T1 │   ┌───┤   ├───┐                        ┌───┤   ├───┐   │T4 │
    /// │   │   │   │r:1│   │     ────────────►      │   │r:0│   │   │   │
    /// └───┘ ┌─▼─┐ └───┘ ┌─▼─┐                    ┌─▼─┐ └───┘ ┌─▼─┐ └───┘
    ///       │   │       │   │                    │   │       │   │
    ///       │T23│       │T4 │                    │T1 │       │T23│
    ///       │   │       │   │                    │   │       │   │
    ///       └───┘       └───┘                    └───┘       └───┘
    /// ```
    /// See also the [wikipedia article on AVL-tree
    /// rebalancing](https://en.wikipedia.org/wiki/AVL_tree#Rebalancing).
    pub fn rotate_left(&mut self, z: usize, x: usize) {
        #[cfg(debug_assertions)]
        println!("left-rotate N{x} (x) and N{z} (z, lower and right child)");
        debug_assert!(self[x].rank == 2);
        debug_assert!(self[z].rank == 1);
        self.rotate_left_new(z, x);
    }

    #[inline]
    fn rotate_left_new(&mut self, z: usize, x: usize) {
        // new implementation of rotate_left, not yet supporting more complex rotations
        // 1
        self[z].parent = self[x].parent;
        // 2
        if let Some(p) = self[z].parent {
            self[p].replace_child_with(x as isize, z as isize);
        } else {
            self.root = z;
        }
        // 3
        self[x].parent = Some(z);
        // 4
        self[x].right = self[z].left;

        // 5
        let r = self[x].right.unwrap();
        if r >= 0 {
            // node
            self[r as usize].parent = Some(x);
        } else {
            // leaf
            self[r].parent = x;
        }

        // 6
        self[z].left = Some(x as isize);

        // 7
        self[z].rank = 0;
        self[x].rank = 0;

        // 8
        // let (n, o) = self.full_nums_ones(x as isize);
        self[z].nums += self[x].nums;
        self[z].ones += self[x].ones;
    }

    #[inline]
    fn rotate_left_old(&mut self, z: usize, x: usize) {
        let mut trace = false;
        let grand_parent = self[x].parent;
        // update parents
        self[z].parent = grand_parent;
        self[x].parent = Some(z);

        // move T23
        self[x].right = self[z].left;
        self[z].left = Some(x as isize);

        if x == self.root {
            // grand_parent == None
            self.root = z;
        } else {
            self[grand_parent.unwrap()].replace_child_with(x as isize, z as isize);
        }

        // zero is only possible after deletion
        if self[z].rank != 0 {
            self[z].rank = 0;
            self[x].rank = 0;
        } else {
            // according to wikipedia
            self[x].rank = 1;
            self[z].rank = -1;
            // deletion requires additional tracing of changes
            trace = true;
        }

        let (n, o) = self.full_nums_ones(x as isize);
        self[z].nums = n;
        self[z].ones = o;

        // properly set parent of T23 to X
        if let Some(r) = self[x].right {
            // can it be None here?
            if r >= 0 {
                // node
                self[r as usize].parent = Some(x);
            } else {
                // leaf
                self[r].parent = x;
            }
        }
        if trace {
            if let Some(g) = grand_parent {
                self.retrace(g, -1);
            }
        }
    }

    /// Right rotation of [`Node`]s `z` and `x` to reestablish rank-difference invariant.
    ///
    /// Assumes that `z` is left child of `x`, `x.rank == -2` and `z.rank == -1|1|0`
    /// (0 can only happens after deletion, and correct ranks might not be zero in all situations
    /// after rotation). We won't need to recursively update `nums` and `ones, as they won't
    /// change from the perspective of `parent`.
    /// ```text
    ///               │parent                               │parent
    ///               │                                     │
    ///             ┌─▼──┐                                 ┌─▼─┐
    ///         left│ X  │right                        left│ Z │right
    ///         ┌───┤    ├───┐                         ┌───┤   ├───┐
    ///         │   │r:-2│   │                         │   │r:0│   │
    ///       ┌─▼──┐└────┘ ┌─▼─┐   right rotation    ┌─▼─┐ └───┘ ┌─▼─┐
    ///   left│ Z  │right  │   │                     │   │   left│ X │right
    ///   ┌───┤    ├───┐   │T4 │    ───────────►     │T1 │   ┌───┤   ├───┐
    ///   │   │r:-1│   │   │   │                     │   │   │   │r:0│   │
    /// ┌─▼─┐ └────┘ ┌─▼─┐ └───┘                     └───┘ ┌─▼─┐ └───┘ ┌─▼─┐
    /// │   │        │   │                                 │   │       │   │
    /// │T1 │        │T23│                                 │T23│       │T4 │
    /// │   │        │   │                                 │   │       │   │
    /// └───┘        └───┘                                 └───┘       └───┘
    /// ```
    /// See also the [wikipedia article on AVL-tree
    /// rebalancing](https://en.wikipedia.org/wiki/AVL_tree#Rebalancing).
    pub fn rotate_right(&mut self, z: usize, x: usize) {
        #[cfg(debug_assertions)]
        println!("right-rotate N{x} (x) and N{z} (z, lower and left child)");
        debug_assert!(self[x].rank == -2);
        debug_assert!(self[z].rank == -1);
        self.rotate_right_new(z, x);
    }

    #[inline]
    fn rotate_right_new(&mut self, z: usize, x: usize) {
        // new implementation of rotate_right, not yet fully featured
        // 1
        self[z].parent = self[x].parent;
        // 2
        if let Some(p) = self[z].parent {
            self[p].replace_child_with(x as isize, z as isize);
        } else {
            self.root = z;
        }
        // 3
        self[x].parent = Some(z);
        // 4
        self[x].left = self[z].right;

        // 5
        let r = self[x].left.unwrap();
        if r >= 0 {
            // node
            self[r as usize].parent = Some(x);
        } else {
            // leaf
            self[r].parent = x;
        }

        // 6
        self[z].right = Some(x as isize);

        // 7
        self[z].rank = 0;
        self[x].rank = 0;

        // 8
        // let (n, o) = self.full_nums_ones(x as isize);
        self[x].nums -= self[z].nums;
        self[x].ones -= self[z].ones;
    }

    #[inline]
    fn rotate_right_old(&mut self, z: usize, x: usize) {
        // if we need to trace back changes in rank later, which we only might in case of deletion
        // (as it might cascade for up to `log n` rotations).
        let mut trace = false;

        // update parent pointers of x and z
        let grand_parent = self[x].parent;
        self[z].parent = grand_parent;
        self[x].parent = Some(z);

        // moving of T23
        self[x].left = self[z].right;

        self[z].right = Some(x as isize);

        if x == self.root {
            // it means that `grand_parent` was None
            self.root = z;
        } else {
            self[grand_parent.unwrap()].replace_child_with(x as isize, z as isize);
        }

        // only possible in case of deletion
        if self[z].rank == 0 {
            self[x].rank = 1; // not sure for right rotation
            self[z].rank = -1; // not sure for right rotation, maybe switch?
            trace = true;
        } else {
            self[z].rank = 0;
            self[x].rank = 0;
        }

        if let Some(l) = self[x].left {
            let (n, o) = self.full_nums_ones(l);
            self[x].nums = n;
            self[x].ones = o;
        } else {
            self[x].nums = 0;
            self[x].ones = 0;
        }

        // update parent pointer of T23, which might actually not exist (happened before)
        #[cfg(debug_assertions)]
        println!("left of {x}: {:?}", self[x].left);
        if let Some(l) = self[x].left {
            if l >= 0 {
                // node
                self[l as usize].parent = Some(x);
            } else {
                // leaf
                self[l].parent = x;
            }
        }
        if trace {
            if let Some(g) = grand_parent {
                self.retrace(g, -1);
            }
        }
    }

    // BALANCING

    /// Rebalance tree to reestablish the rank difference invariance (valid values -1, 0, 1).
    /// This is done via combinations of left and right rotations. For insertions, at most two
    /// rotations are necessary, deletions might require up until `log(depth)` rotations to
    /// reestablish balance. (rebalancing after deletion requires additional retracing which is not
    /// yet implemented)
    ///
    /// - `parent` is [`Node`] with temporary rank / balance factor violation
    /// - `node` is child of `parent` with higher inbalance
    pub fn rebalance(&mut self, node: usize, parent: usize) {
        #[cfg(debug_assertions)]
        println!(
            ".rebalance: rank of parent[{parent}]: {}, node[{node}]: {}",
            self[parent].rank, self[node].rank
        );
        #[cfg(debug_assertions)]
        println!("rebalance node ids: parent {} node {}", parent, node);
        self.viz();
        // invariance has been broken at `parent`, while `node` is the 'higher' child. (unclear
        // which side)
        // match self.get_node_side // TODO: update
        if let Some(r) = self[parent].right {
            if r == node as isize {
                // node is right child
                if self[node].rank >= 0 {
                    #[cfg(debug_assertions)]
                    println!(" Right Right violation");
                    self.rotate_left(node, parent);
                } else {
                    #[cfg(debug_assertions)]
                    println!(" Right Left violation");
                    let y = self[node].left.unwrap() as usize;
                    self.rotate_right(y, node);
                    self.viz();
                    self.rotate_left(y, parent);
                }
            }
        }
        if let Some(l) = self[parent].left {
            if l == node as isize {
                // node is left child
                if self[node].rank <= 0 {
                    #[cfg(debug_assertions)]
                    println!(" Left Left violation");
                    self.rotate_right(node, parent);
                } else {
                    #[cfg(debug_assertions)]
                    println!(" Left Right violation");
                    let y = self[node].right.unwrap() as usize;
                    self.rotate_left(y, node);
                    self.viz();
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
            if l >= 0 && i8::abs(self[l as usize].rank) == 1 {
                self.rebalance(l as usize, parent);
            }
        }
        if let Some(r) = self[parent].right {
            if r >= 0 && i8::abs(self[r as usize].rank) == 1 {
                self.rebalance(r as usize, parent);
            }
        }
        unreachable!("Node has no child with |rank| == 1 but achieved |rank| == 2 somehow")
    }

    // INSERT

    /// Handle inserting `bit` at position `index` in given `leaf`
    fn insert_leaf(&mut self, leaf: isize, index: usize, bit: bool) -> Result<(), &'static str> {
        // check for leaf full, split, traverse, rebalance, insert if true.
        if u32::from(self[leaf].nums) >= LeafValue::BITS && self[self[leaf].parent].left.is_none() {
            self.move_right_child_left(self[leaf].parent);

            let values = self[leaf].split_to_right();
            let leaf_id = self.create_right_leaf(self[leaf].parent);
            self[leaf_id].value = values;
            self[leaf_id].nums = (LeafValue::BITS / 2) as u8;
            self.update_left_values_only(self[leaf].parent, leaf);

            self.insert_node(self[leaf].parent, index, bit)?;
        } else if u32::from(self[leaf].nums) >= LeafValue::BITS {
            let node = self.split_leaf(leaf);
            // try insertion at the newly created node
            self.insert_node(node, index, bit)?;
        } else {
            self[leaf].insert(index, bit)?;
        }
        Ok(())
    }

    /// Handle inserting `bit` at position `index` in given `node`.
    ///
    /// Not to be confused with `?`, which is for inserting a `Node`.
    fn insert_node(&mut self, node: usize, index: usize, bit: bool) -> Result<(), &'static str> {
        // update `nums` and `ones` values during descent
        if self[node].nums <= index {
            // enter right side
            if let Some(right_id) = self[node].right {
                if right_id >= 0 {
                    self.insert_node(right_id as usize, index - self[node].nums, bit)?;
                } else {
                    // leaf
                    self.insert_leaf(right_id, index - self[node].nums, bit)?;
                }
            } else {
                // create right side leaf and insert
                let leaf_id = self.create_right_leaf(node);
                // even retracing won't disturb order
                self.insert_leaf(leaf_id, index - self[node].nums, bit)?;
            }
        } else {
            // enter left side
            let left_id = self[node].left.unwrap();
            self[node].nums += 1;
            if bit {
                self[node].ones += 1;
            }
            if left_id >= 0 {
                self.insert_node(left_id as usize, index, bit)?;
            } else {
                // leaf
                self.insert_leaf(left_id, index, bit)?;
            }
        }
        Ok(())
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

    /// Delete bit at position `index` in `leaf`, handle all cases.
    /// Returns `leaf` where bit got deleted.
    #[inline]
    fn delete_leaf(&mut self, leaf: isize, index: usize) -> Result<isize, &'static str> {
        self[leaf].delete(index)?;
        // check for leaf empty, merge, traverse, rebalance if true
        if u32::from(self[leaf].nums) <= LeafValue::BITS / 4 {
            self.merge_away(leaf);
        }
        Ok(leaf)
    }

    fn delete_node(&mut self, node: usize, index: usize) -> Result<isize, &'static str> {
        // TODO: update `nums` and `ones` during descent
        // update `nums` and `ones` values during descent
        if self[node].nums <= index {
            // enter right side
            let right_id = self[node].right.unwrap();
            if right_id >= 0 {
                self.delete_node(right_id as usize, index - self[node].nums)
            } else {
                // leaf
                self.delete_leaf(right_id, index - self[node].nums)
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
                self.delete_node(left_id as usize, index)
            } else {
                // leaf
                self.delete_leaf(left_id, index)
            }
        }
    }

    // CLOSEST_NEIGHBOR_*

    /// Return closest immediately sequential neighbor to given [`Leaf`] `leaf`, should it exist.
    /// `Either` additionally tells if it was a right or left child.
    #[must_use]
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
    #[must_use]
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
        unreachable!(".descend_leftmost: Node does not have children")
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
        unreachable!(".descend_rightmost: Node does not have children")
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
            if u32::from(self[n].nums) <= { 3 * LeafValue::BITS / 4 } {
                // neighbor has enough room to spare, merge
                let parent = self[leaf].parent;
                self.merge_leafs(leaf, neighbor);
                self.update_left_values_node(parent);
            } else {
                // steal so many that the other leaf will keep exactly half
                let stolen_bits = self[n].nums - HALF as u8;
                // let extension = neighbor.map_right(|n| self[n].split_to_left()).map_left(|n| self[n].split_to_right());
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
                self[l].extend_from(&leaf);
                // left child gets removed, increase rank balance towards right
                self[parent].rank += 1;
            }
            Right(r) => {
                self[r].prepend(&leaf);
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

    #[inline]
    fn flip_leaf(&mut self, leaf: isize, index: usize) -> isize {
        self[leaf].flip(index);
        leaf
    }

    #[inline]
    fn flip_node(&mut self, node: usize, index: usize) -> isize {
        self.apply_node(node, Self::flip_leaf, index)
    }

    // RANK

    #[inline]
    fn rank_leaf(&self, leaf: isize, index: usize, bit: bool) -> usize {
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
    #[must_use]
    pub fn get_side(&self, child: isize) -> Option<Side<usize>> {
        if child >= 0 {
            self.get_node_side(child as usize)
        } else {
            Some(self.get_leaf_side(child))
        }
    }

    /// Given Leaf `child`, return side on parent and parent index
    #[must_use]
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
        unreachable!("leaf L{child} is not child of supposed parent N{parent}")
    }

    /// Given Node `child`, return side on parent and parent index
    #[must_use]
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

    // MISC

    /// Output current tree state to file for visualization and pause execution until some input is
    /// given
    #[inline]
    #[cfg(debug_assertions)]
    fn viz_stop(&self) {
        self.viz();
        print!("stopped for visualization. continue by pressing [Enter]");
        std::io::stdout().flush().unwrap();
        commands::wait_continue();
        println!();
    }

    /// Write current tree state to file for visualization, but don't pause execution
    #[inline]
    #[cfg(debug_assertions)]
    fn viz(&self) {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let h = hasher.finish(); // {h:x}
        let fname = format!("/tmp/tmp_{h:x}");

        commands::write_file(&fname, &self.dotviz(0)).unwrap();
        println!("wrote current tree state to '{fname}'");
    }

    #[cfg(not(debug_assertions))]
    fn viz_stop(&self) {}

    #[cfg(not(debug_assertions))]
    fn viz(&self) {}

    /// Non-recursive updating of parent `nums` and `ones` values.
    ///
    /// Returns if `child` is left side child of `node` or not.
    pub fn update_left_values_only(&mut self, node: usize, child: isize) -> bool {
        // check if child is left child
        if let Some(l) = self[node].left {
            if l == child {
                // was left child
                let (n, o) = self.full_nums_ones(child);
                self[node].nums = n;
                self[node].ones = o;
                return true;
            }
            // ignore if we came from right child
        }
        // no left child? no values to update
        false
    }

    /// Recursively update parent values in case of left-child modification of `nums` or `ones`,
    /// coming from `child`.
    pub fn update_left_values(&mut self, node: usize, child: isize) {
        // do most of actual work first
        if self.update_left_values_only(node, child) {
            // recurse if values got updated and parent exists
            if let Some(p) = self[node].parent {
                self.update_left_values(p, node as isize);
            }
        }
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
        self.update_left_values_only(new_node, leaf);

        // making leaf right child of new Node
        new_node
    }

    // MISC

    /// Return the id of leaf for `index`
    #[inline]
    fn leaf_id(&mut self, leaf: isize, index: usize) -> isize {
        leaf
    }

    /// Return `nums` and `ones` of `child` (`N2`) from both its left and right subtrees.
    ///
    /// Graphically, return fully redundant indexing support values `nums` and `ones` for `N1` by
    /// adding left-values from `N2` and `N3`, as well as right-values from `N3` (recursively until
    /// leaf).
    /// ```text
    ///                           ┌───┐
    ///                           │   │
    ///                        ┌──┤N1 │
    ///                        │  │num│
    ///                      ┌─▼─┐└───┘
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
    #[must_use]
    pub fn full_nums_ones(&self, child: isize) -> (usize, usize) {
        if child >= 0 {
            let node = child as usize;
            // node
            if let Some(r) = self[node].right {
                let (n, o) = self.full_nums_ones(r);
                (n + self[node].nums, o + self[node].ones)
            } else {
                (self[node].nums, self[node].ones)
            }
        } else {
            // leaf
            (self[child].nums as usize, self[child].ones())
        }
    }

    // VALIDATION

    /// Validate correctness off all values `nums` and `ones` throughout the tree.
    /// Returns both `nums` and `ones` as tuple or failure node otherwise.
    ///
    /// `add` is additional 'source'-string, as traceback where the failed validation happened.
    #[inline]
    fn validate(&self, add: &str) -> Result<(usize, usize), &str> {
        self.viz();
        self.validate_node(self.root, add)
    }

    fn validate_node(&self, node: usize, add: &str) -> Result<(usize, usize), &str> {
        let (mut n, mut o) = (0, 0);
        if let Some(l) = self[node].left {
            if l >= 0 {
                let (nl, ol) = self.validate_node(l as usize, add)?;
                n += nl;
                o += ol;
            } else {
                // leaf
                n += self[l].nums();
                o += self[l].ones();
            }
        }
        // validate correctness
        assert_eq!(
            self[node].nums, n,
            "`nums` is wrong in Node[{node}]: {} != {n}\n{add}",
            self[node].nums
        );
        assert_eq!(
            self[node].ones, o,
            "`ones` is wrong in Node[{node}]: {} != {o}\n{add}",
            self[node].ones
        );

        // check right side, add to return value
        if let Some(r) = self[node].right {
            if r >= 0 {
                let (nr, or) = self.validate_node(r as usize, add)?;
                n += nr;
                o += or;
            } else {
                // leaf
                n += self[r].nums();
                o += self[r].ones();
            }
        }
        Ok((n, o))
    }
}

// further modules with implementations
mod impls;

#[cfg(test)]
mod tests;
