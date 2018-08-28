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
however it does not allow modification of an element inside the bar.

To allow modification, the bar configuration polls the components for updates.
These can then modify the bar configuration directly. Once the config is modified,
the bar is notified that the configuration is dirty and can redraw the components.

## Bar->Config communication

The first interaction between bar and configuration is always the bar initializing
the configuration. Then the configuration can setup everything necessary to supply
the bar with updates.

If an event like mouse motion is received by the bar, it is propagated to the
config. Since components are not aware of their own position, a method needs to be
attached to each event which allows translating the global event to a
component-relative event. This allows both handling global events like mouse button
releases and handling clicks/motion at specific positions.

Handling only events inside the component could look like this:
```rust
impl Component {
    pub fn notify(&mut self, event: Event) {
        if let Some(event) = event.to_relative(...) {
            // Handle only events inside the component
        }
    }
}
```

## Config->Component communication

Besides updating a component based on event notifications, it must also be
possible to update components based on asynchronous callbacks or timed intervals.
This is initiated by the config by polling all components for updates. If no
event is available the configuration goes to sleep until at least one new event
is available. Each event can modify the configuration which will then lead to
the bar getting notified about the required redraw.

To make it possible that timed intervals can increase granularity when required
(for example during user interaction), it should be possible to change the
polling rate dynamically. This can be done by providing a method which allows
returing any interval, which is then polled every time an update is received.

![Architecture](docs/arch.png)
