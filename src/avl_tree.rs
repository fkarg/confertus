#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(unused_variables)]

/// AVL-Tree for Dynamic Bit Vector implementation
#[derive(Debug, PartialEq, Clone)]
pub enum AVL {
    /// Leaf
    /// 0: bitvector containing values
    /// 1: `nums`, number of used bits
    /// 2: `parent` reference of Leaf
    Leaf(u64, usize, Option<Box<AVL>>),
    /// Node which might contain a right and left subtree, and counts on left-sided `nums` and
    /// `ones`.
    ///
    /// `nums` counts the number used bits on the left side, required for indexing.
    ///
    /// `ones` counts the number of one-bits in the left side, used for efficient `rank` and
    /// `select` queries.
    Node {
        /// reference to parent Node
        parent: Option<Box<AVL>>,
        /// left side Node
        left: Option<Box<AVL>>,
        /// right side Node
        right: Option<Box<AVL>>,
        /// number of ones on the left subtree
        ones: usize,
        /// number of 'filled' bits on the left  subtree
        nums: usize,
        /// total number of filled bits ... (unused currently)
        size: usize,
    },
}

/// Various support functions and functionality for [`AVL`]
impl AVL {
    // /// Create new empty AVL-Tree root and subtrees for total available capacity `capacity`.
    // #[inline]
    // pub fn new_with_capacity(capacity: usize) -> Self {
    //     AVL::create(None, None, 0, 0, 0)
    // }

    /// Create new empty AVL-Tree root
    #[inline]
    pub fn new() -> Self {
        AVL::create(None, None, 0, 0, 0)
    }

    /// Create new empty AVL-Tree Leaf
    #[inline]
    pub fn empty() -> Self {
        AVL::Leaf(0_u64, 0, None)
    }

    /// Create AVL-Tree singleton with value `bit`.
    #[inline]
    pub fn singleton(bit: bool) -> Self {
        AVL::Leaf(bit as u64, 1, None)
    }

    /// shorthand function for creating an [`AVL::Node`]
    #[inline]
    pub fn create(
        left: Option<Box<AVL>>,
        right: Option<Box<AVL>>,
        ones: usize,
        nums: usize,
        size: usize,
    ) -> Self {
        AVL::Node {
            left,
            right,
            ones,
            nums,
            size,
            parent: None,
        }
    }

    // TODO later
    // pub fn insert_leaf(mut self, index: usize, val: T) {
    //     match &self {
    //         AVL::Leaf(v, s) => {
    //             assert!(index == 0 && v == 0);
    //             AVL::Leaf(val,
    //         }
    //         AVL::Node {..} => todo!(),
    //         _ => todo!(),
    //     }
    // }

    // fn insert_tree(mut self, index: usize, tree: AVL) {
    //     todo!()
    // }

    /// Returns number of bits used in left subtree or current [`AVL::Leaf`]
    #[inline]
    pub fn nums(self) -> usize {
        match self {
            AVL::Leaf(_, n, _) => n,
            AVL::Node { nums, .. } => nums,
        }
    }

    /// Returns number of 1-bits in left subtree or current [`AVL::Leaf`]
    #[inline]
    pub fn ones(self) -> usize {
        match self {
            AVL::Leaf(v, _, _) => v.count_ones().try_into().unwrap(),
            AVL::Node { ones, .. } => ones,
        }
    }

    /// Return currently available capacity across all subtrees
    pub fn capacity(self) -> usize {
        match self {
            AVL::Leaf(_, s, _) => 64 - s,
            AVL::Node { left, right, .. } => {
                let mut sum = 0;
                if let Some(l) = left {
                    sum += l.capacity();
                }
                if let Some(r) = right {
                    sum += r.capacity();
                }
                sum
            }
        }
    }

    /// Length of total bits currently held
    #[inline(always)]
    pub fn len(self) -> usize {
        match self {
            AVL::Leaf(_, s, _) => s,
            AVL::Node { size, .. } => size,
        }
    }

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    fn rank(self, index: usize) -> u64 {
        todo!()
    }

    fn select(self, index: usize) -> u64 {
        todo!()
    }

