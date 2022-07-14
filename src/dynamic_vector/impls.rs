use crate::traits::*;
use crate::{BitSize, DynamicBitVector, Leaf, Node};
use std::fmt;
use std::ops::{Add, Index, IndexMut};

impl<T> BitSize for DynamicBitVector<T>
where
    T: BitSize + StaticBitVec,
{
    fn bitsize_full(&self) -> usize {
        448 + self.leafs.len() * 17 * 8 + self.nodes.len() * 325
    }
}

impl<T: StaticBitVec> Dot for DynamicBitVector<T> {
    fn dotviz(&self, self_id: isize) -> String {
        format!(
            "\n\ndigraph tree {{\n\
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

/// Really just the `Debug` output
impl<T: StaticBitVec> fmt::Display for DynamicBitVector<T> {
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
impl<T: StaticBitVec> Index<usize> for DynamicBitVector<T> {
    type Output = Node;

    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}

impl<T: StaticBitVec> IndexMut<usize> for DynamicBitVector<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}

/// Return [`Leaf`] for `isize` indexing
///
/// When creating a new container with [`DynamicBitVector::new`], a [`Leaf`] on position 0 (which
/// cannot be accessed) is created, as all attempted (later) indexing to values `>= 0` are
/// converted to `usize` first and return a [`Node`] instead.
impl<T> Index<isize> for DynamicBitVector<T>
where
    T: StaticBitVec,
{
    type Output = Leaf<T>;

    fn index(&self, index: isize) -> &Self::Output {
        let uidx = if index < 0 {
            -index as usize
        } else {
            index as usize
        };
        &self.leafs[uidx]
    }
}

impl<T: StaticBitVec> IndexMut<isize> for DynamicBitVector<T> {
    fn index_mut(&mut self, index: isize) -> &mut Self::Output {
        let uidx = if index < 0 {
            -index as usize
        } else {
            index as usize
        };
        &mut self.leafs[uidx]
    }
}
