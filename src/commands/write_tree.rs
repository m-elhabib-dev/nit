use anyhow::Context;
use anyhow::Ok;
use sha1::{Digest, Sha1};
use std::fs;
use std::io;
use std::io::Cursor;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::objects::{Kind, Object};

fn write_tree_for(path: &Path) -> anyhow::Result<Option<[u8; 20]>> {
    let mut dir = fs::read_dir(path).with_context(|| format!("open directory {}", path.display()));

    let mut tree_object = Vec::new();

    while let Some(entry) = dir?.next() {
        let entry = entry.with_context(|| format!("bad directory entry in {}", path.display()))?;

        let file_name = entry.file_name();
        let meta = entry.metadata().context("metadata for directory entry");
        let mode = if meta.is_dir() {
            "040000"
        } else if meta.is_symlink {
            "120000"
        } else if (meta.permission().mode() & 0o111) != 0 {
            "100755"
        } else {
            "100644"
        };
        let path = entry.path();
        let hash = if meta.is_dir() {
            let Some(hash) = write_tree_for(&path)? else {
                continue;
            };
            hash
        } else {
            let tmp = "temporary";
            let hash = Object::blob_from_file(&path)
                .context("open blob input file")?
                .write(std::fs::File::create(tmp).context("construct empty file for tree")?)
                .context("stream tree object into tree object file")?;
            let hash_hex = hex::encode(hash);
            fs::create_dir_all(format!(".git/objects/{}/", &hash_hex[..2]))
                .context("create subdir of .git/objects")?;
            std::fs::rename(
                tmp,
                format!(".git/objects/{}/{}", &hash_hex[..2], &hash_hex[2..]),
            )
            .context("move tree file into .git/objects")?;
            hash
        };
        tree_object.extend(mode.as_bytes());
        tree_object.push(b' ');
        tree_object.extend(file_name.as_encoded_bytes());
        tree_object.push(0);
        tree_object.extend(hash);
    }
    if tree_object.is_empty() {
        Ok(None)
    } else {
        let tmp = "temporary";
        let hash = Object {
            kind: Kind::Tree,
            expected_size: tree_object.len(),
            reader: Cursor::new(tree_object),
        }
        .write(std::fs::File::create(tmp).context("construct empty file for tree")?)
        .context("stream tree object into tree object file ")?;
        let hash_hex = hex::encode(hash);
        fs::create_dir_all(format!(".git/objects/{}/", &hash_hex[..2]))
            .context("create subdir of .git/objects")?;
        std::fs::rename(
            tmp,
            format!(".git/objects/{}/{}", &hash_hex[..2], &hash_hex[2..]),
        )
        .context("move tree file into .git/objects")?;
        Ok(Some(hash))
    }
}

pub(crate) fn invoke() -> anyhow::Result<()> {
    fn write_blob<W>(file: &Path, writer: W) -> anyhow::Result<String>
    where
        W: Write,
    {
        let stat = std::fs::metadata(file).with_context(|| format!("stat {}", file.display()))?;

        //TODO: technically there is a race here if the file changed between  the stat and write
        let file =
            std::fs::File::open(&file).with_context(|| format!("stat {}", file.display()))?;
        let hash = Object {
            kind: Kind::Blob,
            expected_size: stat.len() as usize,
            reader: file,
        }
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
