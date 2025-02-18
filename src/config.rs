use std::{process::Output, sync::Mutex};

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
            .copied()
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Connection {
    Ble(BLEConnection),
}

impl Connection {
    pub fn get_ble(&self) -> Option<&BLEConnection> {
        match self {
            Connection::Ble(ble) => Some(ble),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct BLEConnection {
    pub name: String,
    pub mac: String,
    pub actions: Option<Vec<Action>>,
}

impl BLEConnection {
    pub fn run_proximity_actions(&self, distance: f32) {
        if let Some(actions) = &self.actions {
            for action in actions {
                match action {
                    Action::Nearby(action) => {
                        if distance > action.threshold {
                            continue;
                        }

                        action.command.run().unwrap();
                    }
                    Action::Away(action) => {
                        if distance < action.threshold {
                            continue;
                        }

                        action.command.run().unwrap();
                    }
                }
            }
        }
    }
}

pub fn run(cmd: &str) -> anyhow::Result<Output> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()?;
    Ok(output)
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Nearby(ProximityAction),
    Away(ProximityAction),
}

#[derive(Deserialize, Debug, Clone)]
pub struct ProximityAction {
    #[serde(default)]
    pub threshold: f32,
    pub command: Command,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Command {
    Unlock,
    Lock,
    String(String),
}

// fixme: don't use sudo, use proper permissions
// fixme: use dbus to lock/unlock
static LOCKED: Mutex<bool> = Mutex::new(false);
impl Command {
    pub fn run(&self) -> anyhow::Result<()> {
        let locked = *LOCKED.lock().unwrap();
        match self {
            Command::Unlock => {
                if locked {
                    println!("Unlocking desktop...");
                    run("sudo loginctl unlock-sessions")?;
                    *LOCKED.lock().unwrap() = false;
                };
            }

            // fixme: unlocking might not be good idea if it wasn't locked automatically
            Command::Lock => {
                if !locked {
                    println!("Locking desktop...");
                    run("sudo loginctl lock-sessions")?;
                    *LOCKED.lock().unwrap() = true;
                }
            }
            Command::String(cmd) => {
                run(cmd)?;
            }
        };
        Ok(())
    }
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
