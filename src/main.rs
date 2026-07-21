use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

pub(crate) mod commands;
pub(crate) mod objects;

/// Doc comment
#[derive(Debug, Subcommand)]
enum Command {
    /// Doc comment
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,

        object_hash: String,
    },
    HashObject {
        #[clap(short = 'w')]
        write: bool,

        file: PathBuf,
    },
    LsTree {
        #[clap(long)]
        name_only: bool,

        tree_hash: String,
    },

    WriteTree,
    CommitTree {
        #[clap(short = 'm')]
        message: String,
        tree_hash: String,
        #[clap(short = 'p')]
        parent_hash: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
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
    }
    Ok(())
}
