# Contributing

## Build and test

The toolchain is pinned in `rust-toolchain.toml` (rustup installs it on first use).

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

CI runs these four on Linux and Windows. The tests build their own XNB and MGFX bytes in memory and need no game
files (`tests/real_data.rs` reads STUNTBOOST's `Content` folder when `GAME_DATA` names it; see the README). The parsers deny panicking constructs (`unwrap`, unchecked arithmetic, indexing and truncating casts) outside
tests, so a new read goes through the checked reads of `Content` and returns an `Error` naming the offset.

## Releasing

1. Raise the version (a patch, 0.x.Y, for fixes and data or manifest additions; a minor, 0.X.0, for new API or any
   breaking change before 1.0; then `cargo check`, so `Cargo.lock` follows) and put the change's CHANGELOG lines under a
   `## <version> (<date>)` heading, in one commit.
2. Push it to main. Once every check has passed, release-plz publishes each crate whose version crates.io does not have
   yet (trusted publishing: no token is stored anywhere), tags the commit `v<version>` and creates the GitHub Release
   with that CHANGELOG section as its notes.

A commit that raises no version only runs CI. Never release by hand.

A new crate's first version can't come from trusted publishing: the token holder publishes it by hand, then adds its
trusted-publishing config (workflow `ci.yml`).

## Reporting a file the crate cannot read

Open an issue with the "Unreadable file" template. Please include:

- the error message, whole (it names the byte offset and what was expected);
- the game, its version or build, and the MonoGame/XNA version it uses if you know it;
- the first 64 bytes of the file as hex (`xxd -l 64 file.xnb`), and its size;
- for an effect, the same for the effect's bytes if you can extract them.

Please do not attach game files, extracted assets or anything else copyrighted. If the hex header and error are not
enough, say so in the issue and we will work out a smaller excerpt together.

## Changes

Keep a change small and tested with a fixture built in the test itself. A format fact should say how it is known (what
was checked, on how many files).
