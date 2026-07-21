use crate::objects::{Kind, Object};
use anyhow::Context;
use std::fmt::Write;
use std::io::Cursor;

pub(crate) fn invoke(
    message: String,
    tree_hash: String,
    parent_hash: Option<String>,
) -> anyhow::Result<()> {
    let mut commit = String::new();
    writeln!(commit, "tree {tree_hash}")?;

    if let Some(parent_hash) = parent_hash {
        writeln!(commit, "parent {parent_hash}")?;
    }

    writeln!(
        commit,
        "author m-alhbyb <mohammedalhbyb@gmail.com> 1784646297 +0200"
    )?;
    writeln!(
        commit,
        "committer m-alhbyb <mohammedalhbyb@gmail.com> 1784646297 +0200"
    )?;
    writeln!(commit, "")?;
    writeln!(commit, "{message}")?;
    writeln!(commit, "")?;

    let hash = Object {
        kind: Kind::Commit,
        expected_size: commit.len(),
        reader: Cursor::new(commit),
    }
    .write_to_objects()
    .context("write commit object")?;

    println!("{}", hex::encode(hash));

    Ok(())
}
