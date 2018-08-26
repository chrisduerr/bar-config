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
