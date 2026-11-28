<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="media/lockup-dark.svg">
    <img src="media/lockup-light.svg" alt="secondtake" width="780">
  </picture>
</p>

<p align="center">
  <a href="https://github.com/david-engelmann/secondtake/actions/workflows/ci.yml"><img src="https://github.com/david-engelmann/secondtake/actions/workflows/ci.yml/badge.svg" alt="ci"></a>
  <a href="https://github.com/david-engelmann/secondtake/actions/workflows/clean-install.yml"><img src="https://github.com/david-engelmann/secondtake/actions/workflows/clean-install.yml/badge.svg" alt="clean-install"></a>
  <a href="https://github.com/david-engelmann/secondtake/actions/workflows/docs.yml"><img src="https://github.com/david-engelmann/secondtake/actions/workflows/docs.yml/badge.svg" alt="docs"></a>
  <a href="https://github.com/david-engelmann/secondtake/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-emerald.svg" alt="MIT License"></a>
</p>

# secondtake

The regression watch for setup changes: traces from before and after any change, matched by task class and compared on cost, time, rework, and rescue rate, ending in a keep, tune, or revert proposal with the numbers attached.

Secondtake compares two sets of flighttape tapes, the runs recorded before a setup change and the runs recorded after it. A change is registered by hand, with its date, before any measurement is claimed. The tool matches the tapes by task, compares each pair on cost, time, rework, and rescue rate, and prints one document carrying the pairs, the aggregates, and a single proposal. Where exactly one measure regresses, a decision model judges the comparison from a deterministic card, and its choice stands only at the confidence the document states.

<p align="center">
  <img src="media/money-shot.gif" alt="secondtake matched-pair comparison CLI showcase" width="880">
</p>

## Install

### Prebuilt Binary (Linux & macOS)

```sh
curl -fsSL https://raw.githubusercontent.com/david-engelmann/secondtake/main/install.sh | sh
```

### Cargo Binstall or Build from Source

```sh
cargo binstall secondtake

# Or build from source (fetches pinned flighttape crate)
git clone https://github.com/david-engelmann/secondtake.git
cd secondtake
cargo install --path . --locked
```

Live one-regression judgment runs through an external connector named by `JEV_CALL` (see [`examples/jev_call_example.py`](https://github.com/david-engelmann/secondtake/tree/main/examples/jev_call_example.py)).

## Quickstart

From a local checkout of this repository:

Run directly against the ready-to-score fixtures in `examples/`:

```sh
secondtake compare examples/before examples/after --registration examples/registration.json -o comparison.json
grep '"proposal"' comparison.json
```

```text
  "proposal": "keep",
```

## Architecture

<p align="center">
  <img src="media/architecture.svg" alt="secondtake four-axis evaluation architecture" width="900">
</p>

## Usage

Run without `-o` to emit the complete comparison document to stdout, or pass `-o <path>` for atomic sibling-tempfile output. `secondtake` refuses any `-o` path inside `before` or `after`, or matching `--registration` or `--judge`:

```text
$ secondtake compare examples/before examples/after --registration examples/registration.json -o examples/before/out.json
error, secondtake writes nothing inside the tape sets it reads
```

Below `min_pairs` the proposal is `abstain`. With zero regressions it is `keep`, and with two or more it is `revert`, with no model consulted. Only in the one-regression band is `--judge <file|live>` consulted, standing at confidence `0.6` or higher.

## Performance

<p align="center">
  <img src="media/benchmark-chart.svg" alt="benchmark profile" width="900">
</p>

Measured on Apple M3 Max (`--release`, `891.4 KiB` binary; details in [docs/PERFORMANCE.md](performance.md)):

| Matched Pairs | Total Tapes Scored | `compare` Latency (`-o`) | Throughput |
| ---: | ---: | ---: | ---: |
| `10` | `20` | `22.7 ms` | `881 tapes/s` |
| `50` | `100` | `54.39 ms` | `1,839 tapes/s` |
| `250` | `500` | `314.04 ms` | `1,592 tapes/s` |

## Where `secondtake` Fits in the Retrospective Line

<p align="center">
  <img src="media/retrospective-arc.svg" alt="Retrospective Line ten-repository architecture" width="900">
</p>

## Documentation

- [Documentation Index](introduction.md) & [Searchable Handbook](https://david-engelmann.github.io/secondtake/)
- [Operator & Integrator Guide](guide.md)
- [Technical Architecture](architecture.md)
- [Wire Formats & JSON Schemas](formats.md)
- [Ecosystem Comparison](comparison.md)
- [Performance Benchmarks](performance.md)
- [FAQ](faq.md)

## Status

The compare command is complete for its scope and is held to the Python it replaced by a differential harness over synthetic tape sets. Closed perfection rounds, 48 of 100. A round counts only when a full cycle of work, audit, review, red team, and approval has closed.

## License

MIT. See [LICENSE](https://github.com/david-engelmann/secondtake/blob/main/LICENSE).

## Rust Library API Reference

Programmatic callers embedding `secondtake` as a Rust library crate can browse the hosted `rustdoc` reference at [`https://david-engelmann.github.io/secondtake/api/secondtake/index.html`](https://david-engelmann.github.io/secondtake/api/secondtake/index.html) (or run `cargo doc --no-deps --open` locally).
