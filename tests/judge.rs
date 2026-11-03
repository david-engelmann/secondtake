//! ST-13 to ST-19. Judgment in the band. Ported from
//! tests/test_judge.py, case for case: the combination
//! rule is pinned with scripted transports returning
//! answers in Jev's recorded shapes, the card rendering
//! is pinned against a hand-written text, and no test
//! performs a network call.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use flighttape::identity::tape_digest;
use flighttape::json::parse;
use flighttape::value::Value;
use secondtake::compare::{compare, load_registration, Registration};
use secondtake::emit::dumps_document;
use secondtake::judge::{
    build_card, judge_card, CardCounts, FixtureTransport, JudgeError, LiveTransport, Transport,
};
use secondtake::util::{obj, text};

const CARD: &str = "Setup change: new system prompt, registered 2026-01-04.\n\
Matched pairs: 1. Ambiguous task keys excluded: 0. Unmatched tapes: 0. Excluded tapes: 0.\n\
Cost per pair: before 150, after 180, change 20 percent against a margin of 10 percent: regressed.\n\
Time per pair: before 60, after 60, change 0 percent against a margin of 10 percent: within margin.\n\
Rework per pair: before 0, after 0, change 0 against a margin of 1 per pair: within margin.\n\
Rescue rate: before 100 percent over 1 pairs holding an error, after 100 percent over 1, change 0 points against a margin of 5 points down: within margin.\n\
Regressions past margins: cost.\n\
The deterministic rule proposes tune.";

fn stamp(seconds: i64) -> String {
    let total = 9 * 3600 + seconds;
    format!(
        "2026-01-05T{:02}:{:02}:{:02}Z",
        total / 3600,
        (total % 3600) / 60,
        total % 60
    )
}

fn json(text_in: &str) -> Value {
    parse(text_in).expect("test json parses")
}

/// Escape a string for embedding in a JSON text.
fn json_escape(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            _ => out.push(ch),
        }
    }
    out
}

fn make_tape(tokens: (i64, i64), error: bool, outcome: &str) -> Value {
    json(&format!(
        "{{\"tape_version\": \"1.0\", \"run\": {{\"harness\": \"synthetic\", \
         \"session_id\": \"s-1\", \"totals\": {{\"input_tokens\": {}, \
         \"output_tokens\": {}}}}}, \"events\": [\
         {{\"seq\": 0, \"kind\": \"session_start\", \"sidechain\": false, \"timestamp\": \"{}\"}}, \
         {{\"seq\": 1, \"kind\": \"message\", \"sidechain\": false, \"role\": \"user\", \
         \"text\": \"Fix the parser\", \"timestamp\": \"{}\"}}, \
         {{\"seq\": 2, \"kind\": \"tool_call\", \"sidechain\": false, \"tool\": \"edit\", \
         \"call_id\": \"c0\", \"input\": {{\"path\": \"a.py\"}}, \"timestamp\": \"{}\"}}, \
         {{\"seq\": 3, \"kind\": \"tool_result\", \"sidechain\": false, \"call_id\": \"c0\", \
         \"is_error\": {}, \"timestamp\": \"{}\"}}, \
         {{\"seq\": 4, \"kind\": \"session_end\", \"sidechain\": false, \"timestamp\": \"{}\"}}], \
         \"outcome\": {{\"status\": \"{}\"}}}}",
        tokens.0,
        tokens.1,
        stamp(0),
        stamp(1),
        stamp(2),
        if error { "true" } else { "false" },
        stamp(3),
        stamp(60),
        outcome
    ))
}

fn temp_root(tag: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("secondtake-judge-{}-{}", std::process::id(), tag));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("temp root");
    root
}

fn write_set(directory: &Path, tapes: &[(&str, Value)]) -> PathBuf {
    std::fs::create_dir_all(directory).expect("set dir");
    for (name, tape) in tapes {
        std::fs::write(
            directory.join(name),
            dumps_document(tape).expect("the tape renders"),
        )
        .expect("tape file");
    }
    directory.to_path_buf()
}

