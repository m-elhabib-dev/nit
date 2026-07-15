use anyhow::Context;
use clap::{Parser, Subcommand};
use sha1::{Digest, Sha1};
use std::ffi::CStr;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::{
    fs,
    io::{BufRead, BufReader},
};

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

/// Doc comment
#[derive(Debug, Subcommand)]
enum Command {
    /// Doc comment
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,

        object_hash: String,
    },
    HashObject {
        #[clap(short = 'w')]
        write: bool,

        file: PathBuf,
    },
}

enum Kind {
    Blob,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Intialized git directory");
        }
        Command::CatFile {
            pretty_print,
            object_hash,
        } => {
            anyhow::ensure!(
                pretty_print,
                "mode must be given without -p, and we donot support mode"
            );
            //TODO: support shortest-unique object hashes
            let f = std::fs::File::open(format!(
                ".git/objects/{}/{}",
                &object_hash[..2],
                &object_hash[2..]
            ))
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
                anyhow::bail!(
                    ".git/objects file header did not start with a known type: '{header}'",
                );
            };
            let kind = match kind {
                "blob" => Kind::Blob,
                _ => anyhow::bail!("we donot know ho to print a '{kind}'"),
            };
            let size = size
                .parse::<usize>()
                .context(".git/objects file header has invalid size: {size}")?;

            //NOTE: this won't error if decompressed file is too long, but at least not spam stdout
            //      and be vulnerable to a zipbomb.
            let mut z = z.take(size as u64);
            match kind {
                Kind::Blob => {
                    let stdout = std::io::stdout();
                    let mut stdout = stdout.lock();
                    let n = std::io::copy(&mut z, &mut stdout)
                        .context("write .git/objects to stdout")?;
                    anyhow::ensure!(
                        n as usize == size,
                        ".git/object file was not the expected size (expected: {size}, actual: {n})"
                    );
                }
            }
        }

        Command::HashObject { write, file } => {
            fn write_blob<W>(file: &Path, writer: W) -> anyhow::Result<String>
            where
                W: Write,
            {
                let stat =
                    std::fs::metadata(file).with_context(|| format!("stat {}", file.display()))?;
                let writer = HashWriter {
                    writer,
                    hasher: Sha1::new(),
                };

                let mut e = ZlibEncoder::new(writer, Compression::default());

                write!(e, "blob ")?;
                write!(e, "{}\0", stat.len())?;

                let compressed = e.finish()?;
                let hash = compressed.hasher.finalize();
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
        }
    }
    Ok(())
}

//struct LimitReader<R> {
//    reader: R,
//    limit: usize,
//}
//
//impl<R> Read for LimitReader<R>
//where
//    R: Read,
//{
//    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
//        if buf.len() > self.limit {
//            buf = &mut buf[..self.limit + 1];
//        }
//        let n = self.reader.read(buf)?;
//        if n > self.limit {
//            return Err(io::Error::new(io::ErrorKind::Other, "too many bytes"));
//        }
//        self.limit -= n;
//        Ok(n)
//    }
//}

struct HashWriter<W> {
    writer: W,
    hasher: Sha1,
}

impl<W> Write for HashWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.writer.write(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
