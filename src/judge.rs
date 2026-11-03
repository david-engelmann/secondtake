//! ST-13 to ST-19. The decision card, the judge's
//! three calls, and the two transports that answer
//! them.

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use flighttape::identity::tape_digest;
use flighttape::json::{parse, ParseError};
use flighttape::value::{Text, Value};

use crate::emit::py_float;

use crate::compare::Registration;
use crate::util::{int_is_negative, number_as_f64, obj, text};

pub const MODEL: &str = "jev-1.13.0";
pub const LABELS: [&str; 4] = ["keep", "tune", "revert", "abstain"];
pub const CONFIDENCE_FLOOR: f64 = 0.6;

const FRAMING_SEVERITY: &str = "\n\nRate the severity of the tradeoff between \
the measures that improved and the measure that regressed, on the scale given.";
const FRAMING_MATCH: &str = "\n\nRate the confidence that the matched pairs \
compare like with like, given the ambiguous, unmatched, and excluded counts, \
on the scale given.";

#[derive(Debug, Clone, PartialEq)]
pub struct JudgeError(pub String);

impl std::fmt::Display for JudgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for JudgeError {}

fn err(message: impl Into<String>) -> JudgeError {
    JudgeError(message.into())
}

/// The counts the card's second line states.
pub struct CardCounts {
    pub ambiguous: usize,
    pub unmatched: usize,
    pub excluded: usize,
}

/// Python's f"{value:.2f}" with trailing zeros and a
/// bare point stripped, the card's number rendering.
/// Integers format exactly in Python until the int is
/// too large to convert to float, where the Python
/// raises OverflowError building a card at all; the
/// port prints the digits (named in the dossier).
pub fn num_text(value: &Value) -> String {
    match value {
        Value::Int(digits) => digits.clone(),
        Value::Float(number) => {
            let rendered = format!("{number:.2}");
            let trimmed = rendered.trim_end_matches('0');
            let trimmed = trimmed.trim_end_matches('.');
            trimmed.to_string()
        }
        _ => unreachable!("the card renders numbers only"),
    }
}

/// How a value prints inside a sentence: the Python's
/// str() of the stored number.
pub fn value_text(value: &Value) -> String {
    match value {
        Value::Int(digits) => digits.clone(),
        Value::Float(number) => py_float(*number),
        _ => unreachable!("sentences carry numbers only"),
    }
}

fn verdict(delta: Option<f64>, regressed: bool) -> &'static str {
    if regressed {
        return "regressed";
    }
    if let Some(delta) = delta {
        if delta != 0.0 {
            return if delta < 0.0 {
                "improved"
            } else {
                "within margin"
            };
        }
    }
    "within margin"
}

fn entry_f64(entry: &Value, key: &str) -> Option<f64> {
    entry.get(key).and_then(number_as_f64)
}

fn counted_line(
    label: &str,
    entry: &Value,
    delta_key: &str,
    margin_text: String,
    regressed: bool,
) -> String {
    let delta = entry.get(delta_key).cloned().unwrap_or(Value::Null);
    let change = match &delta {
        Value::Null => "n/a".to_string(),
        other => num_text(other),
    };
    let delta_f = number_as_f64(&delta);
    format!(
        "{label}: before {}, after {}, change {change} {margin_text}: {}.",
        num_text(entry.get("before_mean").expect("a counted entry has means")),
        num_text(entry.get("after_mean").expect("a counted entry has means")),
        verdict(delta_f, regressed),
    )
}

const UNMEASURED: &str = "unmeasured, no pair records it on both sides.";

