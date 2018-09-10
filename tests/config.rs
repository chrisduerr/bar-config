use std::io::Cursor;

use bar_config::bar::Bar;

#[test]
fn parse_colors() {
    let input = Cursor::new(String::from(
        "\
         height: 30\n\
         monitors:\n\
         - { name: \"DVI-1\" }\n\
         left:\n\
         - { foreground: \"#FF00FF99\" }",
    ));

    let bar = Bar::load(input).unwrap();

    let foreground = bar.left()[0].settings().foreground.unwrap();
    assert_eq!(foreground.r, 255);
    assert_eq!(foreground.g, 0);
    assert_eq!(foreground.b, 255);
    assert_eq!(foreground.a, 153);
}

#[test]
fn colors_as_f64() {
    let input = Cursor::new(String::from(
        "\
         height: 30\n\
         monitors:\n\
         - { name: \"DVI-1\" }\n\
         left:\n\
         - { foreground: \"#FF00FF99\" }",
    ));

    let bar = Bar::load(input).unwrap();

    let foreground = bar.left()[0].settings().foreground.unwrap().as_f64();
    assert_eq!(foreground.0, 1.0);
    assert_eq!(foreground.1, 0.0);
    assert_eq!(foreground.2, 1.0);
    assert_eq!(foreground.3, 0.6);
}
