//! The comparison's rendering: Python's json.dumps
//! with indent=2 and ensure_ascii=False, the exact
//! bytes the command line emits (ST-12), and the
//! Python's stdio law for the bytes that cannot be
//! written: a lone surrogate in the surrogateescape
//! range (DC80 to DCFF) writes as its raw byte, and
//! any other lone surrogate makes the write fail, so
//! serialization reports it instead of rendering a
//! substitute.

use flighttape::value::{Text, Value};

/// A document carrying text no Python write could
/// emit: the CLI reports the cannot-write error for
/// whichever destination was asked.
#[derive(Debug, PartialEq)]
pub struct EmitError;

/// Python's float repr, which is also the JSON
/// encoder's rendering: the shortest digits that
/// round trip, presented fixed inside the band
/// -4 <= exp10 < 16 and scientifically outside it,
/// with a signed two-digit exponent. The digits are
/// computed exactly in this module: neither Rust's
/// own float formatting (which rounds exact decimal
/// ties up where CPython rounds to even) nor the
/// flighttape crate's renderer (whose digit sequence
/// for an exactly terminating expansion can run one
/// digit past the shortest, as at 2^-24) is CPython's
/// sequence in every case.
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
    let (digits, point) = shortest_digits(value.abs());
    let count = digits.len() as i64;
    let exp10 = point - 1;
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

/// The shortest round-trip digit sequence of a
/// positive finite nonzero double, and the position
/// of the decimal point: the value is
/// 0.digits * 10^point. Computed exactly: the
/// double's own decimal expansion is finite (it is
/// m * 2^e), so the expansion is built digit by digit
/// with small integer arithmetic, and the shortest
/// length whose rounding of the expansion parses back
/// to the same double wins, the closer candidate
/// first and an exact tie going to the even last
/// digit. Parsing back goes through Rust's correctly
/// rounded reader, so the true (asymmetric, at powers
/// of two) rounding interval is honored by
/// construction.
fn shortest_digits(value: f64) -> (String, i64) {
    let bits = value.to_bits();
    let biased = ((bits >> 52) & 0x7FF) as i32;
    let fraction = bits & ((1u64 << 52) - 1);
    let (mantissa, exponent) = if biased == 0 {
        (fraction, -1074)
    } else {
        (fraction + (1u64 << 52), biased - 1075)
    };
    // The exact expansion: digits of mantissa * 2^e
    // when e >= 0, and of mantissa * 5^-e with the
    // point shifted by e when e < 0.
    let mut digits: Vec<u8> = mantissa
        .to_string()
        .bytes()
        .map(|byte| byte - b'0')
        .collect();
    let point: i64 = if exponent >= 0 {
        for _ in 0..exponent {
            multiply_small(&mut digits, 2);
        }
        digits.len() as i64
    } else {
        for _ in 0..-exponent {
            multiply_small(&mut digits, 5);
        }
        digits.len() as i64 + exponent as i64
    };
    while digits.len() > 1 && *digits.last().expect("nonempty") == 0 {
        digits.pop();
    }
    let exact: String = digits.iter().map(|d| (b'0' + d) as char).collect();
    for len in 1..=exact.len() {
        let low = &exact[..len];
        let rest = &exact[len..];
        let high = increment_digits(low);
        let (high_digits, high_point) = match &high {
            Some(high) => (high.clone(), point),
            None => ("1".to_string(), point + 1),
        };
        let low_ok = parses_back(low, point, value);
        let high_ok = parses_back(&high_digits, high_point, value);
        match (low_ok, high_ok) {
            (false, false) => continue,
            (true, false) => return (low.to_string(), point),
            (false, true) => return (high_digits, high_point),
            (true, true) => {
                // The closer candidate wins; the tail
                // against the half decides, and an
                // exact half goes to the even last
                // digit.
                let cmp = cmp_tail_to_half(rest);
                let pick_high = match cmp {
                    std::cmp::Ordering::Greater => true,
                    std::cmp::Ordering::Less => false,
                    std::cmp::Ordering::Equal => (low.as_bytes()[len - 1] - b'0') % 2 == 1,
                };
                return if pick_high {
                    (high_digits, high_point)
                } else {
                    (low.to_string(), point)
                };
            }
        }
    }
    unreachable!("the exact digits themselves round trip")
}

/// Multiply a most-significant-first digit vector by
/// a small factor, in place.
fn multiply_small(digits: &mut Vec<u8>, factor: u8) {
    let mut carry = 0u16;
    for digit in digits.iter_mut().rev() {
        let product = *digit as u16 * factor as u16 + carry;
        *digit = (product % 10) as u8;
        carry = product / 10;
    }
    let mut prefix = Vec::new();
    while carry > 0 {
        prefix.push((carry % 10) as u8);
        carry /= 10;
    }
    prefix.reverse();
    let tail = std::mem::take(digits);
    *digits = prefix;
    digits.extend(tail);
}

/// A digit string plus one, or None when every digit
/// was 9 (the result gains a digit: 1 followed by
/// zeros, which the caller renders as "1" with the
/// point moved).
fn increment_digits(digits: &str) -> Option<String> {
    let mut bytes: Vec<u8> = digits.bytes().collect();
    let mut index = bytes.len();
    loop {
        if index == 0 {
            return None;
        }
        index -= 1;
        if bytes[index] == b'9' {
            bytes[index] = b'0';
        } else {
            bytes[index] += 1;
            return Some(String::from_utf8(bytes).expect("digits are ascii"));
        }
    }
}