pub fn build_card(
    registration: &Registration,
    aggregates: &Value,
    regressions: &[String],
    proposal: &str,
    counts: &CardCounts,
) -> String {
    let margins = &registration.margins;
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!(
        "Setup change: {}, registered {}.",
        registration.change, registration.date
    ));
    lines.push(format!(
        "Matched pairs: {}. Ambiguous task keys excluded: {}. \
         Unmatched tapes: {}. Excluded tapes: {}.",
        entry_f64(aggregates, "pairs").expect("aggregates carry the pair count") as i128,
        counts.ambiguous,
        counts.unmatched,
        counts.excluded
    ));
    for (key, label, margin) in [
        ("cost", "Cost per pair", &margins[0]),
        ("time", "Time per pair", &margins[1]),
    ] {
        let entry = aggregates.get(key).expect("aggregates carry the measure");
        if entry_f64(entry, "pairs_counted") == Some(0.0) {
            lines.push(format!("{label}: {UNMEASURED}"));
        } else {
            lines.push(counted_line(
                label,
                entry,
                "delta_pct",
                format!("percent against a margin of {} percent", num_text(margin)),
                regressions.iter().any(|r| r == key),
            ));
        }
    }
    let entry = aggregates.get("rework").expect("aggregates carry rework");
    if entry_f64(entry, "pairs_counted") == Some(0.0) {
        lines.push(format!("Rework per pair: {UNMEASURED}"));
    } else {
        lines.push(counted_line(
            "Rework per pair",
            entry,
            "delta",
            format!("against a margin of {} per pair", num_text(&margins[2])),
            regressions.iter().any(|r| r == "rework"),
        ));
    }
    let rescue = aggregates.get("rescue").expect("aggregates carry rescue");
    let before_rate = entry_f64(rescue, "before_rate");
    let after_rate = entry_f64(rescue, "after_rate");
    match (before_rate, after_rate) {
        (Some(before), Some(after)) => {
            let delta =
                entry_f64(rescue, "delta_points").expect("rates on both sides carry a delta");
            let regressed = regressions.iter().any(|r| r == "rescue rate");
            let verdict = if regressed {
                "regressed"
            } else if delta > 0.0 {
                "improved"
            } else {
                "within margin"
            };
            lines.push(format!(
                "Rescue rate: before {} percent over {} pairs holding an error, \
                 after {} percent over {}, change {} points against a margin of \
                 {} points down: {verdict}.",
                num_text(&Value::Float(before)),
                entry_f64(rescue, "before_denominator").expect("denominator") as i128,
                num_text(&Value::Float(after)),
                entry_f64(rescue, "after_denominator").expect("denominator") as i128,
                num_text(&Value::Float(delta)),
                num_text(&margins[3]),
            ));
        }
        _ => {
            lines.push("Rescue rate: unmeasured, a side holds no pair with an error.".to_string());
        }
    }
    lines.push(format!(
        "Regressions past margins: {}.",
        if regressions.is_empty() {
            "none".to_string()
        } else {
            regressions.join(", ")
        }
    ));
    lines.push(format!("The deterministic rule proposes {proposal}."));
    lines.join("\n")
}

fn answer_decision(response: &Value) -> Result<Value, JudgeError> {
    let object = match response.as_object() {
        Some(object) => object,
        None => return Err(err("the judge's answer is not an object")),
    };
    let _ = object;
    let model_ok = matches!(response.get("model"), Some(Value::Str(m)) if m.as_str().as_deref() == Some(MODEL));
    if !model_ok {
        return Err(err("the judge's answer names an unpinned model"));
    }
    let answers = match response.get("answers") {
        Some(answers) if answers.as_object().is_some() => answers,
        _ => return Err(err("the judge's answer carries no answers")),
    };
    match answers.get("decision") {
        Some(decision) if decision.as_object().is_some() => Ok(decision.clone()),
        _ => Err(err("the judge's answer carries no decision")),
    }
}

fn bounded(value: Option<&Value>, low: f64, high: f64) -> bool {
    match value.and_then(number_as_f64) {
        Some(number) => number.is_finite() && number >= low && number <= high,
        None => false,
    }
}