    /// Inserts bit `val` at the current last position.
    pub fn push(&mut self, val: bool) {
        match self {
            AVL::Leaf(ref mut v, ref mut s, ref mut parent) => {
                if *s >= 63 {
                    // u64::BITS.try_into().unwrap()

                    // split apart
                    todo!("split leaf apart in two")
                } else {
                    *v |= (val as u64) << *s;
                    *s += 1;
                }
            }
            AVL::Node {
                left,
                ref mut right,
                ones,
                nums,
                ..
            } => {
                if let Some(r) = right {
                    r.push(val);
                    todo!("backprop ones and nums")
                } else {
                    // create leaf
                    *right = Some(Box::new(AVL::singleton(val)));
                    todo!("backprop ones and nums")
                }
            }
        }
    }

    /// Inserts bit `val` at a given position `index`, shifting all bits after it one spot to the
    /// right.
    ///
    /// `index` may be any value up to *and including* `self.len()`.
    ///
    /// # Panics
    /// This panics if `index` is out of bounds (including `self.len()`).
    pub fn insert(&mut self, index: usize, val: bool) {
        match self {
            AVL::Leaf(ref mut values, ref mut num, ref mut parent) => {
                // check for size of current leaf
                if index >= 64 {
                    // u64::BITS.try_into().unwrap()
                    // split apart in two leafs, create node from this one.
                    // so ... usually I'd split in the middle, is it reasonable to assume that
                    // things will usually continue to be added to the right? so maybe put 75% to
                    // the left?
                    todo!("split leaf apart in two")
                } else if *num == index {
                    // insert at last position
                    *num += 1;
                    *values |= (val as u64) << index;
                    // TODO: can potentially be removed for just 'in the middle' code eventually
                } else if *num >= index {
                    // insert somewhere in the middle.
                    let lmask = u64::MAX.rotate_left((64 - index).try_into().unwrap());
                    let rmask = u64::MAX.rotate_right(index.try_into().unwrap());
                    *values = (*values & lmask) | (1 << index) | ((*values & rmask) >> 1);
                    // prints pointer to v instead of v ... but dereferencing not easy
                    println!("nums: {num}, values: {}, index: {index}", values);
                    // todo!("insert elements in the middle")
                } else {
                    // index to insert is further than current size of bitvector
                    panic!("Invalid command: tried to insert at non-existing position")
                }
            }

            AVL::Node {
                ref mut left,
                ref mut right,
                ref mut ones,
                ref mut nums,
                ..
            } => {
                if index < *nums {
                    if let Some(l) = left {
                        l.insert(index, val);
                        todo!("backprop ones and nums")
                    } else {
                        // doesn't happen?
                        // unreachable code, right?

                        // create leaf
                        *left = Some(Box::new(AVL::singleton(val)));
                        // update nums and ones for current node
                        *nums += 1;
                        if val {
                            *ones += 1;
                        }
                        todo!("backprop ones and nums")
                    }
                } else if index >= *nums {
                    if let Some(r) = right {
                        r.insert(index - *nums, val);
                        todo!("backprop ones and nums")
                    } else {
                        // create leaf
                        *right = Some(Box::new(AVL::singleton(val)));
                        todo!("backprop ones and nums")
                    }
                }
            }
        }
    }
}

impl Default for AVL {
    fn default() -> Self {
        Self::new()
    }
}

// struct Index(usize);

struct TreeNode<T> {
    value: Option<T>,
    left: Option<Box<TreeNode<T>>>,
    right: Option<Box<TreeNode<T>>>,
    ones: Option<usize>,
    nums: Option<usize>,
}

// /// Invariance:
// /// - either value is Some
// /// - or ones, nums, and one of left/right is Some
// impl<T> TreeNode<T> {
//     fn insert(mut self, index: usize, val: T) {
//         if let Some(num) = self.nums && index < num {
//         // if self.nums.is_some_and(|&num| index < num) {
//             // insert in left subtree, adapt index
//         // } else if self.nums.is_some_and(|&num| index >= num) {
//         } else if let Some(num) = self.nums && index >= num {
//             // insert in right subtree, adapt index
//         } else if let Some(mut value) = self.value {
//             // insert in value according to index
//         }
//     }
// }
