<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="media/lockup-dark.svg">
    <img src="media/lockup-light.svg" alt="secondtake" width="760">
  </picture>
</p>

<p align="center">
  <img src="media/money-shot.gif" alt="secondtake matched-pair comparison showcase" width="880">
</p>

<p align="center">
  <img src="media/architecture.svg" alt="secondtake four-axis evaluation architecture" width="900">
</p>

# Secondtake Operator and Integrator Guide

`secondtake` evaluates agent setup changes (system prompts, `AGENTS.md` / `CLAUDE.md` edits, model upgrades, tool harness updates) by pairing `flighttape` `1.0` session tapes recorded before and after a registered change, scoring four behavioral measures, and emitting a single deterministic JSON comparison document with a `keep`, `tune`, `revert`, or `abstain` proposal.

JSON Schemas (Draft 2020-12) for every input and output document ship under `schema/`:

- `schema/registration.schema.json`
- `schema/judgments.schema.json`
- `schema/comparison.schema.json`

## 1. Installation

### Prebuilt Binary (`install.sh`)

On Linux and macOS (`x86_64` and `aarch64`), `install.sh` downloads a release archive, verifies its SHA-256 checksum against `SHA256SUMS`, and installs `secondtake` to `$HOME/.local/bin` (or `--bin-dir`):

```sh
./install.sh
./install.sh --version v0.1.0 --bin-dir "$HOME/.local/bin"
```

### Building from Source (`cargo`)

With Git and Rust `1.78+` installed:

```sh
git clone https://github.com/david-engelmann/secondtake.git
cd secondtake
cargo install --path .
```

Ensure `$HOME/.cargo/bin` is on your `PATH`.

## 2. Quickstart (`examples/`)

Ready-to-run synthetic fixtures live in `examples/` and work identically on Linux, macOS, and Windows without shell editing steps:

```sh
secondtake compare examples/before examples/after \
  --registration examples/registration.json
```

To write the comparison atomically to a file:

```sh
secondtake compare examples/before examples/after \
  --registration examples/registration.json \
  -o comparison.json
```

`secondtake` writes to an unpredictable sibling temporary file in the destination directory and renames it into place. Its write guard refuses `-o` paths that resolve inside `before` or `after`, or that match the `--registration` or `--judge` input files.

## 3. Registration Schema (`registration.json`)

Every comparison requires `--registration <path>` pointing to a JSON object written before measurement begins.

```json
{
  "change": "tighten edit retry rule in AGENTS.md",
  "date": "2026-11-27",
  "margins": {
    "cost_pct": 10,
    "time_pct": 10,
    "rework_per_pair": 1,
    "rescue_points": 5
  },
  "min_pairs": 5
}
```

### Fields and Defaults

- `change` (required): Non-empty single-line UTF-8 string naming the setup change (`
`, `
`, `	`, ASCII control characters `< 0x20`, `0x7F`, `U+0085`, `U+2028`, and `U+2029` are refused).
- `date` (required): Non-empty single-line UTF-8 string recording the registration date (conventionally `YYYY-MM-DD`).
- `margins` (optional, or `null`): Object overriding one or more non-negative numeric regression margins. Unknown keys inside `margins` are refused immediately. Defaults when omitted:
  - `cost_pct`: `10` (percent increase allowed)
  - `time_pct`: `10` (percent increase allowed)
  - `rework_per_pair`: `1` (additional redo moments per pair allowed)
  - `rescue_points`: `5` (percentage-point drop allowed)
- `min_pairs` (optional): Positive integer (`>= 1`) setting the minimum matched pair count required before claiming `keep`, `tune`, or `revert`. Defaults to `1`.

Validate registrations in CI against `schema/registration.schema.json` (`additionalProperties: false`) so misspelled top-level keys (such as `"min_pair"` instead of `"min_pairs"`) are caught before evaluation.

## 4. Tape Validation and Task-Key Pairing

`secondtake` reads top-level entries in `before` and `after` directories sorted by Unicode codepoint order:

1. **Format and shape validation (`excluded`)**: Non-files, unreadable files, invalid UTF-8/JSON, tapes not at `tape_version == "1.0"`, tapes with non-consecutive `seq` numbers or unpaired `tool_result` IDs, negative or `> 10^18` values in `run.totals`, invalid `outcome.status`, or nesting at/above Canonical Form 1 limits are listed under `before.excluded` or `after.excluded` with exact `reason` and `defects` while scoring continues on valid tapes.
2. **Task key (`matching_rule`)**: For each valid tape, `secondtake` finds the first main-chain user message (`kind == "message"`, `role == "user"`, `sidechain == false`) carrying a string `text`, and computes the `flighttape` Canonical Form 1 SHA-256 hex digest of that `text` string.
   - Tapes with no main-chain user text message are recorded in `unmatchable`.
   - Keys appearing exactly once in `before` and once in `after` become matched `pairs`.
   - Keys appearing on both sides with count `> 1` on either side are recorded in `ambiguous` and excluded from `pairs` so duplicate runs never pair arbitrarily.
   - Keys appearing on only one side are recorded in `unmatched.before` or `unmatched.after`.

