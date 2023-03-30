use gmec::types::error_chain::ErrorChain;
use gmec::types::error_chain::ErrorPropogation;

use crate::utils::*;


const LOAD_PREV: &str = "prev";
const LOAD_NEXT: &str = "next";
const LOAD_LATEST: &str = "latest";

enum LoadMode {
    Prev,
    Next,
    Latest,
    ID(String),
}

pub(crate) fn load_process<I>(mut args_iter: I) -> Result<String, ErrorChain>
where I: Iterator<Item = String> {
    let mutliple_load_mode_set_error = format!("load mode ('{}' or '{}' or '{}' or '<hash>' or '<tag>' or '<branch>') set more than once", LOAD_PREV, LOAD_NEXT, LOAD_LATEST);
    let mut load_mode: LoadMode = LoadMode::Latest;
    let mut load_mode_set = false;
    while let Some(next_arg) = args_iter.next() {
        let next_arg_lower = next_arg.to_lowercase();
        match next_arg_lower.as_str() {
            LOAD_PREV | LOAD_NEXT | LOAD_LATEST => {
                if load_mode_set {
                    return Err(ErrorChain::new(mutliple_load_mode_set_error));
                }
                load_mode = match next_arg_lower.as_str() {
                    LOAD_PREV => LoadMode::Prev,
                    LOAD_NEXT => LoadMode::Next,
                    LOAD_LATEST => LoadMode::Latest,
                    _ => LoadMode::Latest // Impossible
                };
                load_mode_set = true;
            }
            id => {
                if load_mode_set {
                    return Err(ErrorChain::new(mutliple_load_mode_set_error));
                }
                load_mode = LoadMode::ID(id.to_owned());
                load_mode_set = true;
            }
        }
    }
    let current_branch = if is_detatched_mode().on_error("could not verify head attatchement")? {
        last_attatched_head_branch().on_error("could not get last branch before detatching")?
    } else {
        get_branch_name().on_error("could not get branch name")?
    };
    let current_commit = get_current_commit().on_error("could not get current commit")?;
    let commiterator = get_reverse_chron_iterator_over_commits_in_branch(&current_branch).on_error("could not obtain an iterator over all commits in branch")?;
    let load_output = match load_mode {
        LoadMode::Prev => {
            let mut next_commit_is_before_current = false;
            let mut id_to_checkout: Option<String> = None;
            for (commit, _) in commiterator {
                if next_commit_is_before_current {
                    id_to_checkout = Some(commit);
                    break;
                } else if commit == current_commit {
                    next_commit_is_before_current = true;
                }
            }
            let good_commit = id_to_checkout.on_error("no commit found earlier than current")?;
            git_checkout(&good_commit).on_error(format!("error checking out commit {}", good_commit))?;
            format!("checked out previous commit ({})", good_commit)
        },
        LoadMode::Next => {
            let mut id_to_checkout: Option<String> = None;
            for (commit, _) in commiterator {
                if commit == current_commit {
                    break;
                } else {
                    id_to_checkout = Some(commit);
                }
            }
            let good_commit = id_to_checkout.on_error("no commit found later than current")?;
            git_checkout(&good_commit).on_error(format!("error checking out commit {}", good_commit))?;
            format!("checked out next commit ({})", good_commit)
        },
        LoadMode::Latest => {
            git_checkout(&current_branch).on_error(format!("error checking out latest commit in branch {}", current_branch))?;
            format!("checked out latest commit in branch {}", current_branch)
        },
        LoadMode::ID(id) => {
            git_checkout(&id).on_error(format!("error checking out {}, must be a tag, commit hash, branch name, or one of 'last', 'next', or 'latest'", id))?;
            format!("checked out {}", id)
        },
    };
    return Ok(load_output)
}