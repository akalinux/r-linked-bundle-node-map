use std::f64::consts::PI;

/// Used to slightly oversize link boxes for mouse over intersections preventing misses.
pub const TRIANGLE_MARGINE_FOR_ERROR: f64 = 1.00004;

/// Half a circle
pub const RAD2DEG: f64 = 180.0 / PI;
/// Full circle in radians.
pub const FULL_CIRCLE: f64 = 2.0 * PI;

/// Default height and width of a node
pub const DEFAULT_NODE_R: f64 = 12.0;

/// how much to scale a link down to relative to the size of a node.
pub const DEFAULT_LINK_SCALE: f64 = 0.96;

/// Default name for options.
pub const DEFAULT_OPT_NAME: &'static str = "defaults";

/// Default color used for nodesand links.
pub const DEFAULT_COLOR: &'static str = "#00828A";
// Default color used for bundles
pub const DEFAULT_BUNDLE_COLOR: &'static str = "#e18f38";

// default animation color
pub const DEFAULT_ANIMATION: &'static str = "#9319cc";
pub const DEFAULT_HIGHLIGHT: &'static str = "#d4d4d468";

pub const DEFAULT_ANIMATION_DASHES: [f64; 2] = [5.0, 15.0];
