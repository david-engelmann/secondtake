//! ST-7, ST-7a. The four measures of one tape.

use flighttape::identity::tape_digest;
use flighttape::validate::tape_duration;
use flighttape::value::Value;

use crate::moments::redo_moments;
use crate::util::{int_to_i128, obj};

pub const MATCHING_RULE: &str = "tapes match by task key: the canonical form 1 digest \
of the text of the first main-chain user message";

/// The ST-6a task key, or None when unmatchable.
pub fn task_key(tape: &Value) -> Option<String> {
    let events = tape.get("events")?.as_array()?;
    for event in events {
        let is_user_message = matches!(event.get("kind"), Some(Value::Str(k)) if k.as_str().as_deref() == Some("message"))
            && matches!(event.get("role"), Some(Value::Str(r)) if r.as_str().as_deref() == Some("user"))
            && matches!(event.get("sidechain"), Some(Value::Bool(false)));
        if is_user_message {
            return match event.get("text") {
                Some(text @ Value::Str(_)) => tape_digest(text).ok(),
                _ => None,
            };
        }
    }
    None
}

/// The four measures of one tape, in the document's
/// own key order.
#[derive(Clone, Debug)]
pub struct Measures {
    pub cost: Option<i128>,
    pub time_seconds: Option<f64>,
    pub rework: i128,
    pub rescued: bool,
    pub has_error: bool,
}

impl Measures {
    pub fn to_value(&self) -> Value {
        obj(vec![
            (
                "cost",
                self.cost.map(crate::util::int_i128).unwrap_or(Value::Null),
            ),
            (
                "time_seconds",
                self.time_seconds.map(Value::Float).unwrap_or(Value::Null),
            ),
            ("rework", crate::util::int_i128(self.rework)),
            ("rescued", Value::Bool(self.rescued)),
            ("has_error", Value::Bool(self.has_error)),
        ])
    }
}

fn totals_int(tape: &Value, key: &str) -> Option<i128> {
    match tape.get("run")?.get("totals")?.get(key) {
        Some(Value::Int(digits)) => int_to_i128(digits),
        _ => None,
    }
}

pub fn tape_measures(tape: &Value) -> Measures {
    let cost = match (totals_int(tape, "input_tokens"), totals_int(tape, "output_tokens")) {
        (Some(inputs), Some(outputs)) => Some(inputs + outputs),
        _ => None,
    };
    let events = tape
        .get("events")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let has_error = events.iter().any(|event| {
        matches!(event.get("kind"), Some(Value::Str(k)) if k.as_str().as_deref() == Some("tool_result"))
            && matches!(event.get("is_error"), Some(Value::Bool(true)))
    });
    let outcome = tape
        .get("outcome")
        .and_then(|outcome| outcome.get("status"))
        .and_then(|status| match status {
            Value::Str(text) => text.as_str(),
            _ => None,
        });
    Measures {
        cost,
        time_seconds: tape_duration(tape),
        rework: redo_moments(tape).len() as i128,
        rescued: has_error && outcome.as_deref() == Some("completed"),
        has_error,
    }
}
