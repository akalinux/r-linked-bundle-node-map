#![cfg(test)]


use linked_bundle_node_map::{
    calc::Options,
    node::{LabelPosition, NodeOpt},
};
use wasm_bindgen_test::*;
//wasm_bindgen_test_configure!(run_in_browser); 

#[wasm_bindgen_test]
#[test]
fn option_tests() {
    let mut opt = Options::new();
    assert_eq!(opt.get_node(&1).id, NodeOpt::defaults().id);
    opt.set_node(NodeOpt {
        id: 1,
        label: String::from("Does not exist"),
        img: String::from(""),
        color: String::from("pink"),
        label_position: LabelPosition::Bottom,
    });
    assert_eq!(opt.get_node(&1).id, 1,);
    opt.rm_node(&1);

    assert_eq!(opt.get_node(&1).id, NodeOpt::defaults().id,);
}
