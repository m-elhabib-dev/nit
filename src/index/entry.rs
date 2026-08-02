use std::cmp::min;
use std::io::Write;

use anyhow::Ok;

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
    pub path: String,
}

impl IndexEntry {
    pub(crate) fn new(oid: [u8; 20], path: String) -> Self {
        Self {
            ctime_sec: 0,
            ctime_nsec: 0,
            mtime_sec: 0,
            mtime_nsec: 0,
            dev: 0,
            ino: 0,
            mode: 0,
            uid: 0,
            gid: 0,
            size: 0,
            oid,
            flags: min(path.len() as u16, 0x0fff),
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
        let flags = min(self.path.len() as u16, 0x0fff);
        writer.write_all(&flags.to_be_bytes())?;
        writer.write_all(self.path.as_bytes())?;
        writer.write_all(&[0])?;
        let entry_size = 62 + self.path.len() + 1;
        let padding = (8 - (entry_size % 8)) % 8;
        for _ in 0..padding {
            writer.write_all(&[0])?;
        }
        Ok(())
    }
}
