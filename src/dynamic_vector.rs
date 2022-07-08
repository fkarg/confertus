// use super::traits;
use std::ops::{Index, IndexMut};

/// Implementation of Dynamic Bit Vector using AVL/RB tree.
#[derive(Debug, PartialEq, Clone)]
pub struct DynamicBitVector {
    /// index to root [`Node`]
    pub root: usize,
    // positively indexed, usize
    /// Vector containing [`Node`]
    pub nodes: Vec<Node>,
    // negatively indexed, isize
    /// Vector containing [`Leaf`]
    pub leafs: Vec<Leaf>,
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


impl DynamicBitVector {
    pub fn new() -> Self {
        DynamicBitVector {
            root: 0,
            nodes: vec![Node::new()], // create root node, but no children yet
            leafs: vec![],
        }
    }

    pub fn push(&mut self, bit: bool) {
        let root = self.root;
        self[root].push(bit);
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

/// Node element of [`DynamicBitVector`]. Contains references (indices) to parent Node,
#[derive(Debug, PartialEq, Clone)]
pub struct Node {
    // TODO: remove option from values to reduce used bit sizes
    /// reference to parent Node
    pub parent: Option<usize>,
    /// left side subtree where `isize` is the index to child element
    pub left: Option<isize>,
    /// right side subtree where `isize` is the index to child element
    pub right: Option<isize>,
    /// total number of filled bits, across both subtrees
    pub size: usize,
    /// number of 'filled' bits on the left  subtree
    pub nums: usize,
    /// number of ones on the left subtree
    pub ones: usize,
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
        }
    }
    pub fn push(&mut self, bit: bool) {
        // cases:
        // - simply insert to rightmost child?
        todo!()
    }
}

/// Leaf element of [`DynamicBitVector`]. Next to its value ([`u128`]) and bits used inside
/// (`nums`), it contains a reference to its parent [`Node`].
#[derive(Debug, PartialEq, Clone)]
pub struct Leaf {
    /// reference to parent [`Node`]
    pub parent: usize,
    /// container for actual bit values
    pub value: u128,
    /// number of bits used in `value`-container. Below `u128::BITS == 128`, so `u8::MAX = 255` is
    /// sufficient
    pub nums: u8, // realistically below u128::BITS, so u8::MAX = 255 is sufficient
}

impl Leaf {
    pub fn new(parent: usize) -> Self {
        Leaf {
            parent,
            value: 0,
            nums: 0,
        }
    }
}
