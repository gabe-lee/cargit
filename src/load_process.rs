use std::{fs, process::Command};

use gmec::types::error_chain::ErrorChain;
use gmec::types::error_chain::ErrorPropogation;


const LOAD_LAST: &str = "last";
const LOAD_NEXT: &str = "next";
const LOAD_LATEST: &str = "latest";

const LOAD_COMMIT: &str = "-c";


enum LoadMode {
    Last,
    Next,
    Latest,
    Commit(String),
    Tag(String),
}

pub(crate) fn load_process<I>(mut args_iter: I) -> Result<String, ErrorChain>
where I: Iterator<Item = String> {
    let mutliple_load_mode_set_error = format!("load mode ('{}' or '{}' or '{}' or '{} <hash>' or '<tag>') set more than once", LOAD_LAST, LOAD_NEXT, LOAD_LATEST, LOAD_COMMIT);
    let mut load_mode: LoadMode = LoadMode::Latest;
    let mut load_mode_set = false;
    while let Some(next_arg) = args_iter.next() {
        let next_arg_lower = next_arg.to_lowercase();
        match next_arg_lower.as_str() {
            LOAD_LAST | LOAD_NEXT | LOAD_LATEST => {
                if load_mode_set {
                    return Err(ErrorChain::new(mutliple_load_mode_set_error));
                }
                load_mode = match next_arg_lower.as_str() {
                    LOAD_LAST => LoadMode::Last,
                    LOAD_NEXT => LoadMode::Next,
                    LOAD_LATEST => LoadMode::Latest,
                    _ => LoadMode::Latest // Impossible
                };
                load_mode_set = true;
            }
            LOAD_COMMIT => {
                if load_mode_set {
                    return Err(ErrorChain::new(mutliple_load_mode_set_error));
                }
                match args_iter.next() {
                    Some(msg) => {
                        load_mode = LoadMode::Commit(msg);
                        load_mode_set = true;
                    }
                    None => return Err(ErrorChain::new("the '-c' option must be followed by a commit hash"))
                }
            }
            tag => {
                if load_mode_set {
                    return Err(ErrorChain::new(mutliple_load_mode_set_error));
                }
                load_mode = LoadMode::Tag(tag.to_owned());
                load_mode_set = true;
            }
        }
    }
    match load_mode {
        LoadMode::Last => todo!(),
        LoadMode::Next => todo!(),
        LoadMode::Latest => todo!(),
        LoadMode::Commit(commit) => todo!(),
        LoadMode::Tag(tag) => todo!(),
    }
    // fs::metadata(CARGO_MANIFEST).on_error("No Cargo.toml file found! This command must be run from a valid Rust crate root directory")?;
    // let cargo_toml_str: String = fs::read_to_string(CARGO_MANIFEST).on_error("Cargo.toml could not be parsed to String")?;
    // let (cargo_toml_before_version, cargo_toml_version, cargo_toml_after_version) = split_version_from_cargo_toml(&cargo_toml_str)?;
    // let (major_ver, minor_ver, patch_ver) = split_parts_from_version(&cargo_toml_version)?;
    // let mut major_num: u32 = major_ver.parse().on_error("Error parsing major version into u32")?;
    // let mut minor_num: u32 = minor_ver.parse().on_error("Error parsing minor version into u32")?;
    // let mut patch_num: u32 = patch_ver.parse().on_error("Error parsing patch version into u32")?;
    // match update_part {
    //     VersionPart::Major => {
    //         major_num += 1;
    //         minor_num = 0;
    //         patch_num = 0;
    //     },
    //     VersionPart::Minor => {
    //         minor_num += 1;
    //         patch_num = 0;
    //     }
    //     VersionPart::Patch => {
    //         patch_num += 1;
    //     }
    // }
    // let new_version = format!("{}.{}.{}", major_num, minor_num, patch_num);
    // let new_cargo_toml_str = format!("{}{}{}", cargo_toml_before_version, new_version, cargo_toml_after_version);
    // fs::write(CARGO_MANIFEST, new_cargo_toml_str).on_error("failed to write to Cargo.toml")?;
    // Command::new("cargo").arg("generate-lockfile").status().on_error("error running 'cargo generate-lockfile'")?;
    // Command::new("git").arg("add").arg(".").status().on_error("error running 'git add .'")?;
    // Command::new("git").arg("commit").arg("-m").arg(format!("\"{}\"", update_message)).status().on_error(format!("error running 'git commit -m \"{}\"'", update_message))?;
    // Command::new("git").arg("push").status().on_error("error running 'git push'")?;
    // Command::new("git").arg("tag").arg(new_version.as_str()).status().on_error(format!("error running 'git tag {}'", new_version))?;
    // Command::new("git").arg("push").arg("--tags").status().on_error("error running 'git push --tags'")?;
    // if publish_after_push {
    //     Command::new("cargo").arg("publish").status().on_error("error running 'cargo publish'")?;
    //     return Ok(format!{"Saved, Commited, and Published!!! New version: {}", new_version})
    // }
    // return Ok(format!{"Saved and Commited! New version: {}", new_version})
    return Ok(String::new())
}