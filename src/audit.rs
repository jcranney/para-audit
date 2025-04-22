use std::path::PathBuf;
use colored::Colorize;
use para::*;
use regex::Regex;

use std::collections::HashMap;

use crate::search;

#[derive(Debug)]
enum Violation {
    RootDirClutter(PathBuf),
    ModDirClutter(PathBuf),
    ModDirName(PathBuf),
    ModRequiredFileMissing{
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
}

enum Fix {
    MoveFile(PathBuf),
    ModName(PathBuf),
    CreateFile {
        file: String,
        module: PathBuf,
    },
    Delete(PathBuf),
    EditFile(PathBuf),
    None,
}

impl Violation {
    fn fix(self) -> Fix {
        match self {
            Violation::RootDirClutter(p) | 
            Violation::ModDirClutter(p) => Fix::MoveFile(p),
            Violation::ModDirName(p) => Fix::ModName(p),
            Violation::ModRequiredFileMissing { file, module } => Fix::CreateFile { file, module },
            Violation::DisallowedFile(p) | Violation::EmptyModule(p) => Fix::Delete(p),
            Violation::NoTags(p) => Fix::EditFile(p),
            _ => Fix::None,
        }
    }
}

impl std::fmt::Display for Fix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fix::MoveFile(p) => {
                let source = p.clone();
                let destination = search::find_root("projects")
                    .unwrap()
                    .join("CLUTTER")
                    .join(p.file_name().unwrap());
                writeln!(f, "mv \"{}\" \"{}\"", source.display(), destination.display())?;
            },
            Fix::ModName(p) => {
                let source = p.clone();
                let destination = p.clone()
                    .parent()
                    .unwrap()
                    .join(
                        p.file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string()
                        .replace(['-',' ','.'], "_")
                        .to_lowercase()
                    );
                writeln!(f, "mv \"{}\" \"{}\"", source.display(), destination.display())?;
            },
            Fix::CreateFile { file, module } => {
                writeln!(f, "touch \"{}\"", module.join(file).display())?;
            },
            Fix::Delete(p) => {
                writeln!(f, "rm {}\"{}\"", match p.is_dir() {
                    true => "-rf ",
                    false => "",
                }, p.display())?;
            },
            Fix::EditFile(p) => {
                writeln!(f, "vim {}", p.display())?;
            }
            Fix::None => (),
        };
        Ok(())
    }
}

pub fn propose_fixes(level: u32) {
    let violations = get_violations();
    for v in violations {
        if v.level() <= level {
            print!("{}", v.fix());
        }
    }
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Violation::RootDirClutter(pathbuf) => {
                format!("{}: {}", "root dir clutter".red(), pathbuf.display())
            },
            Violation::ModDirClutter(pathbuf) => {
                format!("{}: {}", "module dir clutter".red(), pathbuf.display())
            },
            Violation::ModDirName(pathbuf) => {
                format!("{}: {}", "invalid module name".red(), pathbuf.display())
            },
            Violation::ModRequiredFileMissing{file,module} => {
                format!(
                    "{} {}: {}",
                    "module missing".red(),
                    file.yellow(),
                    module.display())
            },
            Violation::DisallowedFile(pathbuf) => {
                format!(
                    "{}: {}",
                    "disallowed file".red(),
                    pathbuf.display(),
                )
            },
            Violation::EmptyModule(pathbuf) => {
                format!("{}: {}", "empty module".red(), pathbuf.display())
            },
            Violation::DuplicateModules(a,b) => {
                format!("{}: {} {}", "duplicate module".red(), a.display(), b.display())
            },
            Violation::TooManyFiles { module, filecount } => {
                format!(
                    "{}: {} {}",
                    "too many files".red(),
                    filecount.to_string().yellow(),
                    module.display()
                )
            },
            Violation::NoTags(yamlfile) => {
                format!("{}: {}", "no tags".red(), yamlfile.display())
            },
        })?;
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
            Violation::TooManyFiles{..} => 3,
            Violation::NoTags(..) => 4,
        }
    }
}

fn get_violations() -> Vec<Violation> {
    let mut violations: Vec<Violation> = vec![];

    let home_path = get_home_path();
    let root_paths = get_root_paths();

    // Check home dir for extra files/directories
    for root_entry in home_path.read_dir().expect("failed to read dir").flatten() {
        if root_paths.contains(&root_entry.path()) {
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
    
    let module_paths = get_module_paths();
    let re = Regex::new("[-, ,\\.,A-Z]").unwrap();
    // this is a list of all module directories:
    module_paths.iter().for_each(|mod_entry| {
        if re.is_match(mod_entry.file_name().unwrap().to_str().unwrap()) {
            violations.push(Violation::ModDirName(mod_entry.clone()));
        }
    });

    // Check each second level dir to see if it contains the required files:
    let required_files: Vec<String> = vec![
        "README.md".to_string(),
        "para.yaml".to_string(),
    ];

    // go into each module directory and verify that the required files are there:
    for module in &module_paths {
        let files: Vec<String> = module.read_dir()
        .expect("failed to read module")
        .filter_map(|mod_element| mod_element.ok())
        .filter_map(|entry| 
            entry.path().file_name().and_then(|name|
                name.to_str().map(|s| s.to_string())
            )
        )
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
        let tags = search::get_module_tags(module);
        if tags.is_empty() {
            violations.push(Violation::NoTags(module.join("para.yaml")))
        }
    }

    let disallowed_files: Vec<String> = [
        ".git",
        ".svn",
        "package-lock.json",
        ".gitignore",
        "node_modules",
        "venv",
        "build",
        "target",
        ".mypy_cache",
        "__pycache__",
        "tmp",
    ].iter().map(|x| x.to_string()).collect();
    
    // for the next tests, we need to check every single file/directory
    visit_all(&home_path, &mut |pathbuf| {
        let filename = pathbuf.file_name().unwrap()
        .to_str().unwrap()
        .to_string();
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
                    module_j.clone()
                ));
            }
        }
    }

    // check for too many files
    module_paths.iter()
    .map(|p| {
        let mut count: u64 = 0;
        para::visit_all(p, &mut |_| {count += 1;});
        (p,count)
    })
    .filter(|(_,x)| *x > 1000)
    .for_each(|(p,count)| violations.push(Violation::TooManyFiles { 
        module: p.clone(), 
        filecount:  count
    }));
    violations
}

pub fn audit(level: u32) {
    let violations = get_violations();

    // print results
    for v in &violations {
        if v.level() <= level {
            println!("{}", v);
        }
    }

    // print summary
    println!("para: {}",
        match violations.len() {
            x if x > 0 => format!("{} violations", x.to_string().red()),
            _ => format!("{} violations", "zero".green()),
        }
    );
}

pub fn stats(min_count: u32) {
    let mut filecount: u32 = 0;
    para::visit_all(&para::get_home_path(), &mut |_| {filecount += 1;} );
    para::print_count("total files", filecount);

    let mut ext_count: HashMap<String,u32> = HashMap::new();
    para::visit_all(
        &para::get_home_path(),
        &mut |path: &PathBuf| {
            if path.is_file() {
                ext_count
                .entry(
                    match path.extension().map(|x| x.to_str().unwrap()) {
                        Some(ext) => ext.to_string(),
                        None => "none".to_string(),
                    }
                ).and_modify(|x| *x+=1)
                .or_insert(1);
            }
        }
    );

    let mut results = ext_count
        .into_iter()
        .filter(|(_,c)| c >= &min_count)
        .collect::<Vec<(String,u32)>>();
    results.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap());
    results.into_iter().for_each(|(a,b)|
        para::print_count(&a[..], b)
    );
}