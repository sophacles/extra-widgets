//! Additional widgets for [tui-rs](https://crates.io/crates/tui).
//!
//! These widgets are designed to operate similar to the built-in widgets in tui-rs, and should fit
//! into your app cleanly.
//!
//! TODO: Add example gifs
//!
//! ## Using the widgets
//! By default all the widgets are built and available:
//! ```toml
//! [dependencies]
//! extra-widgets = "0.0.1"
//! tui = "0.18"
//! ```
//! Alternately, each widget can be enabled individually. The feature names are the same as the
//! module name for the widget. To just use the calendar widget:
//! ```toml
//! [dependencies]
//! extra-widgets = {"0.0.1" default-features = false, features = ["calendar"] }
//! tui = "0.18"
//! ```
//! Dependencies for each widget are only pulled if the feature is enabled.
//!
#[cfg(feature = "calendar")]
pub mod calendar;
#[cfg(feature = "separated_list")]
pub mod separated_list;
