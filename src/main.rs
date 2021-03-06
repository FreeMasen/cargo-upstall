extern crate docopt;
extern crate reqwest;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;
extern crate cargo_upstall;

use std::{
    process::{exit, Command as Cmd},
};

use docopt::Docopt;
use semver::Version;

use cargo_upstall::manifest::*;

const USAGE: &str = r#"
Upstall (update or install) a cargo package

Usage:
    upstall <command> [options]

Options:
    -h, --help           Show this message
    -v, --version        Print the version to the screen
    --max VERSION        A maximum target version
    --features FEATURES  Space-separated list of features to pass to cargo install

"#;

#[derive(Debug, Deserialize)]
struct Args {
    arg_command: String,
    flag_max: Option<Version>,
    flag_features: Vec<String>,
}

fn main() {
    // parse arguments
    let args: Args = Docopt::new(USAGE)
                        .and_then(|d| d.deserialize())
                        .unwrap_or_else(|e| e.exit());
    println!("Checking for currently installed version of {}", &args.arg_command);
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
            println!("Found version {}", cmd.version);
            // If we find a command we want to check it against crates.io using
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
    cmd.arg(args.arg_command);
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
        cmd.arg("--features");
        for feat in args.flag_features {
            cmd.arg(feat);
        }
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
    match cargo_upstall::installed::get_installed_commands() {
        Ok(installed) => Some(installed),
        Err(e) => {
            eprintln!("Error getting installed commands\n{}", e);
            None
        }
    }
}
