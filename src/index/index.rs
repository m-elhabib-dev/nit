use std::io::Read;

use crate::index::entry::IndexEntry;
use crate::index::header::IndexHeader;
use anyhow::Ok;

pub struct Index {
    pub header: IndexHeader,
    pub entries: Vec<IndexEntry>,
}

impl Index {
    pub fn read(mut reader: impl Read) -> anyhow::Result<Index> {
        let header = IndexHeader::read(&mut reader)?;
        let mut entries = Vec::new();
        for _ in 0..header.entries {
            let entry = IndexEntry::read(&mut reader)?;
            entries.push(entry);
        }
        let mut checksum = [0u8; 20];
        reader.read_exact(&mut checksum)?;
        Ok(Index { header, entries })
    }
}
