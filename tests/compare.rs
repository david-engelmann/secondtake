//! ST-5 to ST-12. The matched comparison. Ported from
//! tests/test_compare.py, case for case.

use std::path::{Path, PathBuf};

use flighttape::identity::tape_digest;
use flighttape::json::parse;
use flighttape::value::{Text, Value};
use secondtake::compare::{compare, load_registration, CompareError, Registration};
use secondtake::emit::dumps_document;
use secondtake::util::{int, obj, text};

const GOLDEN: &str = "cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3";

fn stamp(seconds: i64) -> String {
    let total = 9 * 3600 + seconds;
    format!(
        "2026-01-05T{:02}:{:02}:{:02}Z",
        total / 3600,
        (total % 3600) / 60,
        total % 60
    )
}

struct TapeOpts {
    task: &'static str,
    tokens: Option<(i64, i64)>,
    seconds: i64,
    redos: usize,
    error: bool,
    outcome: &'static str,
    user: bool,
}

impl Default for TapeOpts {
    fn default() -> Self {
        TapeOpts {
            task: "Fix the parser",
            tokens: Some((100, 50)),
            seconds: 60,
            redos: 0,
            error: false,
            outcome: "completed",
            user: true,
        }
    }
}

fn st(value: &Value) -> String {
    match value {
        Value::Str(text) => text.as_str().expect("well-formed text"),
        _ => panic!("not a string"),
    }
}

fn make_tape(opts: &TapeOpts) -> Value {
    let mut events: Vec<Value> = Vec::new();
    events.push(obj(vec![
        ("seq", int("0")),
        ("kind", text("session_start")),
        ("sidechain", Value::Bool(false)),
        ("timestamp", text(&stamp(0))),
    ]));
    let mut seq = 1i64;
    if opts.user {
        events.push(obj(vec![
            ("seq", int(&seq.to_string())),
            ("kind", text("message")),
            ("sidechain", Value::Bool(false)),
            ("role", text("user")),
            ("text", text(opts.task)),
            ("timestamp", text(&stamp(1))),
        ]));
        seq += 1;
    }
    let calls = 1 + opts.redos;
    for index in 0..calls {
        let call_id = format!("c{index}");
        events.push(obj(vec![
            ("seq", int(&seq.to_string())),
            ("kind", text("tool_call")),
            ("sidechain", Value::Bool(false)),
            ("tool", text("edit")),
            ("call_id", text(&call_id)),
            ("input", obj(vec![("path", text("a.py"))])),
            ("timestamp", text(&stamp(2 + 2 * index as i64))),
        ]));
        seq += 1;
        events.push(obj(vec![
            ("seq", int(&seq.to_string())),
            ("kind", text("tool_result")),
            ("sidechain", Value::Bool(false)),
            ("call_id", text(&call_id)),
            ("is_error", Value::Bool(opts.error && index == 0)),
            ("timestamp", text(&stamp(3 + 2 * index as i64))),
        ]));
        seq += 1;
    }
    events.push(obj(vec![
        ("seq", int(&seq.to_string())),
        ("kind", text("session_end")),
        ("sidechain", Value::Bool(false)),
        ("timestamp", text(&stamp(opts.seconds))),
    ]));
    let mut run: Vec<(&str, Value)> =
        vec![("harness", text("synthetic")), ("session_id", text("s-1"))];
    if let Some((input, output)) = opts.tokens {
        run.push((
            "totals",
            obj(vec![
                ("input_tokens", int(&input.to_string())),
                ("output_tokens", int(&output.to_string())),
            ]),
        ));
    }
    obj(vec![
        ("tape_version", text("1.0")),
        ("run", obj(run)),
        ("events", Value::Array(events)),
        ("outcome", obj(vec![("status", text(opts.outcome))])),
    ])
}

fn temp_root(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("secondtake-test-{}-{}", std::process::id(), tag));
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
    load_registration(&path, path.to_str().expect("utf8 path")).expect("registration loads")
}

