# Cargo Upstall
This is a cargo sub-command that wraps around `cargo install` checking the
currently installed version against [crates.io](https://crates.io) and forcing install
if the currently installed version is out of date.

Currently this will only work with [crates.io](https://crates.io) crates but hopefully
I will be able to support [github](https://github.com/) crates in the near future.

## Why?

When working on [another rust project](https://freemasen.github.io/wasm_tutorial) I wanted
to deploy a Rust web server to Heroku. Unfortunately the build times for my application were getting
close to the time limit and the biggest culprit was forcing install of some rust binary crates.
This sub-command does cost a little since we need to check the registry and the installed versions
but it drastically reduced my build times from commit to commit.

## How?
Cargo keeps an inventory of the installed binaries in a file called `.crates.toml`. For global
installs this is located inside of `CARGO_HOME`. Once we have the currently installed version
we can validate that against `https://crates.io/api/v1/crates/:crate_name`, which should provide the
latest published version. If the two don't match we perform `cargo install [name] --force` otherwise
move on.

## To Do
- [] Add GitHub Crate Installs
- [] Add GitLab Crate Installs
- [] Add BitBucket Crate Installs
- [] Apply lessons learned to `cargo install` itself by adding the `--update` flag