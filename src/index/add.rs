use std::fs::Metadata;
use std::path::{Path, PathBuf};

use anyhow::{Context, Ok};
use sha1::{Digest, Sha1};

use crate::index::entry::IndexEntry;
use crate::index::header::IndexHeader;
use crate::objects::Object;
use crate::objects::walk::read_dir_sorted;

pub(crate) fn collect_entries(
    entries: Vec<(PathBuf, Metadata)>,
) -> anyhow::Result<Vec<IndexEntry>> {
    let mut entries_vec = Vec::new();
    for (entry_path, meta) in entries {
        if meta.is_dir() {
            let children = read_dir_sorted(&entry_path)?;
            let nested = collect_entries(children)?;
            entries_vec.extend(nested);
        } else {
            let entry_path = entry_path.strip_prefix(".")?.to_path_buf();
            let object = Object::blob_from_file(&entry_path)?;
            let oid = object.write_to_objects()?;
            let entry = IndexEntry::new(oid, entry_path, &meta);
            entries_vec.push(entry);
        }
    }
    Ok(entries_vec)
}

pub(crate) fn invoke(path: PathBuf) -> anyhow::Result<()> {
    let buffer = create_index(path)?;
    std::fs::write(".git/index", buffer)?;
    Ok(())
}

pub(crate) fn create_index(path: PathBuf) -> anyhow::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    if path == Path::new(".") {
        let entries = read_dir_sorted(&path)
            .with_context(|| format!("could not read path {}", path.display()))?;
        let entries_result = collect_entries(entries)?;
        let header = IndexHeader::new(entries_result.len() as u32);
        header.write(&mut buffer)?;
        for entry in entries_result {
            entry.write(&mut buffer)?;
        }
    } else {
        let header = IndexHeader::new(1);
        header.write(&mut buffer)?;
        let meta = std::fs::metadata(&path)?;
        let object = Object::blob_from_file(&path)?;
        let oid = object.write_to_objects()?;
        let entry = IndexEntry::new(oid, path, &meta);
        entry.write(&mut buffer)?;
    }
    let checksum = Sha1::digest(&buffer);
    buffer.extend_from_slice(&checksum);
    Ok(buffer)
}
