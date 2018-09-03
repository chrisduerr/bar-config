use std::io::Cursor;

use bar_config::Bar;

fn main() {
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

    let mut bar = Bar::load(input).unwrap();

    print_bar(&bar);
    loop {
        let _ = bar.recv();
        print_bar(&bar);
    }
}

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
