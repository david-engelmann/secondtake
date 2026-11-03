//! ST-12, ST-17. A thin command line: compare.
//! Failures are named errors with exit 1. The parse
//! reproduces the Python's argparse behavior for this
//! grammar byte for byte: its help texts, its error
//! shapes, and the order they fire in (a scan error
//! before help, help before the required check, the
//! required check before unrecognized arguments).

use std::io::Write;
use std::path::{Component, Path, PathBuf};

use flighttape::pyrepr::repr_text;
use flighttape::value::Text;

use crate::compare::{compare, load_registration};
use crate::emit::dumps_document;
use crate::helptext::{COMPARE_HELP, COMPARE_USAGE, TOP_HELP, TOP_USAGE};
use crate::judge::{FixtureTransport, JudgeError, LiveTransport, Transport};

struct Parsed {
    before: String,
    after: String,
    registration: String,
    judge: Option<String>,
    output: Option<String>,
}

enum ParseOutcome {
    Run(Box<Parsed>),
    Exit(i32),
}

fn parse_error(usage: &str, prog: &str, message: &str) -> ParseOutcome {
    eprintln!("{usage}\n{prog}: error: {message}");
    ParseOutcome::Exit(2)
}

/// Whether a token is shaped like an option: it starts
/// with a dash, is not a bare dash, and does not look
/// like a negative number (no option in this grammar
/// does, so argparse lets those through as values).
fn option_shaped(token: &str) -> bool {
    if token == "-" || !token.starts_with('-') {
        return false;
    }
    let rest = &token[1..];
    let negative_number = {
        let mut chars = rest.chars();
        matches!(chars.next(), Some(c) if c.is_ascii_digit())
            || (rest.starts_with('.')
                && matches!(chars.next(), Some(c) if c.is_ascii_digit()))
    };
    !negative_number
}

/// Whether a long-option name is --help or an
/// unambiguous prefix of it.
fn is_help_name(name: &str) -> bool {
    !name.is_empty() && "--help".starts_with(name) && name.starts_with("--h")
}

const LONG_OPTIONS: [&str; 3] = ["--registration", "--judge", "--output"];

fn match_long(name: &str) -> Option<&'static str> {
    let mut found: Option<&'static str> = None;
    for option in LONG_OPTIONS {
        if option == name {
            return Some(option);
        }
        if option.starts_with(name) {
            found = Some(option);
        }
    }
    found
}

fn canonical_label(canonical: &str) -> &'static str {
    match canonical {
        "--registration" => "--registration",
        "--judge" => "--judge",
        _ => "-o/--output",
    }
}

/// Take the value for an option: a glued form wins,
/// else the next token, which an option-shaped token
/// can never be. On success `index` still points at
/// the option token; the caller advances past both.
fn take_value(
    tokens: &[String],
    index: &mut usize,
    glued: Option<String>,
    canonical: &str,
) -> Result<String, ParseOutcome> {
    if let Some(value) = glued {
        return Ok(value);
    }
    match tokens.get(*index + 1) {
        Some(next) if !option_shaped(next) => {
            *index += 1;
            Ok(next.clone())
        }
        _ => Err(parse_error(
            COMPARE_USAGE,
            "secondtake compare",
            &format!(
                "argument {}: expected one argument",
                canonical_label(canonical)
            ),
        )),
    }
}