fn registration(root: &Path, extra: &str) -> Registration {
    let path = root.join("registration.json");
    let merged = if extra.is_empty() {
        "{\"change\": \"new system prompt\", \"date\": \"2026-01-04\"}".to_string()
    } else {
        format!("{{\"change\": \"new system prompt\", \"date\": \"2026-01-04\", {extra}}}")
    };
    std::fs::write(&path, merged).expect("registration file");
    load_registration(&path, "registration").expect("registration loads")
}

fn band(root: &Path) -> (PathBuf, PathBuf) {
    let before = write_set(
        &root.join("b"),
        &[("a.json", make_tape((100, 50), true, "completed"))],
    );
    let after = write_set(
        &root.join("a"),
        &[("a.json", make_tape((130, 50), true, "completed"))],
    );
    (before, after)
}

fn number(value: f64) -> Value {
    if value == value.trunc() && value.abs() < 9.0e15 {
        secondtake::util::int(&(value as i64).to_string())
    } else {
        Value::Float(value)
    }
}

fn choice_response(choice: &str, confidence: f64) -> Value {
    obj(vec![
        ("model", text("jev-1.13.0")),
        (
            "answers",
            obj(vec![(
                "decision",
                obj(vec![
                    ("type", text("choice")),
                    ("choice", text(choice)),
                    ("confidence", number(confidence)),
                    (
                        "probabilities",
                        obj(vec![
                            ("keep", Value::Float(0.05)),
                            ("tune", Value::Float(0.1)),
                            ("revert", Value::Float(0.8)),
                            ("abstain", Value::Float(0.05)),
                        ]),
                    ),
                ]),
            )]),
        ),
        (
            "usage",
            obj(vec![
                ("input_tokens", secondtake::util::int("545")),
                ("output_tokens", secondtake::util::int("50")),
            ]),
        ),
    ])
}

fn score_response(score: f64) -> Value {
    let legend: Vec<(String, Value)> = (0..10)
        .map(|i| (i.to_string(), text(&format!("Level {}", i + 1))))
        .collect();
    let probabilities: Vec<(String, Value)> = (0..10)
        .map(|i| (i.to_string(), Value::Float(0.1)))
        .collect();
    let pairs = |items: Vec<(String, Value)>| -> Value {
        Value::Object(
            items
                .into_iter()
                .map(|(k, v)| (flighttape::value::Text::from_str(&k), v))
                .collect(),
        )
    };
    obj(vec![
        ("model", text("jev-1.13.0")),
        (
            "answers",
            obj(vec![(
                "decision",
                obj(vec![
                    ("type", text("score")),
                    ("score", number(score)),
                    ("confidence", Value::Float(0.5)),
                    ("legend", pairs(legend)),
                    ("probabilities", pairs(probabilities)),
                ]),
            )]),
        ),
        (
            "usage",
            obj(vec![
                ("input_tokens", secondtake::util::int("480")),
                ("output_tokens", secondtake::util::int("44")),
            ]),
        ),
    ])
}

struct Scripted {
    responses: HashMap<String, Value>,
    calls: RefCell<Vec<String>>,
}

impl Scripted {
    fn new(responses: HashMap<String, Value>) -> Scripted {
        Scripted {
            responses,
            calls: RefCell::new(Vec::new()),
        }
    }
}

impl Transport for Scripted {
    fn ask(&self, _card: &str, kind: &str) -> Result<Value, JudgeError> {
        self.calls.borrow_mut().push(kind.to_string());
        Ok(self.responses.get(kind).expect("scripted kind").clone())
    }
}

struct Down;

impl Transport for Down {
    fn ask(&self, _card: &str, _kind: &str) -> Result<Value, JudgeError> {
        Err(JudgeError("the judge call failed".to_string()))
    }
}

struct Exploding;

impl Transport for Exploding {
    fn ask(&self, _card: &str, _kind: &str) -> Result<Value, JudgeError> {
        panic!("consulted outside the band");
    }
}

fn judged(choice: &str, confidence: f64) -> Scripted {
    Scripted::new(HashMap::from([
        ("choice".to_string(), choice_response(choice, confidence)),
        ("severity".to_string(), score_response(7.5)),
        ("match".to_string(), score_response(8.25)),
    ]))
}

