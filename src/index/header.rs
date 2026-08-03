use std::fs::{File, Metadata};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Ok};
use sha1::{Digest, Sha1};

use crate::index::entry::IndexEntry;
use crate::objects::Object;
use crate::objects::walk::read_dir_sorted;

const INDEX_SIGNATURE: [u8; 4] = *b"DIRC";
const INDEX_VERSION: u32 = 2;
pub const HEADER_SIZE: usize = 12;

pub struct IndexHeader {
    pub entries: u32,
}

impl IndexHeader {
    pub fn new(entries: u32) -> Self {
        Self { entries }
    }
    pub fn write(&self, mut writer: impl Write) -> anyhow::Result<()> {
        writer.write_all(&INDEX_SIGNATURE)?;
        writer.write_all(&INDEX_VERSION.to_be_bytes())?;
        writer.write_all(&self.entries.to_be_bytes())?;
        Ok(())
    }
    pub fn read(mut reader: impl Read) -> anyhow::Result<IndexHeader> {
        // Header buffer
        let mut header = [0u8; HEADER_SIZE];
        // read header
        reader
            .read_exact(&mut header)
            .context("cannot read headr")?;
        // parse version and entries count from the header buffer
        let version = u32::from_be_bytes(header[4..8].try_into()?);
        let entries_count = u32::from_be_bytes(header[8..12].try_into()?);
        // verify the header signature and version
        if !(header[0..4] == INDEX_SIGNATURE && version == INDEX_VERSION) {
            anyhow::bail!("incorrect header of the index file");
        }
        // print header
        println!("Signature: {}", std::str::from_utf8(&INDEX_SIGNATURE)?);
        println!("Version: {INDEX_VERSION}");
        println!("Entries: {entries_count}");

        // create new index header with entries count
        let indexheader = IndexHeader::new(entries_count);
        Ok(indexheader)
    }
    pub fn read_entries(mut reader: impl Read) -> anyhow::Result<Vec<IndexEntry>> {
        let header = IndexHeader::read(&mut reader)?;
        let mut entries = Vec::new();
        for i in 0..header.entries {
            let entry = IndexEntry::read(&mut reader)?;
            println!("entry: {}", i + 1);
            println!("ctime_sec: {}", entry.ctime_sec);
            println!("ctime_nsec: {}", entry.ctime_nsec);
            println!("mtime_sec: {}", entry.mtime_sec,);
            println!("mtime_nsec: {}", entry.mtime_nsec);
            println!("dev: {}", entry.dev);
            println!("ino: {}", entry.ino);
            println!("mode: {}", entry.mode);
            println!("uid: {}", entry.uid);
            println!("gid: {}", entry.gid);
            println!("size: {}", entry.size);
            println!("oid: {:?}", entry.oid);
            println!("flags: {}", entry.flags);
            println!("path: {} \n", entry.path.display());
            entries.push(entry);
        }
        let mut checksum = [0u8; 20];
        reader.read_exact(&mut checksum)?;
        println!("checksum: {:?}", checksum);
        Ok(entries)
    }
}

fn collect_entries(entries: Vec<(PathBuf, Metadata)>) -> anyhow::Result<Vec<IndexEntry>> {
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
    std::fs::write(".git/myindex", buffer)?;
    let p = File::open(".git/myindex")?;
    let _ = IndexHeader::read_entries(&p)?;

    Ok(())
}
