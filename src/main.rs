use std::{fs::File, env, io::{Read, Write, Stdout, stdout}, slice::SliceIndex, fmt::format, process::Command};

use Patterns::{PatternMatcher, PatternMatch};

mod Patterns;

// fn main() {}

const CARGO_MANIFEST: &str = "Cargo.toml";

const SAVE_MODE_ARG: &str = "save";
const HELP_MODE_ARG: &str = "help";
const VERSION_PREFIX: &str = "version = \"";

const HELP_MESSAGE :&str = "
The cargo gitkit command is used to automate git processes that would normally take several independant commands, while also making relevant changes to the crate's Cargo.toml manifest (when relevant)

Usage:
  cargo gitkit help                                     # Display this help message
  cargo gitkit save [major|minor|patch] [-m <message>]  # this command performs the following actions, in order:
      (Increment relevent part of the Version in Cargo.toml, and set any lesser version parts to 0)
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

fn save_process(args: Vec<String>) -> Result<String, String> {
    let mut update_part: VersionPart = VersionPart::Patch;
    let mut update_message: String = String::from("(patch update)");
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
            _ => return Err(String::from("invalid argument to 'save' mode, see 'help' for more info"))
        };
    }
    if args.len() >= 4 {
        if next_arg_is_msg {
            update_message = args[3].clone();
        } else if args[3].to_lowercase() == String::from("-m") {
            next_arg_is_msg = true
        } else {
            return Err(String::from("invalid argument to 'save' mode, see 'help' for more info"))
        }
    }
    if args.len() >= 5 && next_arg_is_msg {
        update_message = args[4].clone();
    }
    let cargo_toml_result = File::options()
        .read(true)
        .write(true)
        .truncate(false)
        .open(CARGO_MANIFEST);
    if cargo_toml_result.is_err() {
        return Err(format!("No Cargo.toml file found! This command must be run from a valid Rust crate root directory\nOriginal error: {}", cargo_toml_result.unwrap_err()));
    }
    let mut cargo_toml = cargo_toml_result.unwrap();
    let mut cargo_toml_str: String = String::new();
    let cargo_toml_read_result = cargo_toml.read_to_string(&mut cargo_toml_str);
    if cargo_toml_read_result.is_err() {
        return Err(format!("Cargo.toml could not be parsed to String\nOriginal error: {}", cargo_toml_read_result.unwrap_err()))
    };
    let (cargo_toml_before_version, cargo_toml_version, cargo_toml_after_version) = split_version_from_cargo_toml(&cargo_toml_str)?;
    let (major_ver, minor_ver, patch_ver) = split_parts_from_version(&cargo_toml_version)?;
    let mut major_num: u32 = match major_ver.parse() {
        Ok(val) => val,
        Err(err) => {
            return Err(format!("Error parsing major version into u32: {}", err))
        }
    };
    let mut minor_num: u32 = match minor_ver.parse() {
        Ok(val) => val,
        Err(err) => {
            return Err(format!("Error parsing minor version into u32: {}", err))
        }
    };
    let mut patch_num: u32 = match patch_ver.parse() {
        Ok(val) => val,
        Err(err) => {
            return Err(format!("Error parsing patch version into u32: {}", err))
        }
    };
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
    cargo_toml.write(new_cargo_toml_str.as_bytes());
    match Command::new("git").arg("add").arg(".").output() {
        Ok(out) => {
            stdout().write_all(&out.stdout);
        },
        Err(err) => {
            return Err(format!("ERROR RUNNING COMMAND: git add .\n{}", err));
        }
    };
    match Command::new("git").arg("commit").arg("-m").arg(format!("\"{}\"", update_message)).output() {
        Ok(out) => {
            stdout().write_all(&out.stdout);
        },
        Err(err) => {
            return Err(format!("ERROR RUNNING COMMAND: git commit -m \"{}\"\n{}", update_message, err));
        }
    };
    match Command::new("git").arg("push").output() {
        Ok(out) => {
            stdout().write_all(&out.stdout);
        },
        Err(err) => {
            return Err(format!("ERROR RUNNING COMMAND: git push -m\n{}", err));
        }
    };
    match Command::new("git").arg("tag").arg(new_version.as_str()).output() {
        Ok(out) => {
            stdout().write_all(&out.stdout);
        },
        Err(err) => {
            return Err(format!("ERROR RUNNING COMMAND: git tag {}\n{}", new_version, err));
        }
    };
    match Command::new("git").arg("push").arg("--tags").output() {
        Ok(out) => {
            stdout().write_all(&out.stdout);
        },
        Err(err) => {
            return Err(format!("ERROR RUNNING COMMAND: git push --tags\n{}", err));
        }
    };
    return Ok(format!{"Saved and Commited! New version: {}", new_version})
}


fn split_version_from_cargo_toml<'a>(cargo_toml_str: &'a str) -> Result<(&'a str, &'a str, &'a str), &str> {
    match cargo_toml_str.find_first(&VERSION_PREFIX) {
        Some(version_prefix_location) => {
            match cargo_toml_str.find_first_from(&"\"", version_prefix_location.end()) {
                Some(version_end_location) => {
                    let before_version = &cargo_toml_str[..version_prefix_location.end()];
                    let version_string = &cargo_toml_str[version_prefix_location.end()..version_end_location.start()];
                    let after_version = &cargo_toml_str[version_end_location.start()..];
                    return Ok((before_version, version_string, after_version))
                },
                None => {
                    return Err("could not locate a \" after the version numbers in the Cargo.toml file")
                }
            }
        },
        None => {
            return Err("could not locate the string-> version = \" <-in the Cargo.toml file")
        }
    }
}

fn split_parts_from_version<'a>(version_string: &'a str) -> Result<(&'a str, &'a str, &'a str), &str> {
    match version_string.find_first(&".") {
        Some(first_version_period) => {
            match version_string.find_first_from(&".", first_version_period.end()) {
                Some(second_version_period) => {
                    let major = &version_string[..first_version_period.start()];
                    let minor = &version_string[first_version_period.end()..second_version_period.start()];
                    let patch = &version_string[second_version_period.end()..];
                    return Ok((major, minor, patch))
                },
                None => {
                    return Err("could not locate a '.' after the minor version")
                }
            }
        },
        None => {
            return Err("could not locate a '.' after the major version")
        }
    }
}