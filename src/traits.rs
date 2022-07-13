pub trait StaticBitVec {
    type Intern;

    /// Constructor
    fn new() -> Self;

    /// `access i` return bit value at position i
    fn access(self, i: usize) -> bool;

    /// `rank [0|1] i` return rank0 or rank1 up to position i
    /// Maschinenbefehl: popcount (?)
    fn rank(&self, bit: bool, i: usize) -> usize;

    /// `select [0|1] i` return select0 or select1 for the i-th occurrence
    fn select(&self, bit: bool, i: usize) -> usize;
}

pub trait DynBitVec<T: StaticBitVec> {
    /// `insert i [0|1]` insert a 0 or 1 at the i-th position of the bit vector
    /// concurrently updates all relevant `ones` and `num` values when traversing to location `i`,
    /// rebalance if necessary
    fn insert(self, i: usize, bit: bool);

    /// `delete i` delete the i-th bit
    /// concurrently updates all relevant `ones` and `num` values when traversing to location `i`,
    /// rebalancing if necessary
    fn delete(self, i: usize);

    /// `flip i` flip the i-th bit
    /// updates `ones` and `num` accordingly
    fn flip(self, i: usize);

    /// `bitset  i` sets `i`-th bit to 1
    /// updates `ones` and `num` accordingly
    fn bitset(self, i: usize);

    /// `bitclear i` sets `i`-th bit to 0
    /// updates `ones` and `num` accordingly
    fn bitclear(self, i: usize);
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
pub trait BitSize {
    /// Return total number of bits used by Type
    fn bitsize(&self) -> usize
    where
        Self: Sized,
    {
        std::mem::size_of::<Self>()
    }

    /// Return total number of bits used by objects managed by structures. Includes all elements on
    /// different areas of heap.
    fn bitsize_full(&self) -> usize;
}
