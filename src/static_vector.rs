use super::traits;
use std::ops::Index;

#[derive(Debug)]
pub struct SBitVec(Vec<u64>);

impl Index<usize> for SBitVec {
    type Output = bool;

    /// Return immutable reference to boolean value of position `index` in bit vector.
    fn index(&self, index: usize) -> &Self::Output {
        let block = self.0[index / 64];
        // &((block >> (index % 64) & 1) as bool)
        if block & (1 << index) != 0 {
            &true
        } else {
            &false
        }
    }
}

impl SBitVec {
    fn append(mut self, bit: bool) {
        self.0[1] = 0;
    }
}

impl traits::StaticBitVec for SBitVec {
    type Intern = u64;

    fn new() -> Self {
        SBitVec(Vec::new())
    }

    fn access(self, i: usize) -> bool {
        self[i]
    }

    fn rank(&self, bit: bool, i: usize) -> usize {
        todo!()
    }

    fn select(&self, bit: bool, i: usize) -> usize {
        todo!()
    }
}
