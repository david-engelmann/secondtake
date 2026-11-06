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

Built in Round 1, last in the line with verdictloop.
The round also added the amendments below (ST-5a to
ST-9a before code, ST-5b, ST-6b, and ST-9b after the
reviews, before the repair code).

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

## Round 2. Judgment in the band (ST-13 to ST-19)

Settles the question ST-9 left open and ST-10 promised:
judgment enters only where the measures argue, and an
abstain outcome can sit with ST-4 as a terminal outcome
that is not a proposal, stated with its reason. The
numbers still own everything the outcome stands on.

- ST-13. The decision card is a deterministic text
  built from the finished comparison: the registered
  change and its date; the matched pair count; the
  ambiguous, unmatched, and excluded counts; one line
  per measure naming its before and after means, its
  change, its margin, and whether it regressed, with a
  measure counted on no pair stated as unmeasured; the
  regressions past margins; and the deterministic
  proposal. The card's digest is the canonical form 1
  digest of the card text, and the card and its digest
  are recorded on the comparison whenever judgment is
  consulted. Test, tests/test_judge.py.
- ST-14. A comparison with fewer matched pairs than
  the floor ends in the abstain outcome with the
  reason stated (insufficient pairs, the count against
  the floor), deterministically, with no model call.
  The floor is the registration's min_pairs where it
  states one, else 1; a stated floor under 1 is a named
  refusal. The default is 1 because the floor exists
  to end the zero-pair keep, not to invent a sample
  doctrine Round 1 never had; registrations state a
  higher floor where the fleet wants one. This
  supersedes the ST-9 rule's outcome at zero pairs:
  no pairs is insufficient evidence, never a keep.
  Test, tests/test_compare.py.
- ST-15. The judge is consulted only in the band:
  exactly one measure regressed past its margin, and
  the floor met. No regression is a keep and two or
  more a revert, deterministically, with the judgment
  recorded as not consulted and the reason stated. A
  band comparison with no judge configured ends in
  the deterministic tune, recorded as not consulted.
  Test, tests/test_compare.py.
- ST-16. The judge answers three calls over the card:
  a choice among keep, tune, revert, and abstain with
  its confidence and per-label probabilities; a score
  for the tradeoff's severity; and a score for the
  confidence the matching deserves. The judged choice
  stands as the proposal when it is not abstain and
  its confidence is at least 0.6. Otherwise the
  outcome is abstain, with the judge's answers
  recorded in full and the reason stated. A judge
  that cannot answer (an unrecorded card, a failed
  call) leaves the deterministic tune standing,
  recorded as unavailable with the reason, never as a
  judged outcome. Test, tests/test_judge.py.
- ST-17. Two transports answer the judge's calls. The
  fixture transport replays recorded responses, keyed
  by the card's digest, from a judgments file; a card
  the file does not record is unavailable. The live
  transport calls Jev (model jev-1.13.0) through the
  program's Typesafe connector route, named by the
  JEV_CALL environment variable; with the variable
  unset, live judging is a named refusal at the
  command line, never a silent skip. Tests use
  fixtures and scripted transports only; no test
  performs a network call. Test, tests/test_judge.py.
- ST-18. The comparison carries a judgment section in
  every case: the status (not consulted, judged,
  abstained, unavailable), the reason where one
  applies, and, when consulted, the card digest, the
  model name, the choice, the confidence, the
  per-label probabilities, and both scores. The
  proposal field carries the terminal outcome: keep,
  tune, revert, or abstain. Determinism (ST-11) is
  unchanged for any fixed judge. Test,
  tests/test_compare.py.
- ST-19. Recorded judgments are fixtures of synthetic
  cards only, each carrying the model name and the
  three recorded responses. A fixture whose recorded
  response is malformed is unavailable, never
  repaired in flight. Test, tests/test_judge.py.

## Round 2 post-review amendments, recorded before the repairs

- ST-13a. The registration's change and date are
  single-line texts: a line break or control character
  in either is a named refusal. The card is lines, and
  the fields stated on it may not forge new ones. The
  first implementation let a newline in the change
  text write extra lines, and an instruction to the
  judge, into the card the live judge reads. Test,
  tests/test_compare.py.
- ST-16a. Judge answers are validated for range as
  well as shape: the confidence and every per-label
  probability are finite numbers between 0 and 1, the
  probability labels are exactly the four, their sum
  is within 0.02 of 1, both scores are finite and
  between 0 and 10, and every response names the
  pinned model of ST-17. Anything else is malformed
  and unavailable under ST-19. The first
  implementation's shape checks passed a NaN
  confidence, which is never below the floor, into a
  judged outcome and out into the emitted document,
  which then was not strict JSON. Test,
  tests/test_judge.py.
