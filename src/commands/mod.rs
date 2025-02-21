use serde::Deserialize;
use std::process::Output;

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Command {
    Unlock,
    KeepUnlocked,
    Lock,
    String(String),
}

// fixme: use dbus to lock/unlock

pub fn run(cmd: &str) -> anyhow::Result<Output> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()?;
    Ok(output)
}
