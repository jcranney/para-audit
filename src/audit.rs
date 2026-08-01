use colored::Colorize;
use git2::{Repository, Status};
use regex::Regex;
use std::{fmt::Debug, path::PathBuf};

use anyhow::Result;
use std::collections::HashMap;

use crate::{
    config, get_home_path, get_module_paths, get_root_paths, print_count, read_yaml,
    visit_all,
};

#[derive(Debug)]
enum Violation {
    RootDirClutter(PathBuf),
    ModDirClutter(PathBuf),
    ModDirName(PathBuf),
    ModRequiredFileMissing {
        file: String,
        module: PathBuf,
    },
    DisallowedFile(PathBuf),
    EmptyModule(PathBuf),
    DuplicateModules(PathBuf, PathBuf),
    TooManyFiles {
        module: PathBuf,
        filecount: u64,
    },
    NoTags(PathBuf),
    BrokenParaYaml {
        module: PathBuf,
        error: String,
    },
    GitNotSynced {
        module: String,
        count: usize,
        repo: PathBuf,
    },
    GitBroken {
        code: String,
        path: PathBuf,
    },
    GitNameInvalid {
        module: String,
        name: String,
    },
    GitNoRemote {
        repo: PathBuf,
    },
    GitBrokenSymlink(PathBuf),
}

enum Fix {
    ModName(PathBuf),
    CreateFile { file: String, module: PathBuf },
    Delete(PathBuf),
    EditFile(PathBuf),
    None,
}

impl Violation {
    fn fix(self) -> Fix {
        match self {
            Violation::ModDirName(p) => Fix::ModName(p),
            Violation::ModRequiredFileMissing { file, module } => Fix::CreateFile { file, module },
            Violation::DisallowedFile(p) | Violation::EmptyModule(p) => Fix::Delete(p),
            Violation::NoTags(p) => Fix::EditFile(p),
            Violation::GitBrokenSymlink(path) => Fix::Delete(path),
            Violation::BrokenParaYaml { module, .. } => Fix::EditFile(module),
            _ => Fix::None,
        }
    }
}

impl std::fmt::Display for Fix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fix::ModName(p) => {
                let source = p.clone();
                let destination = p.clone().parent().unwrap().join(
                    p.file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string()
                        .replace(['-', ' ', '.'], "_")
                        .to_lowercase(),
                );
                writeln!(
                    f,
                    "mv \"{}\" \"{}\"",
                    source.display(),
                    destination.display()
                )?;
            }
            Fix::CreateFile { file, module } => {
                writeln!(f, "touch \"{}\"", module.join(file).display())?;
            }
            Fix::Delete(p) => {
                writeln!(
                    f,
                    "rm {}\"{}\"",
                    match p.is_dir() || p.is_symlink() {
                        true => "-rf ",
                        false => "",
                    },
                    p.display()
                )?;
            }
            Fix::EditFile(p) => {
                writeln!(f, "vim \"{}\"", p.display())?;
            }
            Fix::None => (),
        };
        Ok(())
    }
}

pub fn propose_fixes(level: u32, config: &config::Config) -> Result<()> {
    let violations = get_violations(config)?;
    for v in violations {
        if v.level() <= level {
            print!("{}", v.fix());
        }
    }
    Ok(())
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Violation::RootDirClutter(pathbuf) =>
                    format!("{}: {}", "root dir clutter".red(), pathbuf.display()),
                Violation::ModDirClutter(pathbuf) =>
                    format!("{}: {}", "module dir clutter".red(), pathbuf.display()),
                Violation::ModDirName(pathbuf) =>
                    format!("{}: {}", "invalid module name".red(), pathbuf.display()),
                Violation::ModRequiredFileMissing { file, module } => format!(
                    "{} {}: {}",
                    "module missing".red(),
                    file.yellow(),
                    module.display()
                ),
                Violation::DisallowedFile(pathbuf) =>
                    format!("{}: {}", "disallowed file".red(), pathbuf.display(),),
                Violation::EmptyModule(pathbuf) =>
                    format!("{}: {}", "empty module".red(), pathbuf.display()),
                Violation::DuplicateModules(a, b) => format!(
                    "{}: {} {}",
                    "duplicate module".red(),
                    a.display(),
                    b.display()
                ),
                Violation::TooManyFiles { module, filecount } => format!(
                    "{}: {} {}",
                    "too many files".red(),
                    filecount.to_string().yellow(),
                    module.display()
                ),
                Violation::NoTags(yamlfile) =>
                    format!("{}: {}", "no tags".red(), yamlfile.display()),
                Violation::GitNotSynced {
                    repo,
                    count,
                    module,
                } => format!(
                    "{}, {}: {}, {}",
                    "git not synced".red(),
                    format!("{} files", count).red().italic(),
                    module,
                    repo.display(),
                ),
                Violation::GitBroken { code, path } => format!(
                    "{}, {}: {}",
                    "can't open git".red(),
                    code.red(),
                    path.display()
                ),
                Violation::GitNameInvalid { module, name } =>
                    format!("{}, {}: {}", "invalid git".red(), module, name),
                Violation::GitNoRemote { repo } =>
                    format!("{}: {}", "no remote repo".red(), repo.display()),
                Violation::GitBrokenSymlink(path_buf) =>
                    format!("{}: {}", "broken git symlink".red(), path_buf.display()),
                Violation::BrokenParaYaml { module, error } => format!(
                    "{} {}: {}",
                    "para.yaml".red(),
                    error.red(),
                    module.display()
                ),
            }
        )?;
        Ok(())
    }
}