fn run_compare(
    root: &Path,
    before: &[(&str, Value)],
    after: &[(&str, Value)],
    reg: &Registration,
) -> Value {
    let before_dir = write_set(&root.join("b"), before);
    let after_dir = write_set(&root.join("a"), after);
    compare(
        &before_dir,
        before_dir.to_str().unwrap(),
        &after_dir,
        after_dir.to_str().unwrap(),
        reg,
        None,
    )
    .expect("comparison runs")
}

fn get<'a>(value: &'a Value, key: &str) -> &'a Value {
    value.get(key).expect("key present")
}

fn as_f64(value: &Value) -> f64 {
    match value {
        Value::Float(number) => *number,
        Value::Int(digits) => digits.parse().expect("int as f64"),
        _ => panic!("not a number"),
    }
}

#[test]
fn the_digest_is_pinned_to_the_family_golden() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tape_synthetic.json");
    let tape = parse(&std::fs::read_to_string(fixture).expect("fixture")).expect("parses");
    assert_eq!(tape_digest(&tape).expect("digests"), GOLDEN);
}

#[test]
fn a_clean_pair_keeps() {
    let root = temp_root("keep");
    let reg = registration(&root, "");
    let document = run_compare(
        &root,
        &[("a.json", make_tape(&TapeOpts::default()))],
        &[("a.json", make_tape(&TapeOpts::default()))],
        &reg,
    );
    assert_eq!(get(&document, "proposal"), &text("keep"));
    assert_eq!(get(&document, "regressions"), &Value::Array(vec![]));
    assert_eq!(get(get(&document, "aggregates"), "pairs"), &int("1"));
    let pair = &get(&document, "pairs").as_array().unwrap()[0];
    assert_eq!(get(get(pair, "before"), "cost"), &int("150"));
    assert_eq!(get(get(pair, "deltas"), "cost"), &int("0"));
}

#[test]
fn one_regression_tunes_and_two_revert() {
    let root = temp_root("band");
    let reg = registration(&root, "");
    let errored = TapeOpts {
        error: true,
        ..Default::default()
    };
    let costlier = TapeOpts {
        tokens: Some((130, 50)),
        error: true,
        ..Default::default()
    };
    let document = run_compare(
        &root,
        &[("a.json", make_tape(&errored))],
        &[("a.json", make_tape(&costlier))],
        &reg,
    );
    assert_eq!(
        get(&document, "regressions"),
        &Value::Array(vec![text("cost")])
    );
    assert_eq!(get(&document, "proposal"), &text("tune"));
    let failed = TapeOpts {
        tokens: Some((130, 50)),
        error: true,
        outcome: "failed",
        ..Default::default()
    };
    let root2 = temp_root("band2");
    let document = run_compare(
        &root2,
        &[("a.json", make_tape(&errored))],
        &[("a.json", make_tape(&failed))],
        &reg,
    );
    assert_eq!(
        get(&document, "regressions"),
        &Value::Array(vec![text("cost"), text("rescue rate")])
    );
    assert_eq!(get(&document, "proposal"), &text("revert"));
    let rescue = get(get(&document, "aggregates"), "rescue");
    assert_eq!(as_f64(get(rescue, "before_rate")), 100.0);
    assert_eq!(as_f64(get(rescue, "after_rate")), 0.0);
    assert_eq!(as_f64(get(rescue, "delta_points")), -100.0);
}

#[test]
fn rework_regresses_past_one_per_pair() {
    let root = temp_root("rework");
    let reg = registration(&root, "");
    let document = run_compare(
        &root,
        &[(
            "a.json",
            make_tape(&TapeOpts {
                error: true,
                ..Default::default()
            }),
        )],
        &[(
            "a.json",
            make_tape(&TapeOpts {
                error: true,
                redos: 2,
                ..Default::default()
            }),
        )],
        &reg,
    );
    let pair = &get(&document, "pairs").as_array().unwrap()[0];
    assert_eq!(get(get(pair, "after"), "rework"), &int("2"));
    assert_eq!(
        get(&document, "regressions"),
        &Value::Array(vec![text("rework")])
    );
    assert_eq!(get(&document, "proposal"), &text("tune"));
}