/// Whether a validated confidence sits below the
/// floor: ints compare exactly, floats as floats.
pub fn confidence_below_floor(confidence: &Value) -> bool {
    match confidence {
        Value::Int(digits) => int_is_negative(digits) || digits == "0",
        Value::Float(number) => *number < CONFIDENCE_FLOOR,
        _ => false,
    }
}

/// A judge transport: answers a card with the recorded
/// response shapes, or raises JudgeError.
pub trait Transport {
    fn ask(&self, card: &str, kind: &str) -> Result<Value, JudgeError>;
}

/// The three calls and their parsed answers (ST-16).
/// Any malformed answer is unavailable, never repaired
/// in flight (ST-19).
pub fn judge_card(card: &str, transport: &dyn Transport) -> Result<Value, JudgeError> {
    let decision = answer_decision(&transport.ask(card, "choice")?)?;
    let choice = decision.get("choice");
    let confidence = decision.get("confidence");
    let probabilities = decision.get("probabilities");
    let choice_label = match choice {
        Some(Value::Str(label)) => label.as_str(),
        _ => None,
    };
    let probabilities_ok = match probabilities {
        Some(probabilities) if probabilities.as_object().is_some() => {
            let entries = probabilities.as_object().expect("checked object");
            let keys_match = entries.len() == LABELS.len()
                && LABELS.iter().all(|label| {
                    entries
                        .iter()
                        .any(|(key, _)| key.as_str().as_deref() == Some(label))
                });
            keys_match
                && LABELS
                    .iter()
                    .all(|label| bounded(probabilities.get(label), 0.0, 1.0))
                && {
                    let sum: f64 = entries
                        .iter()
                        .filter_map(|(_, value)| number_as_f64(value))
                        .sum();
                    (sum - 1.0).abs() <= 0.02
                }
        }
        _ => false,
    };
    let choice_ok = matches!(decision.get("type"), Some(Value::Str(t)) if t.as_str().as_deref() == Some("choice"))
        && matches!(choice_label.as_deref(), Some(label) if LABELS.contains(&label))
        && bounded(confidence, 0.0, 1.0)
        && probabilities_ok;
    if !choice_ok {
        return Err(err("the judge's choice answer is malformed"));
    }
    let mut scores: Vec<(String, Value)> = Vec::new();
    for kind in ["severity", "match"] {
        let decision = answer_decision(&transport.ask(card, kind)?)?;
        let score_ok = matches!(decision.get("type"), Some(Value::Str(t)) if t.as_str().as_deref() == Some("score"))
            && bounded(decision.get("score"), 0.0, 10.0);
        if !score_ok {
            return Err(err(format!("the judge's {kind} answer is malformed")));
        }
        scores.push((
            kind.to_string(),
            decision
                .get("score")
                .expect("a bounded score exists")
                .clone(),
        ));
    }
    Ok(obj(vec![
        ("model", text(MODEL)),
        ("choice", choice.expect("a valid choice exists").clone()),
        (
            "confidence",
            confidence.expect("a bounded confidence exists").clone(),
        ),
        (
            "probabilities",
            probabilities.expect("valid probabilities exist").clone(),
        ),
        ("severity", scores[0].1.clone()),
        ("match_confidence", scores[1].1.clone()),
    ]))
}

/// Replays recorded responses, keyed by the card's
/// digest (ST-17, ST-19).
#[derive(Debug)]
pub struct FixtureTransport {
    by_digest: HashMap<String, Value>,
}

impl FixtureTransport {
    pub fn load(path: &Path, display: &str) -> Result<FixtureTransport, JudgeError> {
        let raw = match std::fs::read(path) {
            Ok(raw) => raw,
            Err(_) => {
                return Err(err(format!("no judgments file at {display}")));
            }
        };
        let data = match String::from_utf8(raw)
            .map_err(|_| ())
            .and_then(|text| parse(&text).map_err(|_| ()))
        {
            Ok(data) => data,
            Err(_) => {
                return Err(err(format!(
                    "the judgments file at {display} is not valid JSON"
                )));
            }
        };
        let cards = match data.get("cards") {
            Some(cards) if cards.as_array().is_some() => {
                cards.as_array().expect("checked array").clone()
            }
            _ => {
                return Err(err("the judgments file carries no cards list"));
            }
        };
        let mut by_digest: HashMap<String, Value> = HashMap::new();
        for entry in &cards {
            let (digest, responses) = match (entry.get("card_sha256"), entry.get("responses")) {
                (Some(Value::Str(digest)), Some(responses)) if responses.as_object().is_some() => {
                    (digest, responses.clone())
                }
                _ => {
                    return Err(err(
                        "a judgments entry carries no card digest or no responses",
                    ));
                }
            };
            let digest = match digest.as_str() {
                Some(digest) => digest,
                None => {
                    return Err(err(
                        "a judgments entry carries no card digest or no responses",
                    ));
                }
            };
            if by_digest.contains_key(&digest) {
                return Err(err("the judgments file records a card digest twice"));
            }
            if let Some(card) = entry.get("card") {
                let digests = match card {
                    Value::Str(card_text) => tape_digest(&Value::Str(card_text.clone())).ok(),
                    _ => None,
                };
                if digests.as_deref() != Some(digest.as_str()) {
                    return Err(err(
                        "a recorded card does not digest to the digest it is filed under",
                    ));
                }
            }
            by_digest.insert(digest.to_string(), responses);
        }
        Ok(FixtureTransport { by_digest })
    }
}

impl Transport for FixtureTransport {
    fn ask(&self, card: &str, kind: &str) -> Result<Value, JudgeError> {
        let digest = tape_digest(&Value::Str(Text::from_str(card)))
            .map_err(|_| err("the card cannot be digested"))?;
        let responses = match self.by_digest.get(&digest) {
            Some(responses) => responses,
            None => {
                return Err(err("the card is not recorded in the judgments file"));
            }
        };
        match responses.get(kind) {
            Some(response) if response.as_object().is_some() => Ok(response.clone()),
            _ => Err(err(format!("the recorded {kind} answer is missing"))),
        }
    }
}

/// Calls Jev through the connector's call script
/// (ST-17). The Python ran the script with its own
/// interpreter; the port resolves python3 on the PATH
/// (named in the dossier).
#[derive(Debug)]
pub struct LiveTransport {
    script: String,
}

impl LiveTransport {
    pub fn new(script: String) -> LiveTransport {
        LiveTransport { script }
    }
}

impl Transport for LiveTransport {
    fn ask(&self, card: &str, kind: &str) -> Result<Value, JudgeError> {
        let framing = match kind {
            "severity" => FRAMING_SEVERITY,
            "match" => FRAMING_MATCH,
            _ => "",
        };
        let state = format!("{card}{framing}");
        let mut command = Command::new("python3");
        command
            .arg(&self.script)
            .arg("--state")
            .arg(&state)
            .arg("--labels")
            .arg(LABELS.join(","));
        if kind != "choice" {
            command.arg("--question-type").arg("score");
        }
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(_) => return Err(err("the judge call could not be made")),
        };
        let deadline = Instant::now() + Duration::from_secs(180);
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => {
                    if Instant::now() >= deadline {
                        let _ = child.kill();
                        return Err(err("the judge call could not be made"));
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(_) => return Err(err("the judge call could not be made")),
            }
        };
        let mut out = Vec::new();
        if let Some(mut stdout) = child.stdout.take() {
            let _ = stdout.read_to_end(&mut out);
        }
        if !status.success() {
            return Err(err("the judge call failed"));
        }
        let text_out = match String::from_utf8(out) {
            Ok(text) => text,
            Err(_) => return Err(err("the judge's answer could not be read")),
        };
        match parse(&text_out) {
            Ok(value) => Ok(value),
            Err(ParseError::TooLarge) | Err(ParseError::Invalid) => {
                Err(err("the judge's answer is not JSON"))
            }
        }
    }
}
