use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub db_url: String,
    pub listen: String,
    pub avatar_path: String,
    pub upload_path: String,
    pub clear_interval: usize,
    pub activate_link: String,
    pub smtp_sender: Option<String>,
    pub smtp_host: Option<String>,
    pub smtp_passwd: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            db_url: String::from("sqlite://userserv.db"),
            listen: String::from("127.0.0.1:3000"),
            avatar_path: String::from("avatar"),
            upload_path: String::from("upload"),
            activate_link: String::from("127.0.0.1:3000"),
            clear_interval: 60,
            smtp_sender: None,
            smtp_host: None,
            smtp_passwd: None,
        }
    }
}

impl Config {
    pub fn load_config() -> Self {
        let cfg_file = std::env::args()
            .nth(1)
            .unwrap_or_else(|| String::from("config.toml"));
        let cfg = if let Ok(content) = std::fs::read_to_string(cfg_file) {
            let cfg = toml::from_str(&content);
            if cfg.is_err() {
                tracing::error!(
                    "config file is not valid, exit program. err: {:?}",
                    cfg.err()
                );
                std::process::exit(-1);
            }
            cfg.unwrap()
        } else {
            tracing::warn!("config file not exits create default one");
            let cfg = Config::default();
            let cfg_str = toml::to_string(&cfg).unwrap();
            std::fs::write("config.toml", cfg_str).unwrap();
            cfg
        };

        check_path(&cfg.avatar_path);
        check_path(&cfg.upload_path);

        cfg
    }
}

fn check_path(path_str: &str) {
    let path = Path::new(path_str);
    if !path.exists() {
        fs::create_dir_all(path).unwrap();
        tracing::info!("create path: {}", path_str);
    } else {
        tracing::info!("{path_str} is ok");
    }
}
