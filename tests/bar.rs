use std::io::Cursor;

use bar_config;
use bar_config::bar::Bar;
use image::{self, GenericImage};
use time;

#[test]
fn load_config() {
    let input = Cursor::new(String::from(
        "height: 30\nmonitors:\n  - { name: \"DVI-1\" }",
    ));

    let bar = Bar::load(input).unwrap();

    assert_eq!(bar.general().height, 30);
    assert_eq!(bar.general().monitors.len(), 1);
    assert_eq!(bar.general().monitors[0].name, "DVI-1");
}

#[test]
fn undynamic_component() {
    let input = Cursor::new(String::from(
        "\
         height: 30\n\
         monitors:\n\
         - { name: \"DVI-1\" }\n\
         left:\n\
         - { text: \"Hello, World!\", width: 99 }",
    ));

    let bar = Bar::load(input).unwrap();

    assert_eq!(bar.left().len(), 1);
    assert_eq!(bar.left()[0].text(), String::from("Hello, World!"));
    assert_eq!(bar.left()[0].settings().width, Some(99));
}

#[test]
fn clock_component() {
    let input = Cursor::new(String::from(
        "\
         height: 30\n\
         monitors:\n\
         - { name: \"DVI-1\" }\n\
         left:\n\
         - { name: \"clock\", interval: 10 }",
    ));

    let mut bar = Bar::load(input).unwrap();
    let _ = bar.recv();

    let time = time::now();
    let time = time.strftime("%H:%M").unwrap();
    assert_eq!(bar.left()[0].text(), format!("{}", time));
}

#[test]
fn component_fallbacks() {
    let input = Cursor::new(String::from(
        "\
         height: 30\n\
         monitors:\n\
         - { name: \"DVI-1\" }\n\
         defaults: { width: 100, fonts: [{ name: \"font\", size: 3 }] }\n\
         left:\n\
         - { fonts: [{ name: \"primary\", size: 9 }] }",
    ));

    let bar = Bar::load(input).unwrap();

    assert_eq!(bar.left()[0].settings().width, Some(100));
    assert_eq!(bar.left()[0].settings().fonts.len(), 2);
    assert_eq!(bar.left()[0].settings().fonts[0].name, "primary");
    assert_eq!(bar.left()[0].settings().fonts[0].size, 9);
    assert_eq!(bar.left()[0].settings().fonts[1].name, "font");
    assert_eq!(bar.left()[0].settings().fonts[1].size, 3);
}

#[test]
fn load_image() {
    let input = Cursor::new(String::from(
        "\
         height: 30\n\
         monitors:\n\
         - { name: \"DVI-1\" }\n\
         background: \"./tests/test.png\"",
    ));

    let bar = Bar::load(input).unwrap();
    let img = &bar.general().background;

    if let bar_config::Background::Image(img) = img {
        let pixel = img.get_pixel(0, 0);
        let expected = image::Rgba {
            data: [27, 27, 27, 255],
        };
        assert_eq!(pixel, expected);
    } else {
        panic!("expected image but got color");
    }
}
