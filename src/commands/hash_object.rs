use crate::objects::Object;
use anyhow::Context;
use sha1::{Digest, Sha1};
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

pub(crate) fn invoke(write: bool, file: &Path) -> anyhow::Result<()> {
    fn write_blob<W>(file: &Path, writer: W) -> anyhow::Result<String>
    where
        W: Write,
    {
        let hash = Object::blob_from_file(file)
            .context("open blob input file")?
            .write(writer)
            .context("stream file into blob")?;
        Ok(hex::encode(hash))
    }

    let hash = if write {
        let tmp = "temporary";
        let hash = write_blob(
            &file,
            std::fs::File::create(tmp).context("construct temporary file for blob")?,
        )
        .context("write out blob object")?;
        fs::create_dir_all(format!(".git/objects/{}/", &hash[..2]))
            .context("create subdir of .git/objects")?;
        std::fs::rename(tmp, format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
            .context("move blob file into .git/objects")?;
        hash
    } else {
        write_blob(&file, std::io::sink()).context("write out blob object")?
    };
    println!("{hash}");

    Ok(())
}

struct HashWriter<W> {
    writer: W,
    hasher: Sha1,
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