impl Violation {
    fn level(&self) -> u32 {
        match self {
            Violation::RootDirClutter(_) => 1,
            Violation::ModDirClutter(_) => 1,
            Violation::ModDirName(_) => 2,
            Violation::ModRequiredFileMissing { .. } => 3,
            Violation::DisallowedFile(_) => 2,
            Violation::EmptyModule(_) => 1,
            Violation::DuplicateModules(..) => 1,
            Violation::TooManyFiles { .. } => 3,
            Violation::NoTags(..) => 4,
            Violation::GitNotSynced { .. } => 3,
            Violation::GitBroken { .. } => 3,
            Violation::GitNameInvalid { .. } => 3,
            Violation::GitNoRemote { .. } => 3,
            Violation::GitBrokenSymlink(_) => 5,
            Violation::BrokenParaYaml { .. } => 1,
        }
    }
}

fn get_violations(config: &config::Config) -> Result<Vec<Violation>> {
    // TODO: Actually implement the "ignored_files" config element
    let config::Config{ disallowed_files, ignore_files, default_open } = config;
    let mut violations: Vec<Violation> = vec![];

    let home_path = get_home_path()?;
    let root_paths = get_root_paths()?;

    // Check home dir for extra files/directories
    for root_entry in home_path.read_dir().expect("failed to read dir").flatten() {
        if root_paths.contains(&root_entry.path())
            || root_entry.path() == home_path.join(".paradisallow")
        {
            continue;
        } else {
            violations.push(Violation::RootDirClutter(root_entry.path()));
        }
    }

    // Check root dirs for extra files/directories
    for root_path in root_paths {
        for mod_entry in root_path.read_dir().expect("failed to read dir").flatten() {
            if !mod_entry.path().is_dir() {
                violations.push(Violation::ModDirClutter(mod_entry.path()));
            }
        }
    }

    let module_paths = get_module_paths()?;
    let re = Regex::new("[-, ,\\.,A-Z]").unwrap();
    // this is a list of all module directories:
    module_paths.iter().for_each(|mod_entry| {
        if re.is_match(mod_entry.file_name().unwrap().to_str().unwrap()) {
            violations.push(Violation::ModDirName(mod_entry.clone()));
        }
    });

    // Check each second level dir to see if it contains the required files:
    let required_files: Vec<String> = vec!["README.md".to_string(), "para.yaml".to_string()];

    // go into each module directory and verify that the required files are there:
    for module in &module_paths {
        let files: Vec<String> = module
            .read_dir()
            .expect("failed to read module")
            .filter_map(|mod_element| mod_element.ok())
            .filter_map(|entry| {
                entry
                    .path()
                    .file_name()
                    .and_then(|name| name.to_str().map(|s| s.to_string()))
            })
            .collect();
        if files.is_empty() {
            violations.push(Violation::EmptyModule(module.clone()));
        }
        for required_file in &required_files {
            if !files.contains(required_file) {
                violations.push(Violation::ModRequiredFileMissing {
                    file: required_file.clone(),
                    module: module.clone(),
                });
            }
        }
        // empty tags is a violation, but an ill-formatted para.yaml file is
        // a separate violation, checked for in the git-loop later.
        if let Ok(para_yaml) = read_yaml(module) {
            match para_yaml.tags {
                Some(tags) => {
                    if tags.is_empty() {
                        violations.push(Violation::NoTags(module.join("para.yaml")))
                    }
                }
                None => violations.push(Violation::NoTags(module.join("para.yaml"))),
            }
        }
    }

    let mut disallowed_files: Vec<String> = vec![];
    // disallowed files are the union of the files listed in ".paradisallow"
    // and in the para config file.

    if let Some(files) = &config.disallowed_files {
        disallowed_files.append(&mut files.clone());
    }

    // for the next tests, we need to check every single file/directory
    visit_all(&home_path, &mut |pathbuf| {
        let filename = pathbuf.file_name().unwrap().to_str().unwrap().to_string();
        if disallowed_files.contains(&filename) {
            violations.push(Violation::DisallowedFile(pathbuf.clone()))
        }
    });

    // check for name duplicates
    for i in 0..module_paths.len() {
        let module_i = &module_paths[i];
        for module_j in &module_paths[i..] {
            if module_i == module_j {
                continue;
            }
            let modname_i = module_i.file_name().unwrap().to_str().unwrap();
            let modname_j = module_j.file_name().unwrap().to_str().unwrap();
            if strsim::jaro(modname_i, modname_j) > 0.96 {
                violations.push(Violation::DuplicateModules(
                    module_i.clone(),
                    module_j.clone(),
                ));
            }
        }
    }

    // check for too many files
    module_paths
        .iter()
        .map(|p| {
            let mut count: u64 = 0;
            visit_all(p, &mut |_| {
                count += 1;
            });
            (p, count)
        })
        .filter(|(_, x)| *x > 1000)
        .for_each(|(p, count)| {
            violations.push(Violation::TooManyFiles {
                module: p.clone(),
                filecount: count,
            });
        });

    // check the status of all git repos
    for p in module_paths.iter() {
        match read_yaml(p) {
            Ok(para_yaml) => {
                for git in para_yaml.gits {
                    match git.split('/').next_back() {
                        Some(mut n) => {
                            n = n.trim_end_matches(".git");
                            let repo_path = p.join(n);
                            if repo_path.is_dir() {
                                match Repository::open(&repo_path) {
                                    Ok(repo) => {
                                        let statuses = repo.statuses(None).unwrap();
                                        let count: usize = statuses
                                            .iter()
                                            .filter(|s| s.status() != Status::IGNORED)
                                            .count();
                                        if count > 0 {
                                            violations.push(Violation::GitNotSynced {
                                                count,
                                                repo: repo_path.clone(),
                                                module: p.to_str().unwrap().to_string(),
                                            });
                                        }
                                        if repo.remotes().unwrap().is_empty() {
                                            violations
                                                .push(Violation::GitNoRemote { repo: repo_path });
                                        }
                                    }
                                    Err(e) => {
                                        violations.push(Violation::GitBroken {
                                            code: format!("{:?}", e.code()),
                                            path: repo_path,
                                        });
                                    }
                                };
                            } else {
                                // it's not a directory
                                if !repo_path.is_file()  // and it's not a file
                                    && repo_path.is_symlink()
                                // but it is still a symlink
                                {
                                    // then it must be a broken symlink, so it should be deleted
                                    violations.push(Violation::GitBrokenSymlink(repo_path));
                                }
                            }
                        }
                        None => {
                            violations.push(Violation::GitNameInvalid {
                                module: p.to_str().unwrap().to_string(),
                                name: git.to_owned(),
                            });
                        }
                    };
                }
            }
            Err(e) => violations.push(Violation::BrokenParaYaml {
                module: p.into(),
                error: e.to_string(),
            }),
        }
    }

    Ok(violations)
}

