use std::path::{Path,PathBuf};
use colored::Colorize;
use std::collections::HashMap;
use para_audit::*;


pub fn search_modules(s: &str, precision: f64) -> Vec<PathBuf> {
    let module_paths = get_module_paths();
    // we have all the module paths here
    let mut matches = search_by_tag(s);
    let mut other_matches = module_paths.clone().into_iter()
    .filter(|p| {
        let p_str = p.file_name().unwrap().to_str().unwrap().to_string();
        let mut hit: bool = strsim::jaro(&p_str, s) > precision || p_str.contains(s);
        if matches.contains(p) {
            hit = false;
        }
        hit
    }).collect();
    matches.append(&mut other_matches);
    matches
}

pub fn find_module(s: &str) -> Option<PathBuf> {
    get_module_paths()
    .into_iter()
    .find(
        |p| p.file_name().unwrap().to_str().unwrap() == s
    )
}

pub fn find_root(s: &str) -> Option<PathBuf> {
    get_root_paths()
    .into_iter()
    .find(
        |p| p.file_name().unwrap().to_str().unwrap() == s
     )
}

pub fn list_rooted_modules(root: &str) -> Result<Vec<PathBuf>, String> {
    // check that root is actually a root
    let root_paths = get_root_paths();
    let root_paths: Vec<&PathBuf> = root_paths.iter()
    .filter(|r| {
        let r = &r.file_name().unwrap().to_str().unwrap().to_string();
        r == root
    }).collect();

    if root_paths.is_empty() {
        return Err(format!("{}: {}", "invalid root".red(), root));
    }

    let root_path = root_paths[0];

    // get all module paths and filter them
    let modules = get_module_paths().into_iter().filter(|p| 
        p.ancestors().nth(1).unwrap() == root_path
    ).collect();
    Ok(modules)
}

pub fn search_by_tag(tag: &str) -> Vec<PathBuf> {
    let mut modules: Vec<PathBuf> = vec![];
    for module in get_module_paths() {
        if let Some(yaml) = para_audit::read_yaml(&module) {
            if let Some(tags) = yaml["tags"].as_sequence() {
                if tags
                .iter()
                .filter_map(|x| x.as_str())
                .collect::<Vec<&str>>()
                .contains(&tag) {
                    modules.push(module);
                }
            }
        }
    }
    modules
}

pub fn get_module_tags(module: &Path) -> Vec<String> {
    let mut module_tags = vec![];
    if let Some(yaml) = para_audit::read_yaml(module) {
        if let Some(tags) = yaml["tags"].as_sequence() {
            for tag in tags {
                if let Some(t) = tag.as_str().map(|x| x.to_string()) {
                    module_tags.push(t)
                }
            }
        }
    }
    module_tags
}

pub fn get_all_tags() -> Result<Vec<(String,u32)>, String> {
    let mut tags_count: HashMap<String, u32> = HashMap::new();
    for module in &get_module_paths() {
        for tag in get_module_tags(module) {
            tags_count
            .entry(tag)
            .and_modify(|c| *c += 1)
            .or_insert(1);
        }
    }
    Ok(tags_count.into_iter().collect::<Vec<(String,u32)>>())
}