# Secondtake Performance & Scale Benchmarks

Measured from a `--release` build (`891.4 KiB` binary) on **Apple M3 Max** (macOS `14.6`, `rustc 1.99.1 (58cf8f9c3 2026-10-13) (Homebrew)`), median of 5 runs after warmup over validated `flighttape` `1.0` session pairs.

## Matched-Pair Evaluation Throughput

| Matched Pairs | Total Tapes Scored | `compare` Latency (`-o`) | Throughput |
| ---: | ---: | ---: | ---: |
| `10` | `20` | `18.73 ms` | `1,068 tapes/s` |
| `50` | `100` | `25.67 ms` | `3,896 tapes/s` |
| `250` | `500` | `87.32 ms` | `5,726 tapes/s` |

Scoring `500` session tapes (`250` matched before/after tasks) completes in **`87.32 ms`**, making `secondtake compare` fast enough to run inside every PR check or `touchstone` experiment gate.
