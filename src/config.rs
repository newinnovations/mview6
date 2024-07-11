use std::{fs, io, sync::OnceLock};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HomeDir {
    pub name: String,
    pub folder: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub home: Vec<HomeDir>,
}

// static mut my_config : Option<Config> = None;

fn read_config() -> io::Result<Config> {
    let file = fs::File::open("/home/martin/.mview6.json")?;
    let config: Config = serde_json::from_reader(file)?;
    println!("deserialized = {:?}", config);
    Ok(config)
}

pub fn config<'a>() -> &'a Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| read_config().expect("Failed to read config file"))
}
