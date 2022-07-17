use super::*;

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
    l.push(false).ok();
    l.push(false).ok();
    l.push(false).ok();
    l.push(false).ok();
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
    l.push(true).ok();
    l.push(true).ok();
    l.push(true).ok();
    l.push(true).ok();
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
        l.push(false).ok();
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
        l.push(true).ok();
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
    l.insert(0, false).ok();
    l.insert(0, false).ok();
    l.insert(0, false).ok();
    l.insert(0, false).ok();
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
    l.insert(0, true).ok();
    l.insert(0, true).ok();
    l.insert(0, true).ok();
    l.insert(0, true).ok();
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
        l.insert(0, false).ok();
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
        l.insert(0, true).ok();
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
