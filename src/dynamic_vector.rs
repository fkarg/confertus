pub use super::leaf::*;
pub use super::node::*;
use crate::commands;
use crate::traits::Dot;
use std::fmt;
use std::ops::{Index, IndexMut};

type Child = bool;

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
                          // last: isize, // 8 bytes, index to right-most leaf
                          // prev: isize, // 8 bytes, index to previously accessed leaf
}

impl fmt::Display for DynamicBitVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
        // write!(f, "BV[root {}\nnodes: {:?}\nleafs: {:?}]", self.root, self.nodes, self.leafs)
        // f.debug_struct("DynamicBitVector")
        //        .field("root", &self.root)
        //        .field("nodes", &self.nodes.iter().enumerate())
        //        .field("leafs", &self.leafs.iter().enumerate())
        //        .finish()
    }
}

/// Return [`Node`] for `usize` indexing
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

/// Return [`Leaf`] for `isize` indexing
impl Index<isize> for DynamicBitVector {
    type Output = Leaf;

    fn index(&self, index: isize) -> &Self::Output {
        let uidx = if index < 0 {
            -index as usize
        } else {
            index as usize
        };
        &self.leafs[uidx]
    }
}

impl IndexMut<isize> for DynamicBitVector {
    fn index_mut(&mut self, index: isize) -> &mut Self::Output {
        let uidx = if index < 0 {
            -index as usize
        } else {
            index as usize
        };
        &mut self.leafs[uidx]
    }
}

impl DynamicBitVector {
    /// Constructs new `DynamicBitVector` with empty root [`Node`].
    pub fn new() -> Self {
        DynamicBitVector {
            root: 0,
            nodes: vec![Node::new()], // create root node, but no children yet
            leafs: vec![Leaf::new(0)],
        }
    }

    /// Return value at position `index` of `DynamicBitVector`.
    #[inline(always)]
    pub fn get(&self, index: usize) -> bool {
        self.get_node(self.root, index)
    }

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

    /// Insert intermediary [`Node`] `int_node_id` between [`Leaf`] `child_id` and parent Node.
    /// `child_id` will always be inserted as the left child of the new intermediary node.
    /// Updates references accordingly.
    ///
    /// Assumes that intermediary node does not have children (overwrites `left` child otherwise)
    /// or otherwise relevant information (`nums` and `ones` get overwritten too).
    fn insert_node(&mut self, child_id: isize, int_node_id: usize) {
        println!("Insert Node {} for {}", int_node_id, child_id);
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
        println!(".insert_node {}", self);
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
        self.insert_node(leaf, new_node_id);
        new_node_id
    }

    /// Move the right subtree to the left side. Expects the left subtree to be empty (will be
    /// overwritten otherwise) and the right to be nonempty (Panics otherwise).
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

