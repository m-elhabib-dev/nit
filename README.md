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

## Known Limitations 
`nit` is a learning-oriented Git reimplementation, so it currently has several limitations and intentional deviations from Git.

### Platform compatibility
* **Linux-focused**. The current implementation has not been designed or tested for Windows.
* Some filesystem behavior and path handling may therefore be platform-specific.
* Windows-specific concerns such as path separators, file modes, and filesystem metadata are not currently handled.

### `.gitignore`
* **`.gitignore` is not implemented.**
* `nit add .` currently considers files without applying Git's ignore rules.
* This means files such as build artifacts, editor files, and environment files can be added unintentionally.

### Remote repositories
* **Remote cloning is not currently supported.**
* Local repository cloning works, but the Git smart HTTP protocol and packfile handling are still incomplete.
* The remote-cloning experiment reached Git's `PACK` format and successfully parsed normal objects, but delta objects still require implementation.

### Git compatibility
* `nit` does **not aim for full Git compatibility** yet.
* Many Git commands, options, configuration mechanisms, and edge cases are not implemented.
* The implementation currently focuses on understanding and reproducing the core Git object model and index.

### Path and filesystem edge cases
* Git supports filenames and filesystem behavior that are more complicated than the current implementation assumes.
* Path handling has not been fully tested against unusual filenames, platform-specific paths, or non-UTF-8 filenames.

### Object handling
* Packfiles containing **`OFS_DELTA` and `REF_DELTA` objects** are not fully supported.
* The current object implementation primarily works with loose Git objects.
* Full packfile support requires delta resolution and conversion of packed objects into usable Git objects.

### Index compatibility
* The custom index implementation currently targets the Git index format needed by `nit`.
* Advanced index features and edge cases are not implemented.

This section would fit well near the end of your README, under something like **"Known Limitations"**, separate from the project's implemented features.


## Build & Usage Guide
If you donot know how to build and use it donot use it, Bro use Git instead :(

## For Nerds

The name "nit" was given by Me, I described the tool as "the stupid content tracker" and the name as (depending on your mood):
* A random three-letter combination that is pronounceable and not already used by a common UNIX command.
* Something stupid, contemptible, or despicable.
* **"New Information Tracker"**, when everything works perfectly.
* **"Goddamn Idiotic Truckload of Sh*t"**, when everything breaks.

