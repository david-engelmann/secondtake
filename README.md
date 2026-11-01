# secondtake

The regression watch for setup changes, comparing traces from before and after any change on cost, time, rework, and rescue rate, and ending in a keep, tune, or revert proposal with the numbers attached.

## What this is

Setup changes usually ship on faith, and nothing replays the recorded history to prove a new rule or hook would have caught the failures without breaking the passes. Secondtake takes a declared change and its date, matches flight tapes on both sides by task class, and compares the fleet's numbers across the boundary. The output is one verdict, including the uncomfortable case where a celebrated rewrite quietly made everything slower and more expensive. The boundary with its siblings is one sentence. Secondtake measures one change, verdictloop keeps the registry of all of them, keeporkill rules on checks.

## Status

Seeded in October 2026. Local setup is in progress on the setup
branch. The remote seeding pass is waiting on a token access fix, so
work is banked locally until pushes clear. This repo is in phase 3
of the Retrospective Line and its build waits on the phase gate, the
main chat confirmation that the Gate Truth evidence pack schema has
merged and passed acceptance. Design work may proceed, build does
not start before the gate.

## Programme status

- Closed perfection rounds, 0 of 100. A round counts only when a
  full cycle of work, audit, review, red team, and approval has
  closed.
- Contract, stub in place, not yet met.
- Local gate, python3 scripts/codepoint_scan.py and, once tests
  exist, python3 -m pytest tests -q. CI runs the same commands.

## Licence

MIT. See LICENSE.
