mod api_impl;
mod backbone_shell;
mod debug_util;
mod helgobox_plugin_editor;
mod tracing_util;
pub use backbone_shell::*;
mod helgobox_plugin;
mod instance_parameter_container;
mod instance_shell;
pub use instance_shell::*;
mod auto_units;
pub use auto_units::*;
mod actions;
mod ini_util;
pub use actions::*;

#[cfg(debug_assertions)]
mod sandbox;
mod shutdown_detection_panel;
mod toolbar;
mod unit_shell;

#[cfg(feature = "playtime")]
mod clip_matrix_handler;

#[cfg(feature = "playtime")]
pub use clip_matrix_handler::*;

pub use instance_parameter_container::*;

#[allow(unused)]
pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

vst::plugin_main!(helgobox_plugin::HelgoboxPlugin);
