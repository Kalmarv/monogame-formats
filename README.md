# monogame-formats

[![crates.io: monogame-formats](https://img.shields.io/crates/v/monogame-formats?label=monogame-formats)](https://crates.io/crates/monogame-formats) [![docs.rs](https://img.shields.io/docsrs/monogame-formats)](https://docs.rs/monogame-formats) [![CI](https://img.shields.io/github/actions/workflow/status/Kalmarv/monogame-formats/ci.yml?branch=main&label=CI)](https://github.com/Kalmarv/monogame-formats/actions/workflows/ci.yml)

A Rust library that reads MonoGame and XNA content files (`.xnb`) and the compiled MGFX effects stored in them.

## Features

- Reads XNB containers, version 5 (XNA 4.0 and MonoGame), uncompressed or LZX-compressed as XNA 4.0's content pipeline writes them
- Exposes the header's platform byte and HiDef flag, and the type-reader table in file order
- Splits reader type names into their unqualified name, base name and generic arguments
- Reads the values of XNA's built-in string and byte-array readers
- Reads `Texture2D` objects with the `Color` surface format, including every mip level
- Gives a checked cursor over the primary object's bytes for reading a game's own content readers
- Reads MGFX effects, version 12, as written by the Byting Games fork of MonoGame 3.8.1
- Reads an effect's constant buffers, shaders, samplers and sampler states, vertex attributes, parameters with defaults, techniques and passes
- Returns shader code as stored (GLSL for the OpenGL profile, DXBC for DirectX 11) without translating it
- Checked against all 1,875 `.xnb` files of STUNTBOOST, 18 of them effects
- Refuses what it does not read with a named error: LZX or LZ4 compression, other versions, shared resources, other texture formats, pass render states and effect annotations
- Checks that every index inside an effect points at an entry that exists
- Bounds-checks every read; malformed input gives an error naming the byte offset instead of a panic
- Parses bytes the caller supplies, borrows parsed values from them and does no file I/O
- Contains no `unsafe` code

## Getting Started

### Requirements

Rust 1.99 or newer (edition 2024).

### Installation

The crate is published on [crates.io](https://crates.io/crates/monogame-formats):

```sh
cargo add monogame-formats
```

### Usage

This program prints an XNB file's reader and, if the file holds an effect, its shaders and techniques:

```rust
use monogame_formats::mgfx::Effect;
use monogame_formats::xnb::Xnb;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("usage: inspect <file.xnb>")?;
    let bytes = std::fs::read(&path)?;
    let xnb = Xnb::parse(&bytes)?;
    let (reader, _) = xnb.primary()?;
    println!("{path}: {}", reader.type_name());

    if reader.is_effect() {
        let effect = Effect::parse(xnb.effect()?)?;
        for (i, shader) in effect.shaders.iter().enumerate() {
            println!("shader {i}: {:?}, {} bytes of code", shader.stage, shader.code.len());
        }
        for technique in &effect.techniques {
            println!("technique {}: {} passes", technique.name, technique.passes.len());
        }
    }
    Ok(())
}
```

Other reads on `Xnb`:

- `Xnb::string` and `Xnb::byte_array` read objects written by XNA's `StringReader` and `ArrayReader<Byte>`.
- `Xnb::texture2d` reads a `Texture2DReader` object into its width, height and mip levels of RGBA bytes.
- `Xnb::primary` returns the object's `TypeReader` and a `Content` cursor (`seven_bit`, `string`, `i32`, `f32`, `bytes`, `end`, ...) for a game's own readers.

The crate-level documentation in `src/lib.rs` has a doc-tested example that builds a one-string XNB in memory. API documentation is on [docs.rs](https://docs.rs/monogame-formats).

Only MGFX version 12 is read. Upstream MonoGame releases write versions 10 and 11, which are refused. The DirectX 11 profile is parsed the same way as OpenGL but has only been tried on hand-built bytes.

Run the tests with `cargo test --workspace`; the test that reads STUNTBOOST's files skips unless `GAME_DATA` is set, as [docs/testing.md](docs/testing.md) describes. See [CONTRIBUTING.md](CONTRIBUTING.md) for the checks CI runs and how to report a file the crate cannot read. Licensed under the GNU General Public License, version 3 or later (see [LICENSE](LICENSE)).
