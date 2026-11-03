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
    // 54 fraction bits of rem/b by long division, with
    // the surviving remainder as the sticky bit.
    let mut frac: u128 = 0;
    for _ in 0..54 {
        rem <<= 1;
        frac <<= 1;
        if rem >= b {
            frac |= 1;
            rem -= b;
        }
    }
    let sticky = rem != 0;
    // The value is total / 2^54, total holding the
    // integer part above the 54 fraction bits.
    let total = (int_part << 54) | frac;
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
    let (over_half, tie) = if drop > 0 {
        (tail > half || (tail == half && sticky), tail == half && !sticky)
    } else {
        // Nothing was dropped from total itself, but
        // the value continues below it as rem/b of one
        // total unit: round on that fraction instead.
        let twice = rem * 2;
        (twice > b, twice == b)
    };
    let mut exponent = drop as i32 - 54;
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
