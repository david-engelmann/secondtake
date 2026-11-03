//! Small shared helpers: object construction over the
//! flighttape value model, and exact comparisons
//! between the port's floats and the reader's exact
//! integer digit strings. Python compares an int and a
//! float exactly; the helpers here keep that exactness
//! where the spec's comparisons depend on it.

use flighttape::value::{Text, Value};

/// Build an object value from key/value pairs, in the
/// order given. Document key order is part of the
/// emitted comparison's bytes.
pub fn obj(pairs: Vec<(&str, Value)>) -> Value {
    Value::Object(
        pairs
            .into_iter()
            .map(|(key, value)| (Text::from_str(key), value))
            .collect(),
    )
}

pub fn int(digits: &str) -> Value {
    Value::Int(digits.to_string())
}

pub fn int_i128(value: i128) -> Value {
    Value::Int(value.to_string())
}

pub fn int_usize(value: usize) -> Value {
    Value::Int(value.to_string())
}

pub fn text(value: &str) -> Value {
    Value::Str(Text::from_str(value))
}

/// The exact integer in a reader digit string, when it
/// fits an i128. Reader integers are canonical (no
/// leading zeros, "-0" already normalized to "0").
pub fn int_to_i128(digits: &str) -> Option<i128> {
    digits.parse::<i128>().ok()
}

/// Whether a reader digit string names exactly `n`.
pub fn int_eq_usize(digits: &str, n: usize) -> bool {
    digits == n.to_string()
}

/// Compare two non-negative integer digit strings.
pub fn nonneg_int_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let a = a.trim_start_matches('0');
    let b = b.trim_start_matches('0');
    let a = if a.is_empty() { "0" } else { a };
    let b = if b.is_empty() { "0" } else { b };
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

/// Whether the digit string names a negative integer.
pub fn int_is_negative(digits: &str) -> bool {
    digits.starts_with('-') && digits != "-0"
}

/// The numeric value of an int or float as an f64, for
/// the judge's range checks: a huge integer becomes an
/// infinity, which fails every bound exactly as the
/// Python's comparisons do.
pub fn number_as_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Int(digits) => digits.parse::<f64>().ok(),
        Value::Float(number) => Some(*number),
        _ => None,
    }
}

/// Whether `x` (a finite f64) is strictly greater than
/// the non-negative integer named by `digits`, compared
/// exactly: a non-integral x exceeds n exactly when its
/// floor does, and an integral x compares digit string
/// to digit string.
pub fn f64_gt_nonneg_int(x: f64, digits: &str) -> bool {
    use std::cmp::Ordering;
    if x.is_nan() {
        return false;
    }
    if x <= 0.0 {
        // A non-positive x never exceeds a non-negative
        // integer except when the integer is negative,
        // which this helper is never handed.
        return false;
    }
    if x == f64::INFINITY {
        return true;
    }
    let floor = x.floor();
    let floor_digits = if floor < 1.7e38 {
        (floor as i128).to_string()
    } else {
        // Beyond i128 the value's own decimal rendering
        // is still exact (every large f64 is an integer).
        format!("{floor:.0}")
    };
    let order = nonneg_int_cmp(&floor_digits, digits);
    if x == floor {
        order == Ordering::Greater
    } else {
        // A non-integral x exceeds n exactly when its
        // floor reaches n.
        order != Ordering::Less
    }
}

/// Whether a value is a JSON number (int or float).
pub fn is_number(value: &Value) -> bool {
    matches!(value, Value::Int(_) | Value::Float(_))
}

