use anyhow::{Context, Ok};
use std::cmp::min;
use std::ffi::OsString;
use std::fs::Metadata;
use std::io::{Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

const FIXED_SIZE: usize = 62;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndexEntry {
    pub ctime_sec: u32,
    pub ctime_nsec: u32,
    pub mtime_sec: u32,
    pub mtime_nsec: u32,
    pub dev: u32,
    pub ino: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u32,
    pub oid: [u8; 20],
    pub flags: u16,
    pub path: PathBuf,
}

impl IndexEntry {
    pub(crate) fn new(oid: [u8; 20], path: PathBuf, meta: &Metadata) -> Self {
        Self {
            ctime_sec: meta.ctime() as u32,
            ctime_nsec: meta.ctime_nsec() as u32,
            mtime_sec: meta.mtime() as u32,
            mtime_nsec: meta.mtime_nsec() as u32,
            dev: meta.dev() as u32,
            ino: meta.ino() as u32,
            mode: meta.mode(),
            uid: meta.uid(),
            gid: meta.gid(),
            size: meta.size() as u32,
            oid,
            flags: min(path.as_os_str().as_bytes().len() as u16, 0x0fff),
            path,
        }
    }
    pub(crate) fn write(&self, mut writer: impl Write) -> anyhow::Result<()> {
        writer.write_all(&self.ctime_sec.to_be_bytes())?;
        writer.write_all(&self.ctime_nsec.to_be_bytes())?;
        writer.write_all(&self.mtime_sec.to_be_bytes())?;
        writer.write_all(&self.mtime_nsec.to_be_bytes())?;
        writer.write_all(&self.dev.to_be_bytes())?;
        writer.write_all(&self.ino.to_be_bytes())?;
        writer.write_all(&self.mode.to_be_bytes())?;
        writer.write_all(&self.uid.to_be_bytes())?;
        writer.write_all(&self.gid.to_be_bytes())?;
        writer.write_all(&self.size.to_be_bytes())?;
        writer.write_all(&self.oid)?;
        writer.write_all(&self.flags.to_be_bytes())?;
        writer.write_all(self.path.as_os_str().as_bytes())?;
        writer.write_all(&[0])?;

        let entry_size = 62 + self.path.as_os_str().as_bytes().len() + 1;
        let padding = (8 - (entry_size % 8)) % 8;
        for _ in 0..padding {
            writer.write_all(&[0])?;
        }
        Ok(())
    }
    pub(crate) fn read(mut reader: impl Read) -> anyhow::Result<IndexEntry> {
        // Header buffer
        let mut fixed = [0u8; FIXED_SIZE];
        // read header
        reader.read_exact(&mut fixed).context("cannot read entry")?;

        let ctime_sec = u32::from_be_bytes(fixed[0..4].try_into()?);
        let ctime_nsec = u32::from_be_bytes(fixed[4..8].try_into()?);
        let mtime_sec = u32::from_be_bytes(fixed[8..12].try_into()?);
        let mtime_nsec = u32::from_be_bytes(fixed[12..16].try_into()?);
        let dev = u32::from_be_bytes(fixed[16..20].try_into()?);
        let ino = u32::from_be_bytes(fixed[20..24].try_into()?);
        let mode = u32::from_be_bytes(fixed[24..28].try_into()?);
        let uid = u32::from_be_bytes(fixed[28..32].try_into()?);
        let gid = u32::from_be_bytes(fixed[32..36].try_into()?);
        let size = u32::from_be_bytes(fixed[36..40].try_into()?);
        let oid = fixed[40..60].try_into()?;
        let flags = u16::from_be_bytes(fixed[60..62].try_into()?);
        let path_len = (flags & 0x0fff) as usize;
        let mut path_bytes = vec![0; path_len];
        reader.read_exact(&mut path_bytes)?;
        let mut nul = [0u8; 1];
        reader.read_exact(&mut nul)?;
        let path = PathBuf::from(OsString::from_vec(path_bytes));

        let entry_size = 62 + path.as_os_str().as_bytes().len() + 1;
        let padding = (8 - (entry_size % 8)) % 8;
        let mut padding_buf = vec![0u8; padding];
        reader.read_exact(&mut padding_buf)?;

        let index_entry = IndexEntry {
            ctime_sec,
            ctime_nsec,
            mtime_sec,
            mtime_nsec,
            dev,
            ino,
            mode,
            uid,
            gid,
            size,
            oid,
            flags,
            path,
        };

        Ok(index_entry)
    }
}
