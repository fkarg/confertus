use super::*;
use pretty_assertions::{assert_eq, assert_ne};
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;
use rand::Rng;

#[test]
fn creation() {
    let dbv = DynamicBitVector::new();
    assert_eq!(
        dbv,
        DynamicBitVector {
            root: 0,
            nodes: vec![Node::new()],  // existence of root node
            leafs: vec![Leaf::new(0)], // one empty leaf
        }
    );
}

// Tests for StaticBitVec behaviour. test with a few simple trees.

#[test]
fn push_6_true() {
    let mut d = DynamicBitVector::new();
    for _ in 0..(LeafValue::BITS * 6) {
        d.push(true);
    }
    assert_eq!(
        d,
        DynamicBitVector {
            root: 1,
            nodes: vec![
                Node::create(
                    Some(1),
                    Some(-1),
                    Some(-2),
                    LeafValue::BITS as usize,
                    LeafValue::BITS as usize,
                    0
                ),
                Node::create(
                    None,
                    Some(0),
                    Some(3),
                    2 * LeafValue::BITS as usize,
                    2 * LeafValue::BITS as usize,
                    1
                ),
                Node::create(
                    Some(3),
                    Some(-3),
                    Some(-4),
                    LeafValue::BITS as usize,
                    LeafValue::BITS as usize,
                    0
                ),
                Node::create(
                    Some(1),
                    Some(2),
                    Some(4),
                    2 * LeafValue::BITS as usize,
                    2 * LeafValue::BITS as usize,
                    0
                ),
                Node::create(
                    Some(3),
                    Some(-5),
                    Some(-6),
                    LeafValue::BITS as usize,
                    LeafValue::BITS as usize,
                    0
                ),
            ],
            leafs: vec![
                Leaf::new(0),
                Leaf::create(0, LeafValue::MAX, LeafValue::BITS as u8),
                Leaf::create(0, LeafValue::MAX, LeafValue::BITS as u8),
                Leaf::create(2, LeafValue::MAX, LeafValue::BITS as u8),
                Leaf::create(2, LeafValue::MAX, LeafValue::BITS as u8),
                Leaf::create(4, LeafValue::MAX, LeafValue::BITS as u8),
                Leaf::create(4, LeafValue::MAX, LeafValue::BITS as u8),
            ],
        }
    );
}

#[test]
fn push_6_false() {
    let mut d = DynamicBitVector::new();
    for _ in 0..(LeafValue::BITS * 6) {
        d.push(false);
    }
    assert_eq!(
        d,
        DynamicBitVector {
            root: 1,
            nodes: vec![
                Node::create(
                    Some(1),
                    Some(-1),
                    Some(-2),
                    LeafValue::BITS as usize,
                    0,
                    0
                ),
                Node::create(
                    None,
                    Some(0),
                    Some(3),
                    2 * LeafValue::BITS as usize,
                    0,
                    1
                ),
                Node::create(
                    Some(3),
                    Some(-3),
                    Some(-4),
                    LeafValue::BITS as usize,
                    0,
                    0
                ),
                Node::create(
                    Some(1),
                    Some(2),
                    Some(4),
                    2 * LeafValue::BITS as usize,
                    0,
                    0
                ),
                Node::create(
                    Some(3),
                    Some(-5),
                    Some(-6),
                    LeafValue::BITS as usize,
                    0,
                    0
                ),
            ],
            leafs: vec![
                Leaf::new(0),
                Leaf::create(0, 0, LeafValue::BITS as u8),
                Leaf::create(0, 0, LeafValue::BITS as u8),
                Leaf::create(2, 0, LeafValue::BITS as u8),
                Leaf::create(2, 0, LeafValue::BITS as u8),
                Leaf::create(4, 0, LeafValue::BITS as u8),
                Leaf::create(4, 0, LeafValue::BITS as u8),
            ],
        }
    );
}

#[test]
fn insert_0() {
    // test proper insertion at last position before leaf-splitting
    let mut d = DynamicBitVector::new();
    for i in 0..LeafValue::BITS {
        d.insert(i as usize, true).expect("insert failed at {i}");
    }
    assert_eq!(
        d,
        DynamicBitVector {
            root: 0,
            nodes: vec![Node::create(
                None,
                None,
                Some(-1),
                0,
                0,
                1
            ),],
            leafs: vec![
                Leaf::new(0),
                Leaf::create(
                    0,
                    LeafValue::MAX,
                    LeafValue::BITS as u8,
                ),
            ],
        }
    );
}

