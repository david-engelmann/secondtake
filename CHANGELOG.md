# Changelog

All notable milestones in `secondtake` are recorded here. Ongoing perfection rounds and obligation rulings are tracked in `CONTRACT.md` (`ST-1` onward).

## [Unreleased]

### Added

- Ready-to-run synthetic fixtures (`examples/before/session.json`, `examples/after/session.json`, `examples/registration.json`) and a reference `JEV_CALL` connector template (`examples/jev_call_example.py`).
- Draft 2020-12 JSON Schemas (`schema/registration.schema.json`, `schema/judgments.schema.json`, `schema/comparison.schema.json`).
- Operator and integrator handbook (`docs/GUIDE.md`) documenting task-key pairing, the four measures, deterministic proposal rules, and the `jev-1.13.0` judge protocol.
- Tag-triggered multi-architecture release workflow (`.github/workflows/release.yml`) and SHA-256 verifying installer (`install.sh`).
- Crate publication metadata (`repository`, `homepage`, `documentation`, `readme`, `keywords`, `categories`, `rust-version`, `exclude`) in `Cargo.toml`, hardened workflow permissions in `.github/workflows/ci.yml`, and `target` / `.cargo-home` exclusions in `scripts/codepoint_scan.py`.
- `CHANGELOG.md`, `SECURITY.md`, and `CONTRIBUTING.md`.

## [0.1.0] - 2026-11-02

### Added

- Matched before-and-after `flighttape` `1.0` tape comparison CLI and Rust library (`secondtake compare`) scoring `cost`, `time`, `rework`, and `rescue` against registered margins (`ST-1` to `ST-19`).
- Deterministic proposal engine (`keep`, `tune`, `revert`, `abstain`) with `jev-1.13.0` live (`JEV_CALL`) and digest-keyed fixture replay (`--judge`) transports in the single-regression tradeoff band.
- Atomic output file writing (`-o` / `--output`) with symlink-aware input overwrite guards.
