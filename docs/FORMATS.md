# Secondtake Wire Formats & Schemas

## 1. Registration Input (`schema/registration.schema.json`)

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

## 2. Comparison Output (`schema/comparison.schema.json`)

Top-level members emitted by `secondtake compare`:

- `registration`: Resolved registration object with defaults populated.
- `matching_rule` & `proposal_rule`: Self-describing contract strings.
- `before` & `after`: `{ "tapes_read", "tapes_scored", "excluded": [...] }`.
- `pairs[]`: Matched pairs ordered by task key (`key`, `before`, `after`, `before_file`, `after_file`, `deltas`).
- `aggregates`: `{ "pairs", "cost", "time", "rework", "rescue" }`.
- `unmatched`, `ambiguous`, `unmatchable`: Full disposition accounting for every non-paired tape.
- `regressions[]`: Names of measures that crossed their registered margin.
- `proposal`: `"keep" | "tune" | "revert" | "abstain"`.
- `judgment`: `{ "status": "not_consulted" | "answered" | "abstained" | "unavailable", ... }`.

## 3. Judgments Replay & Live Connector (`schema/judgments.schema.json`)

- Replay file (`--judge judgments.json`): Records deterministic answers keyed by card digest for offline CI reproduction.
- Live connector (`--judge live` with `JEV_CALL=/path/to/jev_call_example.py`): Exchanges a `jev-1.13.0` JSON card on stdin/stdout with zero SDK lock-in.
