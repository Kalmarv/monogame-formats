//! Reads games' own content, named by the `GAME_DATA` configuration (see tests/inputs/README.md). Without it a test
//! skips with a line naming its key; no game file is kept in this repository.
//!
//! STUNTBOOST's `Content`: every `.xnb` under the folder must parse as a container, and every one whose reader is
//! `EffectReader` must parse as an MGFX effect (version 12), with all its indices in range.
//!
//! Terraria's `Content` (XNA 4.0, most files LZX-compressed): every `.xnb` decompresses to the content XNA 4.0's own
//! decompressor makes of it (`oracles.xna.terraria/content.jsonl`: the SHA-256 of what follows the header) and the
//! result parses as a container.
#![allow(
    clippy::unwrap_used,
    reason = "a test reading the files you point it at; the library does no I/O"
)]

mod inputs;

use inputs::{Game, OracleSet, install_file, oracle};
use monogame_formats::mgfx::Effect;
use monogame_formats::xnb::{Xnb, decompress};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

fn xnb_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            xnb_files(&path, out);
        } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("xnb")) {
            out.push(path);
        }
    }
}

#[test]
fn every_container_and_effect_of_a_game_reads() {
    let Some(root) = install_file(Game::Stuntboost, "Content") else {
        return;
    };
    let mut files = Vec::new();
    xnb_files(&root, &mut files);
    assert!(!files.is_empty(), "no .xnb under {}", root.display());
    let mut effects = 0usize;
    for path in &files {
        let bytes = std::fs::read(path).unwrap();
        let xnb = Xnb::parse(&bytes).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let Ok((reader, _)) = xnb.primary() else {
            continue;
        };
        if reader.is_effect() {
            let code = xnb.effect().unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            Effect::parse(code).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            effects += 1;
        }
    }
    eprintln!("{} files, {effects} effects", files.len());
}

#[test]
fn every_lzx_file_decompresses_to_what_xna_makes_of_it() {
    let (Some(root), Some(want)) = (
        install_file(Game::Terraria, "Content"),
        oracle(OracleSet::Xna, Game::Terraria, "content.jsonl"),
    ) else {
        return;
    };
    let (mut files, mut compressed, mut bytes) = (0usize, 0usize, 0u64);
    for line in std::fs::read_to_string(&want).unwrap().lines() {
        let theirs: serde_json::Value = serde_json::from_str(line).unwrap();
        let path = theirs["path"].as_str().unwrap();
        assert!(theirs.get("error").is_none(), "{path}: XNA refused it");
        let file = std::fs::read(root.join(path)).unwrap();
        let plain = decompress(&file).unwrap_or_else(|e| panic!("{path}: {e}"));
        let content = &plain[10..];
        let sha: String = Sha256::digest(content)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(
            (content.len() as u64, sha.as_str()),
            (
                theirs["content_bytes"].as_u64().unwrap(),
                theirs["sha256"].as_str().unwrap()
            ),
            "{path}: the content after the header"
        );
        Xnb::parse(&plain).unwrap_or_else(|e| panic!("{path}: {e}"));
        files += 1;
        compressed += usize::from(theirs["flags"].as_u64().unwrap() & 0x80 != 0);
        bytes += content.len() as u64;
    }
    eprintln!("{files} files, {compressed} LZX-compressed, {bytes} bytes of content");
    assert_eq!(
        (files, compressed),
        (15_994, 15_108),
        "the files of Terraria's Content"
    );
}
