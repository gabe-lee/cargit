use std::{fs, env, process::Command};

use anyhow::{Context, Result, Error};

use gmec::pattern::{PatternMatcher};
// fn main() {}

const CARGO_MANIFEST: &str = "Cargo.toml";

const SAVE_MODE_ARG: &str = "save";
const HELP_MODE_ARG: &str = "help";

const VERSION_PREFIX: &str = "version = \"";

const HELP_MESSAGE :&str = "
The cargit command is used to automate git and cargo processes that would normally take several independant commands,
while also making relevant changes to the crate's Cargo.toml and Cargo.lock (when relevant)

Usage:
  cargit help                                         # Display this help message
  cargit save [major|minor|patch] [-m \"<message>\"]  # this command performs the following actions, in order:
      (Increment relevent part of the Version in Cargo.toml, and set any lesser version parts to 0)
      cargo generate-lockfile
      git add .
      git commit -m \"<message>\"
      git push
      git tag <Version from Cargo.toml>
      git push --tags
";

enum RunMode {
    HelpErr,
    Help,
    Save
}

enum VersionPart {
    Major,
    Minor,
    Patch
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mode = if args.len() >= 2 { 
       let mode_string = args[1].to_lowercase(); 
       match mode_string.as_str() {
        SAVE_MODE_ARG => RunMode::Save,
        HELP_MODE_ARG => RunMode::Help,
        _ => RunMode::HelpErr
       }
    } else {
        RunMode::HelpErr
    };
    let mode_result = match mode {
        RunMode::HelpErr => Ok(format!("INVALID USAGE!\nvvvvvvvvvvvvvv\n{}", HELP_MESSAGE)),
        RunMode::Help => Ok(format!("{}", HELP_MESSAGE)),
        RunMode::Save => save_process(args)
    };
    match mode_result {
        Ok(message) => println!("{}", message),
        Err(err) => eprintln!("{}", err)
    }
}

fn save_process(args: Vec<String>) -> Result<String> {
    let mut update_part: VersionPart = VersionPart::Patch;
    let mut update_message: String = String::from("(undocumented update)");
    let mut next_arg_is_msg = false;
    if args.len() >= 3 {
        update_part = match args[2].to_lowercase().as_str() {
            "major" => VersionPart::Major,
            "minor" => VersionPart::Minor,
            "patch" => VersionPart::Patch,
            "-m" => {
                next_arg_is_msg = true;
                VersionPart::Patch
            }
            _ => return Err(Error::msg("invalid argument to 'save' mode, see 'help' for more info"))
        };
    }
    if args.len() >= 4 {
        if next_arg_is_msg {
            update_message = args[3].clone();
        } else if args[3].to_lowercase() == String::from("-m") {
            next_arg_is_msg = true
        } else {
            return Err(Error::msg("invalid argument to 'save' mode, see 'help' for more info"))
        }
    }
    if args.len() >= 5 && next_arg_is_msg {
        update_message = args[4].clone();
    }
    update_message = update_message.trim_matches('\"').to_string();
    fs::metadata(CARGO_MANIFEST).context("No Cargo.toml file found! This command must be run from a valid Rust crate root directory")?;
    let cargo_toml_str: String = fs::read_to_string(CARGO_MANIFEST).context("Cargo.toml could not be parsed to String")?;
    let (cargo_toml_before_version, cargo_toml_version, cargo_toml_after_version) = split_version_from_cargo_toml(&cargo_toml_str)?;
    let (major_ver, minor_ver, patch_ver) = split_parts_from_version(&cargo_toml_version)?;
    let mut major_num: u32 = major_ver.parse().context("Error parsing major version into u32")?;
    let mut minor_num: u32 = minor_ver.parse().context("Error parsing minor version into u32")?;
    let mut patch_num: u32 = patch_ver.parse().context("Error parsing patch version into u32")?;
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
    fs::write(CARGO_MANIFEST, new_cargo_toml_str).context("failed to write to Cargo.toml")?;
    Command::new("cargo").arg("generate-lockfile").status().context("error running 'cargo generate-lockfile'")?;
    Command::new("git").arg("add").arg(".").status().context("error running 'git add .'")?;
    Command::new("git").arg("commit").arg("-m").arg(format!("\"{}\"", update_message)).status().context(format!("error running 'git commit -m \"{}\"'", update_message))?;
    Command::new("git").arg("push").status().context("error running 'git push'")?;
    Command::new("git").arg("tag").arg(new_version.as_str()).status().context(format!("error running 'git tag {}'", new_version))?;
    Command::new("git").arg("push").arg("--tags").status().context("error running 'git push --tags'")?;
    return Ok(format!{"Saved and Commited! New version: {}", new_version})
}


fn split_version_from_cargo_toml<'a>(cargo_toml_str: &'a str) -> Result<(&'a str, &'a str, &'a str)> {
    let version_prefix_location = cargo_toml_str.find_first(&VERSION_PREFIX).context("could not locate the string 'version = \"' in the Cargo.toml file")?;
    let version_end_location = cargo_toml_str.find_first_from(&"\"", version_prefix_location.end()).context("could not locate a '\"' after the version numbers in the Cargo.toml file")?;
    let before_version = &cargo_toml_str[..version_prefix_location.end()];
    let version_string = &cargo_toml_str[version_prefix_location.end()..version_end_location.start()];
    let after_version = &cargo_toml_str[version_end_location.start()..];
    return Ok((before_version, version_string, after_version))
}

fn split_parts_from_version<'a>(version_string: &'a str) -> Result<(&'a str, &'a str, &'a str)> {
    let first_version_period = version_string.find_first(&".").context("could not locate a '.' after the major version")?;
    let second_version_period = version_string.find_first_from(&".", first_version_period.end()).context("could not locate a '.' after the minor version")?;
    let major = &version_string[..first_version_period.start()];
    let minor = &version_string[first_version_period.end()..second_version_period.start()];
    let patch = &version_string[second_version_period.end()..];
    return Ok((major, minor, patch));
}