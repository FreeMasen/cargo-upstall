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

const MYCONFIG: &str = include_str!("../Cargo.toml");
const USAGE: &str = "
Cargo Upstall, safely attempts to upgrade or install a Cargo bin

Usage:
    upstall <command> [options]
    upstall --help | -h
    upstall --version | -v

Options:
    -h, --help           Show this message
    -v, --version        Print the version to the screen
    --max VERSION        A maximum target version
    --git URL            The git repo url if not registered on crates.io
    --features FEATURES  Space-separated list of features to pass to cargo install
";
#[derive(Debug, Deserialize)]
struct Args {
    arg_command: String,
    flag_max: Option<Version>,
    flag_features: Vec<String>,
    flag_git: Option<String>,
    flag_help: bool,
    flag_h: bool,
    flag_version: bool,
    flag_v: bool,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                        .and_then(|d| d.deserialize())
                        .unwrap_or_else(|e| e.exit());
    if args.flag_help || args.flag_h {
        println!("{}", USAGE);
        exit(0);
    }
    if args.flag_version || args.flag_v {
        println!("cargo-upstall {}", get_version_from_toml(MYCONFIG));
        exit(0);
    }
    let action = if let Some(installed) = get_installed_commands() {
        let mut act = Action::Install {force: false, version: args.flag_max.clone() };//if nothing is found we can just install as normal
        for cmd in installed {
            if cmd.name == args.arg_command {
                //If we find something check the installed command against
                //the max arg and crates.io
                act = check_version(&cmd, args.flag_max).expect("Failed to check version");
                break;
            }
        }
        act
    } else {
        println!("Unable to find currently installed commands");
        Action::Nothing
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
                Ok(s) => println!("Process exited with {:?}", s),
                Err(e) => eprintln!("Error waiting on process {:?}", e),
            }
        },
        Err(e) => eprintln!("Error spawning process {:?}", e),
    };
}
#[derive(PartialEq)]
enum Action {
    Install {force: bool, version: Option<Version> },
    Nothing,
}

fn check_version(command: &Command, max_version: Option<Version>) -> Result<Action, String> {
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
    let io_versions = get_crate_versions(&format!("https://crates.io/api/v1/crates/{}", &command.name)).map_err(|e| format!("{:?}", e))?;
    //get the max version that is <= the max_version if defined
    if let Some(target) = get_max_version(&io_versions, max_version) {
        if target > &command.version {
            println!("You have version {} installed, upgrading to {}", &command.version, &target);
            return Ok(Action::Install { force: true, version: Some(target.clone()) })
        }
        println!("You have the most recent version");
        Ok(Action::Nothing)
    } else {
        Err("Unable to find info on crates.io".into())
    }
}

fn get_max_version(versions: &Vec<Version>, max: Option<Version>) -> Option<&Version> {
    if let Some(ref max) = max {
        versions.iter().filter(|v| v <= &max).max()
    } else {
        versions.iter().max()
    }
}

fn get_crate_versions(url: &str) -> Result<Vec<Version>, reqwest::Error> {
    let info: CratesIoEntry = reqwest::get(url)?.json()?;
    Ok(info.versions.iter().map(|e| e.num.clone()).collect())
}

fn get_installed_commands() -> Option<Vec<Command>> {
    let base = get_cargo_path()?;
    let toml_path = base.join(".crates.toml");
    let toml = read_to_string(&toml_path).expect("Unable to read .crates.toml");
    let installed: Installed = from_str(&toml).expect("Invalid toml value for .crates.toml");
    Some(installed.commands())
}

fn get_version_from_toml(toml: &str) -> Version {
    let man: Crate = from_str(toml).expect("Invalid toml value for Cargo.toml");
    man.version()
}

fn get_cargo_path() -> Option<PathBuf> {
    if let Ok(path) = var("CARGOHOME") {
        Some(PathBuf::from(path))
    } else {
        let hd = home_dir()?;
        Some(hd.join(".cargo"))
    }
}
