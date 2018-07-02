extern crate serde;
extern crate serde_any;
#[macro_use]
extern crate serde_derive;

extern crate failure;

use std::path::PathBuf;

#[derive(Deserialize, Clone, Debug)]
struct Settings {
    name: String,
    path: PathBuf,
    size: usize,
    count: i32,
}

fn main() -> Result<(), failure::Error> {
    // This tries to load a `Settings` structure from "settings.json",
    // "settings.toml", "settings.yaml", "settings.yml", or "settings.ron"
    let settings: Settings = serde_any::from_file_stem("settings")?;

    // If any of these files is present and a valid source for deserialization, the
    // `settings` structure will container the parsed configuration values
    println!("{:?}", settings);

    Ok(())
}
