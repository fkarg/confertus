use crate::traits::{Dot, DynBitTree, DynBitVec, StaticBitVec};
use crate::{BitSize, DynamicBitVector, Leaf, LeafValue, Node};
use std::fmt;
use std::ops::{Add, Index, IndexMut};

impl BitSize for DynamicBitVector {
    fn bitsize_full(&self) -> usize {
        448 + self.leafs.len() * 25 * 8 + self.nodes.len() * 325
    }
}

impl Dot for DynamicBitVector {
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

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}

impl IndexMut<usize> for DynamicBitVector {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}

/// Return [`Leaf`] for `isize` indexing
///
/// When creating a new container with [`DynamicBitVector::new`], a [`Leaf`] on position 0 (which
/// cannot be accessed) is created, as all attempted (later) indexing to values `>= 0` are
/// converted to `usize` first and return a [`Node`] instead.
impl Index<isize> for DynamicBitVector {
    type Output = Leaf;

    #[inline]
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
    #[inline]
    fn index_mut(&mut self, index: isize) -> &mut Self::Output {
        let uidx = if index < 0 {
            -index as usize
        } else {
            index as usize
        };
        &mut self.leafs[uidx]
    }
}

impl StaticBitVec for DynamicBitVector {
    type Intern = Vec<LeafValue>;

    #[inline]
    fn ones(&self) -> usize {
        self[self.root].ones
    }

    /// Return value at position `index` of `DynamicBitVector`.
    ///
    /// # Panics
    /// If `index` is out of bounds.
    #[inline]
    fn access(&self, index: usize) -> bool {
        self.get_node(self.root, index)
        // self.apply(Self::get_leaf, index)
        // self.apply(|s, leaf, index| s.get_leaf(leaf, index), index)
    }

    #[inline]
    fn rank(&self, bit: bool, index: usize) -> usize {
        self.apply_bitop(Self::rank_leaf, Self::rank_add, index, bit)
    }

    #[inline]
    fn select(&self, bit: bool, n: usize) -> usize {
        self.select_node(self.root, n, bit)
    }

    /// Return full internal container
    #[inline]
    fn values(&self) -> Self::Intern {
        todo!()
    }
}

impl DynBitVec for DynamicBitVector {
    #[inline]
    #[cfg(debug_assertions)]
    fn insert(&mut self, index: usize, bit: bool) -> Result<(), &'static str> {
        match self.insert_node(self.root, index, bit) {
            Err(e) => {
                let lid = self.apply(Self::leaf_id, index);
                println!("Insert of {bit} at position {index} failed with '{e}' in L{lid}");
                self.viz_stop();
                Err(e)
            }
            Ok(()) => Ok(()),
        }
    }

    #[inline]
    #[cfg(not(debug_assertions))]
    fn insert(&mut self, index: usize, bit: bool) -> Result<(), &'static str> {
        self.insert_node(self.root, index, bit)?;
        Ok(())
    }

    #[inline]
    #[cfg(debug_assertions)]
    fn delete(&mut self, index: usize) -> Result<(), &'static str> {
        let leaf = match self.apply(Self::delete_leaf, index) {
            Err(e) => {
                let lid = self.apply(Self::leaf_id, index);
                println!("Delete at position {index} failed with '{e}' in L{lid}");
                self.viz_stop();
                Err(e)
            }
            Ok(l) => Ok(l),
        }?;
        self.update_left_values(self[leaf].parent, leaf);
        Ok(())
    }

    #[inline]
    #[cfg(not(debug_assertions))]
    fn delete(&mut self, index: usize) -> Result<(), &'static str> {
        let leaf = self.apply(Self::delete_leaf, index)?;
        self.update_left_values(self[leaf].parent, leaf);
        Ok(())
    }

    #[inline]
    fn flip(&mut self, index: usize) {
        let leaf = self.apply(Self::flip_leaf, index);
        self.update_left_values(self[leaf].parent, leaf);
    }

    #[inline]
    fn nums(&self) -> usize {
        self[self.root].nums
    }
}
