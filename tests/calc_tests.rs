#![cfg(test)]

use linked_bundle_node_map::{
    Point, ScreenBox, Transform,
    bsp::PointLookupResult,
    calc::{BulkLoad, Calculator, Options},
    constants::ZERO_TRANSFORM,
    link::{Animation, Bundle, Link, SrcDstIs},
    node::{LabelPosition, Node, NodeOpt},
};
use wasm_bindgen_test::*;

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

fn common_data() -> (Node, Node, Link, Bundle) {
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
    let link = Link::new(0, 0, 1, 0, Animation::None, String::from("This is a test"));
    let bundle = Bundle::new(0, 0, 1, 0, vec![0], String::from("test bundle!"));
    (src, dst, link, bundle)
}
#[wasm_bindgen_test]
#[test]
fn bulk_load_tests() {
    let (src, dst, link, bundle) = common_data();
    let bulk = BulkLoad::nlb(
        vec![src.clone(), dst.clone()],
        vec![link.clone()],
        vec![bundle.clone()],
    );
    let mut calc = Calculator::new();
    let t = Transform {
        x: 0.0,
        y: 0.0,
        k: 1.0,
    };
    calc.bulk_load(bulk);
    assert_eq!(
        calc.in_point(&Point { x: 1.0, y: 1.0 }, &t),
        Some(PointLookupResult::Node(src.clone()))
    );
    assert_eq!(
        calc.in_point(&Point { x: 9.0, y: 9.0 }, &t),
        Some(PointLookupResult::Node(dst.clone()))
    );

    assert!(calc.in_point(&Point { x: 1.0, y: 9.0 }, &t).is_none());
    assert_eq!(
        calc.in_point(&Point { x: 5.0, y: 5.0 }, &t),
        Some(PointLookupResult::Bundle(bundle.clone()))
    );
    assert_eq!(
        calc.in_point(&Point { x: 2.5, y: 2.5 }, &t),
        Some(PointLookupResult::Link(link.clone()))
    );
    assert_eq!(
        calc.in_point(&Point { x: 7.5, y: 7.5 }, &t),
        Some(PointLookupResult::Link(link.clone()))
    );

    {
        let mut iter = calc.on_screen(10, 10, &t);
        assert_eq!(
            iter.next(),
            Some((
                vec![0, 1],
                vec![link.clone().get_container_id(),],
                ScreenBox {
                    width: 768,
                    height: 768,
                    x: 0,
                    y: 0,
                    step: 768
                }
            ))
        );
        assert_eq!(iter.next(), None);
    }
    calc.links_mut().bulk = true;
    calc.link_remove(link.id);
    calc.bundle_remove(bundle.id);
    assert!(calc.links().get(&link.link_id()).unwrap().is_empty());
    calc.finish_bulk_load();
    assert!(calc.links().get(&link.link_id()).is_none());
    {
        let mut iter = calc.on_screen(10, 10, &t);
        assert_eq!(
            iter.next(),
            Some((
                vec![0, 1],
                vec![],
                ScreenBox {
                    width: 768,
                    height: 768,
                    x: 0,
                    y: 0,
                    step: 768
                }
            ))
        );
        assert_eq!(iter.next(), None);
    }
}

#[wasm_bindgen_test]
#[test]
fn wanted_srceen_tests() {
    let mut calc = Calculator::new();
    let ds = calc.default_screen_block();
    assert!(calc.current_screen().is_none());
    assert_eq!(
        calc.wanted_screens(32, 32, &ZERO_TRANSFORM),
        vec![ds.clone()]
    );
    let (src, _, _, _) = common_data();
    calc.node_add(src.clone());

    let step = calc.indexer().step;
    assert_eq!(calc.wanted_screens(32, 32, &ZERO_TRANSFORM), vec![]);

    assert_eq!(
        calc.wanted_screens(
            (calc.indexer().step * 2) as u32,
            (calc.indexer().step * 2) as u32,
            &Transform {
                x: 1.0,
                y: 0.0,
                k: 1.0
            }
        ),
        vec![
            ScreenBox {
                x: step * -1,
                y: 0,
                height: step as u32,
                width: step as u32,
                step
            },
            ScreenBox {
                x: step * -1,
                y: step,
                height: step as u32,
                width: step as u32,
                step
            },
            ScreenBox {
                x: 0,
                y: step,
                height: step as u32,
                width: step as u32,
                step
            },
        ]
    );
    assert_eq!(
        calc.wanted_screens(
            calc.indexer().step as u32,
            calc.indexer().step as u32,
            &Transform {
                x: 1.0,
                y: 0.0,
                k: 0.5
            }
        ),
        vec![
            ScreenBox {
                x: step * -1,
                y: 0,
                height: step as u32,
                width: step as u32,
                step
            },
            ScreenBox {
                x: step * -1,
                y: step,
                height: step as u32,
                width: step as u32,
                step
            },
            ScreenBox {
                x: 0,
                y: step,
                height: step as u32,
                width: step as u32,
                step
            },
        ]
    );
}