#[test]
fn insert_1() {
    // test proper Leaf-splitting when inserting at first position
    let mut d = DynamicBitVector::new();
    for i in 0..=(LeafValue::BITS * 1) {
        d.insert(0, true).expect("insert failed at {i}");
    }
    let half = (LeafValue::BITS / 2) as u32;
    assert_eq!(
        d,
        DynamicBitVector {
            root: 0,
            nodes: vec![Node::create(
                None,
                Some(-1),
                Some(-2),
                (half + 1) as usize,
                (half + 1) as usize,
                0
            ),],
            leafs: vec![
                Leaf::new(0),
                Leaf::create(
                    0,
                    LeafValue::MAX.overflowing_shr(half - 1).0,
                    (half + 1) as u8
                ),
                Leaf::create(0, LeafValue::MAX.overflowing_shr(half).0, half as u8),
            ],
        }
    );
}

#[test]
fn insert_2() {
    // test proper Leaf-splitting when inserting at last position
    let mut d = DynamicBitVector::new();
    for i in 0..=(LeafValue::BITS * 1) {
        d.insert(i as usize, true).expect("insert failed at {i}");
    }
    let half = (LeafValue::BITS / 2) as u32;
    assert_eq!(
        d,
        DynamicBitVector {
            root: 0,
            nodes: vec![Node::create(
                None,
                Some(-1),
                Some(-2),
                half as usize,
                half as usize,
                0
            ),],
            leafs: vec![
                Leaf::new(0),
                Leaf::create(0, LeafValue::MAX.overflowing_shr(half).0, half as u8),
                Leaf::create(
                    0,
                    LeafValue::MAX.overflowing_shr(half - 1).0,
                    (half + 1) as u8
                ),
            ],
        }
    );
}


// #[test]
fn insert_3() {
    let mut d = DynamicBitVector::new();
    for i in 0..(LeafValue::BITS * 3) {
        d.insert(0, true).expect("insert failed at {i}");
    }
    let half = (LeafValue::BITS / 2) as usize;
    assert_eq!(
        d,
        DynamicBitVector {
            root: 1,
            nodes: vec![
                Node::create(Some(1), Some(-3), Some(-2), half, half, 0),
                Node::create(None, Some(2), Some(0), half * 4, half * 4, 0),
                Node::create(Some(1), Some(3), Some(-4), half * 3, half * 3, -1),
                Node::create(Some(2), Some(-1), Some(-5), half * 2, half * 2, 0),
            ],
            leafs: vec![
                Leaf::new(0),
                Leaf::create(3, LeafValue::MAX, (half * 2) as u8),
                Leaf::create(0, LeafValue::MAX.overflowing_shr(half as u32).0, half as u8),
                Leaf::create(0, LeafValue::MAX.overflowing_shr(half as u32).0, half as u8),
                Leaf::create(2, LeafValue::MAX.overflowing_shr(half as u32).0, half as u8),
                Leaf::create(3, LeafValue::MAX.overflowing_shr(half as u32).0, half as u8),
            ],
        }
    );
}

// WIP
// #[test]
fn insert_4() {
    let mut d = DynamicBitVector::new();
    let half = (LeafValue::BITS / 2) as usize;
    let of = half - 1;
    for i in 0..(LeafValue::BITS * 2 + half as u32) {
        d.insert(d[d.root].nums, true)
            .expect("insert failed at {i}");
    }
    assert_eq!(
        d,
        DynamicBitVector {
            root: 0,
            nodes: vec![
                Node::create(None, Some(-1), Some(1), half + 1, half + 1, 1),
                Node::create(Some(0), Some(-2), Some(-3), half + of, half + of, 0),
            ],
            leafs: vec![
                Leaf::new(0),
                Leaf::create(
                    0,
                    LeafValue::MAX.overflowing_shr((half - 1) as u32).0,
                    (half + 1) as u8
                ),
                Leaf::create(
                    1,
                    LeafValue::MAX.overflowing_shr((half - of) as u32).0,
                    (half + of) as u8
                ),
                Leaf::create(1, LeafValue::MAX.overflowing_shr(half as u32).0, half as u8),
            ],
        }
    );
}

