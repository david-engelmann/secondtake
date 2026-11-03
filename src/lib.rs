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
pub mod tapes;
pub mod util;

pub use compare::{compare, load_registration, CompareError, Registration};
pub use judge::JudgeError;
