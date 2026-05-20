use colored::Colorize;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use anyhow::{Result, anyhow};

use crate::{get_git_path, read_yaml};

pub fn open(module: &PathBuf) -> Result<()> {
    // print module path to std for "goto"/"cd" like command
    eprintln!(
        "{}",
        format!("opening: {}", module.file_name().unwrap().to_str().unwrap(),)
            .green()
            .italic()
    );

    if let Some(yaml) = read_yaml(module)? {
        let cmd = yaml["open"].as_sequence().map(|s| {
            s.iter()
                .map(|x| x.as_str().map(|s| s.to_string()))
                .collect::<Vec<Option<String>>>()
        });
        if let Some(cmd) = cmd {
            // command sequence exists
            let mut command: Command;
            if let Some(root_cmd) = &cmd[0] {
                command = Command::new(root_cmd);
            } else {
                return Err(anyhow!("couldn't parse command argument"));
            }
            for arg in cmd[1..].iter() {
                if let Some(arg) = arg {
                    command.arg(arg);
                } else {
                    return Err(anyhow!("failed to parse command arguments in para.yaml"));
                }
            }
            command.current_dir(module);
            command
                .status()
                .or(Err(anyhow!("failed to spawn `open` command from para.yaml")))?;
        }
        if let Some(git) = yaml["git"].as_str() {
            init_git(git, module)?;
        }
    }

    Command::new("zsh")
        .current_dir(module)
        .status()?;
    Ok(())
}

pub fn edit_note(note: PathBuf) -> Result<()> {
    Command::new("code")
        .arg(note)
        .status()
        .or(Err(anyhow!("Couldn't start vim")))?;
    Ok(())
}

fn init_git(git: &str, module: &Path) -> Result<()> {
    // get git repo name (will be dir name)
    let name = match git.split('/').next_back() {
        Some(n) => n.trim_end_matches(".git"),
        None => return Err(anyhow!("para.yaml git url invalid")),
    };

    // git url is defined, confirm that no dir with that name exists yet
    if module.join(name).exists() {
        // already exists, no problem.
        return Ok(());
    } else {
        // file either doesn't exist, or is a broken symlink
        if module.join(name).is_symlink() {
            // must be a broken symlink, we can delete it and move on
            std::fs::remove_file(module.join(name))?;
        }
    }

    // check if repo in downloads, if not, get it
    let original = get_git_path()?.join(name);
    if !original.exists() {
        // doesn't exist, clone it:
        let status = Command::new("git")
            .arg("clone")
            .arg(git)
            .arg(&original)
            .status()?;
        if !status.success() {
            return Err(anyhow!("git clone failed"));
        }
    }
    // now there is a correctly named directory in the downlaods folder,
    // hopefully the git repo but if it's not then that's fine, whatever.

    
    // make symbolic link here linking to cloned repo
    std::os::unix::fs::symlink(original, module.join(name))?;

    // done!
    Ok(())
}
