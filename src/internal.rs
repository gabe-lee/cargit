use std::{process::{Command, Output, ExitStatus}, ffi::OsStr, io::{self, Write}};

use gmec::{patterns::PatternMatcher, types::error_chain::{ErrorChain, ErrorPropogation}};

pub(crate) const CARGO_MANIFEST: &str = "Cargo.toml";
pub(crate) const VERSION_PREFIX: &str = "version = \"";

pub(crate) enum VersionPart {
    Major,
    Minor,
    Patch
}

#[derive(Clone, Copy)]
pub(crate) struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32
}

impl Version {
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

pub(crate) fn cli_affirmative(string: String) -> bool {
    match string.to_lowercase().trim() {
        "y" | "ye" | "yes" => true,
        _ => false
    }
}

pub(crate) struct Commiterator {
    index: usize,
    commits_and_tags: Vec<(String, Vec<String>)>
}

impl Iterator for Commiterator {
    type Item = (String, Vec<String>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.commits_and_tags.len() {
            return None
        }
        let cloned = (self.commits_and_tags[self.index].0.clone(), self.commits_and_tags[self.index].1.clone());
        self.index += 1;
        return Some(cloned)
    }
}

pub(crate) fn get_cli_output<S>(program: S, args: &[S]) -> Result<Output, ErrorChain>
where S: AsRef<str> + AsRef<OsStr> {
    let full_command = collect_full_command(&program, args);
    let out_result = Command::new(program).args(args).output().on_error(format!("error running command: {}", full_command))?;
    return Ok(out_result);
}

pub(crate) fn run_cli<S>(program: S, args: &[S]) -> Result<(), ErrorChain>
where S: AsRef<str> + AsRef<OsStr> {
    let full_command = collect_full_command(&program, args);
    let success = Command::new(program).args(args).status().on_error(format!("error running command: {}", full_command))?.success();
    if success {
        return Ok(())
    }
    return Err(ErrorChain::new("command exited with abnormal status"));
}

fn collect_full_command<S>(program: &S, args: &[S]) -> String
where S: AsRef<str> + AsRef<OsStr> {
    let mut command_string = String::new();
        command_string.push_str(program.as_ref());
        for arg in args.into_iter() {
            command_string.push(' ');
            command_string.push_str(arg.as_ref());
        }
        command_string
}

pub(crate) fn get_cli_output_as_string<S>(program: S, args: &[S]) -> Result<String, ErrorChain>
where S: AsRef<OsStr> + AsRef<str> {
    let cli_output = get_cli_output(program, args).on_error("could not execute command")?;
    let cli_output_string = String::from_utf8(cli_output.stdout).on_error("could not parse to string")?.to_owned();
    return Ok(cli_output_string);
}

pub(crate) fn split_version_from_cargo_toml<'a>(cargo_toml_str: &'a str) -> Result<(&'a str, &'a str, &'a str), ErrorChain> {
    let version_prefix_location = cargo_toml_str.find_first(&VERSION_PREFIX).on_error("could not locate the string 'version = \"' in the Cargo.toml file")?;
    let version_end_location = cargo_toml_str.find_first_from(&"\"", version_prefix_location.end()).on_error("could not locate a '\"' after the version numbers in the Cargo.toml file")?;
    let before_version = &cargo_toml_str[..version_prefix_location.end()];
    let version_string = &cargo_toml_str[version_prefix_location.end()..version_end_location.start()];
    let after_version = &cargo_toml_str[version_end_location.start()..];
    return Ok((before_version, version_string, after_version))
}

pub(crate) fn split_parts_from_version<'a>(version_string: &'a str) -> Result<(&'a str, &'a str, &'a str), ErrorChain> {
    let first_version_period = version_string.find_first(&".").on_error("could not locate a '.' after the major version")?;
    let second_version_period = version_string.find_first_from(&".", first_version_period.end()).on_error("could not locate a '.' after the minor version")?;
    let major = &version_string[..first_version_period.start()];
    let minor = &version_string[first_version_period.end()..second_version_period.start()];
    let patch = &version_string[second_version_period.end()..];
    return Ok((major, minor, patch));
}

pub(crate) fn get_reverse_chron_iterator_over_commits_in_current_branch() -> Result<Commiterator, ErrorChain> {
    let branch = get_branch_name().on_error("could not get current branch name")?;
    return get_reverse_chron_iterator_over_commits_in_branch(branch.as_str());
}

