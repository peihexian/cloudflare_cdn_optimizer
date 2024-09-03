// src/config.rs
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use anyhow::Result; 
use once_cell::sync::Lazy;


pub static GLOBAL_CONFIG: Lazy<Config> = Lazy::new(|| {
    Config::load().expect("Failed to load configuration")
});

#[derive(Debug, Deserialize)]
pub struct Config {
    pub cloudflare: CloudflareConfig,
    pub cdn: CdnConfig,
    pub optimization: OptimizationConfig,
}

#[derive(Debug, Deserialize)]
pub struct CloudflareConfig {
    pub api_token: String,
    pub zone_id: String,
    pub record_id: String,
    pub domain: String,
    pub update_dns: bool,
}

#[derive(Debug, Deserialize)]
pub struct CdnConfig {
    pub cidr_list: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct OptimizationConfig {
    pub ping_threads: usize,
    pub top_ips_to_save: usize,
    pub run_interval_seconds: u64,
    pub debug : bool,
}

impl Config {
    pub fn load() -> Result<Self> {
        let mut file = File::open("config.yaml")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}

pub fn init_global_config() {
    Lazy::force(&GLOBAL_CONFIG);
}