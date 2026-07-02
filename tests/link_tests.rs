#![cfg(test)]

use std::collections::HashMap;

use linked_bundle_node_map::{
    CalculatorTrait, GetCenter, Point,
    calc::Options,
    link::{Animation, Bundle, Link, LinkContainer, LinkContainerOpt, LinkContainsType},
    node::{Node, NodeStates},
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
    let lc_opt = LinkContainerOpt::defaults();
    lc.add_link(Link {
        id: 0,
        src: 0,
        dst: 1,
        opt: 0,
        animation: Animation::None,
        label: String::from("test link 1"),
    });
    let mut cu = lc.compute_link_segement(
        &Point { x: 0.0, y: 0.0 },
        &Point { x: 10.0, y: 0.0 },
        2.0,
        0,
        &lc_opt,
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
        opt: 0,
        animation: Animation::None,
        label: String::from("test link 1"),
    });

    assert_eq!(lc.links.len(), 2);
    cu = lc.compute_link_segement(
        &Point { x: 0.0, y: 0.0 },
        &Point { x: 10.0, y: 0.0 },
        2.0,
        0,
        &lc_opt,
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
        opt: 0,
        animation: Animation::None,
        label: String::from("test link 1"),
    });
    cu = lc.compute_link_segement(
        &Point { x: 0.0, y: 0.0 },
        &Point { x: 10.0, y: 0.0 },
        2.0,
        0,
        &lc_opt,
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

#[test]
fn compute_bundle_tests() {
    let lc = LinkContainer::new(0, 1);

    // does not require distance
    // 0 1 2
    //   1
    let mut sets = Vec::new();

    lc.compute_bunlde_points(
        &Point { x: 0.0, y: 0.0 },
        &Point { x: 2.0, y: 0.0 },
        1,
        &mut sets,
    );
    assert_eq!(sets.len(), 1);
    assert_eq!(sets[0], Point { x: 1.0, y: 0.0 });

    sets.clear();
    // does not require distance
    //  0 1 2 3 4
    //    1   2
    lc.compute_bunlde_points(
        &Point { x: 0.0, y: 0.0 },
        &Point { x: 4.0, y: 0.0 },
        2,
        &mut sets,
    );
    assert_eq!(sets.len(), 2);
    assert_eq!(sets[0], Point { x: 1.0, y: 0.0 });
    assert_eq!(sets[1], Point { x: 3.0, y: 0.0 });

    sets.clear();
    // does not require distance
    //  0 1 2 3 4 5 6
    //    1   2   3
    lc.compute_bunlde_points(
        &Point { x: 0.0, y: 0.0 },
        &Point { x: 6.0, y: 0.0 },
        3,
        &mut sets,
    );
    assert_eq!(sets.len(), 3);
    assert_eq!(sets[0], Point { x: 1.5, y: 0.0 });
    assert_eq!(sets[1], Point { x: 3.0, y: 0.0 });
    assert_eq!(sets[2], Point { x: 4.5, y: 0.0 });

    sets.clear();
    // requires distance
    //  0 1 2 3 4 5 6 7 8
    //    0   1   3   4
    lc.compute_bunlde_points(
        &Point { x: 0.0, y: 0.0 },
        &Point { x: 9.0, y: 0.0 },
        4,
        &mut sets,
    );
    assert_eq!(sets.len(), 4);
    assert_relative_eq!(sets[0].x, 1.0, epsilon = 0.009);
    assert_relative_eq!(sets[0].y as f32, 0.0, epsilon = 0.009);
    assert_relative_eq!(sets[1].x, 3.0, epsilon = 0.009);
    assert_relative_eq!(sets[1].y as f32, 0.0, epsilon = 0.009);
    assert_relative_eq!(sets[2].x, 5.0, epsilon = 0.009);
    assert_relative_eq!(sets[2].y as f32, 0.0, epsilon = 0.009);
    assert_relative_eq!(sets[3].x, 7.0, epsilon = 0.009);
    assert_relative_eq!(sets[3].y as f32, 0.0, epsilon = 0.009);
}

#[test]
fn animation_tests() {
    let mut lc = LinkContainer::new(0, 1);
    let lc_opt = LinkContainerOpt::defaults();
    lc.add_link(Link {
        id: 0,
        src: 0,
        dst: 1,
        opt: 0,
        animation: Animation::ToSrc,
        label: String::from("test link 1"),
    });

    let src = Point { x: 0.0, y: 0.0 };
    let dst = Point { x: 10.0, y: 0.0 };
    let mut cu = lc.compute_link_segement(&src, &dst, 2.0, 0, &lc_opt);

    assert_eq!(cu.animations.len(), 1);
    assert_relative_eq!(cu.animations[0].src.x, dst.x - 2.0);
    assert_relative_eq!(cu.animations[0].src.y, dst.y);
    assert_relative_eq!(cu.animations[0].dst.x, src.x + 2.0);
    assert_relative_eq!(cu.animations[0].dst.y, src.y, epsilon = 0.01);
    assert_relative_eq!(cu.animations[0].width, cu.width * 0.5);
    lc.add_link(Link {
        id: 0,
        src: 1,
        dst: 0,
        opt: 0,
        animation: Animation::ToSrc,
        label: String::from("test link 1"),
    });

    cu = lc.compute_link_segement(&src, &dst, 2.0, 0, &lc_opt);
    assert_relative_eq!(cu.animations[0].src.x, src.x + 2.0);
    assert_relative_eq!(cu.animations[0].src.y, src.y, epsilon = 0.01);
    assert_relative_eq!(cu.animations[0].dst.x, dst.x - 2.0);
    assert_relative_eq!(cu.animations[0].dst.y, dst.y, epsilon = 0.01);
    assert_relative_eq!(cu.animations[0].width, cu.width * 0.5);
    lc.add_link(Link {
        id: 0,
        src: 0,
        dst: 1,
        opt: 0,
        animation: Animation::Both,
        label: String::from("test link 1"),
    });
    cu = lc.compute_link_segement(&src, &dst, 2.0, 0, &lc_opt);
    assert_relative_eq!(cu.animations[0].width, cu.width * 0.3, epsilon = 0.09);
    assert_relative_eq!(cu.animations[1].width, cu.width * 0.3, epsilon = 0.09);
    assert_relative_eq!(cu.animations[0].src.x, src.x + 2.0);
    assert_relative_eq!(cu.animations[0].dst.x, dst.x - 2.0);
    let offset = 0.5;
    assert_relative_eq!(cu.animations[0].dst.y, offset * -1.0, epsilon = 0.02);
    assert_relative_eq!(cu.animations[0].src.y, offset * -1.0, epsilon = 0.02);
    assert_relative_eq!(cu.animations[1].dst.y, offset, epsilon = 0.02);
    assert_relative_eq!(cu.animations[1].src.y, offset, epsilon = 0.02);
}

#[test]
fn point_inside_tests() {
    let mut ns = NodeStates::new(2);
    let src = Node {
        x: 1.0,
        y: 1.0,
        w: 1.0,
        h: 1.0,
        id: 0,
        opt: 0,
        label: String::from("test"),
        groups: Vec::new(),
    };
    let dst = Node {
        x: 9.0,
        y: 9.0,
        w: 1.0,
        h: 1.0,
        id: 1,
        opt: 0,
        label: String::from("test"),
        groups: Vec::new(),
    };
    ns.insert(src.clone());
    ns.insert(dst.clone());
    let mut ops = Options::new();
    let mut animations = HashMap::new();
    let mut lc = LinkContainer::new(src.id, dst.id);
    let link = Link::new(0, 0, 1, 0, Animation::None, String::from("This is a test"));
    lc.add_link(link.clone());
    let bundle = Bundle::new(0, 0, 1, 0, Vec::new(), String::from("value"));
    lc.add_bundle(bundle.clone());
    lc.update(&mut ns, &mut ops, &mut animations);

    assert!(lc.contains_point(&Point { x: 1.0, y: 9.0 }).is_none());
    assert_eq!(
        lc.contains_point(&Point { x: 5.0, y: 5.0 }),
        Some(LinkContainsType::Bundle(bundle.clone()))
    );
    assert_eq!(
        lc.contains_point(&Point { x: 2.5, y: 2.5 }),
        Some(LinkContainsType::Link(link.clone()))
    );
    assert_eq!(
        lc.contains_point(&Point { x: 7.5, y: 7.5 }),
        Some(LinkContainsType::Link(link.clone()))
    );
}