fn run(before: &Path, after: &Path, reg: &Registration, judge: &dyn Transport) -> Value {
    compare(
        before,
        before.to_str().unwrap(),
        after,
        after.to_str().unwrap(),
        reg,
        Some(judge),
    )
    .expect("comparison runs")
}

fn get<'a>(value: &'a Value, key: &str) -> &'a Value {
    value.get(key).expect("key present")
}

fn st(value: &Value) -> String {
    match value {
        Value::Str(text) => text.as_str().expect("well-formed"),
        _ => panic!("not a string"),
    }
}

#[test]
fn the_card_is_the_stated_text() {
    let root = temp_root("card");
    let reg = registration(&root, "");
    let (before, after) = band(&root);
    let document = run(&before, &after, &reg, &judged("revert", 0.9));
    assert_eq!(st(get(get(&document, "judgment"), "card")), CARD);
    let expected = tape_digest(&text(CARD)).expect("digests");
    assert_eq!(st(get(get(&document, "judgment"), "card_sha256")), expected);
}

#[test]
fn a_confident_choice_stands() {
    let root = temp_root("stands");
    let reg = registration(&root, "");
    let (before, after) = band(&root);
    let transport = judged("revert", 0.9);
    let document = run(&before, &after, &reg, &transport);
    assert_eq!(
        *transport.calls.borrow(),
        vec![
            "choice".to_string(),
            "severity".to_string(),
            "match".to_string()
        ]
    );
    assert_eq!(st(get(&document, "proposal")), "revert");
    let judgment = get(&document, "judgment");
    assert_eq!(st(get(judgment, "status")), "judged");
    assert_eq!(st(get(judgment, "model")), "jev-1.13.0");
    assert_eq!(st(get(judgment, "choice")), "revert");
    assert_eq!(get(judgment, "confidence"), &Value::Float(0.9));
    assert_eq!(get(judgment, "severity"), &Value::Float(7.5));
    assert_eq!(get(judgment, "match_confidence"), &Value::Float(8.25));
    assert_eq!(
        get(get(judgment, "probabilities"), "revert"),
        &Value::Float(0.8)
    );
}

#[test]
fn the_judge_abstaining_abstains() {
    let root = temp_root("abstain");
    let reg = registration(&root, "");
    let (before, after) = band(&root);
    let document = run(&before, &after, &reg, &judged("abstain", 0.9));
    assert_eq!(st(get(&document, "proposal")), "abstain");
    let judgment = get(&document, "judgment");
    assert_eq!(st(get(judgment, "status")), "abstained");
    assert_eq!(st(get(judgment, "reason")), "the judge abstained");
}

#[test]
fn low_confidence_abstains() {
    let root = temp_root("lowconf");
    let reg = registration(&root, "");
    let (before, after) = band(&root);
    let document = run(&before, &after, &reg, &judged("keep", 0.4));
    assert_eq!(st(get(&document, "proposal")), "abstain");
    let judgment = get(&document, "judgment");
    assert_eq!(st(get(judgment, "status")), "abstained");
    assert!(st(get(judgment, "reason")).contains("below 0.6"));
}

#[test]
fn an_unavailable_judge_leaves_the_tune() {
    let root = temp_root("down");
    let reg = registration(&root, "");
    let (before, after) = band(&root);
    let document = run(&before, &after, &reg, &Down);
    assert_eq!(st(get(&document, "proposal")), "tune");
    let judgment = get(&document, "judgment");
    assert_eq!(st(get(judgment, "status")), "unavailable");
    assert_eq!(st(get(judgment, "reason")), "the judge call failed");
    assert_eq!(st(get(judgment, "card")), CARD);
}

#[test]
fn the_edges_never_consult() {
    let root = temp_root("edges");
    let reg = registration(&root, "");
    let before = write_set(
        &root.join("b2"),
        &[("a.json", make_tape((100, 50), false, "completed"))],
    );
    let after = write_set(
        &root.join("a2"),
        &[("a.json", make_tape((100, 50), false, "completed"))],
    );
    let document = run(&before, &after, &reg, &Exploding);
    assert_eq!(st(get(&document, "proposal")), "keep");
    assert_eq!(
        st(get(get(&document, "judgment"), "reason")),
        "the deterministic rule is conclusive"
    );
    let root2 = temp_root("edges2");
    let (before, after) = band(&root2);
    let floor_reg = registration(&root2, "\"min_pairs\": 3");
    let document = run(&before, &after, &floor_reg, &Exploding);
    assert_eq!(st(get(&document, "proposal")), "abstain");
    assert_eq!(
        st(get(get(&document, "judgment"), "reason")),
        "insufficient pairs, 1 of 3"
    );
}

