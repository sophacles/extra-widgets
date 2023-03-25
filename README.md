# extra-widgets

Additional widgets for [tui-rs](https://crates.io/crates/tui).

These widgets are designed to operate similar to the built-in widgets in tui-rs, and should fit
into your app cleanly.

TODO: Add example gifs

This isn't yet a published crate, but the widgets are all documented. Docs are best built with:

```
cargo doc --document --private-items
```

Since the internals are documented as well.

## Using the widgets
By default all the widgets are built and available:
```toml
[dependencies]
extra-widgets = "0.0.1"
tui = "0.18"
```
Alternately, each widget can be enabled individually. The feature names are the same as the
module name for the widget. To just use the calendar widget:
```toml
[dependencies]
extra-widgets = {"0.0.1" default-features = false, features = ["calendar"] }
tui = "0.18"
```
Dependencies for each widget are only pulled if the feature is enabled.

Macros (e.g. `bold!(...)`) are gated by the `text_macros` feature.

### Serde support

State structs can be serialized with Serde by enabling the `serde` feature. This can be useful
(for example) in apps that wish to save state to disk and restore it on the next run.

Serializeable states:
  * styled_list::ListState

### About
These started as functionality I wanted in my own projects, and I thought they
might be generally useful, so this project was born. Long term, I'd like to add
this to some collection for tui/ratatui, or as a seed for such a collection. Or 
barring that, come up with a decent name and publish as a crate.
