mod save_process;
mod utils;

use std::{env};

use gmec::types::error_chain::{ErrorChain, ErrorPropogation};

use crate::save_process::save_process;


const SAVE_MODE_ARG: &str = "save";
const HELP_MODE_ARG: &str = "help";

const HELP_MESSAGE :&str = "
The cargit command is used to automate git and cargo processes that would normally take several independant commands,
while also making relevant changes to the crate's Cargo.toml and Cargo.lock (when relevant)

Usage:
  cargit help                                                    # Display this help message
  cargit save [major|minor|patch] [-publish] [-m \"<message>\"]  # this command performs the following actions, in order:
      (Increment relevent part of the version in Cargo.toml, and set any lesser version parts to 0)
      cargo generate-lockfile
      git add .
      git commit -m \"<message>\"
      git push
      git tag <Version from Cargo.toml>
      git push --tags
      cargo publish (if -publish option set)
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

fn main() -> Result<(), ErrorChain> {
    let mut args_iter  = env::args().collect::<Vec<String>>().into_iter();
    let _ = args_iter.next().on_error("missing first cli argument")?;
    let mode_arg = args_iter.next().unwrap_or(String::from("help"));
    let mode = match mode_arg.as_str() {
        SAVE_MODE_ARG => RunMode::Save,
        HELP_MODE_ARG => RunMode::Help,
        _ => RunMode::HelpErr
    };
    let mode_result = match mode {
        RunMode::HelpErr => Ok(format!("invalid mode passed to cargit, see 'cargit help' for more info:\n{}", HELP_MESSAGE)),
        RunMode::Help => Ok(format!("{}", HELP_MESSAGE)),
        RunMode::Save => save_process(args_iter)
    };
    match mode_result {
        Ok(message) => println!("{}", message),
        Err(err) => eprintln!("{}", err)
    }
    return Ok(())
}