/// Parse the compare subcommand's tokens.
fn parse_compare(tokens: &[String], top_extras: &[String]) -> ParseOutcome {
    let mut before: Option<String> = None;
    let mut after: Option<String> = None;
    let mut extras: Vec<String> = top_extras.to_vec();
    let mut registration: Option<String> = None;
    let mut judge: Option<String> = None;
    let mut output: Option<String> = None;
    let mut help_seen = false;
    let mut separated = false;
    let mut index = 0usize;

    let push_positional = |token: &str,
                               before: &mut Option<String>,
                               after: &mut Option<String>,
                               extras: &mut Vec<String>| {
        if before.is_none() {
            *before = Some(token.to_string());
        } else if after.is_none() {
            *after = Some(token.to_string());
        } else {
            extras.push(token.to_string());
        }
    };

    while index < tokens.len() {
        let token = &tokens[index];
        if separated {
            push_positional(token, &mut before, &mut after, &mut extras);
            index += 1;
            continue;
        }
        if token == "--" {
            separated = true;
            index += 1;
            continue;
        }
        if token.starts_with("--") {
            let (name, glued) = match token.find('=') {
                Some(at) => (&token[..at], Some(token[at + 1..].to_string())),
                None => (token.as_str(), None),
            };
            if is_help_name(name) {
                if let Some(value) = glued {
                    return parse_error(
                        COMPARE_USAGE,
                        "secondtake compare",
                        &format!(
                            "argument -h/--help: ignored explicit argument {}",
                            repr_text(&Text::from_str(&value))
                        ),
                    );
                }
                help_seen = true;
                index += 1;
                continue;
            }
            match match_long(name) {
                Some(canonical) => {
                    match take_value(tokens, &mut index, glued, canonical) {
                        Ok(value) => match canonical {
                            "--registration" => registration = Some(value),
                            "--judge" => judge = Some(value),
                            _ => output = Some(value),
                        },
                        Err(outcome) => return outcome,
                    }
                }
                None => extras.push(token.clone()),
            }
            index += 1;
            continue;
        }
        if option_shaped(token) {
            // A short cluster: h fires help, o takes the
            // rest of the cluster (or the next token) as
            // its value; anything else is unrecognized.
            let cluster = &token[1..];
            let mut unknown = false;
            for (at, ch) in cluster.char_indices() {
                match ch {
                    'h' => help_seen = true,
                    'o' => {
                        let rest = &cluster[at + 1..];
                        let glued = if rest.is_empty() {
                            None
                        } else if let Some(stripped) = rest.strip_prefix('=') {
                            Some(stripped.to_string())
                        } else {
                            Some(rest.to_string())
                        };
                        match take_value(tokens, &mut index, glued, "--output") {
                            Ok(value) => output = Some(value),
                            Err(outcome) => return outcome,
                        }
                        break;
                    }
                    _ => {
                        unknown = true;
                        break;
                    }
                }
            }
            if unknown {
                extras.push(token.clone());
            }
            index += 1;
            continue;
        }
        push_positional(token, &mut before, &mut after, &mut extras);
        index += 1;
    }

    if help_seen {
        print!("{COMPARE_HELP}");
        return ParseOutcome::Exit(0);
    }
    let mut missing: Vec<&str> = Vec::new();
    if before.is_none() {
        missing.push("before");
    }
    if after.is_none() {
        missing.push("after");
    }
    if registration.is_none() {
        missing.push("--registration");
    }
    if !missing.is_empty() {
        return parse_error(
            COMPARE_USAGE,
            "secondtake compare",
            &format!(
                "the following arguments are required: {}",
                missing.join(", ")
            ),
        );
    }
    if !extras.is_empty() {
        return parse_error(
            TOP_USAGE,
            "secondtake",
            &format!("unrecognized arguments: {}", extras.join(" ")),
        );
    }
    ParseOutcome::Run(Box::new(Parsed {
        before: before.expect("checked above"),
        after: after.expect("checked above"),
        registration: registration.expect("checked above"),
        judge,
        output,
    }))
}

/// Parse the top level: help, the command word, and
/// the held extras an unknown token becomes.
fn parse_top(tokens: &[String]) -> ParseOutcome {
    let mut extras: Vec<String> = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if token == "--" {
            // A leading "--" is the command word itself
            // to argparse, not a separator.
            return parse_error(
                TOP_USAGE,
                "secondtake",
                &format!(
                    "argument command: invalid choice: {} (choose from 'compare')",
                    repr_text(&Text::from_str(token))
                ),
            );
        }
        if token.starts_with("--") {
            let (name, glued) = match token.find('=') {
                Some(at) => (&token[..at], Some(token[at + 1..].to_string())),
                None => (token.as_str(), None),
            };
            if is_help_name(name) {
                if let Some(value) = glued {
                    return parse_error(
                        TOP_USAGE,
                        "secondtake",
                        &format!(
                            "argument -h/--help: ignored explicit argument {}",
                            repr_text(&Text::from_str(&value))
                        ),
                    );
                }
                print!("{TOP_HELP}");
                return ParseOutcome::Exit(0);
            }
            extras.push(token.clone());
            index += 1;
            continue;
        }
        if option_shaped(token) {
            // A cluster whose first letter is h prints
            // the top help; anything else is held.
            if token[1..].starts_with('h') {
                print!("{TOP_HELP}");
                return ParseOutcome::Exit(0);
            }
            extras.push(token.clone());
            index += 1;
            continue;
        }
        // The first positional-class token is the
        // command word, a leading "--" included.
        if token != "compare" {
            return parse_error(
                TOP_USAGE,
                "secondtake",
                &format!(
                    "argument command: invalid choice: {} (choose from 'compare')",
                    repr_text(&Text::from_str(token))
                ),
            );
        }
        return parse_compare(&tokens[index + 1..], &extras);
    }
    parse_error(
        TOP_USAGE,
        "secondtake",
        "the following arguments are required: command",
    )
}

