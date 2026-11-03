//! ST-5 to ST-9, ST-13 to ST-18. The matched
//! comparison and its judgment.

use std::collections::BTreeMap;
use std::path::Path;

use flighttape::identity::tape_digest;
use flighttape::json::parse;
use flighttape::value::{container_depth, Text, Value};

use crate::judge::{
    build_card, confidence_below_floor, judge_card, value_text, CardCounts, JudgeError, Transport,
};
use crate::measures::{tape_measures, task_key, Measures, MATCHING_RULE};
use crate::pyrepr::repr_value;
use crate::tapes::{load_tape_file, measure_shape_defects, LoadOutcome};
use crate::util::{
    err, f64_gt_nonneg_int, fsdecode_units, int, int_eq_usize, int_is_negative, int_usize,
    is_number, json_text_depth, obj, stderr_render, text, text_cmp_py, FromMessage,
};

pub const PROPOSAL_RULE: &str = "abstain for insufficient evidence below the pair \
floor; keep when no measure regresses past its margin, revert when two or more \
do; in the one-regression band a configured judge's choice stands at \
confidence 0.6 or better, abstains below it, and an unavailable judge leaves \
the tune standing; a measure with no usable pairs neither regresses nor clears";

const MARGIN_KEYS: [&str; 4] = ["cost_pct", "time_pct", "rework_per_pair", "rescue_points"];

#[derive(Debug, Clone, PartialEq)]
pub struct CompareError(pub String);

impl std::fmt::Display for CompareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CompareError {}

impl FromMessage for CompareError {
    fn from_message(message: String) -> CompareError {
        CompareError(message)
    }
}

/// A validated registration: the change and its date,
/// the four margins in their canonical order, and the
/// pair floor.
#[derive(Clone, Debug)]
pub struct Registration {
    pub change: String,
    pub date: String,
    pub margins: [Value; 4],
    pub min_pairs: Value,
}

impl Registration {
    pub fn to_value(&self) -> Value {
        obj(vec![
            ("change", text(&self.change)),
            ("date", text(&self.date)),
            (
                "margins",
                obj(vec![
                    ("cost_pct", self.margins[0].clone()),
                    ("time_pct", self.margins[1].clone()),
                    ("rework_per_pair", self.margins[2].clone()),
                    ("rescue_points", self.margins[3].clone()),
                ]),
            ),
            ("min_pairs", self.min_pairs.clone()),
        ])
    }

    fn floor_exceeds(&self, pair_count: usize) -> bool {
        match &self.min_pairs {
            Value::Int(digits) => {
                let digits = digits.trim_start_matches('-');
                digits.len() > 18 || {
                    let floor: usize = digits.parse().unwrap_or(usize::MAX);
                    pair_count < floor
                }
            }
            _ => false,
        }
    }

    fn floor_text(&self) -> String {
        match &self.min_pairs {
            Value::Int(digits) => digits.clone(),
            other => value_text(other),
        }
    }
}

/// Whether a delta exceeds a margin, compared the way
/// Python compares: float against float directly,
/// float against an integer exactly.
fn exceeds(delta: f64, margin: &Value) -> bool {
    match margin {
        Value::Float(limit) => delta > *limit,
        Value::Int(digits) => {
            if int_is_negative(digits) {
                // Margins are validated non-negative; a
                // negative here cannot occur.
                true
            } else {
                f64_gt_nonneg_int(delta, digits.trim_start_matches('-'))
            }
        }
        _ => false,
    }
}

fn single_line_check(value: &Text) -> Result<String, &'static str> {
    // The Python scans the decoded string a character
    // at a time: refused controls first, then a lone
    // surrogate fails the UTF-8 encoding check.
    let units = value.units();
    let mut index = 0;
    let mut lone_surrogate = false;
    while index < units.len() {
        let unit = units[index];
        let scalar: u32 = if (0xD800..0xDC00).contains(&unit)
            && index + 1 < units.len()
            && (0xDC00..0xE000).contains(&units[index + 1])
        {
            index += 2;
            0x10000 + (((unit as u32 - 0xD800) << 10) | (units[index - 1] as u32 - 0xDC00))
        } else if (0xD800..0xE000).contains(&unit) {
            lone_surrogate = true;
            index += 1;
            0xFFFD
        } else {
            index += 1;
            unit as u32
        };
        if scalar < 32 || scalar == 127 || matches!(scalar, 0x85 | 0x2028 | 0x2029) {
            return Err("line");
        }
    }
    if lone_surrogate {
        return Err("utf8");
    }
    Ok(value.as_str().unwrap_or_default())
}