#[test]
fn ambiguous_unmatched_and_unmatchable_are_reported() {
    let root = temp_root("matching");
    let reg = registration(&root, "");
    let same = TapeOpts {
        task: "Same task",
        ..Default::default()
    };
    let document = run_compare(
        &root,
        &[
            ("a.json", make_tape(&same)),
            ("b.json", make_tape(&same)),
            (
                "c.json",
                make_tape(&TapeOpts {
                    task: "Only before",
                    ..Default::default()
                }),
            ),
            (
                "d.json",
                make_tape(&TapeOpts {
                    user: false,
                    ..Default::default()
                }),
            ),
        ],
        &[
            ("a.json", make_tape(&same)),
            (
                "z.json",
                make_tape(&TapeOpts {
                    task: "Only after",
                    ..Default::default()
                }),
            ),
        ],
        &reg,
    );
    assert_eq!(get(get(&document, "aggregates"), "pairs"), &int("0"));
    let ambiguous = get(&document, "ambiguous").as_array().unwrap();
    assert_eq!(ambiguous.len(), 1);
    assert_eq!(get(&ambiguous[0], "before"), &int("2"));
    let unmatched = get(&document, "unmatched");
    let files = |side: &str| -> Vec<String> {
        get(unmatched, side)
            .as_array()
            .unwrap()
            .iter()
            .map(|u| st(get(u, "file")))
            .collect()
    };
    assert_eq!(files("before"), vec!["c.json".to_string()]);
    assert_eq!(files("after"), vec!["z.json".to_string()]);
    assert_eq!(
        get(&document, "unmatchable"),
        &Value::Array(vec![obj(vec![
            ("side", text("before")),
            ("file", text("d.json")),
            ("reason", text("no main-chain user message with a text")),
        ])])
    );
    assert_eq!(get(&document, "proposal"), &text("abstain"));
    assert_eq!(
        get(&document, "judgment"),
        &obj(vec![
            ("status", text("not_consulted")),
            ("reason", text("insufficient pairs, 0 of 1")),
        ])
    );
}

#[test]
fn excluded_tapes_are_named_with_defects() {
    let root = temp_root("excluded");
    let reg = registration(&root, "");
    let mut broken = make_tape(&TapeOpts::default());
    if let Value::Object(entries) = &mut broken {
        for (key, value) in entries.iter_mut() {
            if key.as_str().as_deref() == Some("tape_version") {
                *value = text("9.9");
            }
        }
    }
    let document = run_compare(
        &root,
        &[
            ("a.json", make_tape(&TapeOpts::default())),
            ("bad.json", broken),
        ],
        &[("a.json", make_tape(&TapeOpts::default()))],
        &reg,
    );
    let excluded = get(get(&document, "before"), "excluded")
        .as_array()
        .unwrap();
    assert_eq!(get(&excluded[0], "file"), &text("bad.json"));
    let defects = get(&excluded[0], "defects").as_array().unwrap();
    assert!(defects.iter().any(|d| st(d) == "tape_version is not 1.0"));
    assert_eq!(get(get(&document, "before"), "tapes_read"), &int("2"));
    assert_eq!(get(get(&document, "before"), "tapes_scored"), &int("1"));
}

#[test]
fn absent_cost_is_unmeasured_not_zero() {
    let root = temp_root("absent-cost");
    let reg = registration(&root, "");
    let document = run_compare(
        &root,
        &[(
            "a.json",
            make_tape(&TapeOpts {
                tokens: None,
                ..Default::default()
            }),
        )],
        &[("a.json", make_tape(&TapeOpts::default()))],
        &reg,
    );
    let cost = get(get(&document, "aggregates"), "cost");
    assert_eq!(get(cost, "pairs_counted"), &int("0"));
    assert_eq!(get(cost, "before_mean"), &Value::Null);
    assert_eq!(get(&document, "regressions"), &Value::Array(vec![]));
}

