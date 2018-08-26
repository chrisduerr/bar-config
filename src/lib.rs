#[macro_use]
extern crate serde_derive;
extern crate serde;

#[cfg(all(feature = "json-fmt", not(feature = "toml-fmt")))]
extern crate serde_json;
#[cfg(not(any(feature = "toml-fmt", feature = "json-fmt")))]
extern crate serde_yaml;
#[cfg(all(feature = "toml-fmt", not(feature = "json-fmt")))]
extern crate toml;

mod config;

pub use config::Bar;

#[test]
fn minimal_config() {
    let height = 30;
    let monitor_name = "DVI-1";
    let input_file = format!(
        "height: {}\n\
         monitors:\n\
         - {{ name: \"{}\" }}\n",
        height, monitor_name
    );

    let config: Bar = serde_yaml::from_str(&input_file).unwrap();

    assert_eq!(config.height, height);
    assert_eq!(config.monitors.len(), 1);
    assert_eq!(config.monitors[0].name, monitor_name);
    assert!(config.monitors[0].fallback_names.is_empty());
}

#[test]
fn full_config() {
    use std::fs::File;
    use std::io::Read;
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config.yml");

    let mut input_text = String::new();
    let mut file = File::open(path).unwrap();
    file.read_to_string(&mut input_text).unwrap();

    let config: Bar = serde_yaml::from_str(&input_text).unwrap();
    let output_text = serde_yaml::to_string(&config).unwrap();

    assert_eq!(input_text, output_text + "\n");
}
