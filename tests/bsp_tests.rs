#![cfg(test)]

use std::collections::HashSet;

use linked_bundle_node_map::{
    Point, PointBox, ScreenBox, Transform,
    bsp::{IdxBoxAction, IdxBoxIter, PointLookupResult, ScreenIndex, ScreenSlot},
    calc::{BacklogUpdates, Options},
    constants::{ZERO_POINT, ZERO_TRANSFORM},
    link::{Animation, Link, LinkContainer, LinkStates},
    node::{Node, NodeStates},
};

#[test]
fn screen_idx_iter_tests() {
    let mut idx = ScreenIndex::new(5);
    let mut t = Transform {
        x: 0.0,
        y: 0.0,
        k: 1.0,
    };
    let mut iter = idx.on_screen(10, 10, &t);

    assert!(iter.next().is_none());
    let src = Node::new(1.0, 1.0, 2.0, 2.0, 0, String::from("test1"), 0, Vec::new());
    let dst = Node::new(9.0, 9.0, 2.0, 2.0, 1, String::from("test2"), 0, Vec::new());
    let j = Node::new(
        -2.5,
        -2.5,
        3.0,
        3.0,
        2,
        String::from("test2"),
        0,
        Vec::new(),
    );
    idx.index(
        ScreenSlot::Node(src.id),
        (None, Some(src.index_bound(idx.step))),
    );

    iter = idx.on_screen(10, 10, &t);
    let screena = ScreenBox {
        width: 5,
        height: 5,
        x: 0,
        y: 0,
        step: 5,
    };
    let screenb = ScreenBox {
        width: 5,
        height: 5,
        x: 5,
        y: 0,
        step: 5,
    };
    let screend = ScreenBox {
        width: 5,
        height: 5,
        x: 5,
        y: 5,
        step: 5,
    };
    assert_eq!(iter.next(), Some((vec![0], Vec::new(), vec![], screena,)));
    assert!(iter.next().is_none());
    idx.index(
        ScreenSlot::Node(dst.id),
        (None, Some(dst.index_bound(idx.step))),
    );
    iter = idx.on_screen(10, 10, &t);
    assert_eq!(iter.next(), Some((vec![0], Vec::new(), vec![], screena)));
    assert_eq!(iter.next(), Some((vec![1], Vec::new(), vec![], screend)));
    assert!(iter.next().is_none());
    let mut ns = NodeStates::new(2);
    ns.insert(src.clone(), false);
    ns.insert(dst.clone(), false);
    let mut opts = Options::new();
    let mut animations = HashSet::new();
    let mut link = LinkContainer::new(0, 1);

    link.link_add(Link {
        id: 0,
        src: 0,
        dst: 1,
        opt: 0,
        label: String::from("link1"),
        animation: Animation::Both,
    });
    link.update(&mut ns, &mut opts, &mut animations);
    idx.index(ScreenSlot::Link(link.id), link.screen_index(idx.step, true));
    iter = idx.on_screen(10, 10, &t);
    assert_eq!(iter.next(), Some((vec![0], vec![link.id], vec![], screena)));
    assert_eq!(iter.next(), Some((vec![1], Vec::new(), vec![], screend)));
    assert!(iter.next().is_none());

    t.x = -5.0;
    iter = idx.on_screen(5, 5, &t);
    assert_eq!(
        iter.next(),
        Some((Vec::new(), vec![link.id], vec![], screenb))
    );
    assert!(iter.next().is_none());
    idx.index(
        ScreenSlot::Link(link.id),
        link.screen_index(idx.step, false),
    );
    iter = idx.on_screen(5, 5, &t);
    assert!(iter.next().is_none());

    idx.index(
        ScreenSlot::Node(j.id),
        (None, Some(j.index_bound(idx.step))),
    );
    assert_eq!(
        idx.max_screen().unwrap(),
        ScreenBox {
            width: 20,
            height: 20,
            x: -5,
            y: -5,
            step: 5
        }
    )
}

