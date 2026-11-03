//! ST-12, ST-17. A thin command line: compare.
//! Failures are named errors with exit 1. The parse
//! reproduces the Python's argparse behavior for this
//! grammar byte for byte: its help texts, its error
//! shapes, and the order they fire in. Help fires at
//! the end of the token that carries it; a scan error
//! (a missing option value, an ignored explicit
//! argument, an ambiguous long option) fires when its
//! token is processed and outranks a pending help;
//! extras are only reported after the required check.
//! Argument tokens are carried as text in the
//! operating system's own decoding (fsdecode), so
//! argument bytes survive as the Python's do.

use std::io::Write;
use std::path::{Path, PathBuf};

use flighttape::value::Text;

use crate::compare::{compare, load_registration};
use crate::emit::dumps_document;
use crate::helptext::{COMPARE_HELP, COMPARE_USAGE, TOP_HELP, TOP_USAGE};
use crate::judge::{FixtureTransport, JudgeError, LiveTransport, Transport};
use crate::pyrepr::repr_text;
use crate::util::{fsdecode_units, fsencode_bytes, normalize_path_display, stderr_render};

/// One argument token: its text, and its decoded
/// scalars for the parser's character work.
struct Token {
    text: Text,
    scalars: Vec<u32>,
}

impl Token {
    fn from_units(units: Vec<u16>) -> Token {
        let mut scalars = Vec::new();
        let mut index = 0;
        while index < units.len() {
            let unit = units[index];
            if (0xD800..=0xDBFF).contains(&unit)
                && index + 1 < units.len()
                && (0xDC00..=0xDFFF).contains(&units[index + 1])
            {
                scalars.push(
                    0x10000 + (((unit as u32) - 0xD800) << 10) + (units[index + 1] as u32 - 0xDC00),
                );
                index += 2;
            } else {
                scalars.push(unit as u32);
                index += 1;
            }
        }
        Token {
            text: Text::from_units(units),
            scalars,
        }
    }

    fn rendered(&self) -> String {
        stderr_render(self.text.units())
    }
}

struct Parsed {
    before: Text,
    after: Text,
    registration: Text,
    judge: Option<Text>,
    output: Option<Text>,
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
fn option_shaped(token: &Token) -> bool {
    let scalars = &token.scalars;
    if scalars.len() < 2 || scalars[0] != '-' as u32 {
        return false;
    }
    let first = scalars[1];
    let digit = |c: u32| ('0' as u32..='9' as u32).contains(&c);
    let negative_number =
        digit(first) || (first == '.' as u32 && scalars.get(2).is_some_and(|c| digit(*c)));
    !negative_number
}

const LONG_OPTIONS: [&str; 4] = ["--help", "--registration", "--judge", "--output"];

fn scalars_of(text: &str) -> Vec<u32> {
    text.chars().map(|ch| ch as u32).collect()
}

/// Take the value for an option inside compare: a
/// glued form wins, else the next token, which an
/// option-shaped token can never be. On success
/// `index` still points at the option token; the
/// caller advances past both.
fn take_value(
    tokens: &[Token],
    index: &mut usize,
    glued: Option<Text>,
    canonical: &str,
) -> Result<Text, ParseOutcome> {
    if let Some(value) = glued {
        return Ok(value);
    }
    match tokens.get(*index + 1) {
        Some(next) if !option_shaped(next) => {
            *index += 1;
            Ok(next.text.clone())
        }
        _ => Err(parse_error(
            COMPARE_USAGE,
            "secondtake compare",
            &format!("argument {}: expected one argument", label_of(canonical)),
        )),
    }
}

fn label_of(canonical: &str) -> &'static str {
    match canonical {
        "--registration" => "--registration",
        "--judge" => "--judge",
        _ => "-o/--output",
    }
}

/// Split a long-option token at its first '=': the
/// name's scalars, and the glued value's text when an
/// '=' is present.
fn split_long(token: &Token) -> (&[u32], Option<Text>) {
    match token.scalars.iter().position(|c| *c == '=' as u32) {
        Some(at) => {
            let units = token.text.units();
            let mut unit_at = 0usize;
            let mut scalar_at = 0usize;
            while scalar_at < at {
                if (0xD800..=0xDBFF).contains(&units[unit_at])
                    && unit_at + 1 < units.len()
                    && (0xDC00..=0xDFFF).contains(&units[unit_at + 1])
                {
                    unit_at += 2;
                } else {
                    unit_at += 1;
                }
                scalar_at += 1;
            }
            (
                &token.scalars[..at],
                Some(Text::from_units(units[unit_at + 1..].to_vec())),
            )
        }
        None => (&token.scalars[..], None),
    }
}

