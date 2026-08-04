use std::io::{Read, Write};

use anyhow::{Context, Ok};

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

        // create new index header with entries count
        let indexheader = IndexHeader::new(entries_count);
        Ok(indexheader)
    }
}
