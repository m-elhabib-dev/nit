use clap::{Parser, Subcommand};
use std::fs::{self, create_dir};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

pub(crate) mod commands;
pub(crate) mod index;
pub(crate) mod objects;

/// Doc comment
#[derive(Debug, Subcommand)]
enum Command {
    /// init` — create a `.git` repository skeleton
    Init,
    /// cat-file -p <hash>` — pretty-print an object
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,

        object_hash: String,
    },
    /// hash-object [-w] <file>` — compute an object hash, optionally write it
    HashObject {
        #[clap(short = 'w')]
        write: bool,

        file: PathBuf,
    },
    /// ls-tree [--name-only] <tree-hash>` — list a tree's entries
    LsTree {
        #[clap(long)]
        name_only: bool,

        tree_hash: String,
    },

    /// write-tree` — write the working tree as a tree object
    WriteTree,
    /// commit-tree -m <msg> [-p <parent>] <tree-hash>` — write a commit object
    CommitTree {
        #[clap(short = 'm')]
        message: String,
        tree_hash: String,
        #[clap(short = 'p')]
        parent_hash: Option<String>,
    },
    /// commit -m <msg>` — write tree + commit, update the current branch
    Commit {
        #[clap(short = 'm')]
        message: String,
    },

    /// add <file>` — stage a file (or `.` for everything) into the index
    Add { file: PathBuf },

    /// status` — show modified, deleted, and untracked files
    Status,
    /// clone <path>` — clone a local repository
    Clone { url: String },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Init => {
            create_dir(".git").unwrap();
            create_dir(".git/objects").unwrap();
            create_dir(".git/refs").unwrap();
            create_dir(".git/refs/heads").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Intialized git directory");
        }
        Command::CatFile {
            pretty_print,
            object_hash,
        } => commands::cat_file::invoke(pretty_print, &object_hash)?,

        Command::HashObject { write, file } => commands::hash_object::invoke(write, &file)?,

        Command::LsTree {
            name_only,
            tree_hash,
        } => commands::ls_tree::invoke(name_only, &tree_hash)?,

        Command::WriteTree => commands::write_tree::invoke()?,

        Command::CommitTree {
            message,
            tree_hash,
            parent_hash,
        } => commands::commit_tree::invoke(message, tree_hash, parent_hash)?,
        Command::Commit { message } => {
            commands::commit::invoke(message)?;
        }
        Command::Add { file } => index::add::invoke(file)?,
        Command::Status => index::status::invoke()?,
        Command::Clone { url } => commands::clone::invoke(url)?,
    }
    Ok(())
}
