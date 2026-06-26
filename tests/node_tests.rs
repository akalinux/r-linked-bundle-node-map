#![cfg(test)]
use linked_bundle_node_map::{CalculatorTrait, ContainsPoint, Point, PointBox, node::Node};

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
