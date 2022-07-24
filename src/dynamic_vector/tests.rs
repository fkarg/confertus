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
fn push_6() {
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

fn push_6_2() {
    let mut d = DynamicBitVector::new();
    for _ in 0..(LeafValue::BITS * 6) {
        d.push(true);
    }
    d.push(true);
    d.viz();
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
fn insert_1() {
    let mut d = DynamicBitVector::new();
    for i in 0..(LeafValue::BITS * 1 + 1) {
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
                Node::create(None, Some(2), Some(0), half * 4, half * 4, -1),
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

// #[test]
// WIP
fn insert_3() {
    let mut d = DynamicBitVector::new();
    for i in 0..(LeafValue::BITS * 2) {
        d.insert(d[d.root].nums, true)
            .expect("insert failed at {i}");
    }
    let half = (LeafValue::BITS / 2) as usize;
    assert_eq!(
        d,
        DynamicBitVector {
            root: 0,
            nodes: vec![
                Node::create(None, Some(-1), Some(1), half + 1, half + 1, 1),
                Node::create(Some(0), Some(-2), Some(-3), half + 3, half + 3, 0),
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
                    LeafValue::MAX.overflowing_shr((half - 3) as u32).0,
                    (half + 3) as u8
                ),
                Leaf::create(1, LeafValue::MAX.overflowing_shr(half as u32).0, half as u8),
            ],
        }
    );
}

// #[test]
// WIP
fn insert_4() {
    let mut d = DynamicBitVector::new();
    let half = (LeafValue::BITS / 2) as usize;
    for i in 0..(LeafValue::BITS * 2 + half as u32) {
        d.insert(d[d.root].nums, true)
            .expect("insert failed at {i}");
    }
    d.viz();
    assert_eq!(
        d,
        DynamicBitVector {
            root: 0,
            nodes: vec![
                Node::create(None, Some(-1), Some(1), half + 1, half + 1, 1),
                Node::create(Some(0), Some(-2), Some(-3), half + 3, half + 3, 0),
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
                    LeafValue::MAX.overflowing_shr((half - 3) as u32).0,
                    (half + 3) as u8
                ),
                Leaf::create(1, LeafValue::MAX.overflowing_shr(half as u32).0, half as u8),
            ],
        }
    );
}

// WIP
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

// Still needs testing (DynamicBitVector):
// - (push)
// - ones
// - access
// - rank
// - select
// - insert
// - delete
// - flip
// - nums
// - (retrace)
// - (rotate_left)
// - (rotate_right)
// - (rebalance)
// - (rebalance_no_child)
// - (create_right_leaf)
// - (closest_neighbor_leaf|child)
// - (merge_away)
// - (swap_remove_leaf|node)
// - (get_side_*)
// - (update_left_values)
// - (split_leaf)
