use std::fs::File;
use std::io::Write;

const INDEX_SIGNATURE: [u8; 4] = *b"DIRC";
const INDEX_VERSION: u32 = 2;

pub struct IndexHeader {
    pub entries: u32,
}

impl IndexHeader {
    pub fn write(&self, mut writer: impl Write) -> std::io::Result<()> {
        writer.write_all(&INDEX_SIGNATURE)?;
        writer.write_all(&INDEX_VERSION.to_be_bytes())?;
        writer.write_all(&self.entries.to_be_bytes())?;
        Ok(())
    }
    pub fn new(entries: u32) -> Self {
        Self { entries }
    }
}

pub(crate) fn invoke() -> anyhow::Result<()> {
    let header = IndexHeader::new(0);
    let file = File::create("../.git/myindex")?;
    header.write(file)?;

    Ok(())
}
