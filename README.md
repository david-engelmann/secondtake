# secondtake

The regression watch for setup changes: traces from before and after any change, matched by task class and compared on cost, time, rework, and rescue rate, ending in a keep, tune, or revert proposal with the numbers attached.

Secondtake compares two sets of flighttape tapes, the runs recorded before a setup change and the runs recorded after it. A change is registered by hand, with its date, before any measurement is claimed. The tool matches the tapes by task, compares each pair on cost, time, rework, and rescue rate, and prints one document carrying the pairs, the aggregates, and a single proposal. Where exactly one measure regresses, a decision model judges the comparison from a deterministic card, and its choice stands only at the confidence the document states.

[![ci](https://github.com/david-engelmann/secondtake/actions/workflows/ci.yml/badge.svg)](https://github.com/david-engelmann/secondtake/actions/workflows/ci.yml)

```
$ secondtake compare demo/before demo/after --registration demo/registration.json -o demo/comparison.json
$ grep '"proposal"' demo/comparison.json
  "proposal": "keep",
```

## Install

The install needs Git and a Rust toolchain.

```sh
git clone https://github.com/david-engelmann/secondtake.git
cd secondtake
export CARGO_HOME="$PWD/.cargo-home"
export PATH="$CARGO_HOME/bin:$PATH"
cargo install --path .
```

Building fetches the flighttape crate at the revision pinned in Cargo.toml. The command builds the crate in release mode and installs the secondtake binary into the scratch Cargo home's bin directory.

Live judgment runs through a call script named by the JEV_CALL environment variable. The script is not shipped with this repository.

## Quickstart

The smallest complete run uses the synthetic tape the test suite ships. The sed step lowers the after tape's recorded output tokens so the comparison has a difference to report.

```
mkdir -p demo/before demo/after
cp tests/fixtures/tape_synthetic.json demo/before/session.json
cp tests/fixtures/tape_synthetic.json demo/after/session.json
sed -i 's/"output_tokens": 50/"output_tokens": 40/' demo/after/session.json
printf '%s\n' '{"change": "new system prompt", "date": "2026-01-04"}' > demo/registration.json
secondtake compare demo/before demo/after --registration demo/registration.json -o demo/comparison.json
grep '"proposal"' demo/comparison.json
```

The grep step prints the proposal line.

```
  "proposal": "keep",
```

## Usage

Run without an output flag, the comparison document prints to stdout in full. This is the quickstart run, printed rather than written.

```
secondtake compare demo/before demo/after --registration demo/registration.json
```

```json
{
  "registration": {
    "change": "new system prompt",
    "date": "2026-01-04",
    "margins": {
      "cost_pct": 10,
      "time_pct": 10,
      "rework_per_pair": 1,
      "rescue_points": 5
    },
    "min_pairs": 1
  },
  "matching_rule": "tapes match by task key: the canonical form 1 digest of the text of the first main-chain user message",
  "proposal_rule": "abstain for insufficient evidence below the pair floor; keep when no measure regresses past its margin, revert when two or more do; in the one-regression band a configured judge's choice stands at confidence 0.6 or better, abstains below it, and an unavailable judge leaves the tune standing; a measure with no usable pairs neither regresses nor clears",
  "before": {
    "tapes_read": 1,
    "tapes_scored": 1,
    "excluded": []
  },
  "after": {
    "tapes_read": 1,
    "tapes_scored": 1,
    "excluded": []
  },
  "pairs": [
    {
      "key": "ac3c7df17a5e22f85e9abf290bc06aaca93c4453865519797dab9b7e4954104f",
      "before": {
        "cost": 200,
        "time_seconds": 60.0,
        "rework": 0,
        "rescued": false,
        "has_error": false
      },
      "after": {
        "cost": 190,
        "time_seconds": 60.0,
        "rework": 0,
        "rescued": false,
        "has_error": false
      },
      "before_file": "session.json",
      "after_file": "session.json",
      "deltas": {
        "cost": -10,
        "time_seconds": 0.0,
        "rework": 0
      }
    }
  ],
  "aggregates": {
    "pairs": 1,
    "cost": {
      "before_mean": 200.0,
      "after_mean": 190.0,
      "pairs_counted": 1,
      "delta_pct": -5.0
    },
    "time": {
      "before_mean": 60.0,
      "after_mean": 60.0,
      "pairs_counted": 1,
      "delta_pct": 0.0
    },
    "rework": {
      "before_mean": 0.0,
      "after_mean": 0.0,
      "pairs_counted": 1,
      "delta": 0.0
    },
    "rescue": {
      "before_rate": null,
      "after_rate": null,
      "before_denominator": 0,
      "after_denominator": 0
    }
  },
  "unmatched": {
    "before": [],
    "after": []
  },
  "ambiguous": [],
  "unmatchable": [],
  "regressions": [],
  "proposal": "keep",
  "judgment": {
    "status": "not_consulted",
    "reason": "the deterministic rule is conclusive"
  }
}
```

With `-o`, the same document lands in a file instead, written atomically beside its target and renamed into place. The tool refuses an output inside either tape set, and an output naming the registration or a judgments file it reads.

```
$ secondtake compare demo/before demo/after --registration demo/registration.json -o demo/before/out.json
error, secondtake writes nothing inside the tape sets it reads
```

The refusal prints to stderr and the command exits 1, having written nothing.

Judgment rides on every document in its own section. Below the registration's pair floor the proposal is abstain for insufficient evidence, reached deterministically. With no regression the proposal is keep, and with two or more it is revert, again with no model consulted. Only in the one-regression band is a judge asked. Pass `--judge` with the path of a judgments file to replay recorded answers, or `--judge live` with the JEV_CALL environment variable naming the connector call script to ask the model. A judged choice stands at confidence 0.6 or better. Below that, or when the judge abstains, the proposal is abstain with the answers recorded on the document. A judge that cannot answer leaves the deterministic tune standing, marked unavailable.

## Status

The compare command is complete for its scope and is held to the Python it replaced by a differential harness over synthetic tape sets. Closed perfection rounds, 13 of 100. A round counts only when a full cycle of work, audit, review, red team, and approval has closed.

## License

MIT. See LICENSE.
