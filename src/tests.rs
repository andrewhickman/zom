use Zom::{self, *};

#[test]
fn push_pop_shrink() {
    let mut zom = Zero;
    zom.push(0);
    assert_eq!(zom, One(0));
    zom.push(1);
    assert_eq!(zom, Many(vec![0, 1]));
    zom.push(2);
    assert_eq!(zom, Many(vec![0, 1, 2]));

    assert_eq!(zom.pop(), Some(2));
    assert_eq!(zom, Many(vec![0, 1]));
    assert_eq!(zom.pop(), Some(1));
    assert_eq!(zom, Many(vec![0]));
    assert_eq!(zom.pop(), Some(0));
    assert_eq!(zom, Many(vec![]));
    assert_eq!(zom.pop(), None);
    assert_eq!(zom, Many(vec![]));

    zom.push(0);
    assert_eq!(zom, Many(vec![0]));
    zom.push(1);
    assert_eq!(zom, Many(vec![0, 1]));
    zom.shrink_to_fit();
    assert_eq!(zom, Many(vec![0, 1]));

    assert_eq!(zom.pop(), Some(1));
    assert_eq!(zom, Many(vec![0]));
    zom.shrink_to_fit();
    assert_eq!(zom, One(0));

    zom.push(1);
    assert_eq!(zom, Many(vec![0, 1]));
    zom.shrink_to_fit();
    assert_eq!(zom, Many(vec![0, 1]));

    zom.clear();
    assert_eq!(zom, Many(vec![]));
    zom.shrink_to_fit();
    assert_eq!(zom, Zero);
}

#[test]
fn iter() {
    let mut zom: Zom<i32> = vec![].into_iter().collect();
    assert_eq!(zom, Zom::Zero);
    zom.extend(vec![]);
    assert_eq!(zom, Zom::Zero);
    zom.extend(vec![0]);
    assert_eq!(zom, Zom::One(0));
    zom.extend(vec![1]);
    assert_eq!(zom, Zom::Many(vec![0, 1]));
    zom.extend(vec![2, 3]);
    assert_eq!(zom, Zom::Many(vec![0, 1, 2, 3]));

    let zom2: Zom<i32> = zom.iter().cloned().collect();
    assert_eq!(zom, zom2);
}
