# secondtake

The second take on a setup change, with the tapes to prove it: before and after, matched by task, compared on cost, time, rework, and rescue rate, ending in one outcome, keep, tune, revert, or an honest abstain when the evidence cannot carry a proposal.

## What this is

Anyone can claim a new prompt, model, or rule set is better. Secondtake demands the receipts. A setup change is registered by hand, with its date, before any measurement is claimed. Then the before tapes and the after tapes are matched by task, the digest of the first user message, and compared on exactly four measures. Cost is the recorded token totals or absent, never estimated. Rework is the redo count under the family's shared definition. Rescue rate is rescued runs over runs that hit an error, per side. The comparison prints its matching rule, its margins, every pair with its numbers, and its denominators, then ends in exactly one outcome under a rule stated on the comparison itself.

Round 2 answers the question Round 1 left open. Below a stated pair floor the outcome is abstain, insufficient evidence, deterministically. Where no measure regresses the outcome is keep, and where two or more do it is revert, with no model consulted. Only in the band between, exactly one regression, does a decision model judge: Jev reads a deterministic card built from the comparison and answers a choice with tradeoff severity and match confidence scores, and its choice stands at confidence 0.6 or better. Below that, or when the judge abstains, the outcome is abstain with the answers recorded; a judge that cannot answer leaves the deterministic tune standing, marked unavailable, never dressed as judged. Every comparison carries its judgment section, consulted or not.

## Status

Round 1 is closed. The compare command matches before
and after tape sets by task and reports four measures
with one deterministic proposal.

Round 2 is closed. The judgment round: a pair floor,
an abstain outcome for insufficient evidence, and a
decision model judging only the one-regression band
from a deterministic card. Setup landed earlier in
October 2026.

Round 3 is closed. The Rust port: the Python package
and its suite leave the tree, and a Rust library crate
with a thin compare binary reproduces the behavior
exactly, consuming canonical form 1 and the tape
reading from the flighttape crate at a pinned
revision. The differential harness byte-compares the
two implementations across the band, and the cargo
suite carries the ported tests.

## Programme status

- Closed perfection rounds, 5 of 100. A round counts only when a
  full cycle of work, audit, review, red team, and approval has
  closed.
- Contract, ST-1 to ST-12 as amended, met for the Round 1 scope.
  ST-13 to ST-19 with amendments ST-13a, ST-16a, and
  ST-17a, written ahead of the code and met in Round 2.
  ST-20 to ST-26, the port obligations, written ahead
  of the code and met in Round 3.
- Local gate, python3 scripts/codepoint_scan.py and cargo test.
  CI runs the same commands.

## Licence

MIT. See LICENSE.