#[test]
fn iter_box_tests() {
    let mut iter = IdxBoxIter::new(None, None, 1);
    assert!(iter.next().is_none());
    iter = IdxBoxIter::new(Some((1..=1, 1..=1)), None, 1);
    assert_eq!(iter.next(), Some((1, 1, IdxBoxAction::Remove)));
    assert!(iter.next().is_none());
    iter = IdxBoxIter::new(None, Some((1..=1, 1..=1)), 1);
    assert_eq!(iter.next(), Some((1, 1, IdxBoxAction::Add)));
    assert!(iter.next().is_none());
    iter = IdxBoxIter::new(Some((1..=1, 1..=1)), Some((1..=1, 1..=1)), 1);
    assert!(iter.next().is_none());
    iter = IdxBoxIter::new(Some((1..=2, 1..=2)), Some((2..=3, 2..=3)), 1);
    assert_eq!(iter.next(), Some((1, 1, IdxBoxAction::Remove)));
    assert_eq!(iter.next(), Some((2, 1, IdxBoxAction::Remove)));
    assert_eq!(iter.next(), Some((1, 2, IdxBoxAction::Remove)));
    assert_eq!(iter.next(), Some((3, 2, IdxBoxAction::Add)));
    assert_eq!(iter.next(), Some((2, 3, IdxBoxAction::Add)));
    assert_eq!(iter.next(), Some((3, 3, IdxBoxAction::Add)));
    assert!(iter.next().is_none());
}

#[test]
fn in_point_tests() {
    let l = String::from("");
    let node_a = Node {
        x: 1.0,
        y: 1.0,
        h: 2.0,
        w: 2.0,
        id: 0,
        label: l.clone(),
        opt: 0,
        groups: Vec::from([0, 1]),
    };
    let node_b = Node {
        x: 9.0,
        y: 9.0,
        h: 2.0,
        w: 2.0,
        id: 1,
        label: l.clone(),
        opt: 0,
        groups: Vec::from([2, 1]),
    };
    let node_box = Node {
        x: 5.0,
        y: 5.0,
        h: 10.0,
        w: 10.0,
        id: 2,
        label: l.clone(),
        opt: 0,
        groups: Vec::from([1, 2]),
    };
    let link_a = Link::new(0, 0, 1, 0, Animation::Both, l.clone());
    let mut ns = NodeStates::new(2);
    ns.insert(node_a.clone(), false);
    ns.insert(node_b.clone(), false);
    ns.insert_box(node_box.clone(), false);
    let step = 10;
    let mut idx = ScreenIndex::new(step);
    let mut ls = LinkStates::new();
    let mut ops = Options::new();
    let mut animations = HashSet::new();
    let mut backlog = BacklogUpdates::new();
    idx.index(ScreenSlot::Node(0), (None, Some(node_a.index_bound(step))));
    idx.index(ScreenSlot::Node(1), (None, Some(node_b.index_bound(step))));
    idx.index(ScreenSlot::Box(2), (None, Some(node_box.index_bound(step))));
    ls.link_add(
        link_a.clone(),
        &mut ns,
        &mut ops,
        &mut animations,
        &mut idx,
        &mut backlog,
    );
    let t = &ZERO_TRANSFORM;
    assert_eq!(
        idx.in_point(&ZERO_POINT, t, &ns, &ls),
        Some(PointLookupResult::Node(node_a.clone()))
    );
    assert_eq!(
        idx.in_point(&Point { x: 10.0, y: 10.0 }, t, &ns, &ls),
        Some(PointLookupResult::Node(node_b.clone()))
    );
    assert_eq!(
        idx.in_point(&Point { x: 5.0, y: 5.0 }, t, &ns, &ls),
        Some(PointLookupResult::Link(link_a.clone()))
    );
    assert_eq!(
        idx.in_point(&Point { x: 10.0, y: 0.0 }, t, &ns, &ls),
        Some(PointLookupResult::Box(node_box.clone()))
    );
    idx.index(ScreenSlot::Box(2), (Some(node_box.index_bound(step)), None));
    assert_eq!(idx.in_point(&Point { x: 10.0, y: 0.0 }, t, &ns, &ls), None,);
    idx.index(ScreenSlot::Box(2), (None, Some(node_box.index_bound(step))));
    assert_eq!(
        idx.in_point(&Point { x: 10.0, y: 0.0 }, t, &ns, &ls),
        Some(PointLookupResult::Box(node_box.clone()))
    );
    ls.link_remove(
        link_a.id,
        &ns,
        &mut ops,
        &mut animations,
        &mut idx,
        &mut backlog,
    );
    assert_eq!(
        idx.in_point(&Point { x: 5.0, y: 5.0 }, t, &ns, &ls),
        Some(PointLookupResult::Box(node_box.clone()))
    );
}
