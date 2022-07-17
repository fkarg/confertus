use super::traits::StaticBitVec;
use core::arch::x86_64::{_pdep_u64, _popcnt64, _tzcnt_u64};

/// So, that one didn't work out as LeafValue, as it still needs to implement bitshifts for various
/// functionality.
impl StaticBitVec for bool {
    type Intern = bool;

    fn ones(&self) -> usize {
        *self as usize
    }

    fn access(&self, index: usize) -> bool {
        assert!(index == 0);
        *self
    }

    fn rank(&self, bit: bool, index: usize) -> usize {
        assert!(index == 0);
        // actually correct implementation
        0
    }

    fn select(&self, bit: bool, n: usize) -> usize {
        assert!(n == 0);
        assert!(bit == *self);
        // not sure if a reasonable implementation is possible?
        0
    }

    fn values(&self) -> Self::Intern {
        *self
    }
}

impl StaticBitVec for u8 {
    type Intern = u8;

    fn ones(&self) -> usize {
        (*self as u64).ones()
    }

    fn access(&self, index: usize) -> bool {
        (*self as u64).access(index)
    }

    fn rank(&self, bit: bool, index: usize) -> usize {
        (*self as u64).rank(bit, index)
    }

    fn select(&self, bit: bool, n: usize) -> usize {
        (*self as u64).select(bit, n)
    }

    fn values(&self) -> Self::Intern {
        *self
    }
}

impl StaticBitVec for u16 {
    type Intern = u16;

    fn ones(&self) -> usize {
        (*self as u64).ones()
    }

    fn access(&self, index: usize) -> bool {
        (*self as u64).access(index)
    }

    fn rank(&self, bit: bool, index: usize) -> usize {
        (*self as u64).rank(bit, index)
    }

    fn select(&self, bit: bool, n: usize) -> usize {
        (*self as u64).select(bit, n)
    }

    fn values(&self) -> Self::Intern {
        *self
    }
}

impl StaticBitVec for u32 {
    type Intern = u32;

    fn ones(&self) -> usize {
        (*self as u64).ones()
    }

    fn access(&self, index: usize) -> bool {
        (*self as u64).access(index)
    }

    fn rank(&self, bit: bool, index: usize) -> usize {
        (*self as u64).rank(bit, index)
    }

    fn select(&self, bit: bool, n: usize) -> usize {
        (*self as u64).select(bit, n)
    }

    fn values(&self) -> Self::Intern {
        *self
    }
}

/// hidden abstraction of internal architecture-dependent unsafe implementations
trait UnsafeBitVec {
    unsafe fn select_internal(&self, bit: bool, n: usize) -> usize;

    unsafe fn rank_internal(&self, bit: bool, index: usize) -> usize;
}

impl UnsafeBitVec for u64 {
    /// Fallback implementation of `select`, not dependent on any specific architecture
    #[cfg(not(all(
        target_arch = "x86_64",
        target_feature = "bmi1",
        target_feature = "bmi2"
    )))]
    unsafe fn select_internal(&self, bit: bool, n: usize) -> usize {
        let mut cnt = n;
        // go over u64 from right to left
        for shift in 0..u64::BITS {
            if (((self >> shift) & 1) != 0) == bit {
                // we're looking for `n`-th match, so check for zero first
                // (insdead of: decrease first)
                if cnt == 0 {
                    return shift as usize;
                }
                cnt -= 1;
            }
        }
        panic!("`{n}`-th `bit`-value '{bit}' not found in {self:b}")
    }

    /// Performant implementation of `select` for `x86_64` architectures with `bmi1` and `bmi2`
    /// features.
    /// ```text
    /// Algorithm for determining the position of the jth 1 in a machine word.
    /// ---
    /// 1: function PTSELECT(x, j)
    /// 2:     i ← SHIFTLEFT(1, j)
    /// 3:     p ← PDEP(i, x)
    /// 4:     return TZCNT(p)
    /// ```
    ///
    /// taken from <https://arxiv.org/pdf/1706.00990.pdf>.
    ///
    /// # Safety
    /// Only available for `x86_64`-based architecuters supporting feature sets `bmi1` and `bmi2`,
    /// which were both introduced by the fourth-generation intel
    /// [haswell](https://en.wikipedia.org/wiki/Haswell_(microarchitecture)) architecture nine
    /// years ago.
    ///
    /// Execute with `RUSTFLAGS="-C target-cpu=native -O" cargo run --release` to get all performance
    /// benefits and enable proper cpu feature recognition.
    #[inline]
    #[cfg(all(
        target_arch = "x86_64",
        target_feature = "bmi1",
        target_feature = "bmi2"
    ))]
    unsafe fn select_internal(&self, bit: bool, n: usize) -> usize {
        _tzcnt_u64(_pdep_u64(1 << n, if bit { *self } else { !self })) as usize
    }

    /// Performant implementation of `rank` for `x86_64` architectures (3 instructions).
    ///
    /// Assumes `index` to be in the range of `0..63`.
    #[cfg(target_arch = "x86_64")]
    unsafe fn rank_internal(&self, bit: bool, index: usize) -> usize {
        if bit {
            _popcnt64(self.overflowing_shl(u64::BITS - index as u32).0 as i64) as usize
        } else {
            _popcnt64((!self).overflowing_shl(u64::BITS - index as u32).0 as i64) as usize
        }
    }

    /// Fallback implementation of `rank`, not depending on any specific architecture
    #[cfg(not(target_arch = "x86_64"))]
    unsafe fn rank_internal(&self, bit: bool, index: usize) -> usize {
        if bit {
            self.overflowing_shl(u64::BITS - index as u32)
                .0
                .count_ones() as usize
        } else {
            (!self)
                .overflowing_shl(u64::BITS - index as u32)
                .0
                .count_ones() as usize
        }
    }
}

