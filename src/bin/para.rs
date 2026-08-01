use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use para_audit::{ParaError, get_git_path, get_home_path, get_module_paths, get_root_paths};
use para_audit::{
    audit,
    config::{self, Config},
    launch, layout, search,
};
use std::process::exit;
use std::{path::PathBuf};

const TICK_MARK: char = char::from_u32(0x2705).unwrap();
const CROSS_MARK: char = char::from_u32(0x274c).unwrap();

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// audit para system
    #[clap(alias = "a")]
    Audit {
        /// level of verbosity to show, 0->10
        level: Option<u32>,
    },
    /// open a module to work on
    #[clap(alias = "o")]
    Open {
        /// module name or substring
        module: String,
    },
    /// search para modules
    #[clap(alias = "s")]
    Search {
        /// string to search for in para modules
        search_string: String,
    },
    /// create a new module, by default in the projects root
    New {
        /// name of the module
        name: String,
        /// which root
        root: Option<String>,
    },
    /// list all para modules, optionally by module type
    #[clap(alias = "ls")]
    List {
        /// module type (e.g., [project], area, resource, archive, all)
        root: Option<String>,
    },
    /// move a module between roots
    #[clap(alias = "mv")]
    Move {
        /// module name or substring
        module: String,
        /// destination root (e.g., projects, areas, resources, archive)
        destroot: String,
    },
    /// print para stats (filecount, etc.)
    #[clap(alias = "st")]
    Stats {
        /// minimum count for showing extensions
        min_count: Option<u32>,
    },
    /// list all tags
    Tags {
        /// hide tags with less than count occurances
        count: Option<u32>,
    },
    /// list fixes to problems identified by audit
    Fix { level: Option<u32> },
    /// check status of para configuration
    Doctor,
}

fn main() -> Result<()> {
    // need a philosophy around config files. I'm thinking:
    //   if PARA_CONFIG env variable exists, then this is the file path.
    //   else $HOME/.config/para-audit/config.yaml is the path.
    //
    //   if file exists at path, then use it as is,
    //   otherwise create it and populate with defaults
    let config_path = para_audit::get_config_path()?;
    let config = match config::Config::read_yaml(&config_path) {
        Ok(config) => config,
        Err(ParaError::IOError(_)) => {
            // coulnd't read a yaml file at the defined config path,
            // create a default one and try to save it there instead.
            let config = config::Config::default();
            config.write_yaml(&config_path)?;
            config
        }
        Err(e) => {
            eprintln!("couldn't parse config file at {}, {}", config_path.display(), e);
            eprintln!("If you move the file out of the way, then I'll create a valid default config there in it's place.");
            exit(1);
        },
    };

    let args = Args::parse();
    match &args.command {
        Commands::Audit { level } => audit::audit(level.unwrap_or(10), &config)?,
        Commands::Search { search_string } => {
            let modules = search::search_modules(search_string, 0.8)?;
            para_audit::print_modules(modules, true);
        }
        Commands::List { root } => match root {
            Some(root) => match &root[..] {
                "all" | "a" => para_audit::print_modules(para_audit::get_module_paths()?, true),
                root => para_audit::print_modules(search::list_rooted_modules(root)?, true),
            },
            None => para_audit::print_modules(search::list_rooted_modules("projects")?, true),
        },
        Commands::Open { module } => {
            let module_to_open = match search::find_module(module)? {
                Some(m) => m,
                None => {
                    let potential_modules = search::search_modules(module, 0.8)?;
                    match potential_modules.len() {
                        1 => potential_modules[0].clone(),
                        x if x > 1 => {
                            para_audit::eprint_modules(potential_modules);
                            return Err(anyhow!("ambiguous module name"));
                        }
                        _ => {
                            return Err(anyhow!("can't find a match"));
                        }
                    }
                }
            };
            launch::open(&module_to_open)?;
        }
        Commands::Move { module, destroot } => {
            let module = match search::find_module(module)? {
                Some(m) => m,
                None => {
                    let potential_modules = search::search_modules(module, 1.0)?;
                    match potential_modules.len() {
                        1 => potential_modules[0].clone(),
                        x if x > 1 => {
                            para_audit::eprint_modules(potential_modules);
                            return Err(anyhow!("ambiguous module name"));
                        }
                        _ => {
                            return Err(anyhow!("can't find a match"));
                        }
                    }
                }
            };
            if let Some(root) = search::find_root(destroot)? {
                layout::mv(module, root)?;
            } else {
                return Err(anyhow!("invalid destination name"));
            }
        }
        Commands::Stats { min_count } => audit::stats(min_count.unwrap_or(100))?,
        Commands::New { name, root } => {
            let mut module_path: PathBuf;
            if let Some(root) = root {
                if let Some(path) = search::find_root(root)? {
                    module_path = path;
                } else {
                    return Err(anyhow!("invalid root name - {}", root));
                }
            } else {
                module_path = match search::find_root("projects")? {
                    Some(p) => p,
                    None => {
                        return Err(anyhow!(
                            "can't find `projects` folder, something very wrong"
                        ));
                    }
                }
            }
            module_path = module_path.join(name);
            layout::new(module_path)?;
        }
        Commands::Tags { count } => {
            let count = count.unwrap_or(5);
            let mut tags = search::get_all_tags()?;
            tags.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            tags.into_iter()
                .filter(|(_, y)| y >= &count)
                .for_each(|(x, y)| para_audit::print_count(&x[..], y));
        }
        Commands::Fix { level } => audit::propose_fixes(level.unwrap_or(10), &config)?,
        Commands::Doctor => {
            // is para-home set?
            match get_home_path() {
                Ok(p) => println!("{TICK_MARK} PARA_HOME: {}", p.display()),
                Err(e) => println!("{CROSS_MARK} {e}"),
            }

            // is the config path set?
            println!("{TICK_MARK} PARA_CONFIG: {}", config_path.display());

            // is there a config file at the path?
            match Config::read_yaml(&config_path) {
                Ok(_) => {
                    println!("{TICK_MARK} Config file exists and is readable",);
                }
                Err(e) => println!("{CROSS_MARK} {e}"),
            };

            // is para git set?
            match get_git_path() {
                Ok(p) => println!("{TICK_MARK} PARA_GIT: {}", p.display()),
                Err(e) => println!("{CROSS_MARK} {e}"),
            }

            // can we load root paths?
            match get_root_paths() {
                Ok(v) => println!("{TICK_MARK} Root paths exist, found {} root paths", v.len()),
                Err(e) => println!("{CROSS_MARK} {e}"),
            }

            // can we load module paths?
                        // can we load root paths?
            match get_module_paths() {
                Ok(v) => println!("{TICK_MARK} Module paths exist, found {} module paths", v.len()),
                Err(e) => println!("{CROSS_MARK} {e}"),
            }
        }
    }
    Ok(())
}