pub(crate) fn get_reverse_chron_iterator_over_commits_in_branch(branch: &str) -> Result<Commiterator, ErrorChain> {
    let all_commits = get_all_commits_in_current_branch().on_error("could not retrieve commits list")?;
    let mut all_commits_tags: Vec<(String, Vec<String>)> = Vec::new();
    for commit in all_commits {
        let tags_on_commit = get_tags_on_commit(&commit).on_error("could_not_retrieve togs on commit")?;
        all_commits_tags.push((commit, tags_on_commit));

    }
    return Ok(Commiterator{ index: 0, commits_and_tags: all_commits_tags })
}

pub(crate) fn get_all_commits_in_current_branch() -> Result<Vec<String>, ErrorChain> {
    let branch = get_branch_name().on_error("could not get current branch name")?;
    return get_all_commits_in_branch(branch.as_str());
}

pub(crate) fn get_all_commits_in_branch(branch: &str) -> Result<Vec<String>, ErrorChain> {
    let command = "git log <branch-name> --pretty=format:'%H'";
    let commits_in_branch = get_cli_output_as_string("git", &["log", branch, "--pretty=format:'%H'"]).on_error(format!("error using command '{}'", command))?;
    let git_log_output = Command::new("git").arg("log").arg("--pretty=format:'%H'").output().on_error("error running command 'git log --pretty=format:'%H''")?;
    let git_log_output_string = String::from_utf8(git_log_output.stdout).on_error("could not parse output from 'git log --pretty=format:'%H'' into utf-8 String")?;
    let git_log_commits: Vec<String> = git_log_output_string.split_whitespace().map(|s| s.trim_matches('\'').to_owned()).collect();
    return Ok(git_log_commits);
}

pub(crate) fn get_tags_on_commit(commit: &String) -> Result<Vec<String>, ErrorChain> {
    let git_tag_output = Command::new("git").arg("tag").arg("--points-at").arg(commit.as_str()).output().on_error(format!("error running command 'git tag --points-at {}'", commit))?;
    let git_tag_output_string = String::from_utf8(git_tag_output.stdout).on_error(format!("could not parse output from 'git tag --points-at {}'", commit))?;
    let git_tags_on_commit: Vec<String> = git_tag_output_string.split_whitespace().map(|s| s.to_owned()).collect();
    return Ok(git_tags_on_commit);
}

pub(crate) fn get_current_commit() -> Result<String, ErrorChain> {
    let git_show_output = Command::new("git").arg("show").arg("-s").arg("--format=%H").output().on_error("error running command 'git show -s --format=%H'")?;
    let git_show_output_string = String::from_utf8(git_show_output.stdout).on_error("could not parse output from 'git show -s --format=%H'")?.trim().to_owned();
    return Ok(git_show_output_string);
}

pub(crate) fn get_branch_name() -> Result<String, ErrorChain> {
    let command = "git rev-parse --abbrev-ref --symbolic-full-name HEAD";
    let branch_name = get_cli_output_as_string("git", &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "HEAD"]).on_error(format!("error using command '{}'", command))?.trim_end().to_owned();
    return Ok(branch_name);
}

pub(crate) fn get_remote_name() -> Result<Option<String>, ErrorChain> {
    let command = "git remote";
    let remote_name = get_cli_output_as_string("git", &["remote"]).on_error(format!("error using command '{}'", command))?.trim().to_owned();
    if remote_name.is_empty() {
        return Ok(None);
    }
    return Ok(Some(remote_name));
}

pub(crate) fn is_detatched_mode() -> Result<bool, ErrorChain> {
    let branch_name = get_branch_name().on_error("could not determine if repo in detatched head mode")?;
    return Ok(branch_name == String::from("HEAD")); // branches CANNOT be named HEAD, HEAD is only returned from get_branch_mode() when in detatched state
}

pub(crate) fn last_attatched_head_branch() -> Result<String, ErrorChain> {
    let command = "git log --walk-reflogs --grep-reflog \"checkout\" -1 --oneline";
    let detatch_log = get_cli_output_as_string("git", &["log", "--walk-reflogs", "--grep-reflog", "checkout", "-1", "--oneline"]).on_error(format!("error using command '{}'", command))?;
    let original_branch_start = detatch_log.find_first(&"checkout: moving from ").on_error("could not locate where HEAD was detatched from branch")?;
    let original_branch_end = detatch_log.find_first_from(&" to ", original_branch_start.end()).on_error("could not locate where HEAD was detatched from branch")?;
    let original_branch = detatch_log[original_branch_start.end()..original_branch_end.start()].to_owned();
    return Ok(original_branch);
}

pub(crate) fn get_all_local_branches_in_repo() -> Result<Vec<String>, ErrorChain> {
    let command = "git branch --list --format=\"%(refname:short)\"";
    let branches_string = get_cli_output_as_string("git", &["branch", "--list", "--format=\"%(refname:short)\""]).on_error(format!("error using command '{}'", command))?;
    let branches: Vec<String> = branches_string.split_whitespace().map(|s| s.trim_matches('"').to_owned()).collect();
    return Ok(branches);
}

