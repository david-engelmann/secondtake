//! `secondtake` pairs before-and-after `flighttape` `"1.0"` session tapes by Canonical Form 1
//! task key and evaluates setup changes across four deterministic measures: token cost, wall
//! duration, redo rework moments, and error rescue rate.
//!
//! Conclusive runs (`0` regressions -> `keep`, `2+` regressions -> `revert`, `< min_pairs` ->
//! `abstain`) resolve without model calls; runs in the `1`-regression tradeoff band consult a
//! replayable or live `jev-1.13.0` judge (`--judge`).
//!
//! ```text
//! let registration = secondtake::load_registration(reg_bytes)?;
//! let report = secondtake::compare(&before_dir, &after_dir, &registration, None)?;
//! ```

pub mod cli;
pub mod compare;
pub mod emit;
#[doc(hidden)]
pub mod helptext;
pub mod judge;
pub mod measures;
pub mod moments;
#[doc(hidden)]
pub mod pyrepr;
#[doc(hidden)]
pub mod repr_table;
pub mod tapes;
pub mod util;

pub use compare::{compare, load_registration, CompareError, Registration};
pub use flighttape;
pub use judge::JudgeError;
pub use measures::{tape_measures, task_key, Measures, MATCHING_RULE};
pub use moments::{redo_moments, RedoMoment};
pub use tapes::{load_tape_file, measure_shape_defects, tape_defects, LoadOutcome};

/// Execute the `secondtake` command-line interface on a dedicated 512 MiB stack thread
/// and return the process exit code (`0`, `1`, or `2`).
pub fn run_cli() -> i32 {
    let child = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(crate::cli::run_cli)
        .expect("the CLI thread starts");
    child.join().unwrap_or(1)
}
