//! secondtake: before and after tapes, matched by task
//! and compared on four measures, with judgment in the
//! one-regression band (ST-5 to ST-19).
//!
//! Round 3 is the Rust port. Canonical form 1, the JSON
//! reader, tape durations and the value model are
//! consumed from the flighttape crate at a pinned
//! revision (ST-21), never re-implemented here; this
//! crate carries the comparison itself.

pub mod cli;
pub mod compare;
pub mod emit;
pub mod helptext;
pub mod judge;
pub mod measures;
pub mod moments;
pub mod pyrepr;
pub mod repr_table;
pub mod tapes;
pub mod util;

pub use compare::{compare, load_registration, CompareError, Registration};
pub use judge::JudgeError;

/// The command line entry, in the cli module.
pub fn run_cli() -> i32 {
    // A generous stack, on the flighttape crate's own
    // precedent: parsed values nest as deep as their
    // files, and cloning or dropping a deep value
    // recurses with its depth.
    let child = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(crate::cli::run_cli)
        .expect("the CLI thread starts");
    child.join().unwrap_or(1)
}
