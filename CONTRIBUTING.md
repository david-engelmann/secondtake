# Contributing

`secondtake` follows the engineering discipline described in [impatient-programming](https://github.com/david-engelmann/impatient-programming): contract obligations before implementation, zero-warning local gates before CI, and strict copy hygiene.

## Repository Rules

1. **Contract before code**: Behavioral changes, bug fixes, and refusal cases are specified in `CONTRACT.md` (`ST-*`) and paired with deterministic tests under `tests/` before implementation.
2. **Copy bar**:
   - Every `.md` and `.txt` file in the repository must contain zero em-dashes (`U+2014`) and zero en-dashes (`U+2013`), enforced by `scripts/codepoint_scan.py`.
   - `README.md` additionally forbids colons and semicolons in prose outside its opening description line (`ST-45`).
3. **Minimal dependency footprint**: `secondtake` consumes `flighttape` for Canonical Form 1 digests, JSON parsing, tape durations, and the `Value` model (`ST-21`), and adds no third-party runtime crates.

## Running the Local Gate

Run all four gate commands from the repository root before pushing any branch:

```sh
python3 scripts/codepoint_scan.py
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

All four commands must exit `0` with zero warnings and all tests passing.
