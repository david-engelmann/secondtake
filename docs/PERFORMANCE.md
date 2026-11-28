# Secondtake Performance & Scale Benchmarks

<p align="center">
  <img src="media/benchmark-chart.svg" alt="secondtake benchmark profile" width="920">
</p>

Measured from a thin-LTO symbol-stripped `--release` build (`683.2 KiB` binary) on **Apple M3 Max** (macOS `14.6`, `rustc 1.99.1 (58cf8f9c3 2026-10-13) (Homebrew)`), median of 5 runs after warmup over validated `flighttape` `1.0` session pairs.

## Matched-Pair Evaluation Throughput

| Matched Pairs | Total Tapes Scored | `compare` Latency (`-o`) | Throughput |
| ---: | ---: | ---: | ---: |
| `10` | `20` | `22.7 ms` | `881 tapes/s` |
| `50` | `100` | `54.39 ms` | `1,839 tapes/s` |
| `250` | `500` | `314.04 ms` | `1,592 tapes/s` |
