#![cfg(test)]

use std::collections::HashMap;

use linked_bundle_node_map::{
    PointBox, ScreenBox, Transform,
    bsp::{IsIndexed, ScreenIndex},
    calc::Options,
    link::{Animation, Link, LinkContainer},
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
    idx.index_node(src.id, (None, Some(src.index_bound(idx.step))));

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
    assert_eq!(iter.next(), Some((vec![0], Vec::new(), screena,)));
    assert!(iter.next().is_none());
    idx.index_node(dst.id, (None, Some(dst.index_bound(idx.step))));
    iter = idx.on_screen(10, 10, &t);
    assert_eq!(iter.next(), Some((vec![0], Vec::new(), screena)));
    assert_eq!(iter.next(), Some((vec![1], Vec::new(), screend)));
    assert!(iter.next().is_none());
    let mut ns = NodeStates::new(2);
    ns.insert(src.clone());
    ns.insert(dst.clone());
    let mut opts = Options::new();
    let mut animations = HashMap::new();
    let mut link = LinkContainer::new(0, 1);

    link.add_link(Link {
        id: 0,
        src: 0,
        dst: 1,
        opt: 0,
        label: String::from("link1"),
        animation: Animation::Both,
    });
    link.update(&mut ns, &mut opts, &mut animations);
    idx.index_link(link.id, link.screen_index(idx.step, true));
    iter = idx.on_screen(10, 10, &t);
    assert_eq!(iter.next(), Some((vec![0], vec![link.id], screena)));
    assert_eq!(iter.next(), Some((vec![1], Vec::new(), screend)));
    assert!(iter.next().is_none());

    t.x = -5.0;
    iter = idx.on_screen(5, 5, &t);
    assert_eq!(iter.next(), Some((Vec::new(), vec![link.id], screenb)));
    assert!(iter.next().is_none());
    idx.index_link(link.id, link.screen_index(idx.step, false));
    iter = idx.on_screen(5, 5, &t);
    assert!(iter.next().is_none());

    idx.index_node(j.id, (None, Some(j.index_bound(idx.step))));
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
fn is_indexed_tests() {
    let mut idx = IsIndexed::<u32>::new(2);
    assert!(idx.is_empty());
    assert!(!idx.is_mouse(0));
    assert!(!idx.is_screen(0));

    idx.add_mouse(0);
    assert!(!idx.is_empty());
    assert!(idx.is_mouse(0));
    assert!(!idx.is_screen(0));

    idx.add_screen(0);
    assert!(!idx.is_empty());
    assert!(idx.is_screen(0));
    assert!(idx.is_mouse(0));
    idx.clear_screen(0);
    assert!(!idx.is_empty());
    assert!(idx.is_mouse(0));
    assert!(!idx.is_screen(0));
    idx.clear_mouse(0);
    assert!(!idx.is_mouse(0));
    assert!(!idx.is_screen(0));
    assert!(idx.is_empty());
}
