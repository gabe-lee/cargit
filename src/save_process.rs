use std::{fs, process::Command};

use gmec::types::error_chain::ErrorChain;
use gmec::types::error_chain::ErrorPropogation;

use crate::VersionPart;
use crate::utils::CARGO_MANIFEST;
use crate::utils::split_parts_from_version;
use crate::utils::split_version_from_cargo_toml;

const SAVE_MAJOR :&str = "major";
const SAVE_MINOR :&str = "minor";
const SAVE_PATCH :&str = "patch";

const SAVE_WITH_MSG: &str = "-m";
const SAVE_PUBLISH: &str = "-publish";

const DEFAULT_MSG: &str = "(undocumented change)";

pub(crate) fn save_process<I>(mut args_iter: I) -> Result<String, ErrorChain>
where I: Iterator<Item = String> {
    let mut update_part: VersionPart = VersionPart::Patch;
    let mut update_part_set = false;
    let mut update_message = DEFAULT_MSG.to_string();
    let mut update_message_set = false;
    let mut publish_after_push: bool = false;
    while let Some(next_arg) = args_iter.next() {
        let next_arg_lower = next_arg.to_lowercase();
        match next_arg_lower.as_str() {
            SAVE_MAJOR | SAVE_MINOR | SAVE_PATCH => {
                if update_part_set {
                    return Err(ErrorChain::new(format!("version part update ('{}' or '{}' or '{}') set more than once", SAVE_MAJOR, SAVE_MINOR, SAVE_PATCH)));
                }
                update_part = match next_arg_lower.as_str() {
                    SAVE_MAJOR => VersionPart::Major,
                    SAVE_MINOR => VersionPart::Minor,
                    SAVE_PATCH => VersionPart::Patch,
                    _ => VersionPart::Patch // Impossible
                };
                update_part_set = true;
            }
            SAVE_WITH_MSG => {
                if update_message_set {
                    return Err(ErrorChain::new("update message argument passed more than once"));
                }
                match args_iter.next() {
                    Some(msg) => {
                        // if msg.len() < 2 || !msg.starts_with("\"") || !msg.ends_with("\"") {
                        //     return Err(ErrorChain::new("the '-m' option must be followed by a quoted message"))
                        // }
                        update_message = msg;
                        update_message_set = true;
                    }
                    None => return Err(ErrorChain::new("the '-m' option must be followed by a quoted message"))
                }
            }
            SAVE_PUBLISH => {
                publish_after_push = true;
            }
            _ => return Err(ErrorChain::new(format!("invalid argument passed to save mode: '{}'", next_arg_lower)))
        }
    }
    fs::metadata(CARGO_MANIFEST).on_error("No Cargo.toml file found! This command must be run from a valid Rust crate root directory")?;
    let cargo_toml_str: String = fs::read_to_string(CARGO_MANIFEST).on_error("Cargo.toml could not be parsed to String")?;
    let (cargo_toml_before_version, cargo_toml_version, cargo_toml_after_version) = split_version_from_cargo_toml(&cargo_toml_str)?;
    let (major_ver, minor_ver, patch_ver) = split_parts_from_version(&cargo_toml_version)?;
    let mut major_num: u32 = major_ver.parse().on_error("Error parsing major version into u32")?;
    let mut minor_num: u32 = minor_ver.parse().on_error("Error parsing minor version into u32")?;
    let mut patch_num: u32 = patch_ver.parse().on_error("Error parsing patch version into u32")?;
    match update_part {
        VersionPart::Major => {
            major_num += 1;
            minor_num = 0;
            patch_num = 0;
        },
        VersionPart::Minor => {
            minor_num += 1;
            patch_num = 0;
        }
        VersionPart::Patch => {
            patch_num += 1;
        }
    }
    let new_version = format!("{}.{}.{}", major_num, minor_num, patch_num);
    let new_cargo_toml_str = format!("{}{}{}", cargo_toml_before_version, new_version, cargo_toml_after_version);
    fs::write(CARGO_MANIFEST, new_cargo_toml_str).on_error("failed to write to Cargo.toml")?;
    Command::new("cargo").arg("generate-lockfile").status().on_error("error running 'cargo generate-lockfile'")?;
    Command::new("git").arg("add").arg(".").status().on_error("error running 'git add .'")?;
    Command::new("git").arg("commit").arg("-m").arg(format!("\"{}\"", update_message)).status().on_error(format!("error running 'git commit -m \"{}\"'", update_message))?;
    Command::new("git").arg("push").status().on_error("error running 'git push'")?;
    Command::new("git").arg("tag").arg(new_version.as_str()).status().on_error(format!("error running 'git tag {}'", new_version))?;
    Command::new("git").arg("push").arg("--tags").status().on_error("error running 'git push --tags'")?;
    if publish_after_push {
        Command::new("cargo").arg("publish").status().on_error("error running 'cargo publish'")?;
        return Ok(format!{"Saved, Commited, and Published!!! New version: {}", new_version})
    }
    return Ok(format!{"Saved and Commited! New version: {}", new_version})
}