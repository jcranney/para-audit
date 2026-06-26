# para_audit

A tool for auditing/organising/interacting with my `para` system.

```
$ para help
A simple CLT for supervising/interfacing with a storage convention based on Tiago Forte's Second Brain - PARA principle.

Usage: para <COMMAND>

Commands:
  audit   audit para system
  open    open a module to work on
  search  search para modules
  new     create a new module, by default in the projects root
  list    list all para modules, optionally by module type
  move    move a module between roots
  stats   print para stats (filecount, etc.)
  tags    list all tags
  fix     list fixes to problems identified by audit
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Introduction
I won't pretend that this is a well documented package, but the main principles are outlined below.

Tiago Forte's book [Building a Second Brain](https://www.buildingasecondbrain.com/) was helpful to me for defining structure in the organisation of my digital data. If you are interested in the detail, then go read his book, but for what is relevant to this command-line-tool, Forte suggests (or more accurately, I interpret Forte's advice as) collecting your digital data into contextual units, and then sort these into top-level categories to ease the mental burden of locating your data. A good storage system should allow you to find what you were looking for when you want to find it. Beyond that, any prescription is probably subjective and opinionated. With that said, here is my working implementation:

For example, take a few ideas from your life that have associated digital data:
 - your project at work, named [MAVIS](https://www.eso.org/sci/facilities/develop/instruments/MAVIS.html),
 - an open-source tiling window manager you've been tinkering with, named [Hyprland](https://github.com/hyprwm/Hyprland),
 - your cat Tabatha who's been on a dieting regime since your pet-sitter overfed her.
 - etc.


**Second Brain** suggests the following top level categories:
```tree
├── projects
├── areas
├── resources
└── archive
```

Then, each of the ideas above would become a directory sitting under a given category:
```
├── projects
│   ├── mavis
│   ├── hyprland
│   └── ...
├── areas
│   ├── tabatha
│   └── ...
├── resources
│   ├── travel
│   └── ...
└── archive
    ├── aaai_2024_review
    └── ...
```

The categorisation choice is something I still think needs work, but this definition is good enough to get you started:
 - A `project` is something that can be finished.
 - An `area` is an area of responsibility.
 - A `resource` is a collection of related material that doesn't fit into a `project` or `area`,
 - and the `archive` is where directories are moved when they are no longer relevant (but still able to be retrieved at a moment's notice).

The first character of these folders defines the name of this tool, `para` (which for me is itself a `project` and hopefully destined one day for the `archive`).

## Installation
With cargo installed, you can install this package via:
```bash
cargo install para-audit
```
and then run it with:
```bash
para --help
```
You are required to define the following two environment variables:
 - `PARA_HOME`, this should point to the root directory where you can find the top level folders (e.g., `projects`,`areas`,`resources`,`archive`).
 - `PARA_GIT`, this should point to a folder where you would like your `git` repositories stored. In my case, I don't like synchronising my git repos to the cloud, so I usually have this set to `$HOME/git` (for example).

For example, in most of my machines I set the environment variables as:
```bash
export PARA_HOME=$HOME/gdrive
export PARA_GIT=$HOME/git
```
where `$HOME/gdrive` is the location of my [insync](https://www.insynchq.com/) google drive directory, and `$HOME/git` is an arbitrary location to dump all of my local clones of git repositories.

## Usage
Once installed, the `para` command allows you to interact with your PARA storage system. For example, I run `para audit` every time a new shell is opened, giving me an update to the health of my organised file system. I also use `para ls` (equivalent to `para ls projects`) often, listing the modules in my `projects` folder, and `para open <module-name>` which allows me to open a module.

## The `para.yaml` file
Each module is unique, but the ways in which I interact with them are similar. For example, it's common that when I switch into a project that I want to open some external application from that directory. E.g., when I'm opening `my_project`, it would be useful to open VS Code at the same time. I can do this by creating a `para.yaml` file inside the directory of `my_project`:
```tree
└── projects
    └── my_project
        └── para.yaml
