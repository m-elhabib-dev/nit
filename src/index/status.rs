use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

use crate::index::index::Index;
use crate::index::{add::collect_entries, entry::IndexEntry};
use anyhow::{Context, Ok};
use colored::Colorize;

pub struct Status {
    modified: Vec<PathBuf>,
    untracked: Vec<PathBuf>,
    deleted: Vec<PathBuf>,
}

pub(crate) fn invoke() -> anyhow::Result<()> {
    let index_path = File::open(".git/index")?;
    let index = Index::read(&index_path).context("couldn't read the index file")?;
    let index_entries = index.entries;

    let working_path = PathBuf::from(".");
    let entries = vec![(working_path, std::fs::metadata(PathBuf::from("."))?)];
    let working_entries = collect_entries(entries)?;

    let index: HashMap<PathBuf, IndexEntry> = index_entries
        .into_iter()
        .map(|e| (e.path.clone(), e))
        .collect();

    let working: HashMap<PathBuf, IndexEntry> = working_entries
        .into_iter()
        .map(|e| (e.path.clone(), e))
        .collect();
    let status = compare(index, working)?;
    print_status(status);
    Ok(())
}

fn compare(
    index: HashMap<PathBuf, IndexEntry>,
    working: HashMap<PathBuf, IndexEntry>,
) -> anyhow::Result<Status> {
    let mut status = Status {
        modified: Vec::new(),
        untracked: Vec::new(),
        deleted: Vec::new(),
    };

    for (path, working_entry) in &working {
        match index.get(path) {
            None => {
                status.untracked.push(path.clone());
            }
            Some(index_entry) => {
                if index_entry != working_entry {
                    status.modified.push(path.clone());
                }
            }
        }
    }

    for path in index.keys() {
        if !working.contains_key(path) {
            status.deleted.push(path.clone());
        }
    }

    Ok(status)
}

pub fn print_status(status: Status) {
    println!("On branch main");
    println!("Your branch is up to date with 'origin/main'.");
    println!();

    if status.modified.is_empty() && status.deleted.is_empty() && status.untracked.is_empty() {
        println!("nothing to commit, working tree clean");
        return;
    }

    if !status.modified.is_empty() || !status.deleted.is_empty() {
        println!("{}", "Changes not staged for commit:".red());
        println!("  (use \"nit add <file>...\" to update what will be committed)");
        println!("  (use \"nit restore <file>...\" to discard changes in working directory)");
        println!();

        print_entries("modified:", &status.modified);
        print_entries("deleted:", &status.deleted);

        println!();
    }

    if !status.untracked.is_empty() {
        println!("{}", "Untracked files:".red());
        println!("  (use \"nit add <file>...\" to include in what will be committed)");
        println!();

        for path in &status.untracked {
            println!("        {}", path.display().to_string().red());
        }

        println!();
    }

    println!("no changes added to commit (use \"nit add\" and/or \"nit commit\")");
}

fn print_entries(label: &str, paths: &[PathBuf]) {
    for path in paths {
        println!("        {:<10} {}", label.red(), path.display());
    }
}
