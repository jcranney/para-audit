# para_audit

A tool for auditing/organising/interacting with my `para` system.

```
$ para help
A simple CLT for supervising/interfacing with a storage convention based on Tiago Forte's Second Brain - PARA principle.

Usage: para <COMMAND>

Commands:
  audit   audit para system
  search  search para modules
  list    list all para modules, optionally by module type
  open    open a module to work on
  move    move a module between roots
  stats   print para stats (filecount, etc.)
  new     create a new module, by default in the projects root
  note    edit the README.md of a particular module
  tags    list all tags
  fix     list fixes to problems identified by audit
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Introduction
I won't pretend that this is a well documented package, but the main principles are outlined below.

Tiago Forte's book [Building a Second Brain](https://www.buildingasecondbrain.com/) was helpful to me for defining structure in the organisation of my digital data. If you are interested in the detail, then go read his book, but for what is relevant to this command-line-tool, Forte suggests (or more accurately, I interpret Forte's advice as) organsing your digital data first into top-level folders-named:
```tree
├── projects
├── areas
├── resources
└── archive
```
then beneath these folders, having "modules" which are each another folder, corresponding to one atomic idea unit. 
```
├── projects
│   ├── tax_return_2025
│   └── ...
├── areas
│   ├── personal_health
│   └── ...
├── resources
│   ├── travel
│   └── ...
└── archive
    ├── aaai_2024_review
    └── ...
```
This is not necessarily canonical Forte. I use the definitions below:
 - A `project`, for example, is something that can be finished.
 - An `area` is an area of responsibility.
 - A `resource` is a collection of related material that doesn't fit into a `project` or `area`,
 - and the `archive` is where modules are moved when they are no longer relevant (but still able to be retrieved at a moment's notice).

The first character of these folders defines the name of this tool, `para` (which itself is a `project` and destined one day for the `archive`).

## Installation
With cargo installed, you can install this package via:
```bash
cargo install para-audit
```
and then run it with:
```bash
para --help
```
The only system configuration required is to set the environment variable `PARA_HOME`. This should point to the directory where you have those 4 folders above. For example, in my case I have:
```bash
# file: ~/.zshrc
# ...

export PARA_HOME=$HOME/gdrive
```
where `$HOME/gdrive` is the location of my [insync](https://www.insynchq.com/) google drive directory.

## Usage
Once installed, the `para` command allows you to interact with your PARA storage system. For example, I run `para audit` every time a new shell is opened, giving me an update to the health of my organised file system. I also use `para ls` (equivalent to `para ls projects`) often, listing the modules in my `projects` folder, and of course `para open <module-name>` which allows me to open a module.

## `para.yaml`
Each module is unique, for the most part. However, I wanted to be able to define some specific behaviour for when I'm interacting with particular modules. For example, when I'm opening `my_new_rust_project`, it would be useful to open VS Code in that module's directory. There are currently 3 types of customisations that can be made per-module:

(WIP)