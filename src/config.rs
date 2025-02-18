use std::vec;

use serde::Deserialize;

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub connection: Option<Vec<Connection>>,
}

impl Config {
    pub fn connections(&self) -> Vec<&Connection> {
        self.connection
            .as_ref()
            .map(|connections| connections.iter().collect())
            .unwrap_or_default()
    }

    pub fn get_connection_by_mac(&self, mac: &str) -> Option<&Connection> {
        self.connections()
            .iter()
            .find(|c| c.get_ble().map_or(false, |ble| ble.mac == mac))
            .map(|c| *c)
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Connection {
    BLE(BLEConnection),
}

impl Connection {
    pub fn get_ble(&self) -> Option<&BLEConnection> {
        match self {
            Connection::BLE(ble) => Some(ble),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct BLEConnection {
    pub name: String,
    pub mac: String,
}

const APP_NAME: &str = "nearby";
pub fn get_config() -> anyhow::Result<Config> {
    let config_dir = dirs::config_dir().expect("Could not find config directory");
    let base_dir = config_dir.join(APP_NAME);
    std::fs::create_dir_all(&base_dir)?;
    let config_file = base_dir.join("config.toml");
    let config: Config = Figment::new()
        .merge(Toml::file(config_file))
        .merge(Env::prefixed(&format!("{APP_NAME}_")))
        .extract()?;

    Ok(config)
}
