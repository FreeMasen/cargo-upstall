use std::{
    env::{current_dir, var},
    fs::read,
    path::PathBuf,
};

use toml::from_slice;

use manifest::{Command, Installed};
use super::Error;

pub fn get_installed_commands() -> Result<Vec<Command>, Error> {
    let global = try_read_global_commands()?;
    let local = try_read_local_commands()?;
    Ok(
        global.commands().into_iter().chain(local.commands().into_iter()).collect()
    )
}

fn try_read_global_commands() -> Result<Installed, Error> {
    let cargo_home = PathBuf::from(var("CARGO_HOME")?);
    let global_path = cargo_home.join(".crates.toml");
    try_parse_file(&global_path)
}

fn try_read_local_commands() -> Result<Installed, Error> {
    let cwd = current_dir()?;
    let local_path = cwd.join(".crates.toml");
    try_parse_file(&local_path)
}

fn try_parse_file(path: &PathBuf) -> Result<Installed, Error> {
    if !path.exists() {
        return Ok(Installed::empty())
    }
    let bytes = read(&path)?;
    Ok(from_slice::<Installed>(&bytes)?)
}