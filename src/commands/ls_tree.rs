use std::{
    ffi::{CStr, OsString},
    io::{BufRead, Read, Write},
    os::unix::ffi::OsStringExt,
};

use crate::objects::{Kind, Object};
use anyhow::{Context, Ok};

#[derive(Debug)]
pub(crate) struct TreeEntry {
    pub(crate) mode: String,
    pub(crate) name: OsString,
    pub(crate) hash: [u8; 20],
    pub(crate) kind: Kind,
}

pub(crate) fn read_tree(object: Object<impl BufRead>) -> anyhow::Result<Vec<TreeEntry>> {
    let mut object = object;
    let mut entries = Vec::new();
    let mut buf = Vec::new();
    let mut hashbuf = [0; 20];
    loop {
        buf.clear();
        let n = object
            .reader
            .read_until(0, &mut buf)
            .context("read next tree object entry")?;
        if n == 0 {
            break;
        }

        object
            .reader
            .read_exact(&mut hashbuf[..])
            .context("read tree entry object hash")?;

        let hash = hex::encode(hashbuf);
        let object = Object::read(&hash)?;

        let mode_and_name = CStr::from_bytes_with_nul(&buf).context("invalid tree entry")?;
        let mut bits = mode_and_name.to_bytes().splitn(2, |&b| b == b' ');

        //TODO replace with split_once
        let mode = bits.next().expect("split always yields once");
        let name = bits
            .next()
            .ok_or_else(|| anyhow::anyhow!("tree entry has no filename"))?;

        let mode = std::str::from_utf8(mode).context("mode is always valid utf-8")?;

        let name = OsString::from_vec(name.to_vec());
        entries.push(TreeEntry {
            mode: mode.to_string(),
            name,
            hash: hashbuf,
            kind: object.kind,
        });
    }
    Ok(entries)
}
pub(crate) fn invoke(name_only: bool, tree_hash: &str) -> anyhow::Result<()> {
    let object = Object::read(tree_hash).context("parse out tree object file")?;
    let mut entries: Vec<TreeEntry> = Vec::new();
    match object.kind {
        Kind::Tree => {
            entries = read_tree(object)?;
        }
        _ => anyhow::bail!("don't yet know how to ls {}", object.kind),
    }
    for entry in entries {
        if name_only {
            println!("{}", entry.name.display());
        } else {
            let mode = entry.mode;
            let kind = entry.kind.to_string();
            let hash = hex::encode(entry.hash);
            let name = entry.name.display();
            println!("{:0>6} {kind} {:?}    {name}\n", mode, hash);
        }
    }
    Ok(())
}
