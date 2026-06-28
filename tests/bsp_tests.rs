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
fn screen_index_tests() {
    let mut idx = ScreenIndex::new(10);

    // Default should say nothing can be on screen!
    assert_eq!(idx.max_screen(), None);
    let mut res = idx.on_screen(
        10,
        10,
        &Transform {
            x: 0.0,
            y: 0.0,
            k: 1.0,
        },
        0,
    );
    assert_eq!((res.0.len(), res.1.len()), (0, 0));
    let src = Node::new(1.0, 1.0, 2.0, 2.0, 0, String::from("test1"), 0, Vec::new());
    let dst = Node::new(9.0, 9.0, 2.0, 2.0, 1, String::from("test2"), 0, Vec::new());
    let mut ns = NodeStates::new(2);
    let mut opts = Options::new();
    let mut animations = HashMap::new();
    ns.insert(src.clone());
    ns.insert(dst.clone());
    let mut link = LinkContainer::new(0, 1);
    idx.index_node(src.id, (None, Some(src.index_bound(idx.step))));
    assert_eq!(
        idx.max_screen().unwrap(),
        ScreenBox {
            width: 10,
            height: 10,
            x: 0,
            y: 0,
            step: 10
        }
    );
    let create_res = |idx: &mut ScreenIndex| {
        return idx.on_screen(
            10,
            10,
            &Transform {
                x: 0.0,
                y: 0.0,
                k: 1.0,
            },
            1,
        );
    };
    res = create_res(&mut idx);
    assert_eq!((res.0.len(), res.1.len()), (1, 0));
    idx.clear_node(src.id, Some(src.index_bound(idx.step)));

    res = create_res(&mut idx);
    assert_eq!((res.0.len(), res.1.len()), (0, 0));
    assert_eq!(idx.max_screen(), None);
    idx.index_link(link.id, link.screen_index(idx.step, true));
    assert_eq!(idx.max_screen(), None);
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

    res = create_res(&mut idx);
    assert_eq!((res.0.len(), res.1.len()), (0, 1));
    idx.index_link(link.id, link.screen_index(idx.step, false));
    res = create_res(&mut idx);
    assert_eq!((res.0.len(), res.1.len()), (0, 0));
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