// WIP
// #[test]
fn insert_6() {
    let mut d = DynamicBitVector::new();
    for i in 0..(LeafValue::BITS * 6) {
        d.insert(0, true).expect("insert failed at {i}");
    }
    assert_eq!(
        d,
        DynamicBitVector {
            root: 1,
            nodes: vec![
                Node::create(
                    Some(1),
                    Some(-1),
                    Some(-2),
                    LeafValue::BITS as usize,
                    LeafValue::BITS as usize,
                    0
                ),
                Node::create(
                    None,
                    Some(0),
                    Some(3),
                    2 * LeafValue::BITS as usize,
                    2 * LeafValue::BITS as usize,
                    1
                ),
                Node::create(
                    Some(3),
                    Some(-3),
                    Some(-4),
                    LeafValue::BITS as usize,
                    LeafValue::BITS as usize,
                    0
                ),
                Node::create(
                    Some(1),
                    Some(2),
                    Some(4),
                    2 * LeafValue::BITS as usize,
                    2 * LeafValue::BITS as usize,
                    0
                ),
                Node::create(
                    Some(3),
                    Some(-5),
                    Some(-6),
                    LeafValue::BITS as usize,
                    LeafValue::BITS as usize,
                    0
                ),
            ],
            leafs: vec![
                Leaf::new(0),
                Leaf::create(0, LeafValue::MAX, LeafValue::BITS as u8),
                Leaf::create(0, LeafValue::MAX, LeafValue::BITS as u8),
                Leaf::create(2, LeafValue::MAX, LeafValue::BITS as u8),
                Leaf::create(2, LeafValue::MAX, LeafValue::BITS as u8),
                Leaf::create(4, LeafValue::MAX, LeafValue::BITS as u8),
                Leaf::create(4, LeafValue::MAX, LeafValue::BITS as u8),
            ],
        }
    );
}

#[test]
fn rotate_left_1() {
    let m = LeafValue::MAX;
    let b = LeafValue::BITS as u8;
    let bs = b as usize;
    let mut d = DynamicBitVector {
        root: 0,
        nodes: vec![
            Node::create(None, Some(-1), Some(1), bs, bs, 2), // x
            Node::create(Some(0), Some(-2), Some(2), bs, bs, 1), // z
            Node::create(Some(1), Some(-3), None, bs, bs, -1), // T4
        ],
        leafs: vec![
            Leaf::new(0),
            Leaf::create(0, m, b), // T1
            Leaf::create(1, m, b), // T23
            Leaf::create(2, m, b), // Child at T4
        ],
    };
    d.rotate_left(1, 0);
    d.viz();
    assert_eq!(d,
            DynamicBitVector {
        root: 1,
        nodes: vec![
            Node::create(Some(1), Some(-1), Some(-2), bs, bs, 0),
            Node::create(None, Some(0), Some(2), 2 * bs, 2 * bs, 0),
            Node::create(Some(1), Some(-3), None, bs, bs, -1),
        ],
        leafs: vec![
            Leaf::new(0),
            Leaf::create(0, m, b),
            Leaf::create(0, m, b),
            Leaf::create(2, m, b),
        ],
        }
    );
}

#[test]
fn rotate_right_1() {
    let m = LeafValue::MAX;
    let b = LeafValue::BITS as u8;
    let bs = b as usize;
    let mut d = DynamicBitVector {
        root: 0,
        nodes: vec![
            Node::create(Some(1), None, Some(-1), 0, 0, 1), // T1
            Node::create(Some(2), Some(1), Some(-2), bs, bs, -1), // z
            Node::create(None, Some(2), Some(-3), 2 * bs, 2 * bs, -2), // x
        ],
        leafs: vec![
            Leaf::new(0),
            Leaf::create(0, m, b), // Child at T1
            Leaf::create(1, m, b), // T23
            Leaf::create(2, m, b), // T4
        ],
    };
    d.rotate_right(1, 2);
    d.viz();
    assert_eq!(d,
            DynamicBitVector {
        root: 1,
        nodes: vec![
            Node::create(Some(1), None, Some(-1), 0, 0, 1),
            Node::create(None, Some(1), Some(2), bs, bs, 0),
            Node::create(Some(1), Some(-2), Some(-3), bs, bs, 0),
        ],
        leafs: vec![
            Leaf::new(0),
            Leaf::create(0, m, b),
            Leaf::create(2, m, b),
            Leaf::create(2, m, b),
        ],
        }
    );
}

