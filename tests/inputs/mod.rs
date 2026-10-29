//! The real-data inputs this crate's tests read, each with what it is and where it comes from
//! (`primase_testdata::inputs!`; README.md here lists them). A test whose input is not configured skips with a line
//! naming its key. An input declared `gate: true` is one a complete run needs: with `[run] require = 'gate'` in the
//! `GAME_DATA` file, a missing one fails the test instead of skipping it. Integration tests reach this module with
//! `mod inputs;`, unit tests with `#[cfg(test)] #[path = "../tests/inputs/mod.rs"] mod inputs;` in the crate root, and
//! tests/real_data_inputs.rs checks that every query the crate's sources write is declared here. Declare an input
//! here when a test first asks for it.

primase_testdata::inputs! {
    games {
        Stuntboost = "stuntboost",
    }
    installs {
        Stuntboost {
            about: "STUNTBOOST's install folder (the folder that holds `Content`: MonoGame 3.8.1 DesktopGL XNB files)",
            source: "the user's own install",
            gate: true,
        },
    }
    keys {}
    tools {}
    oracle_sets {}
    declared {
        Query::Install(Game::Stuntboost, Some("Content")) => {
            status: Live,
            gate: true,
            about: r#"STUNTBOOST's `Content` folder (1875 `.xnb` files, 18 of them effects), every file read by tests/real_data.rs"#,
            source: r#"the user's own install"#,
        },
    }
    debug {}
}
