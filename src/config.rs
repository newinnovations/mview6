use std::{env, fs, io, sync::OnceLock};

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
    let homedir = env::var("HOME").unwrap_or_default();
    let file = fs::File::open(format!("{homedir}/.config/mview/mview6.json"))?;
    let config: Config = serde_json::from_reader(file)?;
    println!("deserialized = {:?}", config);
    Ok(config)
}

pub fn config<'a>() -> &'a Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| read_config().unwrap_or_default())
}
