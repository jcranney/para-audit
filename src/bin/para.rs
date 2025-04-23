use clap::{Parser, Subcommand};
use std::path::PathBuf;
use para_audit::{audit, launch, layout, search};

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
    /// search para modules
    #[clap(alias = "s")]
    Search {
        /// string to search for in para modules
        search_string: String,
    },
    /// list all para modules, optionally by module type
    #[clap(alias = "ls")]
    List {
        /// module type (e.g., [project], area, resource, archive, all)
        root: Option<String>,
    },
    /// open a module to work on
    #[clap(alias = "o")]
    Open {
        /// module name or substring
        module: String,
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
    /// create a new module, by default in the projects root
    New {
        /// name of the module
        name: String,
        /// which root
        root: Option<String>,
    },
    /// edit the README.md of a particular module
    #[clap(alias = "edit")]
    Note {
        /// name of the module
        module: String,
    },
    /// list all tags
    Tags {
        /// hide tags with less than count occurances 
        count: Option<u32>,
    },
    /// list fixes to problems identified by audit
    Fix {
        level: Option<u32>,
    },
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    match &args.command {
        Commands::Audit { level } => audit::audit(level.unwrap_or(10)),
        Commands::Search { search_string } => {
            let modules = search::search_modules(search_string, 0.8);
            para_audit::print_modules(modules, true);
        },
        Commands::List { root } => {
            match root {
                Some(root) => match &root[..] {
                    "all" | "a" => para_audit::print_modules(
                        para_audit::get_module_paths(),
                        true,
                    ),
                    root => para_audit::print_modules(
                        search::list_rooted_modules(root)?,
                        true,
                    ),
                }
                None => para_audit::print_modules(
                    search::list_rooted_modules("projects")?,
                    true,
                ),
            }
        },
        Commands::Open { module } => {
            let module_to_open = match search::find_module(module) {
                Some(m) => {
                    m
                },
                None => {
                    let potential_modules = search::search_modules(module, 0.8);
                    match potential_modules.len() {
                        1 => potential_modules[0].clone(),
                        x if x > 1 => {
                            para_audit::eprint_modules(potential_modules);
                            return Err("ambiguous module name".to_string());
                        },
                        _ => {
                            return Err("can't find a match".to_string());
                        }
                    }
                }
            };
            launch::open(&module_to_open)?;
        },
        Commands::Move { module, destroot } => {
            let module = match search::find_module(module) {
                Some(m) => m,
                None => {
                    let potential_modules = search::search_modules(module, 1.0);
                    match potential_modules.len() {
                        1 => potential_modules[0].clone(),
                        x if x > 1 => {
                            para_audit::eprint_modules(potential_modules);
                            return Err("ambiguous module name".to_string());
                        },
                        _ => {return Err("can't find a match".to_string());}
                    }
                }
            };
            if let Some(root) = search::find_root(destroot) {
                layout::mv(module, root)?;
            } else {
                return Err("invalid destination name".to_string());
            }
        },
        Commands::Stats { min_count } => audit::stats(min_count.unwrap_or(100)),
        Commands::New { name, root } => {
            let mut module_path: PathBuf;
            if let Some(root) = root {
                if let Some(path) = search::find_root(root) {
                    module_path = path;
                } else {
                    return Err(format!("invalid root name - {}", root))
                }
            } else {
                module_path = match search::find_root("projects") {
                    Some(p) => p,
                    None => {
                        return Err("can't find `projects` folder, something very wrong".to_string());
                    }
                }
            }
            module_path = module_path.join(name);
            layout::new(module_path)?;

        },
        Commands::Note { module } => {
            if let Some(module) = search::find_module(module) {
                launch::edit_note(module.join("README.md"))?;   
            } else {
                return Err("can't find module".to_string());
            }
        },
        Commands::Tags {count} => {
            let count = count.unwrap_or(5);
            let mut tags = search::get_all_tags()?;
            tags.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap());
            tags
            .into_iter()
            .filter(|(_,y)| y >= &count)
            .for_each(|(x,y)|
                para_audit::print_count(&x[..], y)
            );
        },
        Commands::Fix { level } => audit::propose_fixes(level.unwrap_or(10)),
    }
    Ok(())
}