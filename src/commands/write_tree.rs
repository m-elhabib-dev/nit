use anyhow::Context;
use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::{cmp::Ordering, fs};

use crate::objects::{Kind, Object};

fn write_tree_for(path: &Path) -> anyhow::Result<Option<[u8; 20]>> {
    let mut dir =
        fs::read_dir(path).with_context(|| format!("open directory {}", path.display()))?;

    let mut entries = Vec::new();

    while let Some(entry) = dir.next() {
        let entry = entry.with_context(|| format!("bad directory entry in {}", path.display()))?;
        let name = entry.file_name();
        let meta = entry.metadata().context("metadata for directory entry")?;
        entries.push((entry, name, meta));
    }

    entries.sort_unstable_by(|a, b| {
        let afn = &a.1;
        let afn = afn.as_encoded_bytes();
        let bfn = &b.1;
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
        } else if a.2.is_dir() {
            Some(b'/')
        } else {
            None
        };

        let c2 = if let Some(c) = bfn.get(common_len).copied() {
            Some(c)
        } else if b.2.is_dir() {
            Some(b'/')
        } else {
            None
        };
        c1.cmp(&c2)
    });

    let mut tree_object = Vec::new();
    for (entry, file_name, meta) in entries {
        if file_name == ".git" || file_name == "target" {
            continue;
        }

        //     .metadata()
        //     .with_context(|| format!("metadata for {}", entry_path.display()))?;
        let mode = if meta.is_dir() {
            "40000"
        } else if meta.is_symlink() {
            "120000"
        } else if (meta.permissions().mode() & 0o111) != 0 {
            "100755"
        } else {
            "100644"
        };
        let path = entry.path();
        let hash = if meta.is_dir() {
            let Some(hash) = write_tree_for(&path)? else {
                continue;
            };
            hash
        } else {
            let tmp = "temporary";
            let hash = Object::blob_from_file(&path)
                .context("open blob input file")?
                .write(std::fs::File::create(tmp).context("construct empty file for tree")?)
                .context("stream tree object into tree object file")?;
            let hash_hex = hex::encode(hash);
            fs::create_dir_all(format!(".git/objects/{}/", &hash_hex[..2]))
                .context("create subdir of .git/objects")?;
            std::fs::rename(
                tmp,
                format!(".git/objects/{}/{}", &hash_hex[..2], &hash_hex[2..]),
            )
            .context("move tree file into .git/objects")?;
            hash
        };
        tree_object.extend(mode.as_bytes());
        tree_object.push(b' ');
        tree_object.extend(file_name.as_encoded_bytes());
        tree_object.push(0);
        tree_object.extend(hash);
    }
    if tree_object.is_empty() {
        Ok(None)
    } else {
        Ok(Some(
            Object {
                kind: Kind::Tree,
                expected_size: tree_object.len(),
                reader: Cursor::new(tree_object),
            }
            .write_to_objects()
            .context("write tree object")?,
        ))
    }
}

pub(crate) fn invoke() -> anyhow::Result<()> {
    let Some(hash) = write_tree_for(Path::new(".")).context("construct root tree object")? else {
        anyhow::bail!("ask to make tree object for empty tree");
    };
    println!("{}", hex::encode(hash));
    Ok(())
}