/// Decode OS bytes the way CPython decodes file system
/// and argument strings: valid UTF-8 decodes normally,
/// and each byte that cannot decode becomes the
/// surrogateescape unit 0xDC00 plus the byte.
pub fn fsdecode_units(bytes: &[u8]) -> Vec<u16> {
    let mut units = Vec::new();
    let mut rest = bytes;
    loop {
        match std::str::from_utf8(rest) {
            Ok(text) => {
                units.extend(text.encode_utf16());
                return units;
            }
            Err(error) => {
                let valid = error.valid_up_to();
                let head = std::str::from_utf8(&rest[..valid]).expect("the valid prefix");
                units.extend(head.encode_utf16());
                match error.error_len() {
                    Some(len) => {
                        for byte in &rest[valid..valid + len] {
                            units.push(0xDC00 + *byte as u16);
                        }
                        rest = &rest[valid + len..];
                    }
                    None => {
                        for byte in &rest[valid..] {
                            units.push(0xDC00 + *byte as u16);
                        }
                        return units;
                    }
                }
            }
        }
    }
}

/// The inverse: encode text units back to OS bytes,
/// mapping surrogateescape units back to their bytes.
/// A lone surrogate outside the escape range cannot
/// be encoded; it renders as its UTF-8 replacement,
/// a case argument strings from the OS never reach
/// (their surrogates come only from fsdecode).
#[cfg(unix)]
pub fn fsencode_bytes(units: &[u16]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xDC80..=0xDCFF).contains(&unit) {
            out.push((unit - 0xDC00) as u8);
            index += 1;
        } else if (0xD800..=0xDBFF).contains(&unit)
            && index + 1 < units.len()
            && (0xDC00..=0xDFFF).contains(&units[index + 1])
        {
            let scalar =
                0x10000 + (((unit as u32) - 0xD800) << 10) + (units[index + 1] as u32 - 0xDC00);
            let mut buf = [0u8; 4];
            out.extend_from_slice(
                char::from_u32(scalar)
                    .expect("a valid pair")
                    .encode_utf8(&mut buf)
                    .as_bytes(),
            );
            index += 2;
        } else if (0xD800..=0xDFFF).contains(&unit) {
            out.extend_from_slice("\u{FFFD}".as_bytes());
            index += 1;
        } else {
            let mut buf = [0u8; 4];
            out.extend_from_slice(
                char::from_u32(unit as u32)
                    .expect("a BMP scalar")
                    .encode_utf8(&mut buf)
                    .as_bytes(),
            );
            index += 1;
        }
    }
    out
}

/// Render text for a stderr message the way CPython's
/// stderr does (its error handler is backslashreplace):
/// valid text passes through, and every lone surrogate
/// becomes its \uXXXX escape. Used where a message
/// names a value the Python names (an unknown margin
/// key, a file name).
pub fn stderr_render(units: &[u16]) -> String {
    let mut out = String::new();
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xD800..=0xDBFF).contains(&unit)
            && index + 1 < units.len()
            && (0xDC00..=0xDFFF).contains(&units[index + 1])
        {
            let scalar =
                0x10000 + (((unit as u32) - 0xD800) << 10) + (units[index + 1] as u32 - 0xDC00);
            out.push(char::from_u32(scalar).expect("a valid pair"));
            index += 2;
        } else if (0xD800..=0xDFFF).contains(&unit) {
            out.push_str(&format!("\\u{unit:04x}"));
            index += 1;
        } else {
            out.push(char::from_u32(unit as u32).expect("a BMP scalar"));
            index += 1;
        }
    }
    out
}

/// Normalize a path's display form the way pathlib
/// does when a Path is printed: empty and dot
/// components drop, duplicate slashes collapse (a
/// leading pair of slashes is preserved), a trailing
/// slash drops, and an empty result is a dot. Dot-dot
/// components are kept; nothing is resolved.
pub fn normalize_path_display(units: &[u16]) -> Vec<u16> {
    let slash = '/' as u16;
    let absolute = units.first() == Some(&slash);
    let double = absolute && units.get(1) == Some(&slash) && units.get(2) != Some(&slash);
    let mut parts: Vec<&[u16]> = Vec::new();
    for part in units.split(|unit| *unit == slash) {
        if part.is_empty() || part == ['.' as u16] {
            continue;
        }
        parts.push(part);
    }
    let mut out: Vec<u16> = Vec::new();
    if double {
        out.extend_from_slice(&[slash, slash]);
    } else if absolute {
        out.push(slash);
    }
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            out.push(slash);
        }
        out.extend_from_slice(part);
    }
    if out.is_empty() {
        out.push('.' as u16);
    }
    out
}

