use std::io::Cursor;

use bar_config::Bar;

#[test]
fn load_config() {
    let input = Cursor::new(String::from(
        "height: 30\nmonitors:\n  - { name: \"DVI-1\" }",
    ));

    let bar = Bar::load(input).unwrap();
    let config = bar.lock();

    assert_eq!(config.height, 30);
    assert_eq!(config.monitors.len(), 1);
    assert_eq!(config.monitors[0].name, "DVI-1");
}

#[test]
fn undynamic_component() {
    let input = Cursor::new(String::from(
        "\
         height: 30\n\
         monitors:\n\
         - { name: \"DVI-1\" }\n\
         left:\n\
         - { text: \"Hello, World!\", settings: { width: 99 } }",
    ));

    let bar = Bar::load(input).unwrap();
    let config = bar.lock();

    assert_eq!(config.left.len(), 1);
    assert_eq!(config.left[0].text(), Some(String::from("Hello, World!")));
    assert_eq!(config.left[0].settings().unwrap().width, Some(99));
}
