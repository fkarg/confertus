use super::traits::StaticBitVec;

trait UnsafeSelect {
    unsafe fn select_internal(&self, bit: bool, n: usize) -> usize;
}

impl UnsafeSelect for u64 {
    #[cfg(not(all(target_feature = "bmi2", target_feature = "bmi1")))]
    unsafe fn select_internal(&self, bit: bool, n: usize) -> usize {
        let mut pos = 0;
        let mut i = n;
        for shift in (0..u64::BITS).rev() {
            if (((self >> shift) & 1) != 0) == bit {
                if i == 0 {
                    return pos;
                }
                i -= 1;
                pos += 1;
            }
        }
        panic!("`n`-th `bit`-value not found in {}", self)
    }

    #[cfg(all(target_feature = "bmi2", target_feature = "bmi1"))]
    unsafe fn select_internal(&self, bit: bool, n: usize) -> usize {
        let array = if bit { *self } else { !self };
        _tzcnt_u64(_pdep_u64(1 << n, array)) as usize
    }
}

impl StaticBitVec for u64 {
    type Intern = u64;

    fn ones(&self) -> usize {
        self.count_ones() as usize
    }

    fn access(&self, index: usize) -> bool {
        self >> index & 1 == 1
    }

    fn rank(&self, bit: bool, index: usize) -> usize {
        if bit {
            (self >> index as u32).count_ones() as usize
        } else {
            ((!self) >> index as u32).count_ones() as usize
        }
    }

    fn values(&self) -> Self::Intern {
        *self
    }

    fn select(&self, bit: bool, n: usize) -> usize {
        unsafe { self.select_internal(bit, n) }
    }
}

impl UnsafeSelect for u128 {
    #[cfg(not(all(target_feature = "bmi2", target_feature = "bmi1")))]
    unsafe fn select_internal(&self, bit: bool, n: usize) -> usize {
        let mut pos = 0;
        let mut i = n;
        for shift in (0..u128::BITS).rev() {
            if (((self >> shift) & 1) != 0) == bit {
                if i == 0 {
                    return pos;
                }
                i -= 1;
                pos += 1;
            }
        }
        panic!("`n`-th `bit`-value not found in {}", self)
    }

    #[cfg(all(target_feature = "bmi2", target_feature = "bmi1"))]
    unsafe fn select_internal(&self, bit: bool, n: usize) -> usize {
        let array = if bit { *self } else { !self };
        // self.value is u64
        // _tzcnt_u64(_pdep_u64(1 << n, array as u64)) as usize

        // self.value is u128
        if n < 64 {
            _tzcnt_u64(_pdep_u64(1 << n, array as u64)) as usize
        } else {
            _tzcnt_u64(_pdep_u64(1 << n, (array >> 64) as u64)) as usize + 64
        }
    }
}

impl StaticBitVec for u128 {
    type Intern = u128;

    fn ones(&self) -> usize {
        self.count_ones() as usize
    }

    fn access(&self, index: usize) -> bool {
        self >> index & 1 == 1
    }

    fn rank(&self, bit: bool, index: usize) -> usize {
        if bit {
            (self >> index as u32).count_ones() as usize
        } else {
            ((!self) >> index as u32).count_ones() as usize
        }
    }

    fn values(&self) -> Self::Intern {
        *self
    }

    fn select(&self, bit: bool, n: usize) -> usize {
        unsafe { self.select_internal(bit, n) }
    }
}