- ST-17a. A judgments file is validated whole when it
  is loaded: every entry carries a card digest and a
  responses object, no digest repeats, and a stored
  card text digests to the digest it is filed under.
  The command refuses to write its output over the
  registration or the judgments file it reads. Test,
  tests/test_judge.py and tests/test_compare.py.

## Round 3. The Rust port (ST-20 to ST-26)

David's ruling of 2026-11-02: the repos are Rust
libraries. This round ports secondtake, and the Python
of Rounds 1 and 2 stands as the behavioral spec the
port must reproduce exactly.

- ST-20. The port reproduces the behavior of Rounds 1
  and 2 exactly: the same compare command with the
  same comparison documents, exit codes, refusal
  texts, decision cards and digests. Anything the
  port cannot reproduce is named in the round's
  dossier as a deviation, never silently dropped.
  Test, the ported suite throughout.
- ST-21. The crate is a library plus a thin binary
  named secondtake carrying the CLI. Canonical form
  1, the tape reading, the duration rule and the
  outcome rule are consumed from the flighttape
  crate at a pinned revision (the Round 11 merge),
  never re-implemented in this repo: the decision
  card's digest is the flighttape golden discipline
  applied one level up. The flighttape crate is the
  only direct dependency; everything else is the
  Rust standard library. The repository's CI gate
  becomes cargo test plus the codepoint scan. Test,
  Cargo.toml and the gate run.
- ST-22. Where this repo's Python reads a tape in its
  own way (the exclusion discipline over a directory,
  the defect and refusal phrases it answers with),
  the Python's behavior governs and the difference
  from the flighttape crate's reading is named in
  the dossier. The crate is a tool under this
  contract, never the spec over it. Test, the
  ported suite throughout.
- ST-23. The judge transport stays a subprocess call
  to the recorded helper, with the payload and
  response shapes of ST-13 to ST-19 unchanged. The
  recorded judgments fixture replays through the
  Rust client with identical proposals and no live
  call. Test, the ported judge suite.
- ST-24. The Python test suite's cases port to cargo
  tests under the same intent, file for file where
  the module maps. Cases that exist only in Python's
  own shapes are named in the dossier. The Python
  files leave the tree only in the landing state of
  this round, when the port has passed every gate
  the Python passed. Test, cargo test on the final
  head.
- ST-25. Before the port lands, a differential
  harness runs both implementations over tape
  directory pairs across the judgment band (below
  the pair floor, at the deterministic edges, in
  the band with a judge, without one, and with the
  judge unavailable) and byte-compares stdout,
  stderr, exit codes, and written files. A single
  differing byte is a port defect to fix, not a
  difference to document. Test, the round's dossier.
- ST-26. The port report stands in the dossier: port
  or rewrite with reasons, the Rust versus Python
  line share measured on the landing tree, the
  bindings determination, and the non-Rust remainder
  register with a justification per item. No Python
  bindings: the consumer shape is a process-boundary
  CLI plus crate consumers.
- ST-27. Round 4 re-pins the flighttape crate
  from ba8d4b5 to its Round 12 merge eebce0be,
  for fleet consistency with the rest of the
  line. The consumption surface is established
  by grep before the pin moves: which
  flighttape modules this crate names decides
  whether the Round 12 delta (the timestamp
  grammar and the float and printability forms)
  has any surface here. A control run of the
  standing differential harness against the
  re-pinned binary, before any other change,
  is the evidence either way.
- ST-28. The round's verification is the full
  local gate plus the differential harness at
  zero failures. CodeRabbit's advisory pass on
  Round 3's PR 4 posted no findings (zero
  entries on the review endpoints), so no
  disposition is owed from it.
- ST-29. Round 5 re-pins flighttape at 74a83298,
  the Round 13 merge, in every table that names
  it. Nothing else in the crate's behavior
  changes.
