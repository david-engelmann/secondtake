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
/// normalization. A relative spelling whose
/// working directory cannot be read fails the
/// resolution (ST-43): the interpreter's resolve
/// raises there, and a substituted root spelling
/// would compare a fiction. So does a spelling
/// whose resolution runs a symlink chain back
/// into itself (ST-49): the interpreter's resolve
/// raises RuntimeError there, and a substituted
/// spelling past the loop is the same fiction.
/// Absolute spellings never consult the working
/// directory.
fn resolve_path(path: &Path) -> Option<PathBuf> {
    if let Ok(canonical) = std::fs::canonicalize(path) {
        return Some(canonical);
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(path)
    };
    if resolution_loops(&absolute) {
        return None;
    }
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
    Some(base)
}

/// Whether resolving this absolute spelling would
/// run a symlink chain back into itself. The walk
/// is the interpreter's own shape: components are
/// taken left to right, a link's target is
/// substituted in place (an absolute target
/// restarts from the root), and a link met again
/// while its own substitution is still in flight
/// is the loop. A link met again after its
/// substitution finished is no loop, and a chain
/// that never revisits a link is no loop however
/// deep it runs, up to the walk's defensive
/// ceiling on total follows (ST-53).
fn resolution_loops(absolute: &Path) -> bool {
    use std::collections::HashMap;
    use std::path::Component;

    enum Step {
        Name(std::ffi::OsString),
        Up,
        Done(PathBuf),
    }

    fn push_path(stack: &mut Vec<Step>, path: &Path) {
        let parts: Vec<Component<'_>> = path.components().collect();
        for component in parts.into_iter().rev() {
            match component {
                Component::Normal(name) => stack.push(Step::Name(name.to_os_string())),
                Component::ParentDir => stack.push(Step::Up),
                _ => {}
            }
        }
    }

    let mut stack: Vec<Step> = Vec::new();
    push_path(&mut stack, absolute);
    let mut resolved = PathBuf::from("/");
    // A link's path, mapped to whether its own
    // substitution is still in flight.
    let mut links: HashMap<PathBuf, bool> = HashMap::new();
    let mut follows = 0usize;
    while let Some(step) = stack.pop() {
        match step {
            Step::Done(link) => {
                links.insert(link, false);
            }
            Step::Up => {
                resolved.pop();
            }
            Step::Name(name) => {
                let candidate = resolved.join(&name);
                let is_link = std::fs::symlink_metadata(&candidate)
                    .map(|meta| meta.file_type().is_symlink())
                    .unwrap_or(false);
                if !is_link {
                    resolved = candidate;
                    continue;
                }
                match links.get(&candidate) {
                    Some(true) => return true,
                    Some(false) => {
                        resolved = candidate;
                        continue;
                    }
                    None => {}
                }
                let target = match std::fs::read_link(&candidate) {
                    Ok(target) => target,
                    Err(_) => {
                        resolved = candidate;
                        continue;
                    }
                };
                follows += 1;
                // A defensive ceiling on the walk:
                // a spelling whose resolution would
                // follow more links than this is
                // reported as a loop, so no spelling
                // can keep the walk substituting
                // forever (ST-53).
                if follows > 4096 {
                    return true;
                }
                links.insert(candidate.clone(), true);
                stack.push(Step::Done(candidate));
                if target.is_absolute() {
                    resolved = PathBuf::from("/");
                }
                push_path(&mut stack, &target);
            }
        }
    }
    false
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

/// Lexical normalization over raw path bytes, the
/// normpath half of os.path.abspath: dot components
/// drop, dot-dot pops the last kept component
/// without following any link, and the result is
/// absolute. A leading pair of slashes is preserved
/// when the input carries exactly two.
fn normpath_bytes(bytes: &[u8]) -> Vec<u8> {
    let double = bytes.len() >= 2
        && bytes[0] == b'/'
        && bytes[1] == b'/'
        && (bytes.len() == 2 || bytes[2] != b'/');
    let mut kept: Vec<&[u8]> = Vec::new();
    for part in bytes.split(|byte| *byte == b'/') {
        if part.is_empty() || part == b"." {
            continue;
        }
        if part == b".." {
            match kept.last() {
                Some(last) if *last != b".." => {
                    kept.pop();
                }
                _ => {}
            }
            continue;
        }
        kept.push(part);
    }
    let mut out: Vec<u8> = Vec::new();
    if double {
        out.push(b'/');
    }
    out.push(b'/');
    for (index, part) in kept.iter().enumerate() {
        if index > 0 {
            out.push(b'/');
        }
        out.extend_from_slice(part);
    }
    out
}

/// The directory mkstemp computes from a parent
/// spelling: os.path.abspath joins the working
/// directory when the spelling is relative, then
/// normalizes lexically. A failed working-directory
/// read fails the write, as the interpreter's
/// getcwd failure fails its mkstemp call.
fn lexical_abspath(parent: &[u8]) -> std::io::Result<Vec<u8>> {
    if parent.first() == Some(&b'/') {
        return Ok(normpath_bytes(parent));
    }
    let cwd = std::env::current_dir()?;
    #[cfg(unix)]
    let cwd_bytes = {
        use std::os::unix::ffi::OsStrExt as _;
        cwd.as_os_str().as_bytes().to_vec()
    };
    #[cfg(not(unix))]
    let cwd_bytes = cwd.to_string_lossy().into_owned().into_bytes();
    let mut joined = cwd_bytes;
    joined.push(b'/');
    joined.extend_from_slice(parent);
    Ok(normpath_bytes(&joined))
}

fn write_atomically(out: &Text, bytes: &[u8]) -> std::io::Result<()> {
    // ST-35: the write consumes the pathlib
    // construction of the output spelling, the
    // display form's own normalization; the rename
    // target is the constructed spelling.
    let constructed = normalize_path_display(out.units());
    // The constructed parent and name, split the way
    // pathlib splits them: the name is the final
    // component, empty when that component is a dot
    // or dot-dot or the path is a root.
    let slash = '/' as u16;
    let (parent_units, name_units): (&[u16], &[u16]) =
        match constructed.iter().rposition(|unit| *unit == slash) {
            Some(index) => {
                let name = &constructed[index + 1..];
                let name: &[u16] = if name == ['.' as u16] || name == ['.' as u16, '.' as u16] {
                    &[]
                } else {
                    name
                };
                let parent: &[u16] = if index == 0 {
                    &constructed[..1]
                } else {
                    &constructed[..index]
                };
                (parent, name)
            }
            None => {
                if constructed == ['.' as u16] || constructed == ['.' as u16, '.' as u16] {
                    (&['.' as u16], &[] as &[u16])
                } else {
                    (&[] as &[u16], &constructed[..])
                }
            }
        };
    let parent_bytes = if parent_units.is_empty() {
        b".".to_vec()
    } else {
        fsencode_bytes(parent_units)
    };
    // ST-36: the temp's home is the lexical
    // absolutization of the constructed parent.
    let temp_dir = lexical_abspath(&parent_bytes)?;
    let name_bytes = fsencode_bytes(name_units);
    let target = os_path(&Text::from_units(constructed.clone()));
    // mkstemp's shape: an unpredictable name in the
    // target directory, created exclusively at mode
    // 0600, so a file already at any guessed name is
    // never touched.
    let mut attempt = 0;
    let temp = loop {
        let suffix = random_suffix(attempt);
        let mut candidate = temp_dir.clone();
        candidate.push(b'/');
        candidate.extend_from_slice(&name_bytes);
        candidate.push(b'.');
        candidate.extend_from_slice(suffix.as_bytes());
        let candidate = os_path_bytes(candidate);
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
    match std::fs::rename(&temp, &target) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = std::fs::remove_file(&temp);
            Err(error)
        }
    }
}