pub fn audit(level: u32, config: &config::Config) -> Result<()> {
    let violations = get_violations(config)?;

    // print results
    for v in &violations {
        if v.level() <= level {
            println!("{}", v);
        }
    }

    // print summary
    println!(
        "para: {}",
        match violations.len() {
            x if x > 0 => format!("{} violations", x.to_string().red()),
            _ => format!("{} violations", "zero".green()),
        }
    );

    Ok(())
}

pub fn stats(min_count: u32) -> Result<()> {
    let mut filecount: u32 = 0;
    visit_all(&get_home_path()?, &mut |_| {
        filecount += 1;
    });
    print_count("total files", filecount);

    let mut ext_count: HashMap<String, u32> = HashMap::new();
    visit_all(&get_home_path()?, &mut |path: &PathBuf| {
        if path.is_file() {
            ext_count
                .entry(match path.extension().map(|x| x.to_str().unwrap()) {
                    Some(ext) => ext.to_string(),
                    None => "none".to_string(),
                })
                .and_modify(|x| *x += 1)
                .or_insert(1);
        }
    });

    let mut results = ext_count
        .into_iter()
        .filter(|(_, c)| c >= &min_count)
        .collect::<Vec<(String, u32)>>();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    results
        .into_iter()
        .for_each(|(a, b)| print_count(&a[..], b));

    Ok(())
}