pub fn load_registration(path: &Path, display: &str) -> Result<Registration, CompareError> {
    let raw = match std::fs::read(path) {
        Ok(raw) => raw,
        Err(_) => {
            return Err(err(format!(
                "no registration at {display}; a comparison is refused without one"
            )));
        }
    };
    let data = (|| -> Result<Value, CompareError> {
        let text_in = String::from_utf8(raw)
            .map_err(|_| err(format!("the registration at {display} is not valid JSON")))?;
        // The Python's reader refuses nesting past its
        // recursion boundary (probed through this
        // surface: a text depth of 9994 reads, 9995
        // refuses); the crate's iterative reader
        // accepts it, so the cap is applied to the text.
        if json_text_depth(&text_in) > 9994 {
            return Err(err(format!(
                "the registration at {display} is not valid JSON"
            )));
        }
        parse(&text_in).map_err(|_| err(format!("the registration at {display} is not valid JSON")))
    })();
    let data = data?;
    if data.as_object().is_none() {
        return Err(err("the registration is not an object"));
    }
    let mut fields: Vec<String> = Vec::new();
    for field in ["change", "date"] {
        let value = data.get(field);
        let text_value = match value {
            Some(Value::Str(text)) if !text.units().is_empty() => text,
            _ => {
                return Err(err(format!("the registration names no {field}")));
            }
        };
        match single_line_check(text_value) {
            Ok(rendered) => fields.push(rendered),
            Err("line") => {
                return Err(err(format!(
                    "the registration {field} is not a single line of text"
                )));
            }
            Err(_) => {
                return Err(err(format!(
                    "the registration {field} is not valid UTF-8 text"
                )));
            }
        }
    }
    let mut margins: [Value; 4] = [int("10"), int("10"), int("1"), int("5")];
    if let Some(stated) = data.get("margins") {
        // A null margins is an absent margins in the
        // Python (`if stated is not None`); anything
        // else that is not an object refuses.
        if !matches!(stated, Value::Null) {
            let stated = match stated.as_object() {
                Some(stated) => stated,
                None => {
                    return Err(err("the registration margins are not an object"));
                }
            };
            for (key, value) in stated {
                let slot = match key.as_str() {
                    Some(key) => MARGIN_KEYS.iter().position(|known| *known == key),
                    None => None,
                };
                let slot = match slot {
                    Some(slot) => slot,
                    None => {
                        return Err(err(format!(
                            "the registration states an unknown margin, {}",
                            stderr_render(key.units())
                        )));
                    }
                };
                let valid = match value {
                    Value::Int(digits) => !int_is_negative(digits),
                    Value::Float(number) => number.is_finite() && *number >= 0.0,
                    _ => false,
                };
                if !valid || !is_number(value) {
                    return Err(err(format!(
                        "the margin {} is not a non-negative number",
                        MARGIN_KEYS[slot]
                    )));
                }
                margins[slot] = value.clone();
            }
        }
    }
    let floor = data.get("min_pairs").cloned().unwrap_or_else(|| int("1"));
    let floor_ok = match &floor {
        Value::Int(digits) => !int_is_negative(digits) && !int_eq_usize(digits, 0),
        _ => false,
    };
    if !floor_ok {
        return Err(err(format!(
            "the registration floor min_pairs is not a positive integer, {}",
            repr_value(&floor)
        )));
    }
    Ok(Registration {
        change: fields[0].clone(),
        date: fields[1].clone(),
        margins,
        min_pairs: floor,
    })
}

struct SetEntry {
    file: Text,
    tape: Value,
}

