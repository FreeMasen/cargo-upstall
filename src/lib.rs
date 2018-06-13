extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

pub mod installed;
pub mod manifest;

pub enum Error {
    Io(::std::io::Error),
    Toml(toml::de::Error),
    Var(::std::env::VarError)
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result  {
        match self {
            Error::Io(e) => e.fmt(f),
            Error::Toml(e) => e.fmt(f),
            Error::Var(e) => e.fmt(f),
        }
    }
}

impl From<::std::io::Error> for Error {
    fn from(other: ::std::io::Error) -> Self {
        Error::Io(other)
    }
}

impl From<toml::de::Error> for Error {
    fn from(other: toml::de::Error) -> Self {
        Error::Toml(other)
    }
}

impl From<::std::env::VarError> for Error {
    fn from(other: ::std::env::VarError) -> Self {
        Error::Var(other)
    }
}