#[test]
fn rotate_left_2() {
    let m = LeafValue::MAX;
    let b = LeafValue::BITS as u8;
    let bs = b as usize;
    let mut d = DynamicBitVector {
        root: 0,
        nodes: vec![
            Node::create(None, Some(-1), Some(1), bs, bs, 2), // x
            Node::create(Some(0), Some(-2), Some(2), bs, bs, 1), // z
            Node::create(Some(1), None, Some(-3), 0, 0, 1), // T4
        ],
        leafs: vec![
            Leaf::new(0),
            Leaf::create(0, m, b), // T1
            Leaf::create(1, m, b), // T23
            Leaf::create(2, m, b), // Child at T4
        ],
    };
    d.rotate_left(1, 0);
    d.viz();
    assert_eq!(d,
            DynamicBitVector {
        root: 1,
        nodes: vec![
            Node::create(Some(1), Some(-1), Some(-2), bs, bs, 0),
            Node::create(None, Some(0), Some(2), 2 * bs, 2 * bs, 0),
            Node::create(Some(1), None, Some(-3), 0, 0, 1),
        ],
        leafs: vec![
            Leaf::new(0),
            Leaf::create(0, m, b),
            Leaf::create(0, m, b),
            Leaf::create(2, m, b),
        ],
        }
    );
}

#[test]
fn rotate_right_2() {
    let m = LeafValue::MAX;
    let b = LeafValue::BITS as u8;
    let bs = b as usize;
    let mut d = DynamicBitVector {
        root: 0,
        nodes: vec![
            Node::create(Some(1), Some(-1), None, bs, bs, -1), // T1
            Node::create(Some(2), Some(1), Some(-2), bs, bs, -1), // z
            Node::create(None, Some(2), Some(-3), 2 * bs, 2 * bs, -2), // x
        ],
        leafs: vec![
            Leaf::new(0),
            Leaf::create(0, m, b), // Child at T1
            Leaf::create(1, m, b), // T23
            Leaf::create(2, m, b), // T4
        ],
    };
    d.rotate_right(1, 2);
    d.viz();
    assert_eq!(d,
            DynamicBitVector {
        root: 1,
        nodes: vec![
            Node::create(Some(1), Some(-1), None, bs, bs, -1),
            Node::create(None, Some(1), Some(2), bs, bs, 0),
            Node::create(Some(1), Some(-2), Some(-3), bs, bs, 0),
        ],
        leafs: vec![
            Leaf::new(0),
            Leaf::create(0, m, b),
            Leaf::create(2, m, b),
            Leaf::create(2, m, b),
        ],
        }
    );
}




// function tests for DynamicBitVector:
// - static: check after each chance for modification
// - [ ] ones: static
// - [ ] nums: static
// - [ ] rank.Node: static
//
// Static BitVec functionality: (should be transcendet from Leaf/Node, low priority testing)
// - [ ] rank
// - [ ] select
// - [ ] access
//
// Dynamic BitVec functionality:
// - [/] creation
//      - [x] `new`
//      - [ ] `with_capacity`
// - [x] push
//      - [x] moving and creation of substructures when required
//      - [x] modification of `nums` and `ones`
//      - [x] including rotation
//      - [ ] when created `with_capacity`
// - [ ] insert
//      - [/] only 'last' place
//      - [/] only 'first' place
//      - [ ] random places
//      - [ ] leaf splitting
//      - [ ] final structure including `nums`, `ones`, `rank`
// - [ ] flip
//      - [ ] random places
//      - [ ] structure modification of `nums` and `ones`
// - [x] rotate_left
// - [x] rotate_right
// - [ ] rotate_right_left
// - [ ] rotate_left_right
// - [ ] delete
//      - [ ] modification of `ones` and `nums`
//      - [ ] bit stealing
//      - [ ] merging (merge_away)
//      - [ ] rotations