#[test]
fn the_floor_refuses_bad_registrations() {
    let root = temp_root("floor");
    for extra in ["\"min_pairs\": 0", "\"min_pairs\": \"3\""] {
        let path = root.join("registration.json");
        std::fs::write(
            &path,
            format!("{{\"change\": \"new system prompt\", \"date\": \"2026-01-04\", {extra}}}"),
        )
        .unwrap();
        let error = load_registration(&path, "registration").expect_err("refuses");
        assert!(error.0.contains("min_pairs"), "{error}");
    }
}

#[test]
fn malformed_answers_are_unavailable() {
    let mut missing_label = choice_response("revert", 0.9);
    if let Some(Value::Object(probs)) = missing_label
        .get_mut("answers")
        .and_then(|a| a.get_mut("decision"))
        .and_then(|d| d.get_mut("probabilities"))
    {
        probs.retain(|(key, _)| key.as_str().as_deref() != Some("abstain"));
    }
    let transport = Scripted::new(HashMap::from([
        ("choice".to_string(), missing_label),
        ("severity".to_string(), score_response(7.5)),
        ("match".to_string(), score_response(8.25)),
    ]));
    let error = judge_card(CARD, &transport).expect_err("malformed");
    assert!(error.0.contains("malformed"), "{error}");

    let mut bad_score = score_response(7.5);
    if let Some(Value::Object(fields)) = bad_score
        .get_mut("answers")
        .and_then(|a| a.get_mut("decision"))
    {
        for (key, value) in fields.iter_mut() {
            if key.as_str().as_deref() == Some("score") {
                *value = text("high");
            }
        }
    }
    let transport = Scripted::new(HashMap::from([
        ("choice".to_string(), choice_response("revert", 0.9)),
        ("severity".to_string(), bad_score),
        ("match".to_string(), score_response(8.25)),
    ]));
    let error = judge_card(CARD, &transport).expect_err("malformed");
    assert!(error.0.contains("malformed"), "{error}");

    let mut no_model = choice_response("revert", 0.9);
    if let Value::Object(fields) = &mut no_model {
        fields.retain(|(key, _)| key.as_str().as_deref() != Some("model"));
    }
    let transport = Scripted::new(HashMap::from([
        ("choice".to_string(), no_model),
        ("severity".to_string(), score_response(7.5)),
        ("match".to_string(), score_response(8.25)),
    ]));
    let error = judge_card(CARD, &transport).expect_err("unpinned");
    assert!(error.0.contains("unpinned model"), "{error}");
}

#[test]
fn the_fixture_transport_replays_by_digest() {
    let root = temp_root("fixture");
    let digest = tape_digest(&text(CARD)).unwrap();
    let path = root.join("judgments.json");
    let payload = format!(
        "{{\"cards\": [{{\"card_sha256\": \"{digest}\", \"responses\": {{\
         \"choice\": {}, \"severity\": {}, \"match\": {}}}}}]}}",
        dumps_value(&choice_response("keep", 0.77)),
        dumps_value(&score_response(3.0)),
        dumps_value(&score_response(9.0))
    );
    std::fs::write(&path, payload).unwrap();
    let transport = FixtureTransport::load(&path, "judgments").unwrap();
    let answers = judge_card(CARD, &transport).unwrap();
    assert_eq!(st(get(&answers, "choice")), "keep");
    assert_eq!(get(&answers, "severity"), &secondtake::util::int("3"));
    assert_eq!(
        get(&answers, "match_confidence"),
        &secondtake::util::int("9")
    );
    let error = judge_card(&format!("{CARD}\n"), &transport).expect_err("not recorded");
    assert!(error.0.contains("not recorded"), "{error}");
    let broken = root.join("broken.json");
    std::fs::write(&broken, "not json").unwrap();
    let error = FixtureTransport::load(&broken, "broken").expect_err("bad json");
    assert!(error.0.contains("not valid JSON"), "{error}");
}