fn load_set(
    directory: &Path,
    display: &str,
    side: &str,
) -> Result<(Vec<SetEntry>, Vec<Value>), CompareError> {
    if !directory.is_dir() {
        return Err(err(format!(
            "the {side} set at {display} is not a directory"
        )));
    }
    let read = match std::fs::read_dir(directory) {
        Ok(read) => read,
        Err(error) => {
            return Err(err(format!(
                "cannot read the {side} set at {display}: {error}"
            )));
        }
    };
    // Entry names are carried in the operating
    // system's own decoding end to end (the Python's
    // listdir strings): the document names them as
    // they are, and paths open by their real bytes.
    // Names sort as the Python sorts strings, by
    // codepoint.
    let mut names: Vec<(std::ffi::OsString, Text)> = Vec::new();
    for entry in read {
        match entry {
            Ok(entry) => {
                let os_name = entry.file_name();
                #[cfg(unix)]
                let bytes = os_name.as_encoded_bytes().to_vec();
                #[cfg(not(unix))]
                let bytes = os_name.to_string_lossy().into_owned().into_bytes();
                let text_name = Text::from_units(fsdecode_units(&bytes));
                names.push((os_name, text_name));
            }
            Err(error) => {
                return Err(err(format!(
                    "cannot read the {side} set at {display}: {error}"
                )));
            }
        }
    }
    names.sort_by(|a, b| text_cmp_py(a.1.units(), b.1.units()));
    let mut tapes = Vec::new();
    let mut excluded = Vec::new();
    for (os_name, name) in names {
        let path = directory.join(&os_name);
        if !path.is_file() {
            excluded.push(obj(vec![
                ("file", Value::Str(name)),
                ("reason", text("not a file")),
                ("defects", Value::Array(Vec::new())),
            ]));
            continue;
        }
        let tape = match load_tape_file(&path) {
            LoadOutcome::Tape(tape) => tape,
            LoadOutcome::Failed { reason, defects } => {
                excluded.push(obj(vec![
                    ("file", Value::Str(name)),
                    ("reason", text(&reason)),
                    (
                        "defects",
                        Value::Array(defects.iter().map(|d| text(d)).collect()),
                    ),
                ]));
                continue;
            }
        };
        let shape_defects = measure_shape_defects(&tape);
        if !shape_defects.is_empty() {
            excluded.push(obj(vec![
                ("file", Value::Str(name)),
                ("reason", text("not a valid tape")),
                (
                    "defects",
                    Value::Array(
                        shape_defects
                            .iter()
                            .map(|d| Value::Str(d.clone()))
                            .collect(),
                    ),
                ),
            ]));
            continue;
        }
        // The Python's canonical digest refuses a tape
        // whose whole value nests to depth 992 (probed:
        // a tool input at depth 988 scores, at 989 the
        // tape is excluded); the crate refuses only
        // past 1000, so the shallower boundary is
        // applied here before the digest runs.
        if container_depth(&tape) >= 992 || tape_digest(&tape).is_err() {
            excluded.push(obj(vec![
                ("file", Value::Str(name)),
                ("reason", text("cannot be cited under canonical form 1")),
                ("defects", Value::Array(Vec::new())),
            ]));
            continue;
        }
        tapes.push(SetEntry { file: name, tape });
    }
    Ok((tapes, excluded))
}

struct Indexed {
    file: Text,
    measures: Measures,
}

fn index_set(tapes: Vec<SetEntry>) -> (BTreeMap<String, Vec<Indexed>>, Vec<Value>) {
    let mut by_key: BTreeMap<String, Vec<Indexed>> = BTreeMap::new();
    let mut unmatchable = Vec::new();
    for entry in tapes {
        match task_key(&entry.tape) {
            None => unmatchable.push(obj(vec![
                ("file", Value::Str(entry.file)),
                ("reason", text("no main-chain user message with a text")),
            ])),
            Some(key) => {
                let measures = tape_measures(&entry.tape);
                by_key.entry(key).or_default().push(Indexed {
                    file: entry.file,
                    measures,
                });
            }
        }
    }
    (by_key, unmatchable)
}

fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}

struct PairRow {
    key: String,
    before: Measures,
    after: Measures,
    before_file: Text,
    after_file: Text,
}

impl PairRow {
    fn to_value(&self) -> Value {
        let delta = |before: Option<i128>, after: Option<i128>| match (before, after) {
            (Some(b), Some(a)) => crate::util::int_i128(a - b),
            _ => Value::Null,
        };
        let delta_f = |before: Option<f64>, after: Option<f64>| match (before, after) {
            (Some(b), Some(a)) => Value::Float(a - b),
            _ => Value::Null,
        };
        obj(vec![
            ("key", text(&self.key)),
            ("before", self.before.to_value()),
            ("after", self.after.to_value()),
            ("before_file", Value::Str(self.before_file.clone())),
            ("after_file", Value::Str(self.after_file.clone())),
            (
                "deltas",
                obj(vec![
                    ("cost", delta(self.before.cost, self.after.cost)),
                    (
                        "time_seconds",
                        delta_f(self.before.time_seconds, self.after.time_seconds),
                    ),
                    (
                        "rework",
                        delta(Some(self.before.rework), Some(self.after.rework)),
                    ),
                ]),
            ),
        ])
    }
}

