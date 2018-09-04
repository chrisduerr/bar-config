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