/// Render a test response the way a file would carry
/// it (score_response renders integral floats through
/// the f64 Display, which drops the fraction; the
/// file form keeps the recorded values as JSON).
fn dumps_value(value: &Value) -> String {
    let bytes = dumps_document(value).expect("the value renders");
    String::from_utf8(bytes)
        .expect("utf8")
        .trim_end()
        .to_string()
}

#[test]
fn the_shipped_fixtures_replay() {
    let shipped =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/judgments_synthetic.json");
    let data = parse(&std::fs::read_to_string(&shipped).unwrap()).unwrap();
    assert_eq!(st(get(&data, "model")), "jev-1.13.0");
    let cards = get(&data, "cards").as_array().unwrap();
    assert_eq!(cards.len(), 8);
    for entry in cards {
        let digest = tape_digest(entry.get("card").expect("card text")).unwrap();
        assert_eq!(st(get(entry, "card_sha256")), digest);
        let responses = get(entry, "responses").as_object().unwrap();
        assert_eq!(responses.len(), 3);
    }
    let transport = FixtureTransport::load(&shipped, "shipped").unwrap();
    let answers = judge_card(CARD, &transport).unwrap();
    assert_eq!(st(get(&answers, "model")), "jev-1.13.0");
    assert_eq!(st(get(&answers, "choice")), "tune");
    assert_eq!(get(&answers, "confidence"), &Value::Float(0.85));
}

#[test]
fn out_of_range_answers_are_unavailable() {
    let scripted = |choice: Value, severity: Value, matched: Value| {
        Scripted::new(HashMap::from([
            ("choice".to_string(), choice),
            ("severity".to_string(), severity),
            ("match".to_string(), matched),
        ]))
    };
    let nan_choice = json(
        "{\"model\": \"jev-1.13.0\", \"answers\": {\"decision\": {\
         \"type\": \"choice\", \"choice\": \"revert\", \"confidence\": NaN, \
         \"probabilities\": {\"keep\": 0.1, \"tune\": 0.1, \"revert\": 0.7, \
         \"abstain\": 0.1}}}}",
    );
    let error = judge_card(
        CARD,
        &scripted(nan_choice, score_response(7.5), score_response(8.25)),
    )
    .expect_err("nan malformed");
    assert!(error.0.contains("malformed"), "{error}");
    let big = json(
        "{\"model\": \"jev-1.13.0\", \"answers\": {\"decision\": {\
         \"type\": \"choice\", \"choice\": \"revert\", \"confidence\": 0.9, \
         \"probabilities\": {\"keep\": 5.0, \"tune\": 5.0, \"revert\": 5.0, \
         \"abstain\": 5.0}}}}",
    );
    let error = judge_card(
        CARD,
        &scripted(big, score_response(7.5), score_response(8.25)),
    )
    .expect_err("big malformed");
    assert!(error.0.contains("malformed"), "{error}");
    let extra = json(
        "{\"model\": \"jev-1.13.0\", \"answers\": {\"decision\": {\
         \"type\": \"choice\", \"choice\": \"revert\", \"confidence\": 0.9, \
         \"probabilities\": {\"keep\": 0.05, \"tune\": 0.1, \"revert\": 0.8, \
         \"abstain\": 0.05, \"shrug\": 0.0}}}}",
    );
    let error = judge_card(
        CARD,
        &scripted(extra, score_response(7.5), score_response(8.25)),
    )
    .expect_err("extra label malformed");
    assert!(error.0.contains("malformed"), "{error}");
    let inf_score = json(
        "{\"model\": \"jev-1.13.0\", \"answers\": {\"decision\": {\
         \"type\": \"score\", \"score\": Infinity, \"confidence\": 0.5}}}",
    );
    let error = judge_card(
        CARD,
        &scripted(
            choice_response("revert", 0.9),
            inf_score,
            score_response(8.25),
        ),
    )
    .expect_err("inf malformed");
    assert!(error.0.contains("malformed"), "{error}");
    let wrong_model = json(
        "{\"model\": \"gpt-fabricated-9000\", \"answers\": {\"decision\": {\
         \"type\": \"score\", \"score\": 5, \"confidence\": 0.5}}}",
    );
    let error = judge_card(
        CARD,
        &scripted(
            choice_response("revert", 0.9),
            score_response(7.5),
            wrong_model,
        ),
    )
    .expect_err("wrong model");
    assert!(error.0.contains("unpinned model"), "{error}");
}

