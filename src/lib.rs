use colored::Colorize;
use colored::CustomColor;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::de;
use std::env;
use std::fmt;
use std::fs;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
pub mod audit;
pub mod config;
pub mod launch;
pub mod layout;
pub mod search;
use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
enum ParaError {
    #[error("{0} environment variable not set")]
    EnvVariableNotFound(String),
    #[error("cannot create path for config, set PARA_CONFIG manually")]
    CannotCreateConfigPath,
}

fn get_env_path(envvar: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(&env::var(envvar).map_err(|_| {
        ParaError::EnvVariableNotFound(envvar.to_string())
    })?))
}

pub fn get_home_path() -> Result<PathBuf> {
    get_env_path("PARA_HOME")
}

pub fn get_git_path() -> Result<PathBuf> {
    get_env_path("PARA_GIT")
}

pub fn get_config_path() -> Result<PathBuf> {
    if let Ok(path) = get_env_path("PARA_CONFIG") {
        return Ok(path);
    }
    if let Some(mut path) = std::env::home_dir() {
        path = path.join(".config").join("para-audit");
        std::fs::create_dir_all(&path)?;
        path = path.join("config.yaml");
        return Ok(path);
    }
    Err(ParaError::CannotCreateConfigPath)?
}

pub fn get_root_paths() -> Result<Vec<PathBuf>> {
    let mut root_paths = vec![];
    for entry in get_home_path()?
        .read_dir()
        .expect("failed to read home dir")
        .flatten()
    {
        let filetype = entry.file_type().expect("failed to extract file type");
        if filetype.is_dir() {
            root_paths.push(entry.path());
        }
    }
    Ok(root_paths)
}

pub fn get_module_paths() -> Result<Vec<PathBuf>> {
    // Check each top level dir to see if it only contains folders (no files)
    let root_paths = get_root_paths()?;
    Ok(root_paths
        .iter()
        .flat_map(|root_path| {
            root_path
                .read_dir()
                .expect("failed to read root dirs")
                .map(|mod_entry| mod_entry.unwrap().path())
                .collect::<Vec<PathBuf>>()
        })
        .filter(|module| module.is_dir())
        .collect())
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
                module
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .display()
                    .to_string()
                    .custom_color(CustomColor {
                        r: 100,
                        g: 100,
                        b: 100
                    }),
                module
                    .parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
                    .custom_color(CustomColor {
                        r: 100,
                        g: 140,
                        b: 100
                    }),
                module
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
                    .custom_color(CustomColor {
                        r: 100,
                        g: 255,
                        b: 100
                    }),
            );
        } else {
            println!("{}", module.display());
        }
    }
}

pub fn print_count(item: &str, count: u32) {
    println!("{:5} {}", count.to_string().yellow(), item.green());
}

pub fn read_yaml(module: &Path) -> Result<ParaYaml> {
    // open module/para.yaml file if it exists
    let f = fs::File::open(module.join("para.yaml"))?;
    Ok(yaml_serde::from_reader(f)?)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParaYaml {
    #[serde(default, deserialize_with = "string_or_seq_string")]
    pub gits: Option<Vec<String>>,
    pub open: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

fn string_or_seq_string<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
    where D: Deserializer<'de>
{
    struct StringOrVec(PhantomData<Option<Vec<String>>>);

    impl<'de> de::Visitor<'de> for StringOrVec {
        type Value = Option<Vec<String>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where E: de::Error
        {
            Ok(Some(vec![value.to_owned()]))
        }

        fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
            where S: de::SeqAccess<'de>
        {
            Deserialize::deserialize(de::value::SeqAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(StringOrVec(PhantomData))
}