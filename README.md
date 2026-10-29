# monogame-formats

A Rust library that reads MonoGame and XNA content files (`.xnb`) and the compiled MGFX shader effects inside them.

It reads, from bytes you give it:

- **XNB containers**, version 5 (the XNA 4.0 format, which MonoGame and FNA write), uncompressed: header, platform,
  type-reader table, and the primary object, with typed reads for the values XNA's built-in readers write (strings,
  byte arrays, raw 8-bit `Color` `Texture2D`s). A game's own content readers are named by the table; their bytes are
  handed to you.
- **MGFX effects**, version 12 (the Byting Games fork of MonoGame 3.8.1; upstream MonoGame writes 10 and 11), as an
  XNB of `EffectReader` holds them: constant buffers, shaders (the GLSL MojoShader wrote), samplers, vertex attributes,
  parameters with defaults, techniques and passes.

It was written against the content of STUNTBOOST (a MonoGame 3.8.1 DesktopGL game): its XNBs and all 18 of its
effects. Used by [primase](https://github.com/Kalmarv/primase), a game-asset conversion library.

## Example

```rust
use monogame_formats::xnb::Xnb;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A one-string XNB, built by hand: header, one type reader, the object.
    let reader = "Microsoft.Xna.Framework.Content.StringReader";
    let mut file = vec![b'X', b'N', b'B', b'd', 5, 0, 0, 0, 0, 0, 1, reader.len() as u8];
    file.extend_from_slice(reader.as_bytes());
    file.extend_from_slice(&[0, 0, 0, 0]); // the reader's version
    file.extend_from_slice(&[0, 1, 5]); // no shared resources, type id 1, a 5-byte string
    file.extend_from_slice(b"hello");
    let size = file.len() as u32;
    file[6..10].copy_from_slice(&size.to_le_bytes()); // the header holds the file's size

    let xnb = Xnb::parse(&file)?;
    assert_eq!(xnb.readers[0].type_name(), reader);
    assert_eq!(xnb.string()?, "hello");
    Ok(())
}
```

This is the crate's doc test (`src/lib.rs`), so it is compiled and run by `cargo test`. For an effect, `Xnb::effect`
returns the compiled effect's bytes and `mgfx::Effect::parse` reads them.

## What it guarantees

- Every read is bounds-checked. Malformed or hostile bytes give an `Err` that names the byte offset and what was
  expected, never a panic or an out-of-range index (`unsafe` is denied; arithmetic, indexing and truncating casts are
  linted against in the parsers). Counts cannot promise more items than there are bytes left.
- What it does not read it refuses by name instead of guessing: a compressed file (LZX or LZ4, naming the flag), an
  XNB or MGFX version other than the ones above, unknown header flags, a file whose stored size is not its length, a
  texture surface format other than `Color`, shared resources, effect passes with blend, depth-stencil or rasterizer
  state, and effect annotations.
- Indices inside an effect (a constant buffer's parameters, a shader's buffers and samplers, a pass's shaders) are
  checked to be in range.
- Parsed values borrow from your input; the crate does no I/O.

## What it does not do

- It does not decompress (LZX and LZ4 XNBs are refused), write XNB or MGFX, or compile effects.
- It does not run or interpret a game's own content readers, or read shared resources.
- It reads only `Color` textures, not compressed formats such as DXT.
- It does not translate GLSL or DXBC: shader code is handed over as the bytes stored.
- Only the versions above were checked against real files; other MGFX versions and the DirectX 11 profile are
  not covered by real data.

## Running the tests

```sh
cargo test --workspace
```

runs the tests that build their own small XNB and MGFX files in memory (a truncated-at-every-length test, header and
object strictness, error offsets, compressed-file refusals, a texture and an effect read back as stored), plus the doc
test above. No game file is kept in this repository. The test that reads a real game, `tests/real_data.rs`, skips with a
line naming what it needs when `GAME_DATA` is not set. To run it, point the `GAME_DATA` environment variable at a TOML
file naming your own copy, in the layout of [primase-testdata](https://github.com/Kalmarv/primase-testdata):

```toml
[installs]
stuntboost = 'D:/games/STUNTBOOST'      # the folder that holds Content
[run]
require = 'gate'                        # optional: a missing input fails instead of skipping
```

It parses every `.xnb` under STUNTBOOST's `Content` folder as a container and every effect among them as MGFX; against
the Steam install that is 1875 files, 18 of them effects. [tests/inputs/README.md](tests/inputs/README.md) lists every
input the tests can read (generated from `tests/inputs/mod.rs`).

## Documentation, contributing, license

API documentation: `cargo doc --open` (once published, <https://docs.rs/monogame-formats>). See
[CONTRIBUTING.md](CONTRIBUTING.md) to build or to report a file the crate cannot read.

Licensed under the GNU General Public License, version 3 or later (see [LICENSE](LICENSE)).