- ST-30. The divergences the re-pin retires are
  proven by probes, not by the standing harness
  alone (Round 4's lesson: the corpus was blind
  to both grammar classes). The round's probe
  set carries, as triplet probes against the
  Python original and both pins: the colon-group
  end stamp from Round 4's red team (Python
  reads fraction digits, 120.3), representative
  FT-63 to FT-66 tail shapes in start and end
  stamp positions, one FT-68 NUL completion
  shape reachable through a tape timestamp, and
  the three Round 4 probe shapes (the 2^-24
  digest, the separator-skip duration, the stray
  digit duration). The harness also gains
  standing cells for the shapes a tape can carry
  through its own files: colon-group and
  separator-skip end stamps in a compared pair,
  so the corpus is no longer blind to the
  classes it ships. A NUL cannot ride in a JSON
  string as a raw byte; if the probe shows the
  NUL class unreachable through secondtake's
  surfaces, the contract records that finding
  instead of a cell.
- ST-31. README records 5 of 100 only in the
  closure commit written after this round's
  three reviews, fixes, and owner verification.
- ST-32. The CodeRabbit disposition owed from
  Round 5 discharges here on evidence: the
  Round 5 pull request's late findings were
  read back in the 2026-11-04 rotation sweep
  and numbered zero, so there is nothing to
  fix and the absence is the disposition.
- ST-33. Round 6 takes the two write-seam
  probes the rotation has not yet run on this
  crate, at the compare command's output
  write, the crate's one temp seam. First,
  temp placement: the oracle's mkstemp
  absolutizes the given parent lexically,
  and a port creating its temp under the
  spelled parent diverges exactly where a
  popped symlink makes the lexical and
  physical parents differ in existence. The
  seam is measured on the oracle's own call
  structure (which spelling feeds the temp
  directory, which feeds the rename, which
  the cannot-write message), convicted
  shapes repaired per the measured law,
  agreement shapes pinned. Second, the temp
  suffix: mkstemp's suffix is exactly eight
  characters; this crate's generator (a
  nanos and pid construction) is measured
  for width, and the name-length boundary is
  measured in fresh pid namespaces in both
  directions; a boundary that moves with the
  generator's width is convicted and
  repaired to a fixed width of eight, the
  observable law being suffix length.
  Whichever arms convict, the repair keeps
  the rename target and the message spelling
  as they stand, unless the probes convict
  them too.
- ST-34. README records 6 of 100 only in the
  closure commit written after this round's
  three reviews, fixes, and owner verification.
- ST-35. The probes convict the rename target:
  the write consumes the raw spelling where the
  oracle consumes the pathlib construction of
  it. The write now consumes the construction
  (the display form's own normalization: tails
  drop, redundant separators collapse with a
  leading pair preserved, dot components drop,
  dot-dot components are kept), and the rename
  target is the constructed spelling. The
  cannot-write message keeps the raw argv
  spelling, as the oracle prints it. Convicted
  shapes: an output spelled with a trailing
  slash, a trailing dot component, or a double
  trailing slash, fresh and over an existing
  file, five cells, the port refusing where
  the oracle writes.
- ST-36. The probes convict the temp's home:
  the port creates its temp under the spelled
  parent, physically resolved, where mkstemp
  absolutizes the given parent lexically. The
  temp is now created under the lexical
  absolutization of the constructed parent:
  the parent joined to the working directory
  when relative, then normalized lexically,
  dot components dropping and dot-dot popping
  without following any link. The rename still
  resolves physically, so a popped spelling
  lands where the kernel puts it; only the
  temp's home moves. Convicted shape: a
  read-only lexical parent with an escaping
  pop, where the port wrote at the popped
  location and the oracle refuses cannot
  write; the mirror shape, a read-only
  physical parent, refuses on both sides and
  is pinned as agreement.
- ST-37. The probes convict the temp suffix's
  width: the generator's fixed eighteen hex
  characters move the name-length boundary
  ten characters inside mkstemp's. The suffix
  is now exactly eight characters, four bytes
  hex encoded, drawn from the standing
  construction (urandom first, else the nanos
  and pid mix), so the temp name's overhead
  over the output name is nine characters,
  mkstemp's own. The boundary then stands
  where the oracle's stands: output basenames
  to 246 characters write, 247 refuse, at
  ambient pids and in a fresh pid namespace
  alike. Convicted shapes: basenames of 237,
  240, and 246 characters, and 240 through a
  pop, the port refusing where the oracle
  writes.
- ST-38. The families' agreement shapes are
  pinned in the differential harness beside
  the convicted ones: the popped, split,
  dangling, tailed-parent, absolute, deep,
  escape, and guard shapes, the boundary
  lengths, and the boundary through a pop.
  The read-only lexical parent cell is
  probe-pinned instead: the root-run harness
  cannot stage a read-only parent against
  itself, so the cell stands in the probe
  record, staged as uid 65534.
