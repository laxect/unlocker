use color_eyre::{eyre::eyre, Result};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::File,
    io::Read,
    os::unix::prelude::ExitStatusExt,
    path::PathBuf,
    process::{Command, ExitStatus},
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RunConfigSer {
    before: Vec<String>,
    after: Vec<String>,
}

impl RunConfigSer {
    fn fill(self, name: String) -> RunConfig {
        let Self { before, after } = self;
        RunConfig { name, before, after }
    }
}

struct RunConfig {
    name: String,
    before: Vec<String>,
    after: Vec<String>,
}

impl RunConfig {
    fn before(&self) -> Result<ExitStatus> {
        let cmd = if let Some(cmd) = self.before.first() {
            cmd
        } else {
            return Ok(ExitStatus::from_raw(0));
        };
        let mut command = Command::new(cmd);
        command.args(self.before.iter().skip(1));
        tracing::info!("[before]: {:?}", command);
        Ok(command.status()?)
    }

    fn after(&self) -> Result<ExitStatus> {
        let cmd = if let Some(cmd) = self.after.first() {
            cmd
        } else {
            return Ok(ExitStatus::from_raw(0));
        };
        let mut command = Command::new(cmd);
        command.args(self.after.iter().skip(1));
        tracing::info!("[after]: {:?}", command);
        Ok(command.status()?)
    }
}

fn config(name: String) -> Result<RunConfig> {
    let mut path = PathBuf::from(&name);
    path.set_extension("toml");
    let config_path = dirs::config_dir()
        .ok_or_else(|| eyre!("No XDG config path find"))?
        .join("unlocker")
        .join(path);
    let mut config_file = File::open(config_path)?;
    let mut buffer = String::new();
    config_file.read_to_string(&mut buffer)?;
    let config_precursor: RunConfigSer = toml::from_str(&buffer)?;
    Ok(config_precursor.fill(name))
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let configs: Result<Vec<RunConfig>> = env::args().skip(1).map(|target| config(target)).collect();
    let configs = configs?;
    for config in configs.iter() {
        let status = config.before()?;
        if !status.success() {
            tracing::error!("[before::{}] failed.", config.name);
            return Ok(());
        }
    }
    Command::new(&shell).status()?;
    for config in configs.iter() {
        let status = config.after()?;
        if !status.success() {
            tracing::error!("[after::{}] failed.", config.name);
            return Ok(());
        }
    }
    Ok(())
}
