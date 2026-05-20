use std::io::Write;
use std::path::{Path, PathBuf};
use std::fs;
use colored::Colorize;
use anyhow::{Result, anyhow};

pub fn new(module: PathBuf) -> Result<()> {
    fs::DirBuilder::new().create(&module)?;
    eprintln!("{}", "created module".green().italic());

    let mut f = fs::File::create(module.join("README.md"))?;
    writeln!(f, "# {}", module.file_name().unwrap().to_str().unwrap())?;
    eprintln!("{}", "created readme".green().italic());
    
    let mut f = fs::File::create(module.join("para.yaml"))?;
    writeln!(f, "open: [\"code\", \".\"]")?;
    eprintln!("{}", "created para.yaml".green().italic());
    
    Ok(())
}

pub fn mv(module: PathBuf, root: PathBuf) -> Result<()> {
    // check that root/module.child does not exist yet
    let destination = root.join(module.file_name().unwrap());
    if Path::exists(&destination) {
        return Err(
            anyhow!(
                "cannot move {} to {}, path exists",
                module.display(),
                destination.display(),
            )
        );
    }

    // create root/module.child
    fs::DirBuilder::new().create(&destination)?;

    // rename module to root/module.child
    fs::rename(&module, &destination)?;
    eprintln!("{}",
        format!(
            "moved to {}",
            destination.display(),
        ).green().italic()
    );

    Ok(())
}