#[allow(clippy::too_many_arguments)]
fn judge_outcome(
    registration: &Registration,
    aggregates: &Value,
    regressions: &[String],
    proposal: &str,
    pair_count: usize,
    judge: Option<&dyn Transport>,
    counts: CardCounts,
) -> Result<(String, Value), JudgeError> {
    let conclusive = || {
        obj(vec![
            ("status", text("not_consulted")),
            ("reason", text("the deterministic rule is conclusive")),
        ])
    };
    if registration.floor_exceeds(pair_count) {
        return Ok((
            "abstain".to_string(),
            obj(vec![
                ("status", text("not_consulted")),
                (
                    "reason",
                    text(&format!(
                        "insufficient pairs, {pair_count} of {}",
                        registration.floor_text()
                    )),
                ),
            ]),
        ));
    }
    if regressions.len() != 1 {
        return Ok((proposal.to_string(), conclusive()));
    }
    let judge = match judge {
        Some(judge) => judge,
        None => {
            return Ok((
                proposal.to_string(),
                obj(vec![
                    ("status", text("not_consulted")),
                    ("reason", text("no judge configured")),
                ]),
            ));
        }
    };
    let card = build_card(registration, aggregates, regressions, proposal, &counts);
    let card_digest = tape_digest(&Value::Str(Text::from_str(&card)))
        .map_err(|_| JudgeError("the card cannot be digested".to_string()))?;
    let mut judgment: Vec<(Text, Value)> = vec![
        (Text::from_str("card_sha256"), text(&card_digest)),
        (Text::from_str("card"), text(&card)),
    ];
    let answers = match judge_card(&card, judge) {
        Ok(answers) => answers,
        Err(error) => {
            judgment.push((Text::from_str("status"), text("unavailable")));
            judgment.push((Text::from_str("reason"), text(&error.0)));
            return Ok((proposal.to_string(), Value::Object(judgment)));
        }
    };
    if let Value::Object(fields) = &answers {
        for (key, value) in fields {
            judgment.push((key.clone(), value.clone()));
        }
    }
    let choice = answers
        .get("choice")
        .and_then(|c| match c {
            Value::Str(label) => label.as_str(),
            _ => None,
        })
        .unwrap_or_default();
    if choice == "abstain" {
        judgment.push((Text::from_str("status"), text("abstained")));
        judgment.push((Text::from_str("reason"), text("the judge abstained")));
        return Ok(("abstain".to_string(), Value::Object(judgment)));
    }
    let confidence = answers.get("confidence").cloned().unwrap_or(Value::Null);
    if confidence_below_floor(&confidence) {
        judgment.push((Text::from_str("status"), text("abstained")));
        judgment.push((
            Text::from_str("reason"),
            text(&format!(
                "the judge's confidence {} is below 0.6",
                value_text(&confidence)
            )),
        ));
        return Ok(("abstain".to_string(), Value::Object(judgment)));
    }
    judgment.push((Text::from_str("status"), text("judged")));
    Ok((choice, Value::Object(judgment)))
}