/// Container is [`u64`] Bit Vector, indexed from right to left (big endian).
impl StaticBitVec for u64 {
    type Intern = u64;

    /// TODO: use _popcnt64 für `x86_64` (just for i64?)
    #[inline]
    fn ones(&self) -> usize {
        self.rank(true, Self::BITS as usize)
    }

    /// Right shift self-bits by `index` and `1`-mask for comparison
    #[inline]
    fn access(&self, index: usize) -> bool {
        self >> index & 1 == 1
    }

    #[inline]
    fn rank(&self, bit: bool, index: usize) -> usize {
        if index == 0 {
            return 0;
        }
        unsafe { self.rank_internal(bit, index) }
    }

    #[inline]
    fn select(&self, bit: bool, n: usize) -> usize {
        unsafe { self.select_internal(bit, n) }
    }

    #[inline]
    fn values(&self) -> Self::Intern {
        *self
    }
}

impl UnsafeBitVec for u128 {
    #[cfg(not(all(
        target_arch = "x86_64",
        target_feature = "bmi1",
        target_feature = "bmi2"
    )))]
    unsafe fn select_internal(&self, bit: bool, n: usize) -> usize {
        let mut cnt = n;
        // go over u128 from right to left
        for shift in 0..u128::BITS {
            if (((self >> shift) & 1) != 0) == bit {
                // we're looking for `n`-th match, so check for zero first
                // (insdead of: decrease first)
                if cnt == 0 {
                    return shift as usize;
                }
                cnt -= 1;
            }
        }
        panic!("`{n}`-th `bit`-value '{bit}' not found in {self:b}")
    }

    #[inline]
    #[cfg(all(
        target_arch = "x86_64",
        target_feature = "bmi1",
        target_feature = "bmi2"
    ))]
    unsafe fn select_internal(&self, bit: bool, n: usize) -> usize {
        let array = if bit { *self } else { !self };
        // self.value is u128, but pdep and tzcnt only exist for u64
        // cast to u64 is expected to be lossy.
        // First, check if `n` is in right or left half of u128
        let rank = (*self as u64).rank_internal(bit, n);
        if rank >= n {
            _tzcnt_u64(_pdep_u64(1 << n, array as u64)) as usize
        } else {
            64 + _tzcnt_u64(_pdep_u64(
                1 << (n - rank),
                array.overflowing_shr(64).0 as u64,
            )) as usize
        }
    }

    #[inline]
    #[cfg(not(target_arch = "x86_64"))]
    unsafe fn rank_internal(&self, bit: bool, index: usize) -> usize {
        if bit {
            (self >> index as u32).count_ones() as usize
        } else {
            ((!self) >> index as u32).count_ones() as usize
        }
    }

    #[inline]
    #[cfg(target_arch = "x86_64")]
    unsafe fn rank_internal(&self, bit: bool, index: usize) -> usize {
        let array = if bit { *self } else { !self };

        if index < 64 {
            // only move by u64::BITS instead of u128::BITS to cap left side away in cast to i64
            _popcnt64(array.overflowing_shl(u64::BITS - index as u32).0 as i64) as usize
        } else {
            _popcnt64(array.overflowing_shl(u128::BITS - index as u32).0 as i64) as usize
        }
    }
}

/// Container is [`u128`] Bit Vector, indexed from right to left (big endian).
impl StaticBitVec for u128 {
    type Intern = u128;

    #[inline]
    fn ones(&self) -> usize {
        self.rank(true, Self::BITS as usize)
    }

    #[inline]
    fn access(&self, index: usize) -> bool {
        self >> index & 1 == 1
    }

    #[inline]
    fn rank(&self, bit: bool, index: usize) -> usize {
        if index == 0 {
            return 0;
        }
        unsafe { self.rank_internal(bit, index) }
    }