#[test]
fn margins_come_from_the_registration() {
    let root = temp_root("margins");
    let reg = registration(&root, "\"margins\": {\"cost_pct\": 25}");
    let document = run_compare(
        &root,
        &[("a.json", make_tape(&TapeOpts::default()))],
        &[(
            "a.json",
            make_tape(&TapeOpts {
                tokens: Some((130, 50)),
                ..Default::default()
            }),
        )],
        &reg,
    );
    assert_eq!(get(&document, "regressions"), &Value::Array(vec![]));
    let margins = get(get(&document, "registration"), "margins");
    assert_eq!(get(margins, "cost_pct"), &int("25"));
    assert_eq!(get(margins, "time_pct"), &int("10"));
}

#[test]
fn the_comparison_is_deterministic() {
    let root = temp_root("deterministic");
    let reg = registration(&root, "");
    let errored = TapeOpts {
        error: true,
        ..Default::default()
    };
    let before_dir = write_set(&root.join("b"), &[("a.json", make_tape(&errored))]);
    let after_dir = write_set(&root.join("a"), &[("a.json", make_tape(&errored))]);
    let first = compare(&before_dir, "b", &after_dir, "a", &reg, None).unwrap();
    let second = compare(&before_dir, "b", &after_dir, "a", &reg, None).unwrap();
    assert_eq!(dumps_document(&first), dumps_document(&second));
}

#[test]
fn registration_refusals() {
    let root = temp_root("refusals");
    let missing = root.join("missing.json");
    let error =
        load_registration(&missing, missing.to_str().unwrap()).expect_err("missing refuses");
    assert!(
        matches!(&error, CompareError(m) if m.contains("no registration")),
        "{error}"
    );
    let bad = root.join("bad.json");
    std::fs::write(&bad, "{not json").unwrap();
    let error = load_registration(&bad, "bad").expect_err("bad json refuses");
    assert!(
        matches!(&error, CompareError(m) if m.contains("not valid JSON")),
        "{error}"
    );
    let unknown = root.join("unknown.json");
    std::fs::write(
        &unknown,
        "{\"change\": \"x\", \"date\": \"2026-01-04\", \"margins\": {\"luck\": 3}}",
    )
    .unwrap();
    let error = load_registration(&unknown, "unknown").expect_err("unknown margin refuses");
    assert!(
        matches!(&error, CompareError(m) if m.contains("unknown margin")),
        "{error}"
    );
}

#[test]
fn the_cli_and_its_write_guard() {
    let root = temp_root("cli-guard");
    let before_dir = write_set(
        &root.join("b"),
        &[("a.json", make_tape(&TapeOpts::default()))],
    );
    let after_dir = write_set(
        &root.join("a"),
        &[("a.json", make_tape(&TapeOpts::default()))],
    );
    let reg_path = root.join("registration.json");
    std::fs::write(
        &reg_path,
        "{\"change\": \"new system prompt\", \"date\": \"2026-01-04\"}",
    )
    .unwrap();
    let out = root.join("comparison.json");
    let code = secondtake::cli::run_with_args(vec![
        "compare".into(),
        before_dir.to_str().unwrap().into(),
        after_dir.to_str().unwrap().into(),
        "--registration".into(),
        reg_path.to_str().unwrap().into(),
        "-o".into(),
        out.to_str().unwrap().into(),
    ]);
    assert_eq!(code, 0);
    let document = parse(&std::fs::read_to_string(&out).unwrap()).unwrap();
    assert_eq!(get(&document, "proposal"), &text("keep"));
    let inside = before_dir.join("comparison.json");
    let code = secondtake::cli::run_with_args(vec![
        "compare".into(),
        before_dir.to_str().unwrap().into(),
        after_dir.to_str().unwrap().into(),
        "--registration".into(),
        reg_path.to_str().unwrap().into(),
        "-o".into(),
        inside.to_str().unwrap().into(),
    ]);
    assert_eq!(code, 1);
    assert!(!inside.exists());
    let code = secondtake::cli::run_with_args(vec![
        "compare".into(),
        before_dir.to_str().unwrap().into(),
        after_dir.to_str().unwrap().into(),
        "--registration".into(),
        root.join("nope.json").to_str().unwrap().into(),
    ]);
    assert_eq!(code, 1);
}

