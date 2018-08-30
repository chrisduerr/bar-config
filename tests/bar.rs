use bar_config::config::Bar;
use tempfile::NamedTempFile;

use std::io::Write;

#[test]
fn load_config() {
    let mut tmpfile = NamedTempFile::new().unwrap();
    write!(tmpfile, "height: 30\nmonitors:\n  - {{ name: \"DVI-1\" }}").unwrap();

    let bar = Bar::load(tmpfile.path()).unwrap();

    assert_eq!(bar.height, 30);
    assert_eq!(bar.monitors.len(), 1);
    assert_eq!(bar.monitors[0].name, "DVI-1");
}

#[test]
fn undynamic_component() {
    let mut tmpfile = NamedTempFile::new().unwrap();
    write!(tmpfile, "\
           height: 30\n\
           monitors:\n\
             - {{ name: \"DVI-1\" }}\n\
           left:\n\
             - {{ text: \"Hello, World!\", settings: {{ width: 99 }} }}"
    ).unwrap();

    let bar = Bar::load(tmpfile.path()).unwrap();

    assert_eq!(bar.left.len(), 1);
    assert_eq!(bar.left[0].text(), Some(&String::from("Hello, World!")));
    assert_eq!(bar.left[0].settings().unwrap().width, Some(99));
}
