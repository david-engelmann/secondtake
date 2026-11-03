//! Redo moments (RF-4, the shared definition ST-7
//! stands on).
//!
//! A redo moment is a tool call whose tool and input
//! match an earlier tool call in the same tape, with an
//! error tool result answering the earlier call in
//! between. The moment is recorded at the later call's
//! seq. Inputs are compared by their canonical form 1
//! bytes, so two inputs that differ only in key order
//! are the same call, and two that differ in any value
//! are not. In this port the comparison key is the
//! input's canonical digest from the flighttape crate,
//! which stands or falls with the bytes it digests.
//! The tape has already passed the format checks in
//! tapes.rs, so a result's call id answers an earlier
//! call and arrives after it; an error recorded for a
//! call is therefore always in between that call and
//! any later repeat examined here. Earlier calls are
//! indexed by their tool and input, so detection stays
//! linear in the size of the tape.

use std::collections::HashMap;

use flighttape::identity::tape_digest;
use flighttape::value::{Text, Value};

use crate::util::int_to_i128;

pub struct RedoMoment {
    pub seq: i128,
    pub earlier_seq: i128,
    pub tool: Text,
}

fn seq_of(event: &Value) -> i128 {
    match event.get("seq") {
        Some(Value::Int(digits)) => int_to_i128(digits).unwrap_or(0),
        _ => 0,
    }
}

/// Every redo moment in one valid tape, in seq order.
pub fn redo_moments(tape: &Value) -> Vec<RedoMoment> {
    let mut moments = Vec::new();
    let mut calls: HashMap<Text, ((Text, String), i128)> = HashMap::new();
    let mut first_errored: HashMap<(Text, String), i128> = HashMap::new();
    let events = tape
        .get("events")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for event in &events {
        let kind = event
            .get("kind")
            .and_then(|v| match v {
                Value::Str(text) => text.as_str(),
                _ => None,
            })
            .unwrap_or_default();
        if kind == "tool_result" {
            if matches!(event.get("is_error"), Some(Value::Bool(true))) {
                if let Some(Value::Str(call_id)) = event.get("call_id") {
                    if let Some((key, seq)) = calls.get(call_id) {
                        let entry = first_errored.entry(key.clone()).or_insert(*seq);
                        if *seq < *entry {
                            *entry = *seq;
                        }
                    }
                }
            }
            continue;
        }
        if kind != "tool_call" {
            continue;
        }
        let input = event.get("input").cloned().unwrap_or(Value::Null);
        let digest = match tape_digest(&input) {
            Ok(digest) => digest,
            Err(_) => continue,
        };
        let tool = match event.get("tool") {
            Some(Value::Str(text)) => text.clone(),
            _ => continue,
        };
        let key = (tool, digest);
        if let Some(earlier) = first_errored.get(&key) {
            moments.push(RedoMoment {
                seq: seq_of(event),
                earlier_seq: *earlier,
                tool: key.0.clone(),
            });
        }
        if let Some(Value::Str(call_id)) = event.get("call_id") {
            calls.insert(call_id.clone(), (key, seq_of(event)));
        }
    }
    moments
}
