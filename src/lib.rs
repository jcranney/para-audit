use std::env;
use std::path::{Path,PathBuf};
use colored::Colorize;
use colored::CustomColor;
use std::fs;
pub mod audit;
pub mod search;
pub mod launch;
pub mod layout;

#[must_use]
pub fn get_home_path() -> PathBuf {
    PathBuf::from(
        &env::var("PARA_HOME")
        .expect("PARA_HOME environment variable not defined")
    )
}

pub fn get_root_paths() -> Vec<PathBuf> {
    let home = get_home_path();
    // Check home dir for extra files/directories
    vec![
        "projects",
        "areas",
        "resources",
        "archive",
    ].into_iter()
    .map(|name| home.join(name))
    .collect()
}

pub fn get_module_paths() -> Vec<PathBuf> {
    // Check each top level dir to see if it only contains folders (no files)
    let root_paths = get_root_paths();
    root_paths.iter().flat_map(|root_path|
        root_path.read_dir().expect("failed to read root dirs")
        .map(|mod_entry| mod_entry.unwrap().path())
        .collect::<Vec<PathBuf>>()
    ).filter(|module| module.is_dir()).collect()
}   

pub fn visit_all(path: &PathBuf, cb: &mut dyn FnMut(&PathBuf)) {
    if path.is_dir() && !path.is_symlink() {
        for entry in path.read_dir().unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            visit_all(&path, cb);
        }
    }
    cb(path);
}

pub fn eprint_modules(modules: Vec<PathBuf>) {
    for module in modules {
        eprintln!("{}", module.display().to_string().italic());
    }
}

pub fn print_modules(modules: Vec<PathBuf>, colorised: bool) {
    for module in modules {
        if colorised {
            println!(
                "{}/{}/{}",
                module.parent().unwrap()
                .parent().unwrap()
                .display().to_string()
                .custom_color(CustomColor{r:100,g:100,b:100}),
                module.parent().unwrap()
                .file_name().unwrap()
                .to_str().unwrap().to_string()
                .custom_color(CustomColor{r:100,g:140,b:100}),
                module.file_name().unwrap()
                .to_str().unwrap().to_string()
                .custom_color(CustomColor{r:100,g:255,b:100}),
            );
        } else {
            println!("{}", module.display());
        }
    }
}

pub fn print_count(item: &str, count: u32) {
    println!("{:5} {}", count.to_string().yellow(), item.green());
}

#[must_use]
pub fn read_yaml(
    module: &Path,
) -> Option<serde_yaml::Value> {
    // open module/para.yaml file if it exists
    if let Ok(f) = fs::File::open(module.join("para.yaml")) {
        if let Ok(file) = serde_yaml::from_reader(f) {
            return Some(file)
        }
    }
    None
}