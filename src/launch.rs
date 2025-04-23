use std::{env, path::{Path, PathBuf}, process::Command};
use colored::Colorize;

use crate::read_yaml;

pub fn open(module: &PathBuf) -> Result<(), String> {
    // print module path to std for "goto"/"cd" like command
    eprintln!("{}", format!(
        "opening: {}",
        module.file_name().unwrap().to_str().unwrap(),
    ).green().italic());
    
    if let Some(yaml) = read_yaml(module) {
        let cmd = yaml["open"]
            .as_sequence()
            .map(|s| s.iter()
                .map(|x| x.as_str().map(|s| s.to_string()))
                .collect::<Vec<Option<String>>>()
            );
        if let Some(cmd) = cmd {
            // command sequence exists
            let mut command: Command;
            if let Some(root_cmd) = &cmd[0] {
                command = Command::new(root_cmd);
            } else {
                return Err("couldn't parse command argument".to_string());
            }
            for arg in cmd[1..].iter() {
                if let Some(arg) = arg {
                    command.arg(arg);
                } else {
                    return Err("failed to parse command arguments in para.yaml".to_string())
                }
            }
            command.current_dir(module);
            command.status().or(Err("failed to spawn `open` command from para.yaml"))?;
        } else {
            eprintln!("{}", "couldn't parse para.yaml `open` as command sequence".red().italic());
        }
        if let Some(git) = yaml["git"].as_str() {
            init_git(git, module)?;
        }
    }
    
    Command::new("zsh")
    .current_dir(module)
    .status().or(Err("couldn't start zsh"))?;
    Ok(())
}

pub fn edit_note(note: PathBuf) -> Result<(), String> {
    Command::new("code")
    .arg(note)
    .status().or(Err("Couldn't start vim"))?;
    Ok(())
}

fn init_git(git: &str, module: &Path) -> Result<(), String> {
    // get git repo name (will be dir name)
    let name = match git.split('/').last() {
        Some(n) => n.trim_end_matches(".git"),
        None => return Err("para.yaml git url invalid".to_string()),
    };

    // git url is defined, confirm that no dir with that name exists yet
    if module.join(name).exists() {
        // already exists, no problem.
        return Ok(());
    }

    // check if repo in downloads, if not, get it
    let original = Path::new(
        &env::var("HOME").expect("HOME env var not defined")
    ).to_path_buf().join("Downloads").join(name);
    if !original.exists() {
        // doesn't exist, clone it:
        if let Ok(status) = Command::new("git")
        .arg("clone")
        .arg(git)
        .arg(&original)
        .status() {
            if !status.success() {
                return Err("git clone failed".to_string());
            }
        } else {
            return Err(
                "failed to start git".to_string()
            );
        }
    }
    // now there is a correctly named directory in the downlaods folder,
    // hopefully the git repo but if it's not then that's fine, whatever.

    // make symbolic link here linking to cloned repo
    if let Err(e) = std::os::unix::fs::symlink(original, module.join(name)) {
        return Err(e.to_string())
    }

    // done!
    Ok(())
}