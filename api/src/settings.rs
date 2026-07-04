use config::{Config, ConfigError};
use serde_derive::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct VehicleImporterSettings {
    pub cookie: String,
    pub cache_dir: String,
    pub vehicle_image_path: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct ApiSettings {
    pub bind_address: String,
    pub host: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Vehicle {
    pub vehicle: String,
    pub year: i32,
    pub name: String,
    #[serde(default = "default_true")]
    pub pcss_import: bool,
}

fn default_true() -> bool { true }

fn default_database_url() -> String {
    "sqlite://content.sqlite3".to_string()
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Settings {
    #[serde(default)]
    pub debug: bool,
    pub importer: VehicleImporterSettings,
    #[serde(default = "default_database_url")]
    pub database_url: String,
    pub vehicle: Vec<Vehicle>,
    pub api: ApiSettings,
}

impl Settings {
    pub fn new() -> anyhow::Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(config::File::with_name("config"))
            .add_source(config::Environment::with_prefix("LIBRETTO"))
            .build()?;
        s.try_deserialize()
    }
}