#[cfg(unix)]
#[test]
fn the_write_guard_exits_1_and_leaves_the_loop_link_untouched() {
    // ST-49. The interpreter's resolve raises on a
    // symlink loop, so a guard spelling that runs a
    // chain back into itself fails the resolution
    // and the run exits 1 having written nothing,
    // where the resolution once compared a fiction
    // past the loop (and, for an output that was
    // itself the loop link, wrote over the link).
    let root = temp_root("cli-loop");
    let before_dir = write_set(
        &root.join("b"),
        &[("a.json", make_tape(&TapeOpts::default()))],
    );
    let after_dir = write_set(
        &root.join("a"),
        &[("a.json", make_tape(&TapeOpts::default()))],
    );
    let reg_path = root.join("registration.json");
    std::fs::write(
        &reg_path,
        "{\"change\": \"new system prompt\", \"date\": \"2026-01-04\"}",
    )
    .unwrap();
    std::os::unix::fs::symlink("midb", root.join("mida")).unwrap();
    std::os::unix::fs::symlink("mida", root.join("midb")).unwrap();
    std::os::unix::fs::symlink("selfloop", root.join("selfloop")).unwrap();
    let base_args: Vec<String> = vec![
        "compare".into(),
        before_dir.to_str().unwrap().into(),
        after_dir.to_str().unwrap().into(),
        "--registration".into(),
        reg_path.to_str().unwrap().into(),
    ];
    let run = |extra: Vec<String>| {
        let mut argv = base_args.clone();
        argv.extend(extra);
        secondtake::cli::run_with_args(argv)
    };
    // The output is the loop link itself: the run
    // fails and the link stands untouched.
    let code = run(vec![
        "-o".into(),
        root.join("selfloop").to_str().unwrap().into(),
    ]);
    assert_eq!(code, 1);
    assert!(std::fs::symlink_metadata(root.join("selfloop"))
        .unwrap()
        .file_type()
        .is_symlink());
    // The output spelled through the cycle.
    let code = run(vec![
        "-o".into(),
        root.join("mida").join("out.json").to_str().unwrap().into(),
    ]);
    assert_eq!(code, 1);
    assert!(!root.join("out.json").exists());
    // The stated judge spelled through the cycle.
    let code = run(vec![
        "--judge".into(),
        root.join("mida")
            .join("judgments.json")
            .to_str()
            .unwrap()
            .into(),
        "-o".into(),
        root.join("out.json").to_str().unwrap().into(),
    ]);
    assert_eq!(code, 1);
    assert!(!root.join("out.json").exists());
    // A link that resolves still refuses on the
    // guard's own terms: the output inside the
    // before set through a sound link.
    std::os::unix::fs::symlink("b", root.join("goodlink")).unwrap();
    let code = run(vec![
        "-o".into(),
        root.join("goodlink")
            .join("out.json")
            .to_str()
            .unwrap()
            .into(),
    ]);
    assert_eq!(code, 1);
    assert!(!before_dir.join("out.json").exists());
}

#[test]
fn the_fixture_tape_keys_by_its_real_text() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tape_synthetic.json");
    let tape = parse(&std::fs::read_to_string(fixture).unwrap()).unwrap();
    let key = secondtake::measures::task_key(&tape).expect("keys");
    let expected =
        tape_digest(&text("Synthetic prompt, fix the widget counter.")).expect("digests");
    assert_eq!(key, expected);
}

