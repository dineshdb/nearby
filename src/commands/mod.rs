use crate::delayed::Delayed;
use serde::Deserialize;
use std::{process::Output, sync::Mutex};

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
static DELAYED_LOCK: Mutex<Option<Delayed>> = Mutex::new(None);

impl Command {
    pub fn run(&self) -> anyhow::Result<()> {
        let locked = *LOCKED.lock().unwrap();
        match self {
            Command::Unlock => {
                // cancel the delayed lock
                if let Some(delayed) = &mut *DELAYED_LOCK.lock().unwrap() {
                    delayed.cancel();
                }
                if locked {
                    println!("Unlocking desktop...");
                    run("sudo loginctl unlock-sessions")?;
                    *LOCKED.lock().unwrap() = false;
                };
            }

            // fixme: unlocking might not be good idea if it wasn't locked automatically
            Command::Lock => {
                let duration = std::time::Duration::from_secs(15);
                if !locked {
                    println!("Locking desktop in {:?}", duration);
                    if let Some(delayed) = &mut *DELAYED_LOCK.lock().unwrap() {
                        delayed.cancel();
                    }
                    // wait before actually locking the desktop
                    let delayed = Delayed::new(duration, || async {
                        run("sudo loginctl lock-sessions").expect("error running lock command");
                        *LOCKED.lock().unwrap() = true;
                    });
                    *DELAYED_LOCK.lock().unwrap() = Some(delayed);
                }
            }
            Command::String(cmd) => {
                run(cmd)?;
            }
        };
        Ok(())
    }
}

pub fn run(cmd: &str) -> anyhow::Result<Output> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()?;
    Ok(output)
}
