# secondtake

The second take on a setup change, with the tapes to prove it: before and after, matched by task, compared on cost, time, rework, and rescue rate, ending in one proposal, keep, tune, or revert.

## What this is

Anyone can claim a new prompt, model, or rule set is better. Secondtake demands the receipts. A setup change is registered by hand, with its date, before any measurement is claimed. Then the before tapes and the after tapes are matched by task, the digest of the first user message, and compared on exactly four measures. Cost is the recorded token totals or absent, never estimated. Rework is the redo count under the family's shared definition. Rescue rate is rescued runs over runs that hit an error, per side. The comparison prints its matching rule, its margins, every pair with its numbers, and its denominators, then ends in exactly one proposal under a deterministic rule stated on the comparison itself.

Round 1 calls no model. Whether a comparison whose measures argue may abstain instead of proposing is the decision round's open question, named in the contract and not answered here.

## Status

Round 1 is built and in review on its branch. Setup landed earlier in October 2026.

## Programme status

- Closed perfection rounds, 1 of 100. A round counts only when a
  full cycle of work, audit, review, red team, and approval has
  closed.
- Contract, ST-1 to ST-12 as amended, met for the Round 1 scope.
- Local gate, python3 scripts/codepoint_scan.py and
  python3 -m pytest tests -q. CI runs the same commands.

## Licence

MIT. See LICENSE.
