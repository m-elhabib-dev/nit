# nit

A minimal Git reimplementation written in Rust. `nit` implements the core Git object model, index, and a subset of the plumbing and porcelain commands from scratch — no libgit2, no `git` binary underneath.

Repos created by `nit` use the same on-disk format as Git, so you can inspect them with plain `git` (e.g. `git log`, `git fsck`) and vice versa — as long as you stick to the commands `nit` supports.

## Features

* **Objects**: blobs, trees, and commits stored in `.git/objects` with SHA-1 naming and zlib compression (`flate2`)

* **Index**: binary index file compatible with Git's format (header, entries, trailing SHA-1 checksum)

* **Commands**:

  * `nit init` — create a `.git` repository skeleton
  * `nit hash-object [-w] <file>` — compute an object hash, optionally write it
  * `nit cat-file -p <hash>` — pretty-print an object
  * `nit write-tree` — write the working tree as a tree object
  * `nit ls-tree [--name-only] <tree-hash>` — list a tree's entries
  * `nit commit-tree -m <msg> [-p <parent>] <tree-hash>` — write a commit object
  * `nit commit -m <msg>` — write tree + commit, update the current branch
  * `nit add <file>` — stage a file (or `.` for everything) into the index
  * `nit status` — show modified, deleted, and untracked files
  * `nit clone <path>` — clone a local repository

* **Remote protocol (in progress)**: partial HTTP smart-protocol implementation (`git-upload-pack`, pkt-line framing) — currently only reads the pack header

## Build & Usage Guide

If you donot know how to build and use it donot use it, Bro use Git instead :(

## For Nerds

The name "nit" was given by Me, I described the tool as "the stupid content tracker" and the name as (depending on your mood):
* A random three-letter combination that is pronounceable and not already used by a common UNIX command.
* Something stupid, contemptible, or despicable.
* **"New Information Tracker"**, when everything works perfectly.
* **"Goddamn Idiotic Truckload of Sh*t"**, when everything breaks.

