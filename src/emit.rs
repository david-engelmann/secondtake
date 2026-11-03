//! The comparison's rendering: Python's json.dumps
//! with indent=2 and ensure_ascii=False, the exact
//! bytes the command line emits (ST-12). Floats render
//! by their Python repr, non-ASCII text passes through
//! unescaped, and only the quote, the backslash and
//! the control characters are escaped, as the encoder
//! does.

use flighttape::value::{Text, Value};

/// Python's float repr, which is also the JSON
/// encoder's rendering: the shortest digits that
/// round trip (Rust's own LowerExp formatting yields
/// the same digit sequence), presented fixed inside
/// the band -4 <= exp10 < 16 and scientifically
/// outside it, with a signed two-digit exponent. The
/// flighttape crate's repr_float derives its digits
/// from the canonical ECMAScript form, whose fixed
/// presentation pads the digit sequence with zeros;
/// in the scientific band that padding is not
/// Python's, so this module renders from the digits
/// directly.
pub fn py_float(value: f64) -> String {
    if value.is_nan() {
        return "nan".to_string();
    }
    if value.is_infinite() {
        return if value > 0.0 {
            "inf".to_string()
        } else {
            "-inf".to_string()
        };
    }
    if value == 0.0 {
        return if value.is_sign_negative() {
            "-0.0".to_string()
        } else {
            "0.0".to_string()
        };
    }
    let sign = if value < 0.0 { "-" } else { "" };
    let rendered = format!("{:e}", value.abs());
    let (mantissa, exponent) = rendered
        .split_once('e')
        .expect("LowerExp always carries an exponent");
    let exp10: i64 = exponent.parse().expect("exponent is an integer");
    let digits: String = mantissa.chars().filter(|ch| *ch != '.').collect();
    let count = digits.len() as i64;
    let point = exp10 + 1;
    if !(-4..16).contains(&exp10) {
        let mantissa = if count == 1 {
            digits
        } else {
            format!("{}.{}", &digits[..1], &digits[1..])
        };
        return format!("{sign}{mantissa}e{exp10:+03}");
    }
    if point <= 0 {
        format!("{sign}0.{}{digits}", "0".repeat((-point) as usize))
    } else if count <= point {
        format!("{sign}{digits}{}.0", "0".repeat((point - count) as usize))
    } else {
        format!(
            "{sign}{}.{}",
            &digits[..point as usize],
            &digits[point as usize..]
        )
    }
}

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
                out.push_str(&py_float(*number));
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
