use std::fs::File;
use std::io::{Read, Write};

use anyhow::{Context, Ok};

use crate::index::entry::IndexEntry;
use crate::objects::Object;

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
        let mut header = [0u8; HEADER_SIZE];
        reader
            .read_exact(&mut header)
            .context("cannot read headr")?;

        let version = u32::from_be_bytes(header[4..8].try_into()?);
        let entries = u32::from_be_bytes(header[8..12].try_into()?);

        if !(header[0..4] == INDEX_SIGNATURE && version == INDEX_VERSION) {
            anyhow::bail!("incorrect file");
        }
        println!("Signature: {}", std::str::from_utf8(&INDEX_SIGNATURE)?);
        println!("Version: {INDEX_VERSION}");
        println!("Entries: {entries}");
        let indexheader = IndexHeader::new(entries);
        Ok(indexheader)
    }
}

pub(crate) fn invoke() -> anyhow::Result<()> {
    let path = ".git/myindex";

    let header = IndexHeader::new(1);

    let object = Object::blob_from_file(path)?;
    let oid = object.write_to_objects()?;
    let entry = IndexEntry::new(oid, path.to_owned());

    let mut index = File::create(path)?;

    header.write(&mut index)?;
    entry.write(&mut index)?;

    Ok(())
}
