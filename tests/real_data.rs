//! Reads a game's own content, when you point at it: set `MONOGAME_FORMATS_CONTENT` to a MonoGame game's `Content`
//! folder (STUNTBOOST's, which this crate was written against). Without it these tests do nothing and pass; no game
//! file is kept in this repository.
//!
//! Every `.xnb` under the folder must parse as a container, and every one whose reader is `EffectReader` must parse
//! as an MGFX effect (version 12), with all its indices in range.
#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::unwrap_used,
    reason = "this test reads the files you point it at; the library does no I/O"
)]

use monogame_formats::mgfx::Effect;
use monogame_formats::xnb::Xnb;
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
    let Some(root) = std::env::var_os("MONOGAME_FORMATS_CONTENT") else {
        eprintln!("MONOGAME_FORMATS_CONTENT is not set: skipped");
        return;
    };
    let mut files = Vec::new();
    xnb_files(Path::new(&root), &mut files);
    assert!(!files.is_empty(), "no .xnb under {}", Path::new(&root).display());
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
