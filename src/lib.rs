pub mod shortener;
use serde::Deserialize;
pub use shortener::*;

pub mod db;

#[derive(Deserialize)]
pub struct Config {
    pub backup_file_path: String,
    pub custom_url: String,
    pub max_log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            backup_file_path: "./backup.txt".to_string(),
            custom_url: "ht.h".to_string(),
            max_log_level: "debug".to_string(),
        }
    }
}