/// An OS path from raw bytes, in the operating
/// system's own encoding.
fn os_path_bytes(bytes: Vec<u8>) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt as _;
        PathBuf::from(std::ffi::OsString::from_vec(bytes))
    }
    #[cfg(not(unix))]
    {
        PathBuf::from(String::from_utf8_lossy(&bytes).into_owned())
    }
}

fn random_suffix(attempt: u32) -> String {
    // ST-37: exactly eight characters, four bytes
    // hex encoded, mkstemp's own width, drawn from
    // the standing construction.
    let mut bytes = [0u8; 4];
    let filled = std::fs::File::open("/dev/urandom")
        .and_then(|mut file| std::io::Read::read_exact(&mut file, &mut bytes))
        .is_ok();
    if !filled {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        // ST-40: the sixteen-byte mix is folded by
        // lanes so the pid term reaches the bytes.
        let mixed =
            (nanos ^ ((std::process::id() as u128) << 64) ^ (attempt as u128)).to_le_bytes();
        bytes = [
            mixed[0] ^ mixed[4] ^ mixed[8] ^ mixed[12],
            mixed[1] ^ mixed[5] ^ mixed[9] ^ mixed[13],
            mixed[2] ^ mixed[6] ^ mixed[10] ^ mixed[14],
            mixed[3] ^ mixed[7] ^ mixed[11] ^ mixed[15],
        ];
    }
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn emit(document: &flighttape::value::Value, out: Option<&Text>) -> i32 {
    let bytes = dumps_document(document);
    match out {
        Some(out) => {
            let written = match &bytes {
                Ok(bytes) => write_atomically(out, bytes),
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
        // The guard resolves in the Python's own order,
        // and a failed resolution takes the crash class
        // (ST-43): the interpreter dies uncaught out of
        // the resolve, so the port returns 1 in silence.
        let target = match resolve_path(&os_path(out)) {
            Some(target) => target,
            None => return 1,
        };
        for directory in [&parsed.before, &parsed.after] {
            let root = match resolve_path(&os_path(directory)) {
                Some(root) => root,
                None => return 1,
            };
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
        for path in &stated {
            match resolve_path(&os_path(path)) {
                Some(resolved) if resolved == target => {
                    eprintln!("error, secondtake writes nothing over the files it reads");
                    return 1;
                }
                Some(_) => {}
                None => return 1,
            }
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
