#![cfg(test)]
use linked_bundle_node_map::{
    CalculatorTrait, ContainsPoint, FullBox, Point, PointBox,
    constants::ZERO_POINT,
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

    let node_a = Node {
        x: 0.0,
        y: 0.0,
        h: 2.0,
        w: 2.0,
        id: 0,
        label: l.clone(),
        opt: 0,
        groups: Vec::from([0, 1]),
    };
    let node_b = Node {
        x: 0.0,
        y: 0.0,
        h: 2.0,
        w: 2.0,
        id: 1,
        label: l.clone(),
        opt: 0,
        groups: Vec::from([2, 1]),
    };
    let node_c = Node {
        x: 0.0,
        y: 0.0,
        h: 2.0,
        w: 2.0,
        id: 3,
        label: l.clone(),
        opt: 0,
        groups: Vec::from([0, 3]),
    };
    nodes.insert(node_a.clone(), false);
    nodes.insert(node_b.clone(), false);
    nodes.insert(node_c.clone(), false);
    let mut res = Vec::new();
    for node in nodes.get_related(&[0, 1]) {
        res.push(node.0.id);
    }
    res.sort();
    assert_eq!(res, vec![0, 1, 3]);
    assert!(!nodes.nodes.is_empty());
    nodes.remove(0);
    res.clear();
    for node in nodes.get_related(&[0, 1]) {
        res.push(node.0.id);
    }
    assert_eq!(res, vec![1]);
    nodes.insert(node_a.clone(), false);
    res.clear();
    for node in nodes.get_related(&[0]) {
        res.push(node.0.id);
    }
    res.sort();
    assert_eq!(res, vec![0, 1, 3]);
    // box layer testing
    res.clear();
    nodes.remove(0);
    nodes.insert_box(node_a.clone(), false);
    for node in nodes.get_related(&[0]) {
        res.push(node.0.id);
    }
    res.sort();
    assert_eq!(res, vec![0, 1, 3]);
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
    let c = Node {
        x: 5.0,
        y: 5.0,
        h: 2.0,
        w: 2.0,
        id: 3,
        label: l.clone(),
        opt: 0,
        groups: Vec::from([0, 1]),
    };
    ns.insert(a.clone(), false);
    assert_eq!(ns.get_center(), Point { x: 0.0, y: 0.0 });
    ns.insert(b.clone(), false);
    assert_eq!(ns.get_center(), Point { x: 2.5, y: 2.5 });
    ns.update(b.transform(5.0, 5.0, 0.0, 0.0));
    assert_eq!(ns.get_center(), Point { x: 5.0, y: 5.0 });
    ns.insert(b.clone(), false);
    assert_eq!(ns.get_center(), Point { x: 2.5, y: 2.5 });
    ns.remove(b.id);
    assert_eq!(ns.get_center(), Point { x: 0.0, y: 0.0 });
    ns.insert_box(c, false);
}

#[test]
#[should_panic]
fn add_node_as_box_fail() {
    let a = Node {
        x: 0.0,
        y: 0.0,
        h: 2.0,
        w: 2.0,
        id: 0,
        label: String::from("value"),
        opt: 0,
        groups: Vec::from([0, 1]),
    };
    let mut ns = NodeStates::new(2);
    ns.insert(a.clone(), false);
    ns.insert_box(a.clone(), false);
}

#[test]
#[should_panic]
fn add_box_as_node_fail() {
    let a = Node {
        x: 0.0,
        y: 0.0,
        h: 2.0,
        w: 2.0,
        id: 0,
        label: String::from("value"),
        opt: 0,
        groups: Vec::from([0, 1]),
    };
    let mut ns = NodeStates::new(2);
    ns.insert_box(a.clone(), false);
    ns.insert(a.clone(), false);
}

