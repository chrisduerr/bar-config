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
    # Component containers
    ?left: [Component]
    ?center: [Component]
    ?right: [Component]
    # General configuration options
    !height: u8
    !monitors: [Monitor]
    ?position: Position
    ?background: Background
    # Default fallback values for components
    ?component_defaults: ComponentSettings

# A single component/block/module in the bar
Component
    ?settings: ComponentSettings
    # Name used to identify which component should be loaded
    ?name: String
    # These extra options are passed to the component
    ?component_options: T

# Default options available for every component
ComponentSettings
    ?color: (r: u8, g: u8, b: u8)
    ?background: Background
    ?fonts: [Font]
    ?width: u8
    ?padding: u8
    ?border: Border
    ?offset_x: i8
    ?offset_y: i8

# Background type enum
Background
    !Image(path: String) | Color(r: u8, g: u8, b: u8)

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
    !color: (r: u8, g: u8, b: u8)

# Bar position enum
Position
    !Top | Bottom
```
