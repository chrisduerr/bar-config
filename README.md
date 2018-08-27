# Bar Specification

This is the architecture specification for a system panel/dock/bar.

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
    ?component_options: T

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

## Config->Bar communication

A configuration file allows creating a static version of any bar,
however it does not allow modification of any element inside the bar.

To allow modification, the bar configuration spawns the processes required
for each component, these then can modify the bar configuration directly.
Once the bar configuration is modified, an event is sent to the bar to
notify it that the configuration file is dirty and needs to be redrawn.
