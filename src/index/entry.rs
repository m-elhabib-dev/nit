use anyhow::Ok;
use std::cmp::min;
use std::fs::Metadata;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

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
}