pub(crate) fn git_checkout(identifier: &str) -> Result<(), ErrorChain> {
    return run_cli("git", &["checkout", identifier]);
}

pub(crate) fn git_branch(branch_name: &str) -> Result<(), ErrorChain> {
    return run_cli("git", &["branch", branch_name]);
}

pub(crate) fn cargo_generate_lockfile() -> Result<(), ErrorChain> {
    return run_cli("cargo", &["generate-lockfile"]);
}

pub(crate) fn cargo_publish() -> Result<(), ErrorChain> {
    return run_cli("cargo", &["publish"]);
}

pub(crate) fn git_stage_all_changes() -> Result<(), ErrorChain> {
    return run_cli("git", &["add", "."]);
}

pub(crate) fn git_commit_with_message(message: &String) -> Result<(), ErrorChain> {
    return run_cli("git", &["commit", "-m", message.as_str()]);
}

pub(crate) fn git_create_tag(tag: &String) -> Result<(), ErrorChain> {
    return run_cli("git", &["tag", tag.as_str()]);
}

pub(crate) fn git_push(remote: &String, branch: &String) -> Result<(), ErrorChain> {
    return run_cli("git", &["push", remote.as_str(), branch.as_str(), "--tags"]);
}

pub(crate) fn git_merge(from_branch: &String, into_branch: &String) -> Result<(), ErrorChain> {
    git_checkout(&into_branch)?;
    return run_cli("git", &["merge", from_branch.as_str()]);
}

pub(crate) fn git_delete_branch(branch: &String) -> Result<(), ErrorChain> {
    return run_cli("git", &["branch", "--delete", branch.as_str()]);
}

// pub(crate) git_push_branch()

pub(crate) fn read_stdin_line(output_string: &mut String) -> Result<usize, ErrorChain> {
    io::stdout().flush().on_error("error flushing stdout")?;
    io::stdin().read_line(output_string).on_error("error reading from stdin")
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::Debug;
    
    fn print_or_panic<D: Debug>(result: Result<D, ErrorChain>) {
        match result {
            Ok(output) => println!("{:#?}", output),
            Err(error) => panic!("{}", error)
        }
    }

    fn unwrap_or_panic<D: Debug>(result: Result<D, ErrorChain>) -> D {
        match result {
            Ok(output) => return output,
            Err(error) => panic!("{}", error)
        }
    }

    #[test]
    #[ignore]
    fn test_get_all_commits_in_current_branch() {
        print_or_panic(get_all_commits_in_current_branch());
    }

    #[test]
    #[ignore]
    fn test_get_tags_on_commit() {
        print_or_panic(get_tags_on_commit(&String::from("308ce2999e9d4e53143a0821f23821b199a1f444")));
    }

    #[test]
    #[ignore]
    fn test_get_current_commit() {
        print_or_panic(get_current_commit());
    }

    #[test]
    #[ignore]
    fn test_get_branch_name() {
        print_or_panic(get_branch_name());
    }

    #[test]
    #[ignore]
    fn test_get_remote_name() {
        print_or_panic(get_remote_name());
    }

    #[test]
    #[ignore]
    fn test_is_detatched_mode() {
        print_or_panic(is_detatched_mode());
    }

    #[test]
    #[ignore]
    fn test_last_attatched_head_branch() {
        print_or_panic(last_attatched_head_branch());
    }

    #[test]
    #[ignore]
    fn test_get_all_local_branches_in_repo() {
        print_or_panic(get_all_local_branches_in_repo());
    }

    #[test]
    #[ignore]
    fn test_get_detatched_or_attatched_branch() {
        let branch_result = if unwrap_or_panic(is_detatched_mode()) {
            unwrap_or_panic(last_attatched_head_branch())
        } else {
            unwrap_or_panic(get_branch_name())
        };
        println!("{}", branch_result);
    }

    #[test]
    #[ignore]
    fn test_iterator_over_commits_and_tags() {
        let commiterator_result = get_reverse_chron_iterator_over_commits_in_current_branch();
        match commiterator_result {
            Ok(commiterator) => {
                let mut output = String::new();
                for (commit, tags) in commiterator {
                    let mut tags_on_commit = String::new();
                    for tag in tags {
                        tags_on_commit.push_str(&tag);
                        tags_on_commit.push(',');
                    } 
                    output.push('\n');
                    output.push_str(&commit);
                    output.push(':');
                    output.push_str(&tags_on_commit);
                }
                println!("{}", output);
            }
            Err(commiterator_error) => {
                panic!("{}", commiterator_error)
            }
        }
    }
}