/// Compare a dropped tail to exactly one half: the
/// tail is the fraction of one unit in the last kept
/// digit, in decimal digits.
fn cmp_tail_to_half(rest: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    if rest.is_empty() {
        return Ordering::Less;
    }
    match rest.as_bytes()[0] {
        b'0'..=b'4' => Ordering::Less,
        b'6'..=b'9' => Ordering::Greater,
        _ => {
            if rest[1..].chars().all(|ch| ch == '0') {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        }
    }
}

/// Whether the decimal digits at the given point
/// parse back to exactly this double.
fn parses_back(digits: &str, point: i64, value: f64) -> bool {
    let text = if digits.len() == 1 {
        format!("{digits}e{}", point - 1)
    } else {
        format!("{}.{}e{}", &digits[..1], &digits[1..], point - 1)
    };
    match text.parse::<f64>() {
        Ok(parsed) => parsed.to_bits() == value.to_bits(),
        Err(_) => false,
    }
}

/// Append one text as a JSON string in bytes. Valid
/// scalars escape only the quote, the backslash, and
/// the control characters, as the encoder does; a
/// surrogateescape unit writes as its raw byte; any
/// other lone surrogate is an EmitError.
fn escape_string(text: &Text, out: &mut Vec<u8>) -> Result<(), EmitError> {
    out.push(b'"');
    let units = text.units();
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xD800..=0xDBFF).contains(&unit)
            && index + 1 < units.len()
            && (0xDC00..=0xDFFF).contains(&units[index + 1])
        {
            let scalar =
                0x10000 + (((unit as u32) - 0xD800) << 10) + (units[index + 1] as u32 - 0xDC00);
            push_scalar(scalar, out);
            index += 2;
            continue;
        }
        if (0xDC80..=0xDCFF).contains(&unit) {
            out.push((unit - 0xDC00) as u8);
            index += 1;
            continue;
        }
        if (0xD800..=0xDFFF).contains(&unit) {
            return Err(EmitError);
        }
        match unit {
            0x22 => out.extend_from_slice(b"\\\""),
            0x5C => out.extend_from_slice(b"\\\\"),
            0x08 => out.extend_from_slice(b"\\b"),
            0x09 => out.extend_from_slice(b"\\t"),
            0x0A => out.extend_from_slice(b"\\n"),
            0x0C => out.extend_from_slice(b"\\f"),
            0x0D => out.extend_from_slice(b"\\r"),
            0x00..=0x1F => {
                out.extend_from_slice(format!("\\u{unit:04x}").as_bytes());
            }
            _ => push_scalar(unit as u32, out),
        }
        index += 1;
    }
    out.push(b'"');
    Ok(())
}

fn push_scalar(scalar: u32, out: &mut Vec<u8>) {
    let mut buf = [0u8; 4];
    let encoded = char::from_u32(scalar)
        .expect("a valid scalar")
        .encode_utf8(&mut buf);
    out.extend_from_slice(encoded.as_bytes());
}

fn push_indent(out: &mut Vec<u8>, depth: usize) {
    for _ in 0..depth {
        out.extend_from_slice(b"  ");
    }
}

fn write_value(value: &Value, depth: usize, out: &mut Vec<u8>) -> Result<(), EmitError> {
    match value {
        Value::Null => out.extend_from_slice(b"null"),
        Value::Bool(true) => out.extend_from_slice(b"true"),
        Value::Bool(false) => out.extend_from_slice(b"false"),
        Value::Int(digits) => out.extend_from_slice(digits.as_bytes()),
        Value::Float(number) => {
            if number.is_nan() {
                out.extend_from_slice(b"NaN");
            } else if number.is_infinite() {
                out.extend_from_slice(if *number > 0.0 {
                    b"Infinity"
                } else {
                    b"-Infinity"
                });
            } else {
                out.extend_from_slice(py_float(*number).as_bytes());
            }
        }
        Value::Str(text) => escape_string(text, out)?,
        Value::Array(items) => {
            if items.is_empty() {
                out.extend_from_slice(b"[]");
                return Ok(());
            }
            out.extend_from_slice(b"[\n");
            for (index, item) in items.iter().enumerate() {
                push_indent(out, depth + 1);
                write_value(item, depth + 1, out)?;
                if index + 1 < items.len() {
                    out.push(b',');
                }
                out.push(b'\n');
            }
            push_indent(out, depth);
            out.push(b']');
        }
        Value::Object(entries) => {
            if entries.is_empty() {
                out.extend_from_slice(b"{}");
                return Ok(());
            }
            out.extend_from_slice(b"{\n");
            for (index, (key, item)) in entries.iter().enumerate() {
                push_indent(out, depth + 1);
                escape_string(key, out)?;
                out.extend_from_slice(b": ");
                write_value(item, depth + 1, out)?;
                if index + 1 < entries.len() {
                    out.push(b',');
                }
                out.push(b'\n');
            }
            push_indent(out, depth);
            out.push(b'}');
        }
    }
    Ok(())
}

/// The document as the CLI emits it, trailing newline
/// included, or the cannot-write failure the Python
/// meets when the document carries a lone surrogate
/// outside the escape range.
pub fn dumps_document(value: &Value) -> Result<Vec<u8>, EmitError> {
    let mut out = Vec::new();
    write_value(value, 0, &mut out)?;
    out.push(b'\n');
    Ok(out)
}
