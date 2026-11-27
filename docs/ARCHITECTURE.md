# Secondtake Technical Architecture

<p align="center">
  <img src="media/architecture.svg" alt="secondtake technical architecture" width="920">
</p>

Most agent setup changes ship on anecdotal impressions: an engineer tweaks `AGENTS.md` or swaps models, runs two prompts, likes the output, and merges, unaware that token spend jumped 35% or edit-retry rework doubled on harder tasks. `secondtake` enforces **pre-registered, matched-pair evaluation** over validated `flighttape` `1.0` bundles.

## 1. Pipeline Stages

1. **Pre-Registration Gate (`--registration`)**:
   - Requires a JSON document declaring `change` and `date` (plus optional `margins` and `min_pairs`) before scoring runs. Single-line control-character validation prevents log/card injection.
2. **Tape Validation & Exclusion Accounting**:
   - Reads `before/` and `after/` directories in Unicode codepoint filename order using `flighttape`'s parser and validator. Corrupt or non-`1.0` files are recorded in `before.excluded` / `after.excluded` with named defects rather than crashing the batch.
3. **Canonical Form 1 First-Prompt Task Key**:
   - Extracts the `text` of the first main-chain user message (`kind == "message"`, `role == "user"`, `sidechain == false`) and hashes it under `flighttape` Canonical Form 1 SHA-256.
   - `1:1` matches enter `pairs[]`; `1:0` or `0:1` enter `unmatched`; `N:M` collisions (`N > 1` or `M > 1`) enter `ambiguous[]` so duplicate task runs never bias means.
4. **Four Behavioral Measures**:
   - **Cost**: Monetary `cost_usd` when present on both sides, falling back cleanly to total token counts.
   - **Time**: `duration_seconds` delta percentage against `margins.time_pct` (default `10%`).
   - **Rework**: Per-pair redo/retry moments against `margins.rework_per_pair` (default `+1`).
   - **Rescue Rate**: Share of error-encountering runs that still finish `completed`, checked against `margins.rescue_points` (default `-5` percentage points).
5. **Deterministic Proposal Band + Bounded One-Regression Judge**:
   - Below `min_pairs` -> `abstain` (insufficient evidence).
   - `0` regressions -> `keep` (`judgment.status: "not_consulted"`).
   - `>= 2` regressions -> `revert` (`judgment.status: "not_consulted"`).
   - Exactly `1` regression -> `tune` by default, or consults `--judge <file|live>` (`JEV_CALL` `jev-1.13.0` envelope), standing only when `confidence >= 0.6`.

## 2. Position in the Retrospective Line

<p align="center">
  <img src="media/retrospective-arc.svg" alt="secondtake in the Retrospective Line" width="920">
</p>
