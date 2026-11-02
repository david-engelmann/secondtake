# secondtake Contract, stub

The full Contract is the first build act of this repo phase. This
stub records the obligations the setup already knows it owes. Each
obligation becomes executable before any feature code that depends
on it lands.

- ST-1. A setup change is registered by hand at first, with its date, before any measurement is claimed.
- ST-2. Before and after tapes are matched by task class, and the matching is stated with the result.
- ST-3. Four metrics are compared, cost, time, rework, and rescue rate. No fifth metric ships in the first slice.
- ST-4. Every comparison ends in one proposal, keep, tune, or revert, with the numbers attached.

## Round 1. Matched comparison on four measures (ST-5 to ST-12)

Contract preparation only in this pass. The build opens
when this repo's turn in the line's build order arrives;
secondtake and verdictloop build last in the line.

- ST-5. Inputs are two directories of flighttape tapes
  at format 1.0, the before set and the after set, read
  with the family's exclusion discipline, plus a
  registration naming the setup change and its date, per
  ST-1. A comparison with no registration is refused by
  name. Test, tests/test_compare.py.
- ST-6. Tapes match across the sets by task: the digest,
  under canonical form 1, of the first main-chain user
  message's text is the task key, and the matching rule
  is printed on the comparison itself, per ST-2. A tape
  with no user message is unmatchable and reported. A
  key matching more than one tape on either side is
  reported as ambiguous and none of its tapes are paired
  by force. Test, tests/test_compare.py.
- ST-7. The four measures of ST-3, defined mechanically.
  Cost is the tape's recorded token totals, input plus
  output, or absent when the tape records none, never
  estimated. Time is the tape's duration. Rework is its
  redo count under the shared redo definition. A tape is
  rescued when its outcome is completed and it holds at
  least one error tool result; rescue rate is rescued
  tapes over tapes holding at least one error, per side.
  Test, tests/test_compare.py.
- ST-8. The comparison reports every pair with both
  sides' measures and their deltas, the aggregates per
  measure with their denominators (pairs counted, tapes
  excluded and why, unmatched and ambiguous keys), and
  exactly one proposal, per ST-4. Test,
  tests/test_compare.py.
- ST-9. The Round 1 proposal rule is deterministic and
  stated on the comparison: keep when no measure
  regresses past its margin, revert when two or more
  regress, tune otherwise. The margins are stated in the
  registration: cost and time regress past ten percent,
  rework past one additional redo per pair on average,
  rescue rate past five points down. Whether an abstain
  outcome can ever sit with ST-4 is the decision-model
  round's open question, named here and not answered in
  this round. Test, tests/test_compare.py.
- ST-10. Judgment enters this tool only after the
  deterministic matching and measurement above, in the
  later decision round and only on comparisons whose
  measures argue. Round 1 calls no model. Test,
  tests/test_compare.py.
- ST-11. The comparison is deterministic: the same sets
  and registration produce byte-identical output.
  Test, tests/test_compare.py.
- ST-12. Secondtake writes only the comparison, to
  stdout or a named path, atomically. Every failure is
  a named error with exit 1. Test, tests/test_compare.py.

## Round 1 amendments, the pinned shapes (recorded before code)

- ST-5a. The registration is a JSON file naming the
  change and its date: {change, date}. A missing or
  malformed registration is a named refusal. It may
  carry margins {cost_pct, time_pct, rework_per_pair,
  rescue_points}; where it does not, the ST-9 values
  (10, 10, 1, 5) apply and the comparison prints the
  margins it used. Tapes are excluded under the
  family discipline: the format 1.0 subset, defects
  named, a tape that cannot be digested under
  canonical form 1 excluded with that reason. Files
  are read in sorted name order. Test,
  tests/test_compare.py.
- ST-6a. The task key is the canonical form 1 digest
  of the text of the first main-chain user message
  (kind message, role user, sidechain false). A tape
  with no such message, or whose text is not a
  string, is unmatchable and reported. Test,
  tests/test_compare.py.
- ST-7a. Cost is totals input_tokens plus
  output_tokens when both are recorded, else absent;
  cache token keys are not input or output. Time is
  the tape duration under flighttape's FT-32 rule.
  Rework is the redo count under the shared RF-4
  definition. Rescue rate is per side over the paired
  tapes: rescued over tapes holding at least one
  error; a side with no error-holding paired tape has
  no rate. Cost, time, and rework aggregates run over
  the pairs where both sides carry the measure, and
  the count of such pairs is printed. Test,
  tests/test_compare.py.
- ST-9a. A measure with no usable pairs, or a rescue
  rate absent on a side, is unmeasured: it neither
  regresses nor clears. Cost and time regress when
  the after mean exceeds the before mean by more
  than the margin percent; a zero before mean with a
  positive after mean regresses. Rework regresses
  when the after mean exceeds the before mean by
  more than the margin per pair. Rescue regresses
  when the rate falls by more than the margin
  points. With zero pairs every measure is unmeasured
  and the rule yields keep; the comparison states
  its zero denominators rather than hiding them.
  Test, tests/test_compare.py.
- ST-8a. The comparison document carries the
  registration and the margins used, the matching
  rule printed as one sentence, per-side read counts
  and exclusions, the pairs sorted by task key with
  both sides' measures and deltas, the aggregates
  with their denominators, the unmatched tapes by
  file and key, the ambiguous keys with their counts,
  the unmatchable tapes with reasons, the regressed
  measures, and exactly one proposal. Test,
  tests/test_compare.py.

## Round 1 amendments, after the reviews (recorded before the repair code)

- ST-5b. Secondtake's subset adds the shapes its own
  measures stand on, beyond the family subset:
  run.totals, when present, is an object whose values
  are non-negative integers, and outcome, when
  present, is an object whose status is in the format
  enum. A tape breaking either is excluded with the
  defect named. Test, tests/test_compare.py.
- ST-6b. The message text the key rule reads is the
  format's text field, the one flighttape's converter
  writes. (The first implementation read a field named
  content, which is the tool result field; the golden
  fixture and the converter are the ground truth, and
  the reviews caught the divergence.) A main-chain
  user message without a text string is unmatchable.
  Test, tests/test_compare.py.
- ST-9b. The comparison prints its proposal rule as
  one sentence beside the matching rule, so the
  verdict can be checked against the rule that
  produced it without opening the contract. Test,
  tests/test_compare.py.
