extern crate docopt;
extern crate reqwest;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

use std::{
    env::{var,home_dir},
    fs::{read_to_string},
    path::{PathBuf},
    process::{exit, Command as Cmd},
};

use docopt::Docopt;
use semver::Version;
use toml::{from_str};

mod manifest;
use manifest::*;

const USAGE: &str = r#"
Upstall (update or install) a cargo package

Usage:
    cargo upstall <command> [options]

Options:
    -h, --help           Show this message
    -v, --version        Print the version to the screen
    --max VERSION        A maximum target version
    --git URL            The git repo url if not registered on crates.io
    --features FEATURES  Space-separated list of features to pass to cargo install
"#;

#[derive(Debug, Deserialize)]
struct Args {
    arg_command: String,
    flag_max: Option<Version>,
    flag_features: Vec<String>,
    flag_git: Option<String>,
}

fn main() {
    println!("{:?}", ::std::env::args());
    // parse argumenst
    let args: Args = Docopt::new(USAGE)
                        .and_then(|d| d.deserialize())
                        .unwrap_or_else(|e| e.exit());

    let action = if let Some(installed) = get_installed_commands() {
        // The default action to take would be
        // not to force and just pass over the
        // max flag
        let mut act = Action::Install {
            force: false,
            version: args.flag_max.clone()
        };
        // Try and find an installed command
        if let Some(cmd) = installed.iter().find(|c| c.name == args.arg_command) {
            // If we find a command we want to check it agains crates.io using
            // the max value as an optional upper limit
            match check_version(&cmd, &args.flag_max) {
                Ok(action) => act = action,
                Err(e) => {
                    eprintln!("Failed to check version {}", e);
                    exit(1)
                }
            }
        }
        act
    } else {
        // If we failed to find
        println!("Unable to find currently installed commands");
        exit(0)
    };
    let mut cmd = Cmd::new("cargo");
    cmd.arg("install");
    match action {
        Action::Install {force, version} => {
            if force {
                cmd.arg("--force");
            }
            if let Some(v) = version {
                cmd.arg(&format!("--version={}", v));
            }
        },
        Action::Nothing => exit(0)
    }
    if args.flag_features.len() > 0 {
        cmd.arg(format!(" --features={}", &args.flag_features.join(" ")));
    }
    if let Some(url) = args.flag_git {
        cmd.arg(format!(" --git={}", &url));
    }
    match cmd.spawn() {
        Ok(mut c) => {
            match c.wait() {
                Ok(_) => (), //If successful, just exit
                Err(e) => eprintln!("Error waiting on cargo install {:?}", e),
            }
        },
        Err(e) => eprintln!("Error spawning cargo install {:?}", e),
    };
}
/// The action to take based on the
/// version installed
#[derive(PartialEq)]
enum Action {
    /// Install a new version if force == true then use
    /// the --force flag if a version is provided
    /// then pass that to cargo install --version
    Install {force: bool, version: Option<Version> },
    /// Do not install anything
    Nothing,
}
/// Check the version of an installed command against
/// crates.io
fn check_version(command: &Command, max_version: &Option<Version>) -> Result<Action, String> {
    //currently I don't know know I want to
    //deal with the hashes and whatnot so 
    //just force reinstalling.
    if command.source.kind == "git" {
        println!("Git repos are always re-installed with --force");
        return Ok(Action::Install { force: true, version: None})
    }
    if let Some(ref max) = max_version {
        //If the installed command is not less than
        //the max, we can just move on
        //maybe need to deal with the > case?
        if &command.version >= max {
            println!("You currently have a version at least at your max version");
            return Ok(Action::Nothing)
        } else {
            return Ok(Action::Install { force: true, version: max_version.clone() })
        }
    }
    //get the list of versions from crates.io
    let io_versions = get_crate_versions(
        &format!("https://crates.io/api/v1/crates/{}", &command.name))
            .map_err(|e| format!("{:?}", e))?;
    //get the max version that is <= the max_version if defined
    if let Some(target) = get_max_version(&io_versions, max_version) {
        // If the max version is greater than the installed
        // command we need up upgrade
        if target > command.version {
            println!("You have version {} installed, upgrading to {}", &command.version, &target);
            return Ok(Action::Install { force: true, version: Some(target.clone()) })
        }
        println!("You have the most recent version");
        Ok(Action::Nothing)
    } else {
        Err("Unable to find info on crates.io".into())
    }
}
/// Get the max version from a list of versions provided
/// by crates.io with an optional upper limit
fn get_max_version(versions: &Vec<Version>, max: &Option<Version>) -> Option<Version> {
    if let Some(ref max) = max {
        versions.into_iter().filter_map(|v| {
            if v <= &max {
                Some(v.clone())
            } else {
                None
            }
            }).max()
    } else {
        versions.into_iter().max().map(|v| v.clone())
    }
}
/// Get the list of version from crates.io for
/// a single crate
fn get_crate_versions(url: &str) -> Result<Vec<Version>, reqwest::Error> {
    let info: CratesIoEntry = reqwest::get(url)?.json()?;
    Ok(info.versions.iter().map(|e| e.num.clone()).collect())
}
/// Get a list of currently installed commands located
/// in the $CARGOHOME/.crates.toml
fn get_installed_commands() -> Option<Vec<Command>> {
    let base = get_cargo_path()?;
    let toml_path = base.join(".crates.toml");
    // If the crates.toml file doesn't exist
    // we don't want to try an read it just send
    // back an empty []
    if !toml_path.exists() {
        return Some(vec![])
    }
    if let Ok(toml) = read_to_string(&toml_path) {
        if let Ok(installed) = from_str::<Installed>(&toml) {
            Some(installed.commands())
        } else {
            None
        }

    } else {
        None
    }
}

/// Get the cargo path, either from the environment
/// variable `CARGOHOME` or the default `~/.cargo`
fn get_cargo_path() -> Option<PathBuf> {
    if let Ok(path) = var("CARGOHOME") {
        Some(PathBuf::from(path))
    } else {
        let hd = home_dir()?;
        Some(hd.join(".cargo"))
    }
}
