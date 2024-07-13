use std::{fs, io, sync::OnceLock};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Bookmark {
    pub name: String,
    pub folder: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Config {
    pub bookmarks: Vec<Bookmark>,
}

fn read_config() -> io::Result<Config> {
    let file = fs::File::open("/home/martin/.mview6.json")?;
    let config: Config = serde_json::from_reader(file)?;
    println!("deserialized = {:?}", config);
    Ok(config)
}

pub fn config<'a>() -> &'a Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| read_config().unwrap_or_default())
}
