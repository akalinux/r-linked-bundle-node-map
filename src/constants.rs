use std::f64::consts::PI;

use crate::{Point, Transform};

/// Used to slightly oversize link boxes for mouse over intersections preventing misses.
pub const TRIANGLE_MARGINE_FOR_ERROR: f64 = 1.00004;

/// Half a circle
pub const RAD2DEG: f64 = 180.0 / PI;
/// Full circle in radians.
pub const FULL_CIRCLE: f64 = 2.0 * PI;

/// Default height and width of a node
pub const DEFAULT_NODE_R: f64 = 12.0;

/// how much to scale a link down to relative to the size of a node.
pub const DEFAULT_LINK_SCALE: f64 = 1.0;

/// Default name for options.
pub const DEFAULT_OPT_NAME: &'static str = "defaults";

/// Default color used for nodesand links.
pub const DEFAULT_COLOR: &'static str = "#00828A";
// Default color used for bundles
pub const DEFAULT_BUNDLE_COLOR: &'static str = "#e18f38";
pub const DEFAULT_HOVER_TIMEOUT: i32 = 300;

// default animation color
pub const DEFAULT_ANIMATION_COLOR: &'static str = "#9319cc";
pub const DEFAULT_HIGHLIGHT_COLOR: &'static str = "#d4d4d468";
pub const DEFAULT_DIV_STYLE: &'static str =
    "position: relative; height: 100%;widht: 100%;box-sizing: border-box;overflow: clip;";

pub const DEFAULT_CANVAS_STYLE: &'static str =
    "position: absolute;box-sizing: border-box;overflow: clip;";
pub const DEFAULT_ANIMATION_DASHES: [f64; 2] = [5.0, 15.0];
pub const DEFAULT_ANIMATION_WIDTH_SCALE: f64 = 1.0 / 3.0;

pub const ZERO_POINT: Point = Point { x: 0.0, y: 0.0 };

pub const SCREEN_EPSILON: f64 = 0.001;

pub const ZERO_TRANSFORM: Transform = Transform {
    x: 0.0,
    y: 0.0,
    k: 1.0,
};

pub const DEFAULT_SCREEN_ZOOM: f64 = 0.05;
pub const DEFAULT_HIGHLIGHT_SCALE: f64 = 1.10;
pub const DEFAULT_HIGHLIGHT_ALPHA: f64 = 0.5;
pub const DEFAULT_FONT_FAMILY: &'static str = "10px Arial";
pub const DEFAULT_TEXT_ALIGN: &'static str = "center";