## 5. The Four Measures

Each matched pair is scored on four measures:

| Measure | Per-Tape Definition | Aggregate & Margin Rule |
| :--- | :--- | :--- |
| `cost` (`cost`) | Exact integer `run.totals.input_tokens + run.totals.output_tokens`. Cache token fields are excluded. Missing either field yields `null` (unmeasured, never coerced to `0`). | Counted across pairs where **both** sides record `cost`. Regresses (`"cost"`) when `delta_pct = (after_mean - before_mean) / before_mean * 100 > margins.cost_pct` (or `after_mean > 0` when `before_mean == 0`). |
| `time` (`time_seconds`) | `flighttape::validate::tape_duration` wall seconds across tape events (`null` if timestamps cannot establish a duration). | Counted across pairs where **both** sides record `time_seconds`. Regresses (`"time"`) when `delta_pct > margins.time_pct` (or `after_mean > 0` when `before_mean == 0`). |
| `rework` (`rework`) | Integer count of redo moments (`RF-4`): a `tool_call` whose `tool` name and Canonical Form 1 `input` digest match an earlier `tool_call` that received an `is_error: true` `tool_result`. | Counted across all matched pairs. Regresses (`"rework"`) when `after_mean - before_mean > margins.rework_per_pair`. |
| `rescue` (`rescued` / `has_error`) | `has_error` is `true` if any `tool_result` has `is_error == true`. `rescued` is `true` if `has_error == true` and `outcome.status == "completed"`. | Computed per side over matched pairs holding `has_error == true`: `rate = rescued_count / error_holder_count * 100`. When **both** sides have `denominator >= 1`, `delta_points = after_rate - before_rate`; regresses (`"rescue rate"`) when `before_rate - after_rate > margins.rescue_points`. |

A measure with zero usable pairs (`pairs_counted == 0`, or zero error-holding pairs on either side for `rescue`) neither regresses nor clears.

## 6. Deterministic Proposal Rule and Tradeoff Band

Let `N` be `aggregates.pairs` and `R` be `regressions.len()`:

1. **Pair floor (`N < min_pairs`)**: Proposal is `"abstain"` (`judgment.status = "not_consulted"`, `reason = "insufficient pairs, N of M"`). No judge is consulted.
2. **Zero regressions (`R == 0`)**: Proposal is `"keep"` (`judgment.status = "not_consulted"`, `reason = "the deterministic rule is conclusive"`). No judge is consulted.
3. **Two or more regressions (`R >= 2`)**: Proposal is `"revert"` (`judgment.status = "not_consulted"`, `reason = "the deterministic rule is conclusive"`). No judge is consulted.
4. **Single-regression band (`R == 1`)**: Deterministic baseline proposal is `"tune"`.
   - Without `--judge`: proposal stays `"tune"` (`judgment.status = "not_consulted"`, `reason = "no judge configured"`).
   - With `--judge <judgments.json>` or `--judge live`: `secondtake` renders the deterministic decision card, hashes it (`card_sha256`), and asks the judge three questions (`choice`, `severity`, `match`).
     - Transport error, timeout, unpinned model, or malformed answer: proposal stays `"tune"` (`judgment.status = "unavailable"`, `reason` attached).
     - Judge chooses `"abstain"`: proposal becomes `"abstain"` (`judgment.status = "abstained"`, `reason = "the judge abstained"`).
     - Judge confidence `< 0.6`: proposal becomes `"abstain"` (`judgment.status = "abstained"`, `reason = "the judge's confidence X is below 0.6"`).
     - Judge chooses `"keep"`, `"tune"`, or `"revert"` with `confidence >= 0.6`: the judge's choice stands (`judgment.status = "judged"`).

## 7. Judge Protocol (`--judge live` and `judgments.json`)

### Decision Card Format

In the one-regression band, `secondtake` builds an 8-line plain-text decision card (`judgment.card`):

```text
Setup change: new system prompt, registered 2026-01-04.
Matched pairs: 1. Ambiguous task keys excluded: 0. Unmatched tapes: 0. Excluded tapes: 0.
Cost per pair: before 150, after 180, change 20 percent against a margin of 10 percent: regressed.
Time per pair: before 60, after 60, change 0 percent against a margin of 10 percent: within margin.
Rework per pair: before 0, after 0, change 0 against a margin of 1 per pair: within margin.
Rescue rate: before 100 percent over 1 pairs holding an error, after 100 percent over 1, change 0 points against a margin of 5 points down: within margin.
Regressions past margins: cost.
The deterministic rule proposes tune.
```