#[test]
fn cache_tokens_are_not_cost() {
    let root = temp_root("cache");
    let reg = registration(&root, "");
    let mut rich = make_tape(&TapeOpts::default());
    if let Some(Value::Object(entries)) = rich.get_mut("run").and_then(|run| run.get_mut("totals"))
    {
        entries.push((Text::from_str("cache_read_input_tokens"), int("999999")));
    }
    let document = run_compare(
        &root,
        &[("a.json", rich)],
        &[("a.json", make_tape(&TapeOpts::default()))],
        &reg,
    );
    let pair = &get(&document, "pairs").as_array().unwrap()[0];
    assert_eq!(get(get(pair, "before"), "cost"), &int("150"));
}

#[test]
fn pair_keys_are_the_task_text_digests() {
    let root = temp_root("pair-keys");
    let reg = registration(&root, "");
    let document = run_compare(
        &root,
        &[("a.json", make_tape(&TapeOpts::default()))],
        &[("a.json", make_tape(&TapeOpts::default()))],
        &reg,
    );
    let pair = &get(&document, "pairs").as_array().unwrap()[0];
    let expected = tape_digest(&text("Fix the parser")).unwrap();
    assert_eq!(get(pair, "key"), &text(&expected));
    assert!(document.get("proposal_rule").is_some());
}

#[test]
fn time_regresses_and_naive_stamps_read_as_utc() {
    let root = temp_root("time");
    let reg = registration(&root, "");
    let document = run_compare(
        &root,
        &[("a.json", make_tape(&TapeOpts::default()))],
        &[(
            "a.json",
            make_tape(&TapeOpts {
                seconds: 600,
                ..Default::default()
            }),
        )],
        &reg,
    );
    assert_eq!(
        get(&document, "regressions"),
        &Value::Array(vec![text("time")])
    );
    // Naive stamps: rebuild the before tape with naive
    // first and last stamps at the same instants.
    let mut mixed = make_tape(&TapeOpts::default());
    if let Some(Value::Array(events)) = mixed.get_mut("events") {
        let last = events.len() - 1;
        for (index, stamp) in [
            (0usize, "2026-01-05T09:00:00"),
            (last, "2026-01-05T09:01:00"),
        ] {
            if let Some(Value::Object(fields)) = events.get_mut(index) {
                for (key, value) in fields.iter_mut() {
                    if key.as_str().as_deref() == Some("timestamp") {
                        *value = text(stamp);
                    }
                }
            }
        }
    }
    let root2 = temp_root("time2");
    let document = run_compare(
        &root2,
        &[("a.json", mixed)],
        &[(
            "a.json",
            make_tape(&TapeOpts {
                seconds: 600,
                ..Default::default()
            }),
        )],
        &reg,
    );
    let pair = &get(&document, "pairs").as_array().unwrap()[0];
    assert_eq!(as_f64(get(get(pair, "before"), "time_seconds")), 60.0);
}

#[test]
fn measure_shape_defects_exclude_the_tape() {
    let root = temp_root("shape");
    let reg = registration(&root, "");
    let mut negative = make_tape(&TapeOpts::default());
    if let Some(Value::Object(entries)) = negative
        .get_mut("run")
        .and_then(|run| run.get_mut("totals"))
    {
        for (key, value) in entries.iter_mut() {
            if key.as_str().as_deref() == Some("input_tokens") {
                *value = int("-5");
            }
        }
    }
    let mut bare = make_tape(&TapeOpts::default());
    if let Value::Object(entries) = &mut bare {
        for (key, value) in entries.iter_mut() {
            if key.as_str().as_deref() == Some("outcome") {
                *value = text("completed");
            }
        }
    }
    let document = run_compare(
        &root,
        &[
            ("a.json", make_tape(&TapeOpts::default())),
            ("neg.json", negative),
            ("bare.json", bare),
        ],
        &[("a.json", make_tape(&TapeOpts::default()))],
        &reg,
    );
    let excluded = get(get(&document, "before"), "excluded")
        .as_array()
        .unwrap();
    let defects_of = |name: &str| -> Vec<String> {
        excluded
            .iter()
            .find(|e| st(get(e, "file")) == name)
            .map(|e| {
                get(e, "defects")
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(st)
                    .collect()
            })
            .unwrap_or_default()
    };
    assert!(defects_of("neg.json")
        .iter()
        .any(|d| d.contains("non-negative")));
    assert_eq!(
        defects_of("bare.json"),
        vec!["outcome is not an object".to_string()]
    );
    assert_eq!(get(get(&document, "before"), "tapes_scored"), &int("1"));
}