pub fn compare(
    before_dir: &Path,
    before_display: &str,
    after_dir: &Path,
    after_display: &str,
    registration: &Registration,
    judge: Option<&dyn Transport>,
) -> Result<Value, CompareError> {
    let (before_tapes, before_excluded) = load_set(before_dir, before_display, "before")?;
    let (after_tapes, after_excluded) = load_set(after_dir, after_display, "after")?;
    let before_scored = before_tapes.len();
    let after_scored = after_tapes.len();
    let (before_by_key, before_unmatchable) = index_set(before_tapes);
    let (after_by_key, after_unmatchable) = index_set(after_tapes);
    let mut keys: Vec<&String> = before_by_key.keys().chain(after_by_key.keys()).collect();
    keys.sort();
    keys.dedup();
    let mut pairs: Vec<PairRow> = Vec::new();
    let mut ambiguous: Vec<Value> = Vec::new();
    let mut unmatched_before: Vec<Value> = Vec::new();
    let mut unmatched_after: Vec<Value> = Vec::new();
    for key in keys {
        let left = before_by_key.get(key).map(Vec::as_slice).unwrap_or(&[]);
        let right = after_by_key.get(key).map(Vec::as_slice).unwrap_or(&[]);
        if left.len() == 1 && right.len() == 1 {
            pairs.push(PairRow {
                key: key.clone(),
                before: left[0].measures.clone(),
                after: right[0].measures.clone(),
                before_file: left[0].file.clone(),
                after_file: right[0].file.clone(),
            });
        } else if !left.is_empty() && !right.is_empty() {
            ambiguous.push(obj(vec![
                ("key", text(key)),
                ("before", int_usize(left.len())),
                ("after", int_usize(right.len())),
            ]));
        } else if !left.is_empty() {
            for entry in left {
                unmatched_before.push(obj(vec![
                    ("file", Value::Str(entry.file.clone())),
                    ("key", text(key)),
                ]));
            }
        } else {
            for entry in right {
                unmatched_after.push(obj(vec![
                    ("file", Value::Str(entry.file.clone())),
                    ("key", text(key)),
                ]));
            }
        }
    }
    let margins = &registration.margins;
    let mut regressions: Vec<String> = Vec::new();
    let mut aggregates: Vec<(Text, Value)> =
        vec![(Text::from_str("pairs"), int_usize(pairs.len()))];
    // (document key, margin index, relative?)
    for (name, doc_key, margin_index, relative) in [
        ("cost", "cost", 0usize, true),
        ("time_seconds", "time", 1usize, true),
        ("rework", "rework", 2usize, false),
    ] {
        // Means are computed in each measure's own
        // domain, as the Python computes them: integer
        // measures sum exactly and divide once (the
        // mean_int rounding), durations sum as floats
        // in pair order.
        let (before_mean, after_mean, counted_len) = match name {
            "cost" => {
                let counted: Vec<(i128, i128)> = pairs
                    .iter()
                    .filter_map(|pair| match (pair.before.cost, pair.after.cost) {
                        (Some(b), Some(a)) => Some((b, a)),
                        _ => None,
                    })
                    .collect();
                let n = counted.len();
                let mean_of = |side: usize| -> Option<f64> {
                    if n == 0 {
                        None
                    } else {
                        let sum: i128 = counted
                            .iter()
                            .map(|pair| if side == 0 { pair.0 } else { pair.1 })
                            .sum();
                        Some(crate::util::mean_int(sum, n as i128))
                    }
                };
                (mean_of(0), mean_of(1), n)
            }
            "rework" => {
                let n = pairs.len();
                let mean_of = |side: usize| -> Option<f64> {
                    if n == 0 {
                        None
                    } else {
                        let sum: i128 = pairs
                            .iter()
                            .map(|pair| {
                                if side == 0 {
                                    pair.before.rework
                                } else {
                                    pair.after.rework
                                }
                            })
                            .sum();
                        Some(crate::util::mean_int(sum, n as i128))
                    }
                };
                (mean_of(0), mean_of(1), n)
            }
            _ => {
                let counted: Vec<(f64, f64)> = pairs
                    .iter()
                    .filter_map(
                        |pair| match (pair.before.time_seconds, pair.after.time_seconds) {
                            (Some(b), Some(a)) => Some((b, a)),
                            _ => None,
                        },
                    )
                    .collect();
                let n = counted.len();
                (
                    mean(&counted.iter().map(|(b, _)| *b).collect::<Vec<_>>()),
                    mean(&counted.iter().map(|(_, a)| *a).collect::<Vec<_>>()),
                    n,
                )
            }
        };
        let mut entry: Vec<(Text, Value)> = vec![
            (
                Text::from_str("before_mean"),
                before_mean.map(Value::Float).unwrap_or(Value::Null),
            ),
            (
                Text::from_str("after_mean"),
                after_mean.map(Value::Float).unwrap_or(Value::Null),
            ),
            (Text::from_str("pairs_counted"), int_usize(counted_len)),
        ];
        if let (Some(before), Some(after)) = (before_mean, after_mean) {
            if relative {
                let delta_pct = if before != 0.0 {
                    Some((after - before) / before * 100.0)
                } else {
                    None
                };
                entry.push((
                    Text::from_str("delta_pct"),
                    delta_pct.map(Value::Float).unwrap_or(Value::Null),
                ));
                let regressed = if before == 0.0 {
                    after > 0.0
                } else {
                    exceeds(
                        delta_pct.expect("a nonzero base has a delta"),
                        &margins[margin_index],
                    )
                };
                if regressed {
                    regressions
                        .push(if name == "time_seconds" { "time" } else { name }.to_string());
                }
            } else {
                let delta = after - before;
                entry.push((Text::from_str("delta"), Value::Float(delta)));
                if exceeds(delta, &margins[margin_index]) {
                    regressions.push("rework".to_string());
                }
            }
        }
        aggregates.push((Text::from_str(doc_key), Value::Object(entry)));
    }
    let rate = |before_side: bool| -> (Option<f64>, usize) {
        let holders: Vec<&PairRow> = pairs
            .iter()
            .filter(|pair| {
                if before_side {
                    pair.before.has_error
                } else {
                    pair.after.has_error
                }
            })
            .collect();
        if holders.is_empty() {
            return (None, 0);
        }
        let rescued = holders
            .iter()
            .filter(|pair| {
                if before_side {
                    pair.before.rescued
                } else {
                    pair.after.rescued
                }
            })
            .count();
        (
            Some(rescued as f64 / holders.len() as f64 * 100.0),
            holders.len(),
        )
    };
    let (before_rate, before_den) = rate(true);
    let (after_rate, after_den) = rate(false);
    let mut rescue: Vec<(Text, Value)> = vec![
        (
            Text::from_str("before_rate"),
            before_rate.map(Value::Float).unwrap_or(Value::Null),
        ),
        (
            Text::from_str("after_rate"),
            after_rate.map(Value::Float).unwrap_or(Value::Null),
        ),
        (Text::from_str("before_denominator"), int_usize(before_den)),
        (Text::from_str("after_denominator"), int_usize(after_den)),
    ];
    if let (Some(before), Some(after)) = (before_rate, after_rate) {
        rescue.push((Text::from_str("delta_points"), Value::Float(after - before)));
        if exceeds(before - after, &margins[3]) {
            regressions.push("rescue rate".to_string());
        }
    }
    aggregates.push((Text::from_str("rescue"), Value::Object(rescue)));
    let aggregates = Value::Object(aggregates);
    let proposal = if regressions.len() >= 2 {
        "revert"
    } else if regressions.len() == 1 {
        "tune"
    } else {
        "keep"
    };
    let counts = CardCounts {
        ambiguous: ambiguous.len(),
        unmatched: unmatched_before.len() + unmatched_after.len(),
        excluded: before_excluded.len() + after_excluded.len(),
    };
    let (proposal, judgment) = match judge_outcome(
        registration,
        &aggregates,
        &regressions,
        proposal,
        pairs.len(),
        judge,
        counts,
    ) {
        Ok(outcome) => outcome,
        Err(error) => return Err(err(error.0)),
    };
    let pair_values: Vec<Value> = pairs.iter().map(PairRow::to_value).collect();
    let mut unmatchable: Vec<Value> = Vec::new();
    for entry in &before_unmatchable {
        if let Value::Object(fields) = entry {
            let mut with_side: Vec<(Text, Value)> = vec![(Text::from_str("side"), text("before"))];
            with_side.extend(fields.iter().cloned());
            unmatchable.push(Value::Object(with_side));
        }
    }
    for entry in &after_unmatchable {
        if let Value::Object(fields) = entry {
            let mut with_side: Vec<(Text, Value)> = vec![(Text::from_str("side"), text("after"))];
            with_side.extend(fields.iter().cloned());
            unmatchable.push(Value::Object(with_side));
        }
    }
    Ok(obj(vec![
        ("registration", registration.to_value()),
        ("matching_rule", text(MATCHING_RULE)),
        ("proposal_rule", text(PROPOSAL_RULE)),
        (
            "before",
            obj(vec![
                (
                    "tapes_read",
                    int_usize(before_scored + before_excluded.len()),
                ),
                ("tapes_scored", int_usize(before_scored)),
                ("excluded", Value::Array(before_excluded)),
            ]),
        ),
        (
            "after",
            obj(vec![
                ("tapes_read", int_usize(after_scored + after_excluded.len())),
                ("tapes_scored", int_usize(after_scored)),
                ("excluded", Value::Array(after_excluded)),
            ]),
        ),
        ("pairs", Value::Array(pair_values)),
        ("aggregates", aggregates),
        (
            "unmatched",
            obj(vec![
                ("before", Value::Array(unmatched_before)),
                ("after", Value::Array(unmatched_after)),
            ]),
        ),
        ("ambiguous", Value::Array(ambiguous)),
        ("unmatchable", Value::Array(unmatchable)),
        (
            "regressions",
            Value::Array(regressions.iter().map(|r| text(r)).collect()),
        ),
        ("proposal", text(&proposal)),
        ("judgment", judgment),
    ]))
}
