# Real-data inputs

What this crate's real-data tests read, as tests/inputs/mod.rs declares them: the keys of the configuration file
`GAME_DATA` names (its layout is that of [primase-testdata](https://github.com/Kalmarv/primase-testdata), which the
tests use), the oracle sets and the files of each, and the debug keys `GAME_DATA_DEBUG` sets. A test whose input is not
configured skips with a line naming its key; the "gate" column marks the inputs that fail instead of skipping when the
file says `[run] require = 'gate'`. The tables below are generated from the declarations (the `real_data_inputs` test
fails when they differ); [the README](../../README.md#running-the-tests) says how to set the inputs up.

<!-- real-data inputs: made by primase_testdata::doc::tables from tests/inputs/mod.rs -->

#### Configuration keys

| key | gate | about | source |
| --- | --- | --- | --- |
| `installs.stuntboost` | yes | STUNTBOOST's install folder (the folder that holds `Content`: MonoGame 3.8.1 DesktopGL XNB files) | the user's own install |
| `installs.terraria` | yes | Terraria's install folder (the folder that holds `Content`: XNA 4.0 XNB files, most of them LZX-compressed) | the user's own install |
| `scratch.root` | - | where real-data tests write their outputs (the OS temporary folder when unset) | - |

#### Oracle sets (`<oracles.root>/<set>/<game>/`)

| set | about | source |
| --- | --- | --- |
| `oracles.xna` | XNA 4.0's own readings of a game's content | independent: XNA Framework 4.0 (Microsoft.Xna.Framework 4.0.0.0), run by primase-oracles' xna/XnaContent.cs |

#### Declared files

| key | status | gate | about | source |
| --- | --- | --- | --- | --- |
| `installs.stuntboost/Content` | live | yes | STUNTBOOST's `Content` folder (Steam build 26429886: 1,881 `.xnb` files, 18 of them effects), every file read by tests/real_data.rs | the user's own install |
| `installs.terraria/Content` | live | yes | Terraria's `Content` folder (15,994 `.xnb` files, 15,108 of them LZX-compressed), every file decompressed by tests/real_data.rs | the user's own install |
| `oracles.xna.terraria/content.jsonl` | live | yes | XNA 4.0's reading of every `.xnb` of Terraria's `Content`: its header and the SHA-256 of the content after it, decompressed by XNA's own `DecompressStream` (JSON lines; tests/real_data.rs compares) | independent: XNA Framework 4.0's `Microsoft.Xna.Framework.Content.DecompressStream`, run by primase-oracles' xna/XnaContent.cs |

<!-- end of real-data inputs -->