    /// Append `bit` to the rightmost position in the rightmost [`Leaf`].
    #[inline]
    pub fn push(&mut self, bit: bool) {
        // let root = self.root;
        self.push_to(self.root, bit);
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
    fn push_to(&mut self, node: usize, bit: bool) {
        // First, find rightmost Leaf. Descend tree right-based.
        if let Some(r) = self[node].right {
            // if the id `r` is positive, it's a node, if it's negative, it's a leaf
            if r >= 0 {
                // node found. push there
                self.push_to(r as usize, bit);
            } else {
                // rightmost Leaf found. `push_right_leaf` walks through all possible cases
                self.push_right_leaf(node, r, bit);
            }
        } else {
            // no right-side child. create leaf and insert as right child
            let new_leaf_id = -(self.leafs.len() as isize);
            self.leafs.push(Leaf::new(node));
            println!(
                "Create Leaf {} on right side of {:#?}",
                new_leaf_id, self[node]
            );
            // inseart newly created leaf to right side
            self[node].right = Some(new_leaf_id);
            // self[node].size += LeafValue::BITS as usize;

            // add +1 to rank for creating leaf on right side
            self[node].rank += 1;

            commands::write_file("tmp.txt", &self.dotviz(0)).unwrap();
            commands::wait_continue();

            // update `rank` up to the root
            self.retrace(node, 1);

            // now, we can push into the right side leaf we just created
            self.push_to(node, bit);
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
    fn push_right_leaf(&mut self, node: usize, leaf: isize, bit: bool) {
        match self[leaf].push(bit) {
            // Leaf.push
            Ok(_) => (),
            Err(_) => {
                // Capacity on right leaf full.
                // check if left child exists
                if self[node].left.is_some() {
                    // Something exists on left side. So we need to insert a new node at the right
                    // side at the current position of `leaf`.
                    let new_node_id = self.insert_node_at_leaf(leaf);

                    // update `rank` up to the root
                    self.retrace(new_node_id, 1);
                    // self.propagate_size_balance_up(node)

                    // check if tree needs to rebalance now
                    // if i8::abs(self[node].rank) == 1 {
                    //     // tree needs rebalancing.
                    //     self.rebalance(node);
                    //     // afterwards, insert at top again
                    // }

                    // next, we need to insert a new Leaf and push into it.
                    // Luckily, we already have code for that
                    self.push_to(new_node_id, bit);
                } else {
                    // No child on left side. Move right leaf over to left side.
                    // then we need to insert a new leaf and push into it.
                    // Luckily, we already have code for that
                    self.move_right_child_left(node);
                    self.push_to(node, bit);
                }
            }
        }
    }

    /// retrace rank of parent until root (or cancel, or rebalance)
    pub fn retrace(&mut self, node: usize, diff: i8) {
        if self[node].rank == 0 {
            return;
        }
        // first, find parent
        if let Some(parent) = self[node].parent {
            self[parent].rank += diff;
            if self[parent].rank == 0 {
                // we cancelled out an earlier inbalance. stop.
                return;
            } else if i8::abs(self[parent].rank) == 2 {
                // we can now rebalance, no need to continue tracing
                self.rebalance(node, parent);
                return;
            }
            // propagate to parent
            self.retrace(parent, diff);
        }
        // else: found root, we're done
    }

    /// Left rotation of [`Node`]s `x` and `z`.
    ///
    /// Assumes that `z` is right child of `x`, `x.rank == 2` and `z.rank == 1|0`.
    /// (0 only happens for deletion)
    pub fn rotate_left(&mut self, z: usize, x: usize) {
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
    }

    /// Right rotation of [`Node`]s `x` and `z`.
    ///
    /// Assumes that `z` is left child of `x`, `x.rank == 2` and `z.rank == 1|0`.
    /// (0 only happens for deletion)
    pub fn rotate_right(&mut self, z: usize, x: usize) -> usize {
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
        todo!(".rotate_right {}", self)
    }

    /// Rebalance tree to reestablish the rank difference invariance (valid values -1, 0, 1).
    /// This is done via combinations of left and right rotations. For insertions, at most two
    /// rotations are necessary, deletions might require up until `log(depth)` rotations to
    /// reestablish balance.
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
        if let Some(r) = self[parent].right {
            if r as usize == node {
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
            if l as usize == node {
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
        // todo!(".rebalance {}", self)
    }

    pub fn insert(&mut self, index: usize, bit: bool) {
        todo!(".insert {}", self.dotviz(0))
    }

    pub fn delete(&mut self, index: usize) {
        todo!(".delete {}", self)
    }

    pub fn flip(&mut self, index: usize) {
        todo!(".flip {}", self)
    }

    pub fn rank(&mut self, bit: bool, index: usize) {
        todo!(".rank {}", self)
    }

    pub fn select(&mut self, bit: bool, index: usize) {
        todo!(".select {}", self)
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

impl Dot for DynamicBitVector {
    fn dotviz(&self, self_id: isize) -> String {
        format!(
            "\n\ndigraph tree {{ \
            BV [label=<DynamicBitVector>];\n\
            BV -> N{} [label=<root>];\n\
            {} \n\
            {} \n\
            }}\n\n",
            self.root,
            self.nodes
                .iter()
                .enumerate()
                .map(|(e, x)| x.dotviz(e as isize))
                .collect::<String>(),
            self.leafs
                .iter()
                .enumerate()
                .map(|(e, x)| x.dotviz(e as isize))
                .collect::<String>(),
        )
    }
}
