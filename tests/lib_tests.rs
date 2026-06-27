#![cfg(test)]

use linked_bundle_node_map::{ScreenBox, Transform};

#[test]
fn test_screenbox_new() {
    assert_eq!(
        ScreenBox::new(
            &Transform {
                x: -2.0,
                y: -2.0,
                k: 2.0,
            },
            4,
            4,
            1,
        ),
        ScreenBox {
            x: 1,
            y: 1,
            width: 2,
            height: 2,
            step: 1,
        }
    );
    assert_eq!(
        (ScreenBox {
            x: 1,
            y: 1,
            width: 2,
            height: 2,
            step: 1,
        })
        .scale(),
        2.0,
    );

    assert_eq!(
        ScreenBox {
            x: 0,
            y: 0,
            width: 4,
            height: 4,
            step: 1
        }
        .getxy_bounds(),
        (0..=3, 0..=3)
    );
}

#[test]
fn test_screen_contains() {
    assert_eq!(
        (ScreenBox {
            x: 0,
            y: 0,
            width: 4,
            height: 4,
            step: 1,
        })
        .contains(&ScreenBox {
            width: 4,
            height: 4,
            x: -2,
            y: -2,
            step: 1,
        }),
        Some(ScreenBox {
            width: 2,
            height: 2,
            x: 0,
            y: 0,
            step: 1,
        })
    );
    assert_eq!(
        (ScreenBox {
            x: 0,
            y: 0,
            width: 4,
            height: 4,
            step: 1,
        })
        .contains(&ScreenBox {
            width: 4,
            height: 4,
            x: 2,
            y: 2,
            step: 1,
        }),
        Some(ScreenBox {
            width: 2,
            height: 2,
            x: 2,
            y: 2,
            step: 1,
        })
    );

    assert_eq!(
        (ScreenBox {
            x: 0,
            y: 0,
            width: 4,
            height: 4,
            step: 1,
        })
        .contains(&ScreenBox {
            width: 4,
            height: 4,
            x: 4,
            y: 4,
            step: 1,
        }),
        None
    );

    assert_eq!(
        (ScreenBox {
            x: 0,
            y: 0,
            width: 4,
            height: 4,
            step: 1,
        })
        .contains(&ScreenBox {
            width: 4,
            height: 4,
            x: -4,
            y: -4,
            step: 1,
        }),
        None
    );
    // should not contain y here
    assert_eq!(
        (ScreenBox {
            x: 0,
            y: 0,
            width: 4,
            height: 4,
            step: 1,
        })
        .contains(&ScreenBox {
            width: 4,
            height: 4,
            x: -3,
            y: -4,
            step: 1,
        }),
        None
    );
}
