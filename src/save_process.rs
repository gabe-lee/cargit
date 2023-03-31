use std::{fs, process::Command};

use gmec::types::error_chain::ErrorChain;
use gmec::types::error_chain::ErrorPropogation;

use crate::internal::*;

const SAVE_MAJOR :&str = "major";
const SAVE_MINOR :&str = "minor";
const SAVE_PATCH :&str = "patch";

const SAVE_WITH_MSG: &str = "-m";
const SAVE_PUBLISH: &str = "-publish";

const DEFAULT_MSG: &str = "(undocumented change)";

struct SaveModeOptions {
    update_part: Option<VersionPart>,
    commit_message: Option<String>,
    new_version: Option<Version>,
    publish_after_push: bool,
}

impl SaveModeOptions {
    fn blank() -> SaveModeOptions {
        return SaveModeOptions { 
            update_part: None, 
            commit_message: None, 
            new_version: None,
            publish_after_push: false, 
        }
    }
}

pub(crate) fn save_process<I>(mut args_iter: I) -> Result<String, ErrorChain>
where I: Iterator<Item = String> {
    let mut options = process_args(&mut args_iter)?;
    process_cargo_changes(&mut options)?;
    if is_detatched_mode()? {
        handle_detatched_state()?;
    }
    cargo_generate_lockfile()?;
    git_stage_all_changes()?;
    git_commit_with_message(&options.commit_message.unwrap_or(DEFAULT_MSG.to_owned()))?;
    let mut final_message = String::from("Saved, Committed");
    if let Some(new_version) = options.new_version {
        git_create_tag(&new_version.to_string())?;
        final_message.push_str(", Tagged");
    }
    if let Some(remote_name) = get_remote_name()? {
        let branch_name = get_branch_name()?;
        git_push(&remote_name, &branch_name)?;
        final_message.push_str(", Pushed");
    }
    if options.publish_after_push {
        cargo_publish()?;
        final_message.push_str(", Published");
    }
    final_message.push('!');
    if let Some(new_version) = options.new_version {
        final_message.push_str(format!(" New version: {}", new_version.to_string()).as_str());
    }
    return Ok(final_message)
}

fn process_args<I>(args_iter: &mut I) -> Result<SaveModeOptions, ErrorChain>
where I: Iterator<Item = String> {
    let mut options = SaveModeOptions::blank();
    while let Some(next_arg) = args_iter.next() {
        let next_arg_lower = next_arg.to_lowercase();
        match next_arg_lower.as_str() {
            SAVE_MAJOR | SAVE_MINOR | SAVE_PATCH => {
                if options.update_part.is_some() {
                    return Err(ErrorChain::new(format!("version part update ('{}' or '{}' or '{}') set more than once", SAVE_MAJOR, SAVE_MINOR, SAVE_PATCH)));
                }
                options.update_part = match next_arg_lower.as_str() {
                    SAVE_MAJOR => Some(VersionPart::Major),
                    SAVE_MINOR => Some(VersionPart::Minor),
                    SAVE_PATCH => Some(VersionPart::Patch),
                    _ => Some(VersionPart::Patch) // Impossible
                };
            }
            SAVE_WITH_MSG => {
                if options.commit_message.is_some() {
                    return Err(ErrorChain::new("update message argument passed more than once"));
                }
                match args_iter.next() {
                    Some(msg) => {
                        options.commit_message = Some(msg);
                    }
                    None => return Err(ErrorChain::new("the '-m' option must be followed by a quoted message"))
                }
            }
            SAVE_PUBLISH => {
                options.publish_after_push = true;
            }
            _ => return Err(ErrorChain::new(format!("invalid argument passed to save mode: '{}'", next_arg_lower)))
        }
    }
    return Ok(options);
}

fn handle_detatched_state() -> Result<(), ErrorChain> {
    let original_branch = last_attatched_head_branch()?;
    let mut buffer = String::new();
    print!(r#"Cannot save while in a detatched head state
(while checked out to a commit that isnt the latest in the branch)
Would you like to create a new branch from these changes now? (y/n): "#);
    read_stdin_line(&mut buffer)?;
    if !cli_affirmative(buffer) {
        buffer = String::new();
        print!(r#"Would you like to discard these changes instead?
(WARNING: this may be irreversible) (y/n): "#);
        read_stdin_line(&mut buffer)?;
        if !cli_affirmative(buffer) {
            return Err(ErrorChain::new("Save aborted due to detatched head"))
        }
        git_checkout(&original_branch)?;
        return Err(ErrorChain::new("Save aborted due to detatched head, returned to original branch while discarding changes"));
    } else {
        buffer = String::new();
        print!(r#"Name for the new branch: "#);
        read_stdin_line(&mut buffer)?;
        let branch_name = buffer.trim().to_owned();
        git_branch(&branch_name)?;
        git_checkout(&branch_name)?;
        print!("Created new branch {} and switched to it to it!", branch_name);
    }
    Ok(())
}

fn process_cargo_changes(options: &mut SaveModeOptions) -> Result<(), ErrorChain> {
    fs::metadata(CARGO_MANIFEST).on_error("No Cargo.toml file found! This command must be run from a valid Rust crate root directory")?;
    let cargo_toml_str: String = fs::read_to_string(CARGO_MANIFEST).on_error("Cargo.toml could not be parsed to String")?;
    let (cargo_toml_before_version, cargo_toml_version, cargo_toml_after_version) = split_version_from_cargo_toml(&cargo_toml_str)?;
    let (major_ver, minor_ver, patch_ver) = split_parts_from_version(&cargo_toml_version)?;
    let mut new_ver = Version{ major: 0, minor: 0, patch: 0 };
    new_ver.major = major_ver.parse().on_error("Error parsing major version into u32")?;
    new_ver.minor = minor_ver.parse().on_error("Error parsing minor version into u32")?;
    new_ver.patch = patch_ver.parse().on_error("Error parsing patch version into u32")?;
    match options.update_part.as_ref().unwrap_or(&VersionPart::Patch) {
        VersionPart::Major => {
            new_ver.major += 1;
            new_ver.minor = 0;
            new_ver.patch = 0;
        },
        VersionPart::Minor => {
            new_ver.minor += 1;
            new_ver.patch = 0;
        }
        VersionPart::Patch => {
            new_ver.patch += 1;
        }
    }
    let new_ver_string = new_ver.to_string();
    options.new_version = Some(new_ver);
    let new_cargo_toml_str = format!("{}{}{}", cargo_toml_before_version, new_ver_string, cargo_toml_after_version);
    fs::write(CARGO_MANIFEST, new_cargo_toml_str).on_error("failed to write to Cargo.toml")?;
    Ok(())
}