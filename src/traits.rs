use core::arch::x86_64::{_pdep_u64, _tzcnt_u64};

/// Functions associated with static bit vectors. Not to be confused with specific containers such
/// as [`u64`], [`u128`] or particulary [`Leaf`](crate::Leaf), which additionally tracks the number
/// of used bits, `nums`, and a parent [`Node`](crate::Node).
pub trait StaticBitVec {
    type Intern;

    /// Return number of on-bits in container
    fn ones(&self) -> usize;

    /// Access bit value at position `index`
    fn access(&self, index: usize) -> bool;

    /// Returns number of `bit`-values up to `index` in container
    ///
    /// runtime complexity: O(1) to O(w)
    fn rank(&self, bit: bool, index: usize) -> usize;

    /// Return index of `n`-th `bit`-value in container
    ///
    /// runtime complexity: O(1) to O(w)
    fn select(&self, bit: bool, n: usize) -> usize;

    /// Return full internal container
    fn values(&self) -> Self::Intern;
}

/// Functions associated with dynamic bit vectors.
pub trait DynBitVec: StaticBitVec {
    /// Insert `bit` at position `index` in underlying container
    ///
    /// runtime complexity: O(1) to O(w)
    fn insert(&mut self, index: usize, bit: bool) -> Result<(), &'static str>;

    /// Remove bit value at position `index`
    ///
    /// runtime complexity: O(1) to O(w)
    fn delete(&mut self, index: usize) -> Result<(), &'static str>;

    /// Flip bit at position `index`, updates `ones` and `num` values accordingly
    ///
    /// runtime complexity: O(1)
    fn flip(&mut self, index: usize);

    /// Return used capacity of underlying container
    fn nums(&self) -> usize;

    /// Return used capacity of underlying container
    #[inline]
    fn len(&self) -> usize {
        self.nums()
    }

    /// If the Leaf has active values
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // /// `bitset  i` sets `i`-th bit to 1
    // /// updates `ones` and `num` accordingly
    // fn bitset(self, i: usize);

    // /// `bitclear i` sets `i`-th bit to 0
    // /// updates `ones` and `num` accordingly
    // fn bitclear(self, i: usize);
}

pub trait DynBitTree {
    /// `deletenode v` delete node v
    fn deletenode(self, v: usize);

    /// `insertchild v i k` insert new `i`-th child of node `v` such that the new node becomes
    /// parent of the previously `i`-th to (`i + k - 1`)-th child of `v`
    /// ### Examples
    /// insertchild (T , v , i , 0) inserts new leaf
    /// insertchild (T , v , i , 1) inserts new parent of only the previously i-th child
    /// insertchild (T , v , 1, δ(v )) inserts new parent of all v ’s children
    fn insertchild(self, v: usize, i: usize, bit: bool);

    /// `child v i` write i-th child of v to output file
    fn child(self, v: usize, i: usize);

    /// `subtree size v` write subtree size of v (including v) to output file
    fn subtree_size(self, v: usize);

    /// `parent v` write parent of v to output file
    fn parent(self, v: usize);
}

/// Visualize Tree-based structures with [`graphviz`](https://graphviz.org/) using the `.dot` format.
pub trait Dot {
    /// Return `dot` representation for graph visualization. [Read more](https://graphviz.org/)
    fn dotviz(&self, self_id: isize) -> String;
}

/// Trait to get instance bit size for different structs
pub trait BitSize: Sized {
    /// Return total number of bits used by Type
    fn bitsize(&self) -> usize {
        std::mem::size_of::<Self>()
    }

    /// Return total number of bits allocated by objects managed by structures. Includes all
    /// elements on different areas of heap.
    fn bitsize_full(&self) -> usize {
        self.bitsize()
    }

    /// Return total number of used bits (fewer than allocated) by objects managed by structures.
    /// Includes all elements on different areas of heap.
    fn bitsize_used(&self) -> usize {
        self.bitsize_full()
    }
}
