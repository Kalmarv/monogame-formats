# Changelog

## 0.2.0 (2026-11-06)

- `xnb::decompress`: an LZX-compressed XNB (flag 0x80, as XNA 4.0's content pipeline writes it) becomes the
  uncompressed file it stands for, which `Xnb::parse` reads; an uncompressed file is returned as it is. Its blocks are
  decoded through primase-foundations 0.3.6's `LzxDecoder`, strictly: the output must be the stated size, and only zero
  bytes may follow the block that ends the stream. Before, every compressed file was refused. LZ4-compressed files
  (MonoGame's flag 0x40) are still refused: no game read here has one.
- `Error::Compressed` says what reading the file takes: `decompress it first` for LZX, `not read` for LZ4.
- Every one of Terraria's 15,994 XNB files (15,108 LZX-compressed) decompresses to the content XNA 4.0's own
  decompressor makes of it (`tests/real_data.rs`, `installs.terraria` and `oracles.xna.terraria/content.jsonl` of
  primase-testdata 0.1.13).
- Depends on primase-foundations 0.3.6 with its `compress` feature.

## 0.1.1 (2026-10-29)

- Depends on primase-foundations 0.3.0, which has no type in this crate's API.
- The real-data input's description counts 1,875 `.xnb` files (18 of them effects), the install's count since the
  game's build 26237614.

## 0.1.0 (2026-10-27)

- First release: MonoGame and XNA content, the XNB container and compiled MGFX effects.
