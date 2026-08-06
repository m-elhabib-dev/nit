use std::{
    env,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::{Context, Ok};

use crate::{commands::ls_tree, objects::Object};

// ├── copy_git()
// ├── resolve_head()
// ├── read_commit()
// ├── checkout_tree()
// ├── write_index()
// └── invoke()

fn copy_git(src: &Path, dst: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(
            src_path
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("no file name"))?,
        );
        if src_path.is_dir() {
            copy_git(src_path.as_path(), &dst_path)?;
        } else {
            let copy = fs::copy(src_path, dst_path)?;
        }
    }
    Ok(())
}

fn resolve_head(dst: &Path) -> anyhow::Result<String> {
    let head_path = PathBuf::from(".git/HEAD");
    let head = fs::read_to_string(head_path)?;

    let mut reference = String::from(".git/");
    reference.push_str(head.trim().trim_start_matches("ref: "));

    let commit_path = PathBuf::from(reference);

    let commit = fs::read_to_string(&commit_path)
        .with_context(|| format!("file {} doesnt exist", commit_path.display()))?;
    let commit = commit.trim();
    Ok(commit.to_string())
}

fn read_commit(commit: String) -> anyhow::Result<()> {
    let mut object = Object::read(&commit).context("couldn't read the commit")?;
    let mut commit_text = String::new();
    object.reader.read_to_string(&mut commit_text)?;
    let commit_lines: Vec<&str> = commit_text.split("\n").collect();
    let mut tree_hash = String::new();
    let mut parent_hash = String::new();
    for line in commit_lines {
        if line.starts_with("tree") {
            tree_hash.push_str(line.trim().trim_start_matches("tree "));
        } else if line.starts_with("parent") {
            parent_hash.push_str(line.trim().trim_start_matches("parent "));
        }
    }
    ls_tree::invoke(false, &tree_hash)?;
    Ok(())
}

pub(crate) fn invoke(url: String) -> anyhow::Result<()> {
    let src = Path::new(&url);
    let dst = src
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("invalid repository path"))?;
    let dst = Path::new(dst);
    if dst.exists() {
        anyhow::bail!("destination '{}' already exists", dst.display());
    }
    if !src.join(".git").is_dir() {
        anyhow::bail!("not a git repository");
    }

    println!("Cloning into '{}'...", dst.display());
    copy_git(src, dst)?;
    println!("Done.");

    let cwd = env::current_dir()?;
    println!("current working dir before = {}", cwd.display());
    env::set_current_dir(cwd.join(dst))?;

    let cwd = env::current_dir()?;
    println!("current working dir after = {}", cwd.display());
    println!("resolving HEAD");
    let current_commit = resolve_head(dst)?;
    println!("resolved");
    println!("reading current commit {current_commit} started");
    read_commit(current_commit)?;
    println!("read");
    Ok(())
}
