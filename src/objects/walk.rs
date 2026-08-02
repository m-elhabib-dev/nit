use anyhow::{Context, Ok};
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::{cmp::Ordering, fs};

pub(crate) fn read_dir_sorted(path: &Path) -> anyhow::Result<Vec<(PathBuf, Metadata)>> {
    let dir = fs::read_dir(path).with_context(|| format!("open directory {}", path.display()))?;

    let mut entries = Vec::new();

    for entry in dir {
        let entry = entry.with_context(|| format!("bad directory entry in {}", path.display()))?;
        let entry_path = entry.path();
        if entry.file_name() == ".git" || entry.file_name() == "target" {
            continue;
        }
        let meta = entry.metadata().context("metadata for directory entry")?;
        entries.push((entry_path, meta));
    }

    entries.sort_unstable_by(|a, b| {
        let afn = a.0.file_name().unwrap();
        let afn = afn.as_encoded_bytes();
        let bfn = b.0.file_name().unwrap();
        let bfn = bfn.as_encoded_bytes();
        let common_len = std::cmp::min(afn.len(), bfn.len());

        match afn[..common_len].cmp(&bfn[..common_len]) {
            Ordering::Equal => {}
            o => return o,
        }

        if afn.len() == bfn.len() {
            return Ordering::Equal;
        }

        let c1 = if let Some(c) = afn.get(common_len).copied() {
            Some(c)
        } else if a.1.is_dir() {
            Some(b'/')
        } else {
            None
        };

        let c2 = if let Some(c) = bfn.get(common_len).copied() {
            Some(c)
        } else if b.1.is_dir() {
            Some(b'/')
        } else {
            None
        };
        c1.cmp(&c2)
    });
    Ok(entries)
}
