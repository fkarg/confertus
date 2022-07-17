use super::*;
use pretty_assertions::{assert_eq, assert_ne};
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;
use rand::Rng;

// tests for DynBitVec-trait

#[test]
fn creation() {
    let l = Leaf::new(0);
    assert_eq!(
        l,
        Leaf {
            parent: 0,
            value: 0,
            nums: 0
        }
    );
}

#[test]
fn push_0() {
    let mut l = Leaf::new(0);
    l.push(false).unwrap();
    l.push(false).unwrap();
    l.push(false).unwrap();
    l.push(false).unwrap();
    assert_eq!(
        l,
        Leaf {
            parent: 0,
            value: 0,
            nums: 4
        }
    );
}

#[test]
fn push_1() {
    let mut l = Leaf::new(0);
    l.push(true).unwrap();
    l.push(true).unwrap();
    l.push(true).unwrap();
    l.push(true).unwrap();
    assert_eq!(
        l,
        Leaf {
            parent: 0,
            value: 15,
            nums: 4
        }
    );
}

#[test]
fn push_all_0() {
    let mut l = Leaf::new(0);
    for _ in 0..LeafValue::BITS {
        l.push(false).unwrap();
    }
    assert_eq!(
        l,
        Leaf {
            parent: 0,
            value: 0,
            nums: LeafValue::BITS as u8
        }
    );
}

#[test]
fn push_all_1() {
    let mut l = Leaf::new(0);
    for _ in 0..LeafValue::BITS {
        l.push(true).unwrap();
    }
    assert_eq!(
        l,
        Leaf {
            parent: 0,
            value: LeafValue::MAX,
            nums: LeafValue::BITS as u8
        }
    );
}

#[test]
fn insert_0() {
    let mut l = Leaf::new(0);
    l.insert(0, false).unwrap();
    l.insert(0, false).unwrap();
    l.insert(0, false).unwrap();
    l.insert(0, false).unwrap();
    assert_eq!(
        l,
        Leaf {
            parent: 0,
            value: 0,
            nums: 4
        }
    );
}

#[test]
fn insert_1() {
    let mut l = Leaf::new(0);
    l.insert(0, true).unwrap();
    l.insert(0, true).unwrap();
    l.insert(0, true).unwrap();
    l.insert(0, true).unwrap();
    assert_eq!(
        l,
        Leaf {
            parent: 0,
            value: 15,
            nums: 4
        }
    );
}

#[test]
fn insert_all_0() {
    let mut l = Leaf::new(0);
    for _ in 0..LeafValue::BITS {
        l.insert(0, false).unwrap();
    }
    assert_eq!(
        l,
        Leaf {
            parent: 0,
            value: 0,
            nums: LeafValue::BITS as u8
        }
    );
}

#[test]
fn insert_all_1() {
    let mut l = Leaf::new(0);
    for _ in 0..LeafValue::BITS {
        l.insert(0, true).unwrap();
    }
    assert_eq!(
        l,
        Leaf {
            parent: 0,
            value: LeafValue::MAX,
            nums: LeafValue::BITS as u8
        }
    );
}

/// Insert random bits at random positions.
#[test]
fn insert_all_random_1() {
    let mut rng = rand::thread_rng();
    let mut l = Leaf::create(0, 0, 0);

    for _ in 0..LeafValue::BITS {
        let i = rng.gen_range(0..(l.nums + 1));
        let bit = rng.gen_bool(0.5);
        l.insert(i as usize, bit).unwrap();
    }
    // only test for this not going wrong. Hard to re-calculate value independetly without the same
    // issues.
}

/// Insert out of bounds: ensure that leads to
#[quickcheck]
fn insert_out_of_bounds(n: usize) -> TestResult {
    let mut l = Leaf::new(0);
    if n <= l.nums as usize {
        return TestResult::discard();
    }
    TestResult::from_bool(l.insert(n, true).is_err())
}

#[test]
fn delete_0() {
    let mut l = Leaf::create(0, 0, 4);
    l.delete(0).unwrap();
    l.delete(0).unwrap();
    l.delete(0).unwrap();
    l.delete(0).unwrap();
    assert_eq!(
        l,
        Leaf {
            parent: 0,
            value: 0,
            nums: 0
        }
    );
}

#[test]
fn delete_1() {
    let mut l = Leaf::create(0, 15, 4);
    l.delete(0).unwrap();
    l.delete(0).unwrap();
    l.delete(0).unwrap();
    l.delete(0).unwrap();
    assert_eq!(
        l,
        Leaf {
            parent: 0,
            value: 0,
            nums: 0
        }
    );
}

#[test]
fn delete_2() {
    let mut l = Leaf::create(0, 15, 4);
    l.delete(2).unwrap();
    l.delete(1).unwrap();
    assert_eq!(
        l,
        Leaf {
            parent: 0,
            value: 3,
            nums: 2
        }
    );
}

#[test]
fn delete_all_1() {
    let mut l = Leaf::create(0, LeafValue::MAX, LeafValue::BITS as u8);
    for _ in 0..LeafValue::BITS {
        l.delete(0).unwrap();
    }
    assert_eq!(
        l,
        Leaf {
            parent: 0,
            value: 0,
            nums: 0,
        }
    );
}

#[test]
fn delete_all_0() {
    let mut l = Leaf::create(0, 0, LeafValue::BITS as u8);
    for _ in 0..LeafValue::BITS {
        l.delete(0).unwrap();
    }
    assert_eq!(
        l,
        Leaf {
            parent: 0,
            value: 0,
            nums: 0,
        }
    );
}

#[test]
fn delete_all_reverse() {
    let mut l = Leaf::create(0, LeafValue::MAX, LeafValue::BITS as u8);
    for i in (0..LeafValue::BITS).rev() {
        l.delete(i as usize).unwrap();
    }
    assert_eq!(
        l,
        Leaf {
            parent: 0,
            value: 0,
            nums: 0,
        }
    );
}

#[test]
fn delete_all_random_1() {
    let mut l = Leaf::create(0, LeafValue::MAX, LeafValue::BITS as u8);
    let mut rng = rand::thread_rng();
    // println!("Integer: {}", rng.gen_range(0..10));

    for _ in 0..LeafValue::BITS {
        let i = rng.gen_range(0..l.nums);
        l.delete(i as usize).unwrap();
    }
    assert_eq!(
        l,
        Leaf {
            parent: 0,
            value: 0,
            nums: 0,
        }
    );
}

#[test]
fn delete_all_random_2() {
    let mut rng = rand::thread_rng();
    let mut l = Leaf::create(0, rng.gen_range(0..LeafValue::MAX), LeafValue::BITS as u8);
    dbg!(l.clone());

    for _ in 0..LeafValue::BITS {
        let i = rng.gen_range(0..l.nums);
        l.delete(i as usize).unwrap();
    }
    assert_eq!(
        l,
        Leaf {
            parent: 0,
            value: 0,
            nums: 0,
        }
    );
}

#[test]
#[should_panic]
fn delete_nonexisting() {
    let mut l = Leaf::new(0);
    l.delete(0).unwrap();
}

#[test]
fn flip_0() {
    let mut l = Leaf::new(0);
    l.flip(0);
    assert_eq!(
        l,
        Leaf {
            parent: 0,
            value: 1,
            nums: 0,
        }
    );
}

#[test]
fn flip_1() {
    let mut l = Leaf::create(0, 1, 1);
    l.flip(0);
    assert_eq!(
        l,
        Leaf {
            parent: 0,
            value: 0,
            nums: 1,
        }
    );
}

// tests for other functionality
