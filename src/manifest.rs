use std::{
    collections::HashMap,
};

use semver::Version;

#[derive(Debug, Deserialize)]
/// The .crates.toml schema
pub struct Installed {
    /// This is a toml map of the installed commands
    ///
    /// This map will either appear in $CARGO_HOME/.crates.toml
    /// or possible $PWD/.crates.toml.
    /// The key will be a string comprised of the following format
    /// [crate name] [semver] ([registry|git]+[source url])
    /// cargo-upstall 0.1.4 (registry+https://github.com/rust-lang/crates.io-index)
    /// The value will be a list of binary names that have been installed
    v1: HashMap<String, Vec<String>>,
}
#[derive(Debug, Deserialize)]
/// A parsed entry in the Installed.v1 HashMap
pub struct Command {
    pub name: String,
    pub version: Version,
    pub source: Source,
    pub list: Vec<String>
}
#[derive(Debug, Deserialize)]
/// A part of a parsed
/// key in the Installed.v1 HashMap
/// describing the information in
/// the parentheses
pub struct Source {
    /// registry | git
    pub kind: String,
    /// The source url
    pub url: String,
    /// If kind is git, this will be
    /// this will be the commit hash
    pub commit_hash: Option<String>
}

impl Installed {
    /// Create an instance with an
    /// empty hashmap
    pub fn empty() -> Self {
        Installed {
            v1: HashMap::new()
        }
    }
    /// Convert the v1 HashMap into a Vec of parsed
    /// entries
    pub fn commands(&self) -> Vec<Command> {
        self.v1.iter().filter_map(|(k, v)| Command::from_v1_entry(k, v)).collect()
    }
}

impl Command {
    /// Create a command from an entry in the Installed.v1 hash map
    pub fn from_v1_entry(key: &str, value: &Vec<String>) -> Option<Command> {
            //split the description into its parts. This is formatted
            let mut name_parts = key.split(" ");
            //The name is always the first position
            let name = name_parts.next()?.to_string();
            //Try and parse the next position as a semver::Version
            let version = Version::parse(name_parts.next()?).ok()?;
            let source = Source::from(name_parts.next()?)?;
            // if we are on windows it will add .exe to any commands installed
            // to be consistent, we remove that
            let list = value.iter().map(|s| s.replace(".exe", "")).collect();
            Some(Command {
                name,
                version,
                source,
                list,
            })
    }
}

impl Source {
    /// parse the url string from the Installed.v1 HashMap
    pub fn from(url: &str) -> Option<Source> {
        let full_source = url.trim_matches(|c| c == ')' || c == '(');
        //split the source string on the +
        let mut source_parts = full_source.split("+");
        //capture the source type <git|registry>
        let kind = source_parts.next()?.to_string();
        //if the source type is git we want to pull out the
        //hash from the url
        let (url, commit_hash) = if kind == "git" {
            let full_url = source_parts.next()?.to_string();
            //split the url on the #
            let mut url_parts = full_url.split("#").map(|s| s.to_string());
            //re-assign the first part to url
            let url = url_parts.next()?;
            //capture the commit hash
            (url, url_parts.next())
        } else { 
            //there was no commit hash to capture
            (source_parts.next()?.to_string(), None )
        };
        Some(Source {
            kind,
            url,
            commit_hash
        })
    }
}
#[derive(Deserialize, Debug)]
/// A partial representation of the
/// json returned from a request to
/// https://crates.io/api/v1/crates/:crate_name
pub struct CratesIoEntry {
    #[serde(rename  = "crate")]
    pub cargo_crate: CratesIoCrate,
    pub versions: Vec<CratesIoVersion>
}

#[derive(Deserialize, Debug)]
/// A partial representation of a sub-object
/// of the json returned from a request to
/// https://crates.io/api/v1/crates/:crate_name
pub struct CratesIoCrate {
    /// The name as it appears on crates.io
    pub name: String,
}

#[derive(Deserialize, Debug)]
/// A partial representation of a sub-object
/// of the json returned from a request to
/// https://crates.io/api/v1/crates/:crate_name
pub struct CratesIoVersion {
    /// The semver version of this crate from crates.io
    pub num: Version,
}