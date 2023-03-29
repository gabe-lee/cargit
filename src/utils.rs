use std::{process::Command, ffi::OsStr};

use gmec::{patterns::PatternMatcher, types::error_chain::{ErrorChain, ErrorPropogation}};

pub(crate) const CARGO_MANIFEST: &str = "Cargo.toml";
pub(crate) const VERSION_PREFIX: &str = "version = \"";

pub(crate) enum VersionPart {
    Major,
    Minor,
    Patch
}

pub(crate) struct CommitTagIterator {
    index: usize,
    commits_and_tags: Vec<(String, Vec<String>)>
}

impl Iterator for CommitTagIterator {
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

pub(crate) fn get_cli_output_as_string<S, IIS>(program: S, args: IIS) -> Result<String, ErrorChain>
where S: AsRef<OsStr> + AsRef<str>,
IIS: IntoIterator<Item = S> {
    let cli_output = Command::new(program).args(args).output().on_error("could not execute command")?;
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

pub(crate) fn get_reverse_chron_iterator_over_commits() -> Result<CommitTagIterator, ErrorChain> {
    let all_commits = get_all_commits_in_current_branch().on_error("could not retrieve commits list")?;
    let mut all_commits_tags: Vec<(String, Vec<String>)> = Vec::new();
    for commit in all_commits {
        let tags_on_commit = get_tags_on_commit(&commit).on_error("could_not_retrieve togs on commit")?;
        all_commits_tags.push((commit, tags_on_commit));

    }
    return Ok(CommitTagIterator{ index: 0, commits_and_tags: all_commits_tags })
}

pub(crate) fn get_all_commits_in_current_branch() -> Result<Vec<String>, ErrorChain> {
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
    let branch_name = get_cli_output_as_string("git", ["rev-parse", "--abbrev-ref", "--symbolic-full-name", "HEAD"]).on_error(format!("error using command '{}'", command))?;
    return Ok(branch_name.trim_end().to_owned());
}

pub(crate) fn get_origin_name() -> Result<String, ErrorChain> {
    let command = "git remote";
    let remote_name = get_cli_output_as_string("git", ["remote"]).on_error(format!("error using command '{}'", command))?;
    return Ok(remote_name.trim_end().to_owned());
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
    fn test_get_origin_name() {
        print_or_panic(get_origin_name());
    }

    #[test]
    #[ignore]
    fn test_iterator_over_commits_and_tags() {
        let commiterator_result = get_reverse_chron_iterator_over_commits();
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