#[test]
fn a_nan_fixture_leaves_the_tune_and_stays_json() {
    let root = temp_root("nan-fixture");
    let digest = tape_digest(&text(CARD)).unwrap();
    let path = root.join("judgments.json");
    let nan_choice = json(
        "{\"model\": \"jev-1.13.0\", \"answers\": {\"decision\": {\
         \"type\": \"choice\", \"choice\": \"revert\", \"confidence\": NaN, \
         \"probabilities\": {\"keep\": 0.1, \"tune\": 0.1, \"revert\": 0.7, \
         \"abstain\": 0.1}}}}",
    );
    let payload = dumps_value(&obj(vec![(
        "cards",
        Value::Array(vec![obj(vec![
            ("card_sha256", text(&digest)),
            (
                "responses",
                obj(vec![
                    ("choice", nan_choice),
                    ("severity", score_response(7.5)),
                    ("match", score_response(8.25)),
                ]),
            ),
        ])]),
    )]));
    std::fs::write(&path, &payload).unwrap();
    assert!(payload.contains("NaN"));
    let reg = registration(&root, "");
    let (before, after) = band(&root);
    let transport = FixtureTransport::load(&path, "judgments").unwrap();
    let document = run(&before, &after, &reg, &transport);
    assert_eq!(st(get(&document, "proposal")), "tune");
    assert_eq!(st(get(get(&document, "judgment"), "status")), "unavailable");
    // The emitted document is strict JSON (the
    // Python asserts json.dumps with allow_nan=False):
    // its text carries no NaN or Infinity token.
    let rendered =
        String::from_utf8(dumps_document(&document).expect("the document renders")).unwrap();
    assert!(!rendered.contains("NaN"), "{rendered}");
    assert!(!rendered.contains("Infinity"), "{rendered}");
}

#[test]
fn the_confidence_boundary_is_exact() {
    let root = temp_root("boundary");
    let reg = registration(&root, "");
    let (before, after) = band(&root);
    let document = run(&before, &after, &reg, &judged("keep", 0.6));
    assert_eq!(st(get(&document, "proposal")), "keep");
    assert_eq!(st(get(get(&document, "judgment"), "status")), "judged");
    let document = run(&before, &after, &reg, &judged("keep", 0.59));
    assert_eq!(st(get(&document, "proposal")), "abstain");
    assert_eq!(st(get(get(&document, "judgment"), "status")), "abstained");
}

#[test]
fn two_regressions_never_consult() {
    let root = temp_root("two-regressions");
    let reg = registration(&root, "");
    let before = write_set(
        &root.join("b3"),
        &[("a.json", make_tape((100, 50), true, "completed"))],
    );
    let after = write_set(
        &root.join("a3"),
        &[("a.json", make_tape((130, 50), true, "failed"))],
    );
    let document = run(&before, &after, &reg, &Exploding);
    assert_eq!(
        get(&document, "regressions"),
        &Value::Array(vec![text("cost"), text("rescue rate")])
    );
    assert_eq!(st(get(&document, "proposal")), "revert");
    assert_eq!(
        st(get(get(&document, "judgment"), "reason")),
        "the deterministic rule is conclusive"
    );
}

