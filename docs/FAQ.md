# Secondtake Frequently Asked Questions

### Why does `secondtake` require `--registration`?
Deciding acceptable regression margins *after* seeing benchmark numbers invites rationalizing regressions away. Writing `registration.json` beforehand pins `margins` and `min_pairs` into the emitted `comparison.json` record itself.

### Why did two runs of the same prompt land in `ambiguous` instead of `pairs`?
If `before/` contains two tapes with the identical first user prompt (or `after/` contains two), pairing one-to-one would require an arbitrary choice. `secondtake` quarantines multi-way task-key collisions in `ambiguous[]` so every entry in `pairs[]` is an unambiguous 1:1 task match.

### When is `--judge` consulted?
Only when `pairs >= min_pairs` and **exact one** of the four measures (`cost`, `time`, `rework`, `rescue`) regressed past its margin (for example, cost rose 14% while rescue rate improved 18 points). Zero regressions always produce `keep`; two or more always produce `revert`.