/// The maximum container nesting depth of a JSON text,
/// counted over brackets outside strings: the measure
/// the Python reader's recursion limit turns on. A
/// scalar text has depth 0.
pub fn json_text_depth(text: &str) -> usize {
    let mut depth = 0usize;
    let mut max = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for ch in text.chars() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '[' | '{' => {
                depth += 1;
                max = max.max(depth);
            }
            ']' | '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    max
}

/// Compare two texts as CPython compares strings: by
/// codepoint, a surrogate pair counting as its astral
/// scalar and a lone surrogate at face value.
pub fn text_cmp_py(a: &[u16], b: &[u16]) -> std::cmp::Ordering {
    fn scalars(units: &[u16]) -> Vec<u32> {
        let mut out = Vec::new();
        let mut index = 0;
        while index < units.len() {
            let unit = units[index];
            if (0xD800..=0xDBFF).contains(&unit)
                && index + 1 < units.len()
                && (0xDC00..=0xDFFF).contains(&units[index + 1])
            {
                out.push(
                    0x10000 + (((unit as u32) - 0xD800) << 10) + (units[index + 1] as u32 - 0xDC00),
                );
                index += 2;
            } else {
                out.push(unit as u32);
                index += 1;
            }
        }
        out
    }
    scalars(a).cmp(&scalars(b))
}

/// The error-construction shape the compare and judge
/// modules share: a named refusal built from a
/// message. Each error type implements FromMessage;
/// the one constructor lives here.
pub trait FromMessage: Sized {
    fn from_message(message: String) -> Self;
}

pub fn err<T: FromMessage>(message: impl Into<String>) -> T {
    T::from_message(message.into())
}

/// The mean of an exact integer sum over `n`, rounded
/// the way Python's int/int division rounds: the exact
/// rational sum/n carried past 53 significant bits by
/// integer long division, then rounded to the nearest
/// f64, ties to even. A naive (sum as f64) / n rounds
/// twice and can land a unit in the last place away
/// from the Python for sums past 2^53.
pub fn mean_int(sum: i128, n: i128) -> f64 {
    if sum == 0 || n == 0 {
        return 0.0;
    }
    let negative = (sum < 0) != (n < 0);
    let a = sum.unsigned_abs();
    let b = n.unsigned_abs();
    let int_part = a / b;
    let mut rem = a % b;
    // Carry the value to about 118 significant bits:
    // the integer part plus enough fraction bits of
    // rem/b by long division that the total always
    // holds far more than the 53 the f64 keeps, with
    // the surviving remainder as the sticky bit.
    let int_bits = if int_part > 0 {
        128 - int_part.leading_zeros()
    } else {
        0
    };
    let fraction_bits = if int_bits > 0 { 118 - int_bits } else { 118 };
    let mut frac: u128 = 0;
    for _ in 0..fraction_bits {
        rem <<= 1;
        frac <<= 1;
        if rem >= b {
            frac |= 1;
            rem -= b;
        }
    }
    let sticky = rem != 0;
    // The value is total / 2^fraction_bits.
    let total = (int_part << fraction_bits) | frac;
    if total == 0 {
        return 0.0;
    }
    let bits = 128 - total.leading_zeros();
    let drop = bits.saturating_sub(53);
    let mut head = total >> drop;
    let tail = if drop > 0 {
        total & ((1u128 << drop) - 1)
    } else {
        0
    };
    let half = if drop > 0 { 1u128 << (drop - 1) } else { 0 };
    let over_half = tail > half || (tail == half && sticky);
    let tie = tail == half && !sticky;
    let mut exponent = drop as i32 - fraction_bits as i32;
    if over_half || (tie && head & 1 == 1) {
        head += 1;
        if head == (1u128 << 53) {
            head >>= 1;
            exponent += 1;
        }
    }
    let value = head as f64 * 2f64.powi(exponent);
    if negative {
        -value
    } else {
        value
    }
}
