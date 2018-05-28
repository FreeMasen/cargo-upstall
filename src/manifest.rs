use std::{
    collections::HashMap,
};

use semver::Version;
#[derive(Debug, Deserialize)]
pub struct Crate {
    package: Package,
}

#[derive(Debug, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: Version,
}

#[derive(Debug, Deserialize)]
pub struct Installed {
    v1: HashMap<String, Vec<String>>,
}
#[derive(Debug, Deserialize)]
pub struct Command {
    pub name: String,
    pub version: Version,
    pub source: Source,
    pub list: Vec<String>
}
#[derive(Debug, Deserialize)]
pub struct Source {
    pub kind: String,
    pub url: String,
    pub commit_hash: Option<String>
}

impl Installed {
    pub fn commands(&self) -> Vec<Command> {
        self.v1.iter().filter_map(|(k, v)| Command::from(k, v)).collect()
    }
}

impl Command {
    pub fn from(desc: &str, list: &Vec<String>) -> Option<Command> {
            //split the description into its parts. This is formatted
            //name 0.0.0 (source_type+url[#hash]) it it will generate a 3 part iterator
            let mut name_parts = desc.split(" ");
            //The name is always the first position
            let name = name_parts.next()?.to_string();
            //Try and parse the next position as a semver::Version
            let version = Version::parse(name_parts.next()?).ok()?;
            let source = Source::from(name_parts.next()?)?;
            let list = list.iter().map(|s| s.replace(".exe", "")).collect();
            Some(Command {
                name,
                version,
                source,
                list,
            })
    }
}

impl Source {
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
pub struct CratesIoEntry {
    #[serde(rename  = "crate")]
    pub cargo_crate: CratesIoCrate,
    pub versions: Vec<CratesIoVersion>
}

#[derive(Deserialize, Debug)]
pub struct CratesIoCrate {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct CratesIoVersion {
    pub num: Version,
}