#[test]
fn the_card_names_improvements() {
    let reg = Registration {
        change: "x".to_string(),
        date: "2026-01-01".to_string(),
        margins: [
            secondtake::util::int("10"),
            secondtake::util::int("10"),
            secondtake::util::int("1"),
            secondtake::util::int("5"),
        ],
        min_pairs: secondtake::util::int("1"),
    };
    let aggregates = json(
        "{\"pairs\": 4, \
         \"cost\": {\"before_mean\": 100.0, \"after_mean\": 120.0, \
         \"pairs_counted\": 4, \"delta_pct\": 20.0}, \
         \"time\": {\"before_mean\": 50.0, \"after_mean\": 50.0, \
         \"pairs_counted\": 4, \"delta_pct\": 0.0}, \
         \"rework\": {\"before_mean\": 1.0, \"after_mean\": 0.5, \
         \"pairs_counted\": 4, \"delta\": -0.5}, \
         \"rescue\": {\"before_rate\": 70.0, \"after_rate\": 78.0, \
         \"before_denominator\": 4, \"after_denominator\": 4, \
         \"delta_points\": 8.0}}",
    );
    let card = build_card(
        &reg,
        &aggregates,
        &["cost".to_string()],
        "tune",
        &CardCounts {
            ambiguous: 0,
            unmatched: 0,
            excluded: 0,
        },
    );
    assert!(
        card.contains(
            "Rework per pair: before 1, after 0.5, change -0.5 against a margin of 1 per pair: improved."
        ),
        "{card}"
    );
    assert!(
        card.contains("change 8 points against a margin of 5 points down: improved."),
        "{card}"
    );
}

#[test]
fn the_judgments_file_is_validated_whole() {
    let root = temp_root("whole");
    let digest = tape_digest(&text(CARD)).unwrap();
    let entry = format!(
        "{{\"card_sha256\": \"{digest}\", \"responses\": {{\
         \"choice\": {}, \"severity\": {}, \"match\": {}}}}}",
        dumps_value(&choice_response("revert", 0.9)),
        dumps_value(&score_response(7.5)),
        dumps_value(&score_response(8.25))
    );
    let path = root.join("judgments.json");
    std::fs::write(&path, format!("{{\"cards\": [{entry}, {entry}]}}")).unwrap();
    let error = FixtureTransport::load(&path, "j").expect_err("twice");
    assert!(error.0.contains("twice"), "{error}");
    std::fs::write(
        &path,
        format!(
            "{{\"cards\": [{{\"card_sha256\": \"{digest}\", \"card\": \"{} \", \
             \"responses\": {{\"choice\": {}, \"severity\": {}, \"match\": {}}}}}]}}",
            json_escape(CARD),
            dumps_value(&choice_response("revert", 0.9)),
            dumps_value(&score_response(7.5)),
            dumps_value(&score_response(8.25))
        ),
    )
    .unwrap();
    let error = FixtureTransport::load(&path, "j").expect_err("digest mismatch");
    assert!(error.0.contains("does not digest"), "{error}");
    std::fs::write(
        &path,
        format!("{{\"cards\": [{{\"card_sha256\": \"{digest}\"}}]}}"),
    )
    .unwrap();
    let error = FixtureTransport::load(&path, "j").expect_err("no responses");
    assert!(error.0.contains("no responses"), "{error}");
}

#[test]
fn live_transport_names_undecodable_output() {
    let root = temp_root("live-decode");
    let stub = root.join("jev_stub.py");
    std::fs::write(
        &stub,
        "import sys\nsys.stdout.buffer.write(b'\\xff\\xfe')\n",
    )
    .unwrap();
    let transport = LiveTransport::new(std::ffi::OsString::from(stub.to_str().unwrap()));
    let error = transport.ask(CARD, "choice").expect_err("undecodable");
    assert!(error.0.contains("could not be read"), "{error}");
    // A failing call and a non-JSON answer name
    // themselves as well.
    let stub = root.join("jev_fail.py");
    std::fs::write(&stub, "import sys\nsys.exit(3)\n").unwrap();
    let transport = LiveTransport::new(std::ffi::OsString::from(stub.to_str().unwrap()));
    let error = transport.ask(CARD, "choice").expect_err("fails");
    assert!(error.0.contains("the judge call failed"), "{error}");
    let stub = root.join("jev_text.py");
    std::fs::write(&stub, "print('not json at all')\n").unwrap();
    let transport = LiveTransport::new(std::ffi::OsString::from(stub.to_str().unwrap()));
    let error = transport.ask(CARD, "choice").expect_err("not json");
    assert!(error.0.contains("not JSON"), "{error}");
}
