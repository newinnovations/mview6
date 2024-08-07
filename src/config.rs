use std::{env, fs::{self, File}, io::{self, BufWriter, Write}, sync::OnceLock};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Bookmark {
    pub name: String,
    pub folder: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub bookmarks: Vec<Bookmark>,
}

impl Config {
    pub fn home() -> String {
        env::var("HOME").unwrap_or_default()
    }

    fn dir() -> String {
        format!("{}/.config/mview", Self::home())
    }

    fn filename() -> String {
        format!("{}/mview6.json", Self::dir())
    }

    pub fn save(&self) -> std::io::Result<()> {
        fs::create_dir_all(Self::dir())?;
        let file = File::create(Self::filename())?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, self)?;
        writer.flush()?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        let config = Self {
            bookmarks: vec![
                Bookmark {
                    name: "Home folder".to_string(),
                    folder: Self::home(),
                },
            ],
        };
        match config.save() {
            Ok(_) => println!("Saved default configuration to {}", Self::filename()),
            Err(_) => println!("Failed to save default configuration to {}", Self::filename()),
        };
        config
    }
}

fn read_config() -> io::Result<Config> {
    let file = fs::File::open(Config::filename())?;
    let config: Config = serde_json::from_reader(file)?;
    println!("deserialized = {:?}", config);
    Ok(config)
}

pub fn config<'a>() -> &'a Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| read_config().unwrap_or_default())
}
