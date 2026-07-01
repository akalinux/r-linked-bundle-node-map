#![cfg(test)]
use linked_bundle_node_map::{
    CalculatorTrait, ContainsPoint, FullBox, Point, PointBox,
    node::{Node, NodeStates},
};

#[test]
fn node_box_negative() {
    assert_eq!(
        Node::new(
            -2.0,
            -2.0,
            1.0,
            1.0,
            0,
            String::from("label"),
            0,
            Vec::new(),
        )
        .index_bound(5),
        (-5..=-5, -5..=-5)
    );

    assert_eq!(
        Node::new(0.5, 0.5, 1.0, 1.0, 0, String::from("label"), 0, Vec::new(),).index_bound(5),
        (0..=0, 0..=0)
    );
    assert_eq!(
        Node::new(0.0, 0.0, 1.0, 1.0, 0, String::from("label"), 0, Vec::new(),).index_bound(5),
        (-5..=0, -5..=0)
    );
}
#[test]
fn node_box() {
    let mut node = Node::new(0.5, 0.5, 1.0, 1.0, 0, String::from("label"), 0, Vec::new());

    let (mut nw, mut ne, mut sw, mut se) = node.full_box();
    assert_eq!(ne.x, 1.0);
    assert_eq!(se.x, 1.0);
    assert_eq!(ne.y, 0.0);
    assert_eq!(nw.y, 0.0);
    assert_eq!(nw.x, 0.0);
    assert_eq!(sw.x, 0.0);
    assert_eq!(sw.y, 1.0);
    assert_eq!(se.y, 1.0);
    assert!(node.contains_point(&Point { x: 0.5, y: 0.5 }));
    assert!(!node.contains_point(&Point { x: -0.5, y: -0.5 }));

    let (x, y) = node.index_bound(1);

    assert_eq!(
        node.w * node.h,
        node.triangle_area(nw.x, nw.y, ne.x, ne.y, sw.x, sw.y)
            + node.triangle_area(nw.x, nw.y, ne.x, ne.y, se.x, se.y)
    );
    assert_eq!(x, 0..=1);
    assert_eq!(y, 0..=1);

    // dead center
    assert!(node.inside_box(&node, &Point { x: 0.5, y: 0.5 }));

    // left
    assert!(!node.inside_box(&node, &Point { x: -0.5, y: 0.5 }));
    // right
    assert!(!node.inside_box(&node, &Point { x: 1.5, y: 0.5 }));
    // above
    assert!(!node.inside_box(&node, &Point { x: 0.5, y: -0.5 }));
    // below
    assert!(!node.inside_box(&node, &Point { x: 0.5, y: 1.5 }));
    // top left
    assert!(node.inside_box(&node, &Point { x: 0.0, y: 0.0 }));
    // top right
    assert!(node.inside_box(&node, &Point { x: 1.0, y: 0.0 }));
    // bottom left
    assert!(node.inside_box(&node, &Point { x: 0.0, y: 1.0 }));
    // bottom right
    assert!(node.inside_box(&node, &Point { x: 1.0, y: 1.0 }));

    node = node.transform(0.5, 0.5, 1.0, 1.0);
    (nw, ne, sw, se) = node.full_box();
    assert_eq!(ne.x, 2.0);
    assert_eq!(se.x, 2.0);
    assert_eq!(ne.y, 0.0);
    assert_eq!(nw.y, 0.0);
    assert_eq!(nw.x, 0.0);
    assert_eq!(sw.x, 0.0);
    assert_eq!(sw.y, 2.0);
    assert_eq!(se.y, 2.0);

    assert_eq!(node.get_distance(0.0, 0.0, 2.0, 0.0), 2.0);
    let r = node.get_distance(0.0, 0.0, 2.0, 2.0);
    let angle = node.get_angle(2.0, 2.0, 0.0, 0.0);
    assert_eq!(angle.floor(), 45.0);
    let mut point = node.get_xy(0.0, 0.0, r, angle);
    point.x = point.x.floor();
    point.y = point.y.floor();
    assert_eq!(point, Point { x: 2.0, y: 2.0 });
}

#[test]
fn get_related_node_tests() {
    let mut nodes = NodeStates::new(3);
    let l = String::from("value");
    nodes.insert(Node {
        x: 0.0,
        y: 0.0,
        h: 2.0,
        w: 2.0,
        id: 0,
        label: l.clone(),
        opt: 0,
        groups: Vec::from([0, 1]),
    });
    nodes.insert(Node {
        x: 0.0,
        y: 0.0,
        h: 2.0,
        w: 2.0,
        id: 1,
        label: l.clone(),
        opt: 0,
        groups: Vec::from([2, 1]),
    });
    nodes.insert(Node {
        x: 0.0,
        y: 0.0,
        h: 2.0,
        w: 2.0,
        id: 3,
        label: l.clone(),
        opt: 0,
        groups: Vec::from([0, 3]),
    });
    nodes.group_add(0, &[1, 2]);
    nodes.group_add(1, &[1, 3, 5]);
    let mut iter = nodes.get_related(&[0]);
    assert_eq!(iter.next().unwrap().id, 0);
    assert_eq!(iter.next().unwrap().id, 3);
    assert_eq!(iter.next().unwrap().id, 1);
    assert!(iter.next().is_none());
    assert!(!nodes.nodes.is_empty());
}

#[test]
fn center_tests() {
    let mut ns = NodeStates::new(2);
    assert_eq!(ns.get_center(), Point { x: 0.0, y: 0.0 });
    let l = String::from("value");
    let a = Node {
        x: 0.0,
        y: 0.0,
        h: 2.0,
        w: 2.0,
        id: 0,
        label: l.clone(),
        opt: 0,
        groups: Vec::from([0, 1]),
    };
    let b = Node {
        x: 5.0,
        y: 5.0,
        h: 2.0,
        w: 2.0,
        id: 1,
        label: l.clone(),
        opt: 0,
        groups: Vec::from([0, 1]),
    };
    ns.insert(a.clone());
    assert_eq!(ns.get_center(), Point { x: 0.0, y: 0.0 });
    ns.insert(b.clone());
    assert_eq!(ns.get_center(), Point { x: 2.5, y: 2.5 });
    ns.update(b.transform(5.0, 5.0, 0.0, 0.0));
    assert_eq!(ns.get_center(), Point { x: 5.0, y: 5.0 });
    ns.insert(b.clone());
    assert_eq!(ns.get_center(), Point { x: 2.5, y: 2.5 });
    ns.remove(b.id);
    assert_eq!(ns.get_center(), Point { x: 0.0, y: 0.0 });
}
