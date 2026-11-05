# Testing

```sh
cargo test --workspace
```

The tests build their own XNB and MGFX bytes in memory, and no game files are kept in the repository. `tests/real_data.rs` parses every file in STUNTBOOST's `Content` folder. It skips unless the `GAME_DATA` environment variable names a TOML file in the [primase-testdata](https://github.com/Kalmarv/primase-testdata) layout:

```toml
[installs]
stuntboost = 'D:/games/STUNTBOOST'      # the folder that holds Content
[run]
require = 'gate'                        # optional: a missing input fails instead of skipping
```

[tests/inputs/README.md](../tests/inputs/README.md) lists every real-data input. See [CONTRIBUTING.md](../CONTRIBUTING.md) for the checks CI runs and how to report a file the crate cannot read.
