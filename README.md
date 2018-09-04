# Bar Config

Crate for easily creating system bars/panels/docks.

The goal of this crate is to make it as simple as possible to create complex bars/panels/docks
for linux without having to worry about anything but rendering.

To get started with the crate, a new bar needs to be created. This is done using the `load`
method in the `Bar`.

# Examples

This example shows you how to create a simple bar with three components which prints the state
of every component in the console every time it's updated.

```rust
use std::io::Cursor;

use bar_config::Bar;

fn main() {
    // Create input configuration input with three components
    let input = Cursor::new(String::from(
        "\
         height: 30\n\
         monitors:\n\
         - { name: \"DVI-1\" }\n\
         left:\n\
         - { text: \"Hello, World!\" }\n\
         center:\n\
         - { name: \"clock\" }\n\
         right:\n\
         - { text: \"VOLUME\" }",
    ));

    // Load the bar configuration from the input
    let mut bar = Bar::load(input).unwrap();

    // Render the state of the bar at startup
    print_bar(&bar);

    loop {
        // Wait for any update to the bar
        let _ = bar.recv();

        // Print all components every time one changes
        print_bar(&bar);
    }
}

// Prints the text of every component in the configuration
fn print_bar(bar: &Bar) {
    let config = bar.lock();
    for comp in config
        .left
        .iter()
        .chain(&config.center)
        .chain(&config.right)
    {
        if let Some(text) = comp.text() {
            print!("{}\t", text);
        }
    }
    println!("");
}
```

## Bar Configuration Grammar

This is the grammar for the user configuration. It is designed to map to data formats
like YAML or JSON but should also allow an easy representation in Rust.

```bash
# Legend
#     !   Field is required
#     ?   Field is optional

# Root element of the bar
Bar
    # General configuration options
    !height: u8
    ?position: Position
    ?background: Background
    !monitors: [Monitor]

    # Default fallback values for components
    ?defaults: ComponentSettings

    # Component containers
    ?left: [Component]
    ?center: [Component]
    ?right: [Component]

# A single component/block/module in the bar
Component
    # Name used to identify which component should be loaded
    ?name: String

    # Text which will be displayed inside the component
    ?text: String

    # Options available for every component
    ?settings: ComponentSettings

    # Extra options are passed to the component
    ?_: T

# Default options available for every component
ComponentSettings
    ?foreground: (r: u8, g: u8, b: u8, a: u8)
    ?background: Background
    ?width: u8
    ?padding: u8
    ?offset_x: i8
    ?offset_y: i8
    ?fonts: [Font]
    ?border: Border

# Background of a component or the bar
Background
    !Image(path: String) | Color(r: u8, g: u8, b: u8, a: u8)

# Dinstinct identification for a font
Font
    !description: String
    !size: u8

# Distinct identification for a monitor
Monitor
    !name: String
    ?fallback_names: [String]

# Border separating the bar from the rest of the WM
Border
    !height: u8
    !color: (r: u8, g: u8, b: u8, a: u8)

# Available positions for the bar
Position
    !Top | Bottom
```
