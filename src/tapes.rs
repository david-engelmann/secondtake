//! Reading flighttape tapes (ST-5a). Secondtake reads
//! tapes at flighttape format version 1.0 only, never
//! raw harness session files. The checks here are the
//! format rules the comparison stands on, the same
//! subset the family reads everywhere: the value is an
//! object carrying tape_version 1.0, a run object, and
//! an events array whose seq values run consecutively
//! from 0 and whose kinds are in the format's enum, and
//! the tool call shapes the flighttape validator
//! enforces under its FT-4 pairing rule. A file that
//! fails any check is excluded with its defects named,
//! and the comparison proceeds on the tapes that
//! remain. Per ST-22 these are this repo's own checks
//! and phrases, standing over the flighttape crate's
//! reader, which supplies only the JSON parse.

use std::path::Path;

use flighttape::json::{parse, ParseError};
use flighttape::pyrepr::repr_text;
use flighttape::value::Value;

use crate::util::{int_eq_usize, int_is_negative, nonneg_int_cmp};

pub const EVENT_KINDS: [&str; 8] = [
    "session_start",
    "message",
    "tool_call",
    "tool_result",
    "diff",
    "spend",
    "note",
    "session_end",
];

pub const OUTCOME_STATUSES: [&str; 4] = ["completed", "failed", "aborted", "unknown"];

/// The outcome of reading one tape file: the tape, or
/// the exclusion record's reason and defects.
pub enum LoadOutcome {
    Tape(Value),
    Failed {
        reason: String,
        defects: Vec<String>,
    },
}

fn failed(reason: &str) -> LoadOutcome {
    LoadOutcome::Failed {
        reason: reason.to_string(),
        defects: Vec::new(),
    }
}

/// Read one tape file.
pub fn load_tape_file(path: &Path) -> LoadOutcome {
    let raw = match std::fs::read(path) {
        Ok(raw) => raw,
        Err(_) => return failed("cannot read the tape file"),
    };
    let text = match String::from_utf8(raw) {
        Ok(text) => text,
        Err(_) => return failed("the file is not valid UTF-8"),
    };
    let tape = match parse(&text) {
        Ok(tape) => tape,
        Err(ParseError::TooLarge) => return failed("the file holds a number too large to read"),
        Err(ParseError::Invalid) => return failed("the file is not valid JSON"),
    };
    let defects = tape_defects(&tape);
    if !defects.is_empty() {
        return LoadOutcome::Failed {
            reason: "not a valid tape".to_string(),
            defects,
        };
    }
    LoadOutcome::Tape(tape)
}

fn str_field(value: &Value, key: &str) -> Option<String> {
    match value.get(key) {
        Some(Value::Str(text)) => text.as_str(),
        _ => None,
    }
}

/// The format checks the tool's evidence stands on.
pub fn tape_defects(tape: &Value) -> Vec<String> {
    let mut defects = Vec::new();
    let object = match tape.as_object() {
        Some(object) => object,
        None => return vec!["the tape is not an object".to_string()],
    };
    let _ = object;
    if str_field(tape, "tape_version").as_deref() != Some("1.0") {
        defects.push("tape_version is not 1.0".to_string());
    }
    if tape.get("run").and_then(Value::as_object).is_none() {
        defects.push("run is not an object".to_string());
    }
    let events = match tape.get("events").and_then(Value::as_array) {
        Some(events) if !events.is_empty() => events,
        _ => {
            defects.push("events must hold at least one event".to_string());
            return defects;
        }
    };
    let mut seen_calls: Vec<flighttape::value::Text> = Vec::new();
    for (index, event) in events.iter().enumerate() {
        let where_ = format!("events[{index}]");
        if event.as_object().is_none() {
            defects.push(format!("{where_} is not an object"));
            continue;
        }
        let seq_ok = match event.get("seq") {
            Some(Value::Int(digits)) => int_eq_usize(digits, index),
            _ => false,
        };
        if !seq_ok {
            defects.push(format!("{where_}.seq is not {index}"));
        }
        let kind = str_field(event, "kind");
        let kind = match kind.as_deref() {
            Some(kind) if EVENT_KINDS.contains(&kind) => kind,
            _ => {
                defects.push(format!("{where_}.kind is not in the format enum"));
                continue;
            }
        };
        let call_id = event.get("call_id");
        if kind == "tool_call" {
            let tool_ok = matches!(event.get("tool"), Some(Value::Str(t)) if !t.units().is_empty());
            if !tool_ok {
                defects.push(format!("{where_}.tool must be a non-empty string"));
            }
            match call_id {
                Some(Value::Str(text)) if !text.units().is_empty() => {
                    if seen_calls.contains(text) {
                        defects.push(format!(
                            "{where_}.call_id {} is used twice",
                            repr_text(text)
                        ));
                    } else {
                        seen_calls.push(text.clone());
                    }
                }
                _ => {
                    defects.push(format!("{where_}.call_id must be a non-empty string"));
                }
            }
        }
        if kind == "tool_result" {
            let answers = match call_id {
                Some(Value::Str(text)) => seen_calls.contains(text),
                _ => false,
            };
            if !answers {
                let shown = match call_id {
                    Some(Value::Str(text)) => repr_text(text),
                    Some(other) => flighttape::pyrepr::repr_value(other),
                    None => "None".to_string(),
                };
                defects.push(format!(
                    "{where_}.call_id {shown} answers no earlier tool_call"
                ));
            }
            if let Some(is_error) = event.get("is_error") {
                if !matches!(is_error, Value::Bool(_)) {
                    defects.push(format!("{where_}.is_error must be a boolean"));
                }
            }
        }
    }
    defects
}

/// ST-5b. The shapes secondtake's own measures stand
/// on, beyond the family subset above: totals values
/// the format bounds at zero, and an outcome object
/// whose status is in the format enum. A tape breaking
/// either is excluded with the defect named, never
/// crashed on.
pub fn measure_shape_defects(tape: &Value) -> Vec<String> {
    let mut defects = Vec::new();
    if let Some(run) = tape.get("run") {
        if let Some(totals) = run.get("totals") {
            match totals.as_object() {
                None => defects.push("run.totals is not an object".to_string()),
                Some(entries) => {
                    for (key, value) in entries {
                        let key = key.as_str().unwrap_or_else(|| {
                            // Keys reaching here came from
                            // parsed JSON; a surrogate key
                            // renders lossily in the defect,
                            // where the Python names it in
                            // full. Such a tape is excluded
                            // either way.
                            String::from_utf16_lossy(key.units())
                        });
                        match value {
                            Value::Int(digits) if !int_is_negative(digits) => {
                                if nonneg_int_cmp(
                                    digits.trim_start_matches('-'),
                                    "1000000000000000000",
                                ) == std::cmp::Ordering::Greater
                                {
                                    defects
                                        .push(format!("run.totals.{key} is too large to measure"));
                                }
                            }
                            _ => {
                                defects.push(format!(
                                    "run.totals.{key} is not a non-negative integer"
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    if let Some(outcome) = tape.get("outcome") {
        match outcome.as_object() {
            None => defects.push("outcome is not an object".to_string()),
            Some(_) => {
                let status = str_field(outcome, "status");
                if !matches!(status.as_deref(), Some(s) if OUTCOME_STATUSES.contains(&s)) {
                    defects.push("outcome.status is not in the format enum".to_string());
                }
            }
        }
    }
    defects
}
