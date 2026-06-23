#![cfg(test)]

use linked_bundle_node_map::{
    CalculatorTrait, GetCenter, Point,
    link::{Animation, Link, LinkContainer},
};

use approx::assert_relative_eq;
#[test]
fn link_container_ids() {
    let mut lc = LinkContainer::new(0xffffffff, 0xffffffff);

    assert_eq!(lc.id, 0xffffffffffffffff);
    assert_eq!(lc.get_node_ids(), (0xffffffff, 0xffffffff));
    lc = LinkContainer::new(2, 1);

    assert_eq!(lc.get_node_ids(), (1, 2));
}

#[test]
fn validate_get_xy() {
    let lc = LinkContainer::new(0, 1);
    assert_eq!(lc.get_xy(0.0, 0.0, 10.0, 0.0), Point { x: 10.0, y: 0.0 });
    let p = lc.get_xy(0.0, 0.0, 10.0, 90.0);
    assert_relative_eq!(p.x as f32, 0.0);
    assert_relative_eq!(p.y as f32, 10.0);
}
#[test]
fn compute_link_tests() {
    let mut lc = LinkContainer::new(0, 1);
    lc.add_link(Link {
        id: 0,
        src: 0,
        dst: 1,
        opt: String::from("defaults"),
        animation: Animation::None,
        label: String::from("test link 1"),
    });
    let mut cu = lc.compute_link_segement(
        &Point { x: 0.0, y: 0.0 },
        &Point { x: 10.0, y: 0.0 },
        2.0,
        0,
        1.0,
    );
    assert_eq!(cu.get_center(), Point { x: 5.0, y: 0.0 });
    assert_relative_eq!(cu.width, 2.0);
    assert_relative_eq!(cu.min_x as f32, 0.0);
    assert_relative_eq!(cu.max_x as f32, 10.0);
    assert_relative_eq!(cu.min_y as f32, -2.0);
    assert_relative_eq!(cu.max_y as f32, 2.0);
    assert_relative_eq!(cu.links[0].src.x as f32, 2.0);
    assert_relative_eq!(cu.links[0].src.y as f32, 0.0);
    assert_relative_eq!(cu.links[0].dst.x as f32, 8.0);
    assert_relative_eq!(cu.links[0].dst.y as f32, 0.0);
    lc.add_link(Link {
        id: 1,
        src: 0,
        dst: 1,
        opt: String::from("defaults"),
        animation: Animation::None,
        label: String::from("test link 1"),
    });

    assert_eq!(lc.links.len(), 2);
    cu = lc.compute_link_segement(
        &Point { x: 0.0, y: 0.0 },
        &Point { x: 10.0, y: 0.0 },
        2.0,
        0,
        1.0,
    );

    assert_eq!(cu.get_center(), Point { x: 5.0, y: 0.0 });
    assert_relative_eq!(cu.width, 1.33, epsilon = 0.009);
    assert_relative_eq!(cu.links[0].src.x, 2.0);
    assert_relative_eq!(cu.links[0].src.y, -1.0, epsilon = 0.009);
    assert_relative_eq!(cu.links[0].dst.x, 8.0);
    assert_relative_eq!(cu.links[0].dst.y, -1.0, epsilon = 0.009);
    assert_relative_eq!(cu.links[1].src.x, 2.0, epsilon = 0.01);
    assert_relative_eq!(cu.links[1].src.y, 1.0, epsilon = 0.009);
    assert_relative_eq!(cu.links[1].dst.x, 8.0);
    assert_relative_eq!(cu.links[1].dst.y, 1.0, epsilon = 0.009);

    lc.add_link(Link {
        id: 2,
        src: 0,
        dst: 1,
        opt: String::from("defaults"),
        animation: Animation::None,
        label: String::from("test link 1"),
    });
    cu = lc.compute_link_segement(
        &Point { x: 0.0, y: 0.0 },
        &Point { x: 10.0, y: 0.0 },
        2.0,
        0,
        1.0,
    );
    assert_eq!(lc.links.len(), 3);
    assert_relative_eq!(cu.width, 0.8, epsilon = 0.0009);
    assert_relative_eq!(cu.links[0].src.x as f32, 2.0);
    assert_relative_eq!(cu.links[0].src.y as f32, -1.33, epsilon = 0.009);
    assert_relative_eq!(cu.links[0].dst.x as f32, 8.0);
    assert_relative_eq!(cu.links[0].dst.y as f32, -1.33, epsilon = 0.09);

    assert_relative_eq!(cu.links[1].src.x as f32, 2.0, epsilon = 0.009);
    assert_relative_eq!(cu.links[1].src.y as f32, 0.0);
    assert_relative_eq!(cu.links[1].dst.x as f32, 8.0, epsilon = 0.009);
    assert_relative_eq!(cu.links[1].dst.y as f32, 0.0);

    assert_relative_eq!(cu.links[2].src.x as f32, 2.0);
    assert_relative_eq!(cu.links[2].src.y as f32, 1.33, epsilon = 0.009);
    assert_relative_eq!(cu.links[2].dst.x as f32, 8.0);
    assert_relative_eq!(cu.links[2].dst.y as f32, 1.33, epsilon = 0.009);
}
