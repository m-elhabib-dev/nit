use anyhow::Context;
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use super::{Kind, Object};

impl Object<()> {
    pub(crate) fn blob_from_file(file: impl AsRef<Path>) -> anyhow::Result<Object<impl Read>> {
        let file = file.as_ref();

        let stat = std::fs::metadata(file).with_context(|| format!("stat {}", file.display()))?;

        //TODO: technically there is a race here if the file changed between  the stat and write
        let file = std::fs::File::open(file).with_context(|| format!("stat {}", file.display()))?;
        Ok(Object {
            kind: Kind::Blob,
            expected_size: stat.len() as usize,
            reader: file,
        })
    }

    pub(crate) fn read(hash: &str) -> anyhow::Result<Object<impl BufRead>> {
        let f = std::fs::File::open(format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
            .context("open in .git/objects")?;
        let z = ZlibDecoder::new(f);
        let mut z = BufReader::new(z);
        let mut buf = Vec::new();
        z.read_until(0, &mut buf)
            .context("read header from .git/objects")?;
        let header = CStr::from_bytes_with_nul(&buf)
            .expect("know there is exactly one nul,  and its at the end");
        let header = header
            .to_str()
            .context(".git/objects file header isn't valid UTF-8")?;
        let Some((kind, size)) = header.split_once(' ') else {
            anyhow::bail!(".git/objects file header did not start with a known type: '{header}'",);
        };
        let kind = match kind {
            "blob" => Kind::Blob,
            "tree" => Kind::Tree,
            "commit" => Kind::Commit,
            _ => anyhow::bail!("what even is a '{kind}'"),
        };
        let size = size
            .parse::<usize>()
            .context(".git/objects file header has invalid size: {size}")?;

        //NOTE: this won't error if decompressed file is too long, but at least not spam stdout
        //      and be vulnerable to a zipbomb.
        let z = z.take(size as u64);

        Ok(Object {
            kind,
            expected_size: size,
            reader: z,
        })
    }
}
