use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::{get_module_paths, get_root_paths, read_yaml};

pub fn search_modules(s: &str, precision: f64) -> Result<Vec<PathBuf>> {
    let module_paths = get_module_paths()?;
    // we have all the module paths here
    let mut matches = search_by_tag(s)?;
    let mut other_matches = module_paths
        .clone()
        .into_iter()
        .filter(|p| {
            let p_str = p.file_name().unwrap().to_str().unwrap().to_string();
            let mut hit: bool = strsim::jaro(&p_str, s) > precision || p_str.contains(s);
            if matches.contains(p) {
                hit = false;
            }
            hit
        })
        .collect();
    matches.append(&mut other_matches);

    Ok(matches)
}

pub fn find_module(s: &str) -> Result<Option<PathBuf>> {
    // search for exact name and return first match,
    if let Some(result) = get_module_paths()?
        .into_iter()
        .find(|p| p.file_name().unwrap().to_str().unwrap() == s)
    {
        return Ok(Some(result));
    };

    // if not exact match, check for unique substring match,
    let substring_matches = get_module_paths()?
        .into_iter()
        .filter(|p| p.file_name().unwrap().to_str().unwrap().contains(s))
        .collect::<Vec<PathBuf>>();
    if substring_matches.len() == 1 {
        return Ok(Some(substring_matches.first().unwrap().clone()));
    }

    // if no unique substring match found, return None
    Ok(None)
}

pub fn find_root(s: &str) -> Result<Option<PathBuf>> {
    Ok(get_root_paths()?
        .into_iter()
        .find(|p| p.file_name().unwrap().to_str().unwrap() == s))
}

pub fn list_rooted_modules(root: &str) -> Result<Vec<PathBuf>> {
    // check that root is actually a root
    let root_paths = get_root_paths()?;
    let root_paths: Vec<&PathBuf> = root_paths
        .iter()
        .filter(|r| {
            let r = &r.file_name().unwrap().to_str().unwrap().to_string();
            r == root
        })
        .collect();

    if root_paths.is_empty() {
        return Err(anyhow!("{}: {}", "invalid root", root));
    }

    let root_path = root_paths[0];

    // get all module paths and filter them
    let modules = get_module_paths()?
        .into_iter()
        .filter(|p| p.ancestors().nth(1).unwrap() == root_path)
        .collect();
    Ok(modules)
}

pub fn search_by_tag(tag: &str) -> Result<Vec<PathBuf>> {
    let mut modules: Vec<PathBuf> = vec![];
    for module in get_module_paths()? {
        if let Ok(para_yaml) = read_yaml(&module) {
            match para_yaml.tags {
                Some(tags) => {
                    for module_tag in tags.iter() {
                        if module_tag.contains(tag) {
                            modules.push(module);
                            break;
                        }
                    }
                }
                None => continue,
            }
        }
    }
    Ok(modules)
}

pub fn get_all_tags() -> Result<Vec<(String, u32)>> {
    let mut tags_count: HashMap<String, u32> = HashMap::new();
    for module in &get_module_paths()? {
        if let Ok(para_yaml) = read_yaml(module)
            && let Some(tags) = para_yaml.tags
        {
            for tag in tags {
                tags_count.entry(tag).and_modify(|c| *c += 1).or_insert(1);
            }
        }
    }
    Ok(tags_count.into_iter().collect::<Vec<(String, u32)>>())
}
