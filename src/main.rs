use color_eyre::{eyre::eyre, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use std::{fs::File, path::Path};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RunConfig {
    before: Vec<String>,
    after: Vec<String>,
}

impl RunConfig {
    fn before(&self) -> Result<()> {
        let cmd = if let Some(cmd) = self.before.first() {
            cmd
        } else {
            return Ok(());
        };
        let mut command = Command::new(cmd);
        command.args(self.before.iter().skip(1));
        tracing::info!("[before]: {:?}", command);
        command.status()?;
        Ok(())
    }

    fn after(&self) -> Result<()> {
        let cmd = if let Some(cmd) = self.after.first() {
            cmd
        } else {
            return Ok(());
        };
        let mut command = Command::new(cmd);
        command.args(self.after.iter().skip(1));
        tracing::info!("[after]: {:?}", command);
        command.status()?;
        Ok(())
    }
}

fn config<P: AsRef<Path>>(name: P) -> Result<RunConfig> {
    let mut name = PathBuf::from(name.as_ref());
    name.set_extension("toml");
    let config_path = dirs::config_dir()
        .ok_or_else(|| eyre!("No XDG config path find"))?
        .join("unlocker")
        .join(name);
    let mut config_file = File::open(config_path)?;
    let mut buffer = String::new();
    config_file.read_to_string(&mut buffer)?;
    Ok(toml::from_str(&buffer)?)
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let targets: Vec<String> = env::args().skip(1).collect();
    for target in targets {
        let config = config(target)?;
        config.before()?;
        Command::new(&shell).status()?;
        config.after()?;
    }
    Ok(())
}
