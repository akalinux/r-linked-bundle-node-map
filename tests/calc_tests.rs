#![cfg(test)]

use linked_bundle_node_map::{
    calc::Options,
    node::{LabelPosition, NodeOpt},
};

#[test]
fn option_tests() {
    let mut opt = Options::new();
    assert_eq!(
        opt.get_node(&String::from("Does not exist")).id,
        NodeOpt::defaults().id
    );
    opt.set_node(NodeOpt {
        id: String::from("Does not exist"),
        img: String::from(""),
        color: String::from("pink"),
        layer: -1,
        label: LabelPosition::Bottom,
    });
    assert_eq!(
        opt.get_node(&String::from("Does not exist")).id,
        String::from("Does not exist")
    );
    opt.rm_node(&String::from("Does not exist"));

    assert_eq!(
        opt.get_node(&String::from("Does not exist")).id,
        NodeOpt::defaults().id,
    );
}
