use sha1::{Digest, Sha1};
use std::io;
use std::io::Write;

pub(super) struct HashWriter<W> {
    pub(super) writer: W,
    pub(super) hasher: Sha1,
}

impl<W> Write for HashWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n: usize = self.writer.write(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