`card_sha256` is the `flighttape` Canonical Form 1 SHA-256 digest of that string as a JSON string value.

### Live Subprocess Contract (`JEV_CALL`)

When invoked with `--judge live`, `secondtake` reads the `JEV_CALL` environment variable and spawns `python3 $JEV_CALL` up to three times in sequence:

1. **Choice call**:
   ```sh
   python3 "$JEV_CALL" --state "<card>" --labels "keep,tune,revert,abstain"
   ```
2. **Severity score call** (appends `

Rate the severity of the tradeoff between the measures that improved and the measure that regressed, on the scale given.` to `<card>`):
   ```sh
   python3 "$JEV_CALL" --state "<card + severity framing>" \
     --labels "keep,tune,revert,abstain" --question-type score
   ```
3. **Match-confidence score call** (appends `

Rate the confidence that the matched pairs compare like with like, given the ambiguous, unmatched, and excluded counts, on the scale given.` to `<card>`):
   ```sh
   python3 "$JEV_CALL" --state "<card + match framing>" \
     --labels "keep,tune,revert,abstain" --question-type score
   ```

Subprocess requirements enforced by `src/judge.rs`:

- **Timeout**: 180 seconds wall time per subprocess call.
- **Streams & exit code**: Both `stdout` and `stderr` are drained concurrently and must decode as valid UTF-8; exit status must be `0`.
- **Pinned model**: Every JSON response object on `stdout` must carry `"model": "jev-1.13.0"`.
- **Choice response shape**:
  ```json
  {
    "model": "jev-1.13.0",
    "answers": {
      "decision": {
        "type": "choice",
        "choice": "tune",
        "confidence": 0.85,
        "probabilities": {
          "keep": 0.05,
          "tune": 0.80,
          "revert": 0.10,
          "abstain": 0.05
        }
      }
    }
  }
  ```
  `choice` must be one of `keep`, `tune`, `revert`, `abstain`; `confidence` and all four `probabilities` values must be finite numbers in `[0.0, 1.0]`; the four probabilities must sum to `1.0` within `+/- 0.02`.
- **Score response shape** (`severity` and `match`):
  ```json
  {
    "model": "jev-1.13.0",
    "answers": {
      "decision": {
        "type": "score",
        "score": 4.5,
        "confidence": 0.7
      }
    }
  }
  ```
  `type` must be `"score"` and `score` must be a finite number in `[0.0, 10.0]`.

See `examples/jev_call_example.py` for a complete runnable adapter template:

```sh
JEV_CALL=examples/jev_call_example.py \
  secondtake compare before_dir after_dir --registration registration.json --judge live
```

### Offline Replay Format (`judgments.json`)

Pass `--judge path/to/judgments.json` to replay recorded responses deterministically without network or subprocess calls (see `schema/judgments.schema.json` and `tests/fixtures/judgments_synthetic.json`):

```json
{
  "model": "jev-1.13.0",
  "cards": [
    {
      "card_sha256": "70a8823926a7cf5b76822da00fa84561b7880b534da90110af8cf507d4940653",
      "card": "Setup change: ...",
      "responses": {
        "choice": { "model": "jev-1.13.0", "answers": { "decision": { "...": "..." } } },
        "severity": { "model": "jev-1.13.0", "answers": { "decision": { "...": "..." } } },
        "match": { "model": "jev-1.13.0", "answers": { "decision": { "...": "..." } } }
      }
    }
  ]
}
```

Duplicate `card_sha256` entries or a `card` field whose Canonical Form 1 digest does not match `card_sha256` are refused up front before scoring begins.

## 8. CI Gate Integration

`secondtake compare` exits `0` whenever it successfully computes and writes a valid comparison document (regardless of whether `"proposal"` is `"keep"`, `"tune"`, `"revert"`, or `"abstain"`), and exits `1` or `2` on input/CLI errors. In CI pipelines, inspect `.proposal` explicitly to gate merges:

```sh
secondtake compare artifacts/before artifacts/after \
  --registration benchmarks/registration.json \
  -o artifacts/comparison.json

python3 -c '
import json, sys
doc = json.load(open("artifacts/comparison.json", encoding="utf-8"))
proposal = doc["proposal"]
print(f"secondtake proposal={proposal} regressions={doc["regressions"]}")
sys.exit(0 if proposal == "keep" else 1)
'
```

---

## Shell Completions & `cargo-binstall`

```sh
cargo binstall secondtake

# Shell completions (bash, zsh, fish, PowerShell)
source completions/secondtake.bash
```