    #[inline]
    fn values(&self) -> Self::Intern {
        *self
    }

    #[inline]
    fn select(&self, bit: bool, n: usize) -> usize {
        unsafe { self.select_internal(bit, n) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;
    use test_case::test_case;

    #[test]
    fn ones_u64_1() {
        assert_eq!(4u64.ones(), 1);
    }

    #[test]
    fn ones_u64_2() {
        assert_eq!(5u64.ones(), 2);
    }

    #[test]
    fn ones_u64_3() {
        assert_eq!(7u64.ones(), 3);
    }

    /// Test if generated number with specific number of ones really has them
    #[quickcheck]
    fn ones_u64(n: u64) -> TestResult {
        if n == 0 {
            TestResult::from_bool(0u64.ones() == n as usize)
        } else if n > u64::BITS as u64 {
            TestResult::discard()
        } else {
            TestResult::from_bool((2 ^ (n - 1)).ones() == n as usize)
        }
    }

    /// For u64::MAX, all accessed values should be 1
    #[quickcheck]
    fn access_u64_1(n: usize) -> TestResult {
        if n > u64::BITS as usize {
            TestResult::discard()
        } else {
            TestResult::from_bool(u64::MAX.access(n))
        }
    }

    /// For 0u64, all accessed values should be 0
    #[quickcheck]
    fn access_u64_0(n: usize) -> TestResult {
        if n > u64::BITS as usize {
            TestResult::discard()
        } else {
            TestResult::from_bool(!0u64.access(n))
        }
    }

    #[quickcheck]
    fn rank_u64(n: usize) -> TestResult {
        if n >= u64::BITS as usize {
            return TestResult::discard();
        } else {
            // assert_eq!(u64::MAX.rank(true, 0), 0);
            // assert_eq!(u64::MAX.rank(true, 1), 1);
            assert_eq!(u64::MAX.rank(true, n), n);
            assert_eq!(u64::MAX.rank(false, n), 0);
            assert_eq!(0u64.rank(false, n), n);
            assert_eq!(0u64.rank(true, n), 0);
            TestResult::passed()
        }
    }

    /// Simple intuitive tests for select on u64
    #[test]
    fn select_u64_simpel() {
        assert_eq!(1u64.select(true, 0), 0);
        assert_eq!(1u64.select(false, 0), 1);
        assert_eq!(2u64.select(true, 0), 1);
        assert_eq!(2u64.select(false, 0), 0);
        assert_eq!(3u64.select(true, 1), 1);
        assert_eq!(3u64.select(false, 1), 3);
        assert_eq!(u64::MAX.select(true, 63), 63);
    }

    /// Test if generated number with specific number of ones really has them
    #[quickcheck]
    fn ones_u128(n: u128) -> TestResult {
        if n == 0 {
            TestResult::from_bool(0u128.ones() == n as usize)
        } else if n > u128::BITS as u128 {
            TestResult::discard()
        } else {
            TestResult::from_bool((2 ^ (n - 1)).ones() == n as usize)
        }
    }

    /// For u128::MAX, all accessed values should be 1
    #[quickcheck]
    fn access_u128_1(n: usize) -> TestResult {
        if n > u128::BITS as usize {
            TestResult::discard()
        } else {
            TestResult::from_bool(u128::MAX.access(n))
        }
    }

    /// For 0u128, all accessed values should be 0
    #[quickcheck]
    fn access_u128_0(n: usize) -> TestResult {
        if n > u128::BITS as usize {
            TestResult::discard()
        } else {
            TestResult::from_bool(!0u128.access(n))
        }
    }

    #[quickcheck]
    fn rank_u128(n: usize) -> TestResult {
        if n >= u128::BITS as usize {
            return TestResult::discard();
        } else {
            // assert_eq!(u64::MAX.rank(true, 0), 0);
            // assert_eq!(u64::MAX.rank(true, 1), 1);
            assert_eq!(u128::MAX.rank(true, n), n);
            assert_eq!(u128::MAX.rank(false, n), 0);
            assert_eq!(0u128.rank(false, n), n);
            assert_eq!(0u128.rank(true, n), 0);
            TestResult::passed()
        }
    }

    /// Simple intuitive tests for select on u128
    #[test]
    fn select_u128_simpel() {
        assert_eq!(1u128.select(true, 0), 0);
        assert_eq!(1u128.select(false, 0), 1);
        assert_eq!(2u128.select(true, 0), 1);
        assert_eq!(2u128.select(false, 0), 0);
        assert_eq!(3u128.select(true, 1), 1);
        assert_eq!(3u128.select(false, 1), 3);
        assert_eq!(u128::MAX.select(true, 63), 63);
    }
}
