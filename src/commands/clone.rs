use std::{
    env,
    fs::{self, File, create_dir_all},
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::{Context, Ok};
use flate2::read::ZlibDecoder;

use crate::{
    commands::ls_tree::{self, TreeEntry, read_tree},
    objects::{Kind, Object},
};

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

fn resolve_head() -> anyhow::Result<String> {
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

fn read_commit(commit: String) -> anyhow::Result<Vec<TreeEntry>> {
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
    let object = Object::read(&tree_hash).context("parse out tree object file")?;
    let entries = read_tree(object)?;
    Ok(entries)
}

fn write_content(entries: Vec<TreeEntry>) -> anyhow::Result<()> {
    for entry in entries {
        if entry.kind == Kind::Blob {
            let hash = entry.hash;
            let hash = hex::encode(hash);
            let mut object = Object::read(&hash)?;
            let mut dist = File::create(&entry.name)?;
            std::io::copy(&mut object.reader, &mut dist)?;
        } else if entry.kind == Kind::Tree {
            println!("Tree");
        }
    }
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
    let cwd = env::current_dir()?;
    env::set_current_dir(cwd.join(dst))?;
    let current_commit = resolve_head()?;
    let entries = read_commit(current_commit)?;
    let writen = write_content(entries)?;
    Ok(())
}