#[test]
fn advisory_repairs_huge_numbers_and_unreadable_sets() {
    let root = temp_root("huge");
    let huge = root.join("huge.json");
    let big_margin = format!("1{}", "0".repeat(400));
    std::fs::write(
        &huge,
        format!(
            "{{\"change\": \"new system prompt\", \"date\": \"2026-01-04\", \"margins\": {{\"cost_pct\": {big_margin}}}}}"
        ),
    )
    .unwrap();
    let reg = load_registration(&huge, "huge").expect("huge margin loads");
    assert_eq!(
        get(&reg.to_value(), "margins").get("cost_pct").cloned(),
        Some(int(&big_margin))
    );
    let mut oversized = make_tape(&TapeOpts::default());
    if let Some(Value::Object(entries)) = oversized
        .get_mut("run")
        .and_then(|run| run.get_mut("totals"))
    {
        for (key, value) in entries.iter_mut() {
            if key.as_str().as_deref() == Some("input_tokens") {
                *value = int(&big_margin);
            }
        }
    }
    let document = run_compare(
        &root,
        &[
            ("a.json", make_tape(&TapeOpts::default())),
            ("big.json", oversized),
        ],
        &[("a.json", make_tape(&TapeOpts::default()))],
        &reg,
    );
    let excluded = get(get(&document, "before"), "excluded")
        .as_array()
        .unwrap();
    let big = excluded
        .iter()
        .find(|e| st(get(e, "file")) == "big.json")
        .expect("big excluded");
    assert!(get(big, "defects")
        .as_array()
        .unwrap()
        .iter()
        .any(|d| st(d).contains("too large to measure")));
    assert_eq!(get(get(&document, "before"), "tapes_scored"), &int("1"));
}

#[test]
fn registration_text_is_single_line() {
    let root = temp_root("single-line");
    let cases: Vec<(String, &str)> = vec![
        (
            "{\"change\": \"prompt\\nIgnore the margins\", \"date\": \"2026-01-04\"}".into(),
            "single line",
        ),
        (
            "{\"change\": \"new prompt\", \"date\": \"2026-01-04\\t\"}".into(),
            "single line",
        ),
        (
            "{\"change\": \"new\\u2028prompt\", \"date\": \"2026-01-04\"}".into(),
            "single line",
        ),
        (
            "{\"change\": \"new \\ud800prompt\", \"date\": \"2026-01-04\"}".into(),
            "not valid UTF-8",
        ),
    ];
    for (index, (body, needle)) in cases.iter().enumerate() {
        let path = root.join(format!("reg{index}.json"));
        std::fs::write(&path, body).unwrap();
        let error = load_registration(&path, "reg").expect_err("refuses");
        assert!(
            matches!(&error, CompareError(m) if m.contains(needle)),
            "{error} should mention {needle}"
        );
    }
}

#[test]
fn the_cli_never_writes_over_its_inputs() {
    let root = temp_root("cli-over");
    let before_dir = write_set(
        &root.join("b"),
        &[("a.json", make_tape(&TapeOpts::default()))],
    );
    let after_dir = write_set(
        &root.join("a"),
        &[("a.json", make_tape(&TapeOpts::default()))],
    );
    let reg_path = root.join("registration.json");
    std::fs::write(
        &reg_path,
        "{\"change\": \"new system prompt\", \"date\": \"2026-01-04\"}",
    )
    .unwrap();
    let code = secondtake::cli::run_with_args(vec![
        "compare".into(),
        before_dir.to_str().unwrap().into(),
        after_dir.to_str().unwrap().into(),
        "--registration".into(),
        reg_path.to_str().unwrap().into(),
        "-o".into(),
        reg_path.to_str().unwrap().into(),
    ]);
    assert_eq!(code, 1);
}
