//! The comparison's rendering: Python's json.dumps
//! with indent=2 and ensure_ascii=False, the exact
//! bytes the command line emits (ST-12). Floats render
//! by their Python repr, non-ASCII text passes through
//! unescaped, and only the quote, the backslash and
//! the control characters are escaped, as the encoder
//! does.

use flighttape::pyrepr::repr_float;
use flighttape::value::{Text, Value};

fn escape_string(text: &Text, out: &mut String) {
    out.push('"');
    // Work over the decoded string when the text is
    // well formed; lone surrogates (which the Python's
    // own write would refuse) render as their escape,
    // a divergence confined to documents carrying
    // surrogate text in file names.
    match text.as_str() {
        Some(rendered) => {
            for ch in rendered.chars() {
                match ch {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\u{08}' => out.push_str("\\b"),
                    '\u{09}' => out.push_str("\\t"),
                    '\u{0A}' => out.push_str("\\n"),
                    '\u{0C}' => out.push_str("\\f"),
                    '\u{0D}' => out.push_str("\\r"),
                    ch if (ch as u32) < 0x20 => {
                        out.push_str(&format!("\\u{:04x}", ch as u32));
                    }
                    ch => out.push(ch),
                }
            }
        }
        None => {
            for unit in text.units() {
                match unit {
                    0x22 => out.push_str("\\\""),
                    0x5C => out.push_str("\\\\"),
                    0x08 => out.push_str("\\b"),
                    0x09 => out.push_str("\\t"),
                    0x0A => out.push_str("\\n"),
                    0x0C => out.push_str("\\f"),
                    0x0D => out.push_str("\\r"),
                    0x00..=0x1F => {
                        out.push_str(&format!("\\u{unit:04x}"));
                    }
                    _ => out.push_str(&format!("\\u{unit:04x}")),
                }
            }
        }
    }
    out.push('"');
}

fn write_value(value: &Value, depth: usize, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Int(digits) => out.push_str(digits),
        Value::Float(number) => {
            if number.is_nan() {
                out.push_str("NaN");
            } else if number.is_infinite() {
                out.push_str(if *number > 0.0 {
                    "Infinity"
                } else {
                    "-Infinity"
                });
            } else {
                out.push_str(&repr_float(*number));
            }
        }
        Value::Str(text) => escape_string(text, out),
        Value::Array(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push_str("[\n");
            for (index, item) in items.iter().enumerate() {
                out.push_str(&"  ".repeat(depth + 1));
                write_value(item, depth + 1, out);
                if index + 1 < items.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&"  ".repeat(depth));
            out.push(']');
        }
        Value::Object(entries) => {
            if entries.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push_str("{\n");
            for (index, (key, item)) in entries.iter().enumerate() {
                out.push_str(&"  ".repeat(depth + 1));
                escape_string(key, out);
                out.push_str(": ");
                write_value(item, depth + 1, out);
                if index + 1 < entries.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&"  ".repeat(depth));
            out.push('}');
        }
    }
}

/// The document as the CLI emits it, trailing newline
/// included.
pub fn dumps_document(value: &Value) -> String {
    let mut out = String::new();
    write_value(value, 0, &mut out);
    out.push('\n');
    out
}