#[test]
fn order_tests() {
    let l = String::from("value");
    let mut a = Node {
        x: 0.0,
        y: 0.0,
        h: 2.0,
        w: 2.0,
        id: 2,
        label: l.clone(),
        opt: 0,
        groups: Vec::from([0, 1]),
    };
    let mut b = Node {
        x: 5.0,
        y: 5.0,
        h: 2.0,
        w: 2.0,
        id: 1,
        label: l.clone(),
        opt: 0,
        groups: Vec::from([0, 1]),
    };
    let mut sorted = Vec::from(&[b.clone(), a.clone()]);
    sorted.sort();
    assert_eq!(sorted, Vec::from(&[a.clone(), b.clone()]));
    // a is bigger on the x axis, so it should come before b
    b.x = 0.0;
    b.y = 0.0;
    a.w = 3.0;
    sorted = Vec::from(&[b.clone(), a.clone()]);
    sorted.sort();
    assert_eq!(sorted, Vec::from(&[a.clone(), b.clone()]));
    // a is bigger on the y axis, so it should come before b
    a.w = 2.0;
    a.h = 3.0;
    sorted = Vec::from(&[b.clone(), a.clone()]);
    sorted.sort();
    assert_eq!(sorted, Vec::from(&[a.clone(), b.clone()]));
    // same size and place.. so compare on id.. b is before a.
    a.h = 2.0;
    sorted = Vec::from(&[a.clone(), b.clone()]);
    sorted.sort();
    assert_eq!(sorted, Vec::from(&[b.clone(), a.clone()]));
}

#[test]
fn contains_tests() {
    let a = Node {
        x: 1.0,
        y: 1.0,
        h: 2.0,
        w: 2.0,
        id: 2,
        label: String::from("blah"),
        opt: 0,
        groups: Vec::from([0, 1]),
    };
    assert!(a.x_contains(0.0));
    assert!(a.x_contains(2.0));
    assert!(a.y_contains(0.0));
    assert!(a.y_contains(2.0));
    assert!(!a.x_contains(-1.0));
    assert!(!a.x_contains(3.0));
    assert!(!a.y_contains(-1.0));
    assert!(!a.y_contains(3.0));
    let b = Node {
        x: 4.0,
        y: 4.0,
        h: 2.0,
        w: 2.0,
        id: 2,
        label: String::from("blah"),
        opt: 0,
        groups: Vec::from([0, 1]),
    };
    assert!(a.overlaps(&a));
    assert!(!a.overlaps(&b));
}

#[test]
fn merge_tests() {
    let mut ns = NodeStates::new(2);
    assert_eq!(ns.get_center(), Point { x: 0.0, y: 0.0 });
    let mut node = Node::new(0.0, 0.0, 2.0, 2.0, 0, String::from("testing"), 0, vec![]);
    ns.insert(node.clone(), false);
    assert_eq!(ns.get_center(), ZERO_POINT);
    node.x = 10.0;
    node.y = 10.0;
    ns.update(node.clone());
    let old = node.get_center();
    assert_eq!(ns.get_center(), old.clone());
    node.x = 5.0;
    node.y = 5.0;
    node.label = String::from("test");
    node.opt = 1;
    node.groups = vec![2, 3, 4];
    ns.insert(node.clone(), true);

    assert_eq!(ns.get_center(), old.clone());
    node.x = 10.0;
    node.y = 10.0;
    assert_eq!(&ns.get(0).unwrap().groups, &node.groups);
    assert_eq!(&ns.get(0).unwrap().opt, &node.opt);
    assert_eq!(&ns.get(0).unwrap().label, &node.label);
    assert_eq!(&ns.get(0).unwrap().get_center(), &old);
    node.x = old.x;
    node.y = old.y;
    assert_eq!(ns.get_updates(), (vec![&node], vec![]));
    ns.remove(node.id);
    assert_eq!(ns.get_updates(), (vec![], vec![]));
    assert_eq!(ns.get_center(), ZERO_POINT);
    ns.insert_box(node.clone(), true);
    assert_eq!(ns.get_center(), ZERO_POINT);
    node.to_point(&ZERO_POINT);
    ns.update(node.clone());
    assert_eq!(ns.get_updates(), (vec![], vec![&node]));
    assert_eq!(ns.get_box(0).unwrap().get_center(), ZERO_POINT);
    node.to_point(&old);
    ns.insert_box(node.clone(), true);
    assert_eq!(ns.get_updates(), (vec![], vec![&node]));
}