/// The outcome of matching a long-option name against
/// a level's option set.
enum LongMatch {
    Exact(&'static str),
    Unique(&'static str),
    Ambiguous(Vec<&'static str>),
    None,
}

fn match_long(name: &[u32], options: &[&'static str]) -> LongMatch {
    let mut candidates: Vec<&'static str> = Vec::new();
    for option in options {
        let full = scalars_of(option);
        if full == name {
            return LongMatch::Exact(option);
        }
        if full.len() > name.len() && full[..name.len()] == *name {
            candidates.push(option);
        }
    }
    match candidates.len() {
        0 => LongMatch::None,
        1 => LongMatch::Unique(candidates[0]),
        _ => LongMatch::Ambiguous(candidates),
    }
}

/// Parse the compare subcommand's tokens.
fn parse_compare(tokens: &[Token], top_extras: &[Text]) -> ParseOutcome {
    let mut before: Option<Text> = None;
    let mut after: Option<Text> = None;
    let mut extras: Vec<Text> = top_extras.to_vec();
    let mut registration: Option<Text> = None;
    let mut judge: Option<Text> = None;
    let mut output: Option<Text> = None;
    let mut separated = false;
    let mut index = 0usize;

    fn push_positional(
        token: &Text,
        before: &mut Option<Text>,
        after: &mut Option<Text>,
        extras: &mut Vec<Text>,
    ) {
        if before.is_none() {
            *before = Some(token.clone());
        } else if after.is_none() {
            *after = Some(token.clone());
        } else {
            extras.push(token.clone());
        }
    }

    while index < tokens.len() {
        let token = &tokens[index];
        if separated {
            push_positional(&token.text, &mut before, &mut after, &mut extras);
            index += 1;
            continue;
        }
        if token.scalars == scalars_of("--") {
            // The first "--" always ends option parsing,
            // but when both positionals are already
            // filled argparse keeps the "--" itself
            // among the unrecognized arguments.
            separated = true;
            if before.is_some() && after.is_some() {
                extras.push(token.text.clone());
            }
            index += 1;
            continue;
        }
        if token.scalars.len() >= 2
            && token.scalars[0] == '-' as u32
            && token.scalars[1] == '-' as u32
        {
            // A long option: split at the first '='.
            let (name, glued) = split_long(token);
            let chosen = match match_long(name, &LONG_OPTIONS) {
                LongMatch::Exact(option) | LongMatch::Unique(option) => option,
                LongMatch::Ambiguous(candidates) => {
                    return parse_error(
                        COMPARE_USAGE,
                        "secondtake compare",
                        &format!(
                            "ambiguous option: {} could match {}",
                            token.rendered(),
                            candidates.join(", ")
                        ),
                    );
                }
                LongMatch::None => {
                    extras.push(token.text.clone());
                    index += 1;
                    continue;
                }
            };
            if chosen == "--help" {
                if let Some(value) = glued {
                    return parse_error(
                        COMPARE_USAGE,
                        "secondtake compare",
                        &format!(
                            "argument -h/--help: ignored explicit argument {}",
                            repr_text(&value)
                        ),
                    );
                }
                print!("{COMPARE_HELP}");
                return ParseOutcome::Exit(0);
            }
            match take_value(tokens, &mut index, glued, chosen) {
                Ok(value) => match chosen {
                    "--registration" => registration = Some(value),
                    "--judge" => judge = Some(value),
                    _ => output = Some(value),
                },
                Err(outcome) => return outcome,
            }
            index += 1;
            continue;
        }
        if option_shaped(token) {
            // A short cluster: h arms help, o takes the
            // rest of the cluster (or the next token) as
            // its value; an unknown first letter makes
            // the whole token an extra, and an unknown
            // later letter just ends the cluster. Help
            // fires when the token ends, unless a scan
            // error in it fired first.
            let cluster = &token.scalars[1..];
            if !matches!(cluster[0], c if c == 'h' as u32 || c == 'o' as u32) {
                extras.push(token.text.clone());
                index += 1;
                continue;
            }
            let mut help_pending = false;
            let mut at = 0usize;
            let mut failed: Option<ParseOutcome> = None;
            while at < cluster.len() {
                match cluster[at] {
                    c if c == 'h' as u32 => {
                        help_pending = true;
                        at += 1;
                    }
                    c if c == 'o' as u32 => {
                        let rest = &cluster[at + 1..];
                        let glued = if rest.is_empty() {
                            None
                        } else {
                            // The cluster prefix so far is
                            // ASCII, so the value's units
                            // begin at scalar position
                            // at + 2 of the token.
                            let units = token.text.units();
                            let mut start = at + 2;
                            if rest[0] == '=' as u32 {
                                start += 1;
                            }
                            Some(Text::from_units(units[start.min(units.len())..].to_vec()))
                        };
                        match take_value(tokens, &mut index, glued, "--output") {
                            Ok(value) => output = Some(value),
                            Err(outcome) => failed = Some(outcome),
                        }
                        break;
                    }
                    _ => break,
                }
            }
            if let Some(outcome) = failed {
                return outcome;
            }
            if help_pending {
                print!("{COMPARE_HELP}");
                return ParseOutcome::Exit(0);
            }
            index += 1;
            continue;
        }
        push_positional(&token.text, &mut before, &mut after, &mut extras);
        index += 1;
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
        let rendered: Vec<String> = extras.iter().map(|t| stderr_render(t.units())).collect();
        return parse_error(
            TOP_USAGE,
            "secondtake",
            &format!("unrecognized arguments: {}", rendered.join(" ")),
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
fn parse_top(tokens: &[Token]) -> ParseOutcome {
    let mut extras: Vec<Text> = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if token.scalars.len() >= 2
            && token.scalars[0] == '-' as u32
            && token.scalars[1] == '-' as u32
            && token.scalars != scalars_of("--")
        {
            let (name, glued) = split_long(token);
            match match_long(name, &["--help"]) {
                LongMatch::Exact("--help") | LongMatch::Unique("--help") => {
                    if let Some(value) = glued {
                        return parse_error(
                            TOP_USAGE,
                            "secondtake",
                            &format!(
                                "argument -h/--help: ignored explicit argument {}",
                                repr_text(&value)
                            ),
                        );
                    }
                    print!("{TOP_HELP}");
                    return ParseOutcome::Exit(0);
                }
                _ => {
                    extras.push(token.text.clone());
                    index += 1;
                    continue;
                }
            }
        }
        if token.scalars == scalars_of("--") || !option_shaped(token) {
            // The first positional-class token is the
            // command word, a leading "--" included.
            if token.text == Text::from_str("compare") {
                return parse_compare(&tokens[index + 1..], &extras);
            }
            return parse_error(
                TOP_USAGE,
                "secondtake",
                &format!(
                    "argument command: invalid choice: {} (choose from 'compare')",
                    repr_text(&token.text)
                ),
            );
        }
        // An option-shaped token: a cluster whose first
        // letter is h prints the top help; anything else
        // is held as an extra.
        if token.scalars[1] == 'h' as u32 {
            print!("{TOP_HELP}");
            return ParseOutcome::Exit(0);
        }
        extras.push(token.text.clone());
        index += 1;
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
    let mut rest: Vec<std::path::Component> = Vec::new();
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
            std::path::Component::Normal(name) => base.push(name),
            std::path::Component::ParentDir => {
                base.pop();
            }
            _ => {}
        }
    }
    base
}

/// The OS path for a text argument, in the operating
/// system's own encoding.
fn os_path(text: &Text) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        PathBuf::from(std::ffi::OsString::from_vec(fsencode_bytes(text.units())))
    }
    #[cfg(not(unix))]
    {
        PathBuf::from(String::from_utf16_lossy(text.units()))
    }
}

/// The display form a loader names: the path as
/// pathlib would print it.
fn display_path(text: &Text) -> String {
    stderr_render(&normalize_path_display(text.units()))
}

fn write_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    // mkstemp's shape: an unpredictable name in the
    // target directory, created exclusively at mode
    // 0600, so a file already at any guessed name is
    // never touched.
    let mut attempt = 0;
    let temp = loop {
        let suffix = random_suffix(attempt);
        let candidate = parent.join(format!("{name}.{suffix}"));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt as _;
            options.mode(0o600);
        }
        match options.open(&candidate) {
            Ok(mut handle) => match handle.write_all(bytes) {
                Ok(()) => break candidate,
                Err(error) => {
                    drop(handle);
                    let _ = std::fs::remove_file(&candidate);
                    return Err(error);
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && attempt < 16 => {
                attempt += 1;
            }
            Err(error) => return Err(error),
        }
    };
    match std::fs::rename(&temp, path) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = std::fs::remove_file(&temp);
            Err(error)
        }
    }
}

fn random_suffix(attempt: u32) -> String {
    let mut bytes = [0u8; 9];
    let filled = std::fs::File::open("/dev/urandom")
        .and_then(|mut file| std::io::Read::read_exact(&mut file, &mut bytes))
        .is_ok();
    if !filled {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        bytes = (nanos ^ ((std::process::id() as u128) << 64) ^ (attempt as u128)).to_le_bytes()
            [..9]
            .try_into()
            .expect("nine bytes");
    }
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn emit(document: &flighttape::value::Value, out: Option<&Text>) -> i32 {
    let bytes = dumps_document(document);
    match out {
        Some(out) => {
            let path = os_path(out);
            let written = match &bytes {
                Ok(bytes) => write_atomically(&path, bytes),
                Err(_) => Err(std::io::Error::other("unwritable text")),
            };
            match written {
                Ok(()) => 0,
                Err(_) => {
                    eprintln!(
                        "error, cannot write the comparison, {}",
                        stderr_render(out.units())
                    );
                    1
                }
            }
        }
        None => match &bytes {
            Ok(bytes) => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                match handle.write_all(bytes).and_then(|_| handle.flush()) {
                    Ok(()) => 0,
                    Err(_) => {
                        eprintln!("error, cannot write to stdout");
                        1
                    }
                }
            }
            Err(_) => {
                eprintln!("error, cannot write to stdout");
                1
            }
        },
    }
}

pub fn run_cli() -> i32 {
    let tokens: Vec<Token> = std::env::args_os()
        .skip(1)
        .map(|arg| {
            #[cfg(unix)]
            let bytes = arg.as_encoded_bytes().to_vec();
            #[cfg(not(unix))]
            let bytes = arg.to_string_lossy().into_owned().into_bytes();
            Token::from_units(fsdecode_units(&bytes))
        })
        .collect();
    run_tokens(tokens)
}

/// The CLI over explicit argument tokens, the shape
/// the ported suite drives.
pub fn run_with_args(tokens: Vec<String>) -> i32 {
    let tokens: Vec<Token> = tokens
        .iter()
        .map(|token| Token::from_units(fsdecode_units(token.as_bytes())))
        .collect();
    run_tokens(tokens)
}

fn run_tokens(tokens: Vec<Token>) -> i32 {
    let parsed = match parse_top(&tokens) {
        ParseOutcome::Run(parsed) => parsed,
        ParseOutcome::Exit(code) => return code,
    };
    let registration = match load_registration(
        &os_path(&parsed.registration),
        &display_path(&parsed.registration),
    ) {
        Ok(registration) => registration,
        Err(error) => {
            eprintln!("error, {error}");
            return 1;
        }
    };
    if let Some(out) = &parsed.output {
        let target = resolve_path(&os_path(out));
        for directory in [&parsed.before, &parsed.after] {
            let root = resolve_path(&os_path(directory));
            if target == root || target.starts_with(&root) {
                eprintln!("error, secondtake writes nothing inside the tape sets it reads");
                return 1;
            }
        }
        let mut stated: Vec<&Text> = vec![&parsed.registration];
        if let Some(judge) = &parsed.judge {
            if judge.units() != Text::from_str("live").units() {
                stated.push(judge);
            }
        }
        if stated
            .iter()
            .any(|path| target == resolve_path(&os_path(path)))
        {
            eprintln!("error, secondtake writes nothing over the files it reads");
            return 1;
        }
    }
    let judge: Option<Box<dyn Transport>> = match &parsed.judge {
        Some(judge) if judge.units() == Text::from_str("live").units() => {
            match std::env::var_os("JEV_CALL") {
                Some(script) if !script.is_empty() => Some(Box::new(LiveTransport::new(script))),
                _ => {
                    eprintln!(
                        "error, live judging needs the JEV_CALL environment variable naming the connector call script"
                    );
                    return 1;
                }
            }
        }
        Some(judge) => match FixtureTransport::load(&os_path(judge), &display_path(judge)) {
            Ok(transport) => Some(Box::new(transport)),
            Err(JudgeError(message)) => {
                eprintln!("error, {message}");
                return 1;
            }
        },
        None => None,
    };
    let document = match compare(
        &os_path(&parsed.before),
        &display_path(&parsed.before),
        &os_path(&parsed.after),
        &display_path(&parsed.after),
        &registration,
        judge.as_deref(),
    ) {
        Ok(document) => document,
        Err(error) => {
            eprintln!("error, {error}");
            return 1;
        }
    };
    emit(&document, parsed.output.as_ref())
}