/// Path.resolve() for the write guard: canonicalize
/// what exists, then append the rest with lexical
/// normalization.
fn resolve_path(path: &Path) -> PathBuf {
    if let Ok(canonical) = std::fs::canonicalize(path) {
        return canonical;
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/"))
            .join(path)
    };
    let mut ancestor: &Path = &absolute;
    let mut rest: Vec<Component> = Vec::new();
    while !ancestor.exists() {
        match ancestor.components().next_back() {
            Some(component) => rest.push(component),
            None => break,
        }
        match ancestor.parent() {
            Some(parent) => ancestor = parent,
            None => break,
        }
    }
    let mut base = std::fs::canonicalize(ancestor).unwrap_or_else(|_| ancestor.to_path_buf());
    for component in rest.iter().rev() {
        match component {
            Component::Normal(name) => base.push(name),
            Component::ParentDir => {
                base.pop();
            }
            _ => {}
        }
    }
    base
}

fn write_atomically(path: &Path, text: &str) -> std::io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let temp = parent.join(format!("{name}.{}.tmp", std::process::id()));
    let result = (|| -> std::io::Result<()> {
        std::fs::write(&temp, text)?;
        std::fs::rename(&temp, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

fn emit(document: &flighttape::value::Value, out: Option<&str>) -> i32 {
    let text = dumps_document(document);
    match out {
        Some(out) => match write_atomically(Path::new(out), &text) {
            Ok(()) => 0,
            Err(_) => {
                eprintln!("error, cannot write the comparison, {out}");
                1
            }
        },
        None => {
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            match handle.write_all(text.as_bytes()).and_then(|_| handle.flush()) {
                Ok(()) => 0,
                Err(_) => {
                    eprintln!("error, cannot write to stdout");
                    1
                }
            }
        }
    }
}

pub fn run_cli() -> i32 {
    let tokens: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    let parsed = match parse_top(&tokens) {
        ParseOutcome::Run(parsed) => parsed,
        ParseOutcome::Exit(code) => return code,
    };
    let registration = match load_registration(Path::new(&parsed.registration), &parsed.registration) {
        Ok(registration) => registration,
        Err(error) => {
            eprintln!("error, {error}");
            return 1;
        }
    };
    if let Some(out) = &parsed.output {
        let target = resolve_path(Path::new(out));
        for directory in [&parsed.before, &parsed.after] {
            let root = resolve_path(Path::new(directory));
            if target == root || target.starts_with(&root) {
                eprintln!("error, secondtake writes nothing inside the tape sets it reads");
                return 1;
            }
        }
        let mut stated: Vec<&String> = vec![&parsed.registration];
        if let Some(judge) = &parsed.judge {
            if judge != "live" {
                stated.push(judge);
            }
        }
        if stated.iter().any(|path| target == resolve_path(Path::new(path))) {
            eprintln!("error, secondtake writes nothing over the files it reads");
            return 1;
        }
    }
    let judge: Option<Box<dyn Transport>> = match &parsed.judge {
        Some(judge) if judge == "live" => match std::env::var("JEV_CALL") {
            Ok(script) if !script.is_empty() => Some(Box::new(LiveTransport::new(script))),
            _ => {
                eprintln!(
                    "error, live judging needs the JEV_CALL environment variable naming the connector call script"
                );
                return 1;
            }
        },
        Some(judge) => match FixtureTransport::load(Path::new(judge), judge) {
            Ok(transport) => Some(Box::new(transport)),
            Err(JudgeError(message)) => {
                eprintln!("error, {message}");
                return 1;
            }
        },
        None => None,
    };
    let document = match compare(
        Path::new(&parsed.before),
        &parsed.before,
        Path::new(&parsed.after),
        &parsed.after,
        &registration,
        judge.as_deref(),
    ) {
        Ok(document) => document,
        Err(error) => {
            eprintln!("error, {error}");
            return 1;
        }
    };
    emit(&document, parsed.output.as_deref())
}