```
and specifying the `open: ...` property:
```yaml
open: ["code", "."]
```
This `open` command tells `para` to execute the command: `code .` from the
project directory whenever I run:
```bash
para open my_project
```
`para` will also launch and join a shell session from that directory, whether I have specified an `open` property or not.

---

Similarly, I often have a git repository associated with a project that
I'd like to have "available" whenever I am mentally in the context of the project.
I can specify it in my `para.yaml` file, like so:
```yaml
open: ["code", "."]
gits: git@github.com:jcranney/my_repo
```
Furthermore, as you may have guessed by the plural, it's possible to add multiple
git repositories to the same project. Since they will be symbolically linked
from the `$PARA_GIT` directory, there is usually no cost in linking one repo
to many `para` projects:
```yaml
open: ["code", "."]
gits: 
 - git@github.com:jcranney/my-repo
 - git@github.com:raplonu/an-inspiring-related-project
```
The behaviour of `para open my_project` when there are gits specified is as follows:
   1) the program will retrieve any gits listed in the `my_project/para.yaml` file,
   2) if there is a subfolder in the project directory with the same name as the git repo, e.g., `.../my_project/my-repo`, then there is nothing else to be done. Otherwise,
   3) check for the existence of a folder: `$PARA_GIT/my-repo`, and symbolically link it to `.../my_project/my-repo`. If it does not already exist, clone it from the specified git repo file to `$PARA_GIT/my-repo`, and then create the symbolic link to `.../my_project/my-repo`.

---

I also recommend that you define a list of tags for a project. Tags are searchable
through `para` and can help you search for a specific project based on elements of
the project that you are able to remember. This is done through the `tags: ...` field
of the `para.yaml` file:
```yaml
open: ["code", "."]
gits: 
 - git@github.com:jcranney/my-repo
 - git@github.com:raplonu/an-inspiring-related-project
tags: ["rust","home automation","arduino"]
```

Tags are used by a number of `para` subcommands, and I find them so useful that I
have defined it to be a [violation](#violations)

## The `para audit` command
So far, `para` is just a glorified `cd` command, with some convenience functions
so that you don't have to remember if it's `ln -s <target> <link>` or the other way around (if you have a mnemonic device for this please let me know). *Well*, let me
introduce to you the `para audit` command...

`para audit` scans recursively every single file (ignoring symlinks) and collects
a list of [violations](#violations) along the way, reporting them back to you once it's finished. At the time of writing this, I have about 16,268 (try `para stats`) files under `$PARA_HOME`, belonging to a total of 338 unique modules (try `para ls all`). `para audit` scans through all of them in about 60 milliseconds, and reports a summary back:
```bash
$ para audit
root dir clutter: /home/jcranney/gdrive/matrix_naming.gdsheet
invalid module name: /home/jcranney/gdrive/archive/Taguchi
git not synced, 3 files: /home/jcranney/gdrive/archive/para_audit, /home/jcranney/gdrive/archive/para_audit/para-audit
no remote repo: /home/jcranney/gdrive/projects/risio/rao-sim
para: 4 violations
```
(I promise it's prettier with colour).

I **strongly recommend** using `para audit` regularly, and ideally in a way that
cannot be accidentally avoided. For me, I include `para audit` at the bottom of
my `~/.zshrc` file so that I see the list of violations every time I open a new shell. I cannot explain how satisfying it is to get a green message stating `para: zero violations`, but the closest feeling is that of arriving home after holidays
to find your room clean and bed made as you thoughtfully left it before you set off. Pure bliss.

If you're feeling lazy, you can get `para` to suggest some fixes for these violations by running `para fix`. In the case above, it's clear what to do with the `invalid module name`, but otherwise the solution is ambiguous, so you're on your own there.
```bash
$ para fix
mv "/home/jcranney/gdrive/archive/Taguchi" "/home/jcranney/gdrive/archive/taguchi"
```

## `Violation`
**!Disclaimer!** This part of the project is extremely opinionated, and worse than
that, it's not user-configurable. If you have ideas for how to "configurize" it, please speak them in the Issues. The violations I have defined are ones that matter
to me, but I don't assume they matter to you.

A violation is a rule that can be assessed at some point during the recursive scanning of
the `$PARA_HOME` directory. A violation should contain enough information to imply a
possible solution. There can be many instances of the same violation - since it's possible
to make the same mistakes more than once.

As of version `v0.1.20`, the complete list of Violations can be found in `./src/audit.rs`, and is repeated here:
```rust
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
```

In the code base, `RootDir` refers to the `$PARA_HOME` environment variable, and `ModDir` refers to the immediate children of that directory. When you first run `para audit`, you will likely get hundreds of "violations" listed. Note that the `para audit` program only reads the file-system, only writing to stdout.