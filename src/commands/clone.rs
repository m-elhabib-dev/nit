use flate2::read::{ZlibDecoder, ZlibEncoder};
use reqwest;
use std::{
    env,
    fmt::format,
    fs::{self, File, create_dir_all},
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::{Context, Ok};

use crate::{
    commands::ls_tree::{TreeEntry, read_tree},
    objects::{Kind, Object},
};

fn copy_git(src: &Path, dst: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(
            src_path
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("no file name"))?,
        );
        if src_path.is_dir() {
            copy_git(src_path.as_path(), &dst_path)?;
        } else {
            fs::copy(src_path, dst_path)?;
        }
    }
    Ok(())
}

fn resolve_head() -> anyhow::Result<String> {
    let head_path = PathBuf::from(".git/HEAD");
    let head = fs::read_to_string(head_path)?;

    let mut reference = String::from(".git/");
    reference.push_str(head.trim().trim_start_matches("ref: "));

    let commit_path = PathBuf::from(reference);

    let commit = fs::read_to_string(&commit_path)
        .with_context(|| format!("file {} doesnt exist", commit_path.display()))?;
    let commit = commit.trim();
    Ok(commit.to_string())
}

fn read_commit(commit: String) -> anyhow::Result<Vec<TreeEntry>> {
    let mut object = Object::read(&commit).context("couldn't read the commit")?;
    let mut commit_text = String::new();
    object.reader.read_to_string(&mut commit_text)?;
    let commit_lines: Vec<&str> = commit_text.split("\n").collect();
    let mut tree_hash = String::new();
    let mut parent_hash = String::new();
    for line in commit_lines {
        if line.starts_with("tree") {
            tree_hash.push_str(line.trim().trim_start_matches("tree "));
        } else if line.starts_with("parent") {
            parent_hash.push_str(line.trim().trim_start_matches("parent "));
        }
    }
    let object = Object::read(&tree_hash).context("parse out tree object file")?;
    let entries = read_tree(object)?;
    Ok(entries)
}

fn write_content(entries: Vec<TreeEntry>, cwd: &Path) -> anyhow::Result<()> {
    for entry in entries {
        let hash = entry.hash;
        let hash = hex::encode(hash);
        let mut object = Object::read(&hash)?;
        if entry.kind == Kind::Blob {
            let mut dist = File::create(cwd.join(&entry.name))?;
            std::io::copy(&mut object.reader, &mut dist)?;
        } else if entry.kind == Kind::Tree {
            let dir = cwd.join(&entry.name);
            create_dir_all(&dir)?;
            let entries = read_tree(object)?;
            write_content(entries, &dir)?;
        }
    }
    Ok(())
}

fn fetch_url(url: String) -> anyhow::Result<()> {
    let mut get_url = url.clone();
    get_url.push_str("/info/refs?service=git-upload-pack");
    let response = reqwest::blocking::get(&get_url)?;
    let response = response.text()?;

    let mut sha = None;

    for line in response.lines() {
        if line.contains(" HEAD") {
            sha = Some(line[8..48].to_string());
            break;
        }
    }
    let sha = sha.ok_or_else(|| anyhow::anyhow!("HEAD not found"))?;
    let post_url = format!("{url}/git-upload-pack");
    let mut body = pkt_line(&format!("want {sha}\n"));
    body.extend_from_slice(b"0000");
    body.extend_from_slice(&pkt_line("done\n"));
    let client = reqwest::blocking::Client::new();

    let res = client
        .post(post_url)
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-git-upload-pack-request",
        )
        .body(body)
        .send()?;

    let bytes = res.bytes()?;
    fs::write("response.pack", &bytes)?;
    println!("response size = {}", bytes.len());
    println!("{:02x?}", &bytes[..bytes.len().min(100)]);
    let pack_start = bytes
        .windows(4)
        .position(|window| window == b"PACK")
        .ok_or_else(|| anyhow::anyhow!("PACK not found"))?;
    let pack = &bytes[pack_start..];

    let name = &pack[..4];

    if name != b"PACK" {
        anyhow::bail!("Not PACK");
    }
    let object_count = u32::from_be_bytes(pack[8..12].try_into()?);
    let mut index = 12;
    for _ in 0..object_count {
        index = read_pack_object(&pack, index)?;
    }
    Ok(())
}

fn pkt_line(data: &str) -> Vec<u8> {
    let mut line = Vec::new();
    let len = data.len() + 4;
    let header = format!("{len:04x}");
    line.extend_from_slice(header.as_bytes());
    line.extend_from_slice(data.as_bytes());
    line
}

//INFO: this function it totally written by AI with my guides
fn read_pack_object(pack: &[u8], start: usize) -> anyhow::Result<usize> {
    let mut index = start;

    let first = pack[index];
    index += 1;

    let object_type = (first >> 4) & 0x07;
    let mut size = (first & 0x0f) as u64;

    let mut shift = 4;
    let mut byte = first;

    while byte & 0x80 != 0 {
        byte = pack[index];
        index += 1;

        size += ((byte & 0x7f) as u64) << shift;
        shift += 7;
    }
    //
    // let compressed = &pack[index..];
    //
    // let mut decoder = ZlibDecoder::new(compressed);
    // let mut content = Vec::new();
    //
    // decoder.read_to_end(&mut content)?;
    //
    // let consumed = decoder.total_in() as usize;
    //
    // println!("type: {object_type}");
    // println!("size: {size}");
    // println!("decompressed: {}", content.len());
    // println!("compressed: {consumed}");
    //
    //
    println!("type: {object_type}");
    println!("size: {size}");

    let compressed = &pack[index..];

    let mut decoder = ZlibDecoder::new(compressed);
    let mut content = Vec::new();

    match decoder.read_to_end(&mut content) {
        std::result::Result::Ok(_) => {
            let consumed = decoder.total_in() as usize;

            println!("decompressed: {}", content.len());
            println!("compressed: {consumed}");

            Ok(index + consumed)
        }
        Err(err) => {
            println!("zlib error: {err}");
            println!("object starts at: {start}");
            println!("type: {object_type}");
            Err(err.into())
        }
    }
}

pub(crate) fn invoke(url: String) -> anyhow::Result<()> {
    if url.starts_with("http") {
        //println!("clonning from remote...");
        //fetch_url(url)?;
        //
        anyhow::bail!("We dont support remote clonning yet, try local cloning");
    } else {
        let src = Path::new(&url);
        let dst = src
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("invalid repository path"))?;
        let dst = Path::new(dst);
        if dst.exists() {
            anyhow::bail!("destination '{}' already exists", dst.display());
        }
        if !src.join(".git").is_dir() {
            anyhow::bail!("not a git repository");
        }
        println!("Cloning into '{}'...", dst.display());
        copy_git(src, dst)?;
        let cwd = env::current_dir()?.join(dst);
        env::set_current_dir(&cwd)?;
        println!("resolving HEAD...");
        let current_commit = resolve_head()?;
        println!("reading entries...");
        let entries = read_commit(current_commit)?;
        println!("writing index...");
        write_content(entries, &cwd)?;
        println!("cloning done.");
    }
    Ok(())
}
