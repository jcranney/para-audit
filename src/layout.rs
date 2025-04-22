use std::io::Write;
use std::path::{Path, PathBuf};
use std::fs;
use colored::Colorize;

pub fn new(module: PathBuf) -> Result<(), String> {
    match fs::DirBuilder::new().create(&module) {
        Ok(()) => eprintln!("{}", "created module".green().italic()),
        Err(e) => {
            return Err(e.to_string());
        },
    }

    match fs::File::create(module.join("README.md")) {
        Ok(mut f) => {
            writeln!(f, "# {}", module.file_name().unwrap().to_str().unwrap())
            .or(Err("couldn't write to new README.md"))?;
            eprintln!("{}", "created readme".green().italic());
        }
        Err(e) => {
            return Err(e.to_string());
        }
    }

    match fs::File::create(module.join("para.yaml")) {
        Ok(mut f) => {
            writeln!(f, "open: [\"code\", \".\"]")
            .or(Err("couldn't write to new para.yaml"))?;
            eprintln!("{}", "created para.yaml".green().italic());
        }
        Err(e) => {
            return Err(e.to_string());
        }
    }

    Ok(())
}

pub fn mv(module: PathBuf, root: PathBuf) -> Result<(), String> {
    // check that root/module.child does not exist yet
    let destination = root.join(module.file_name().unwrap());
    if Path::exists(&destination) {
        return Err(
            format!(
                "cannot move {} to {}, path exists",
                module.display(),
                destination.display(),
            )
        );
    }

    // create root/module.child
    if let Err(e) =  fs::DirBuilder::new().create(&destination) {
        return Err(e.to_string());
    }

    // rename module to root/module.child
    if let Err(e) = fs::rename(&module, &destination) {
        return Err(e.to_string());
    } else {
        eprintln!("{}",
            format!(
                "moved to {}",
                destination.display(),
            ).green().italic()
        );
    }
    Ok(())
}