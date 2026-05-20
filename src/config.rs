// everything related to the PARA user-dependent configuration.

use std::path::Path;

use serde::{Deserialize, Serialize};
use anyhow::Result;


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Config {
    // to start with, let's just define the list of disallowed files
    pub disallowed_files: Option<Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        Self { disallowed_files: Some(vec![ "example_disallowed.file".to_string() ]) }
    }
}

impl Config {

    pub fn write_yaml<P: AsRef<Path>>(&self, filename: P) -> Result<()> {
        std::fs::write(filename, yaml_serde::to_string(&self)?)?;
        Ok(())
    }

    pub fn read_yaml<P: AsRef<Path>>(filename: P) -> Result<Config> {
        let config = yaml_serde::from_str(&std::fs::read_to_string(filename)?)?;
        Ok(config)
    }
}