#![cfg(test)]

use linked_bundle_node_map::link::LinkContainer;

#[test]
fn link_container_ids() {
    let mut lc = LinkContainer::new(0xffffffff, 0xffffffff);

    assert_eq!(lc.id, 0xffffffffffffffff);
    assert_eq!(lc.get_node_ids(), (0xffffffff, 0xffffffff));
    lc = LinkContainer::new(2, 1);

    assert_eq!(lc.get_node_ids(), (1, 2));
}
