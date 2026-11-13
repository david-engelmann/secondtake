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
- ST-39. The write seam's name handling is
  byte-counted on both sides. The differential
  harness pins non-UTF8 output names through the
  write beside the boundary cases: a raw byte in
  the name and in the parent, fresh and through a
  pop, and the boundary under raw bytes, where a
  name of fewer characters but more bytes refuses
  exactly when its byte count crosses the limit.
- ST-40. The fallback entropy of ST-37 mixes the
  process id into the four drawn bytes: the
  sixteen-byte little-endian mix of nanos, pid,
  and attempt is folded by lanes, so no term of
  the mix is discarded by the slice that takes
  the four bytes.
- ST-41. The CodeRabbit disposition owed from
  Round 6 discharges here on evidence: the
  Round 6 pull request's record, read back
  through the list endpoints, carries exactly
  one inline finding (a Windows-aware
  normalization note on the temporary path),
  replied to and dismissed with its reason in
  Round 6 itself, a summary walkthrough with
  no further findings, and a follow-up command
  answered already reviewed. Nothing was
  posted after the merge, so there are no late
  findings to fix and the closed record is the
  disposition.
- ST-42. The deleted working directory family
  is measured at this command before any
  repair (the owed item from Round 6's red
  team). Launched from a deleted working
  directory with the registration readable,
  the Python dies uncaught out of its guard
  resolve with a FileNotFoundError traceback
  wherever a guard-resolved spelling is
  relative: the output target, the before
  set, the after set, and a judgments file
  each convict in turn, while the port
  resolves through a substituted root
  spelling and answers a named refusal
  further along (cannot write, the set is
  not a directory, no judgments file). Exit,
  stdout, and the file trees agree on every
  convicted cell; only the stderr form
  splits. The family's edges are agreement:
  a relative registration refuses by name at
  load on both sides, before any resolve
  runs, so the crash family starts only once
  the guard is reached; with no output
  requested, relative tape sets refuse by
  name at load on both sides, so the crash
  is guard-scoped; and with every spelling
  absolute, both sides write and exit zero
  from the deleted directory.
- ST-43. The guard's resolution fails where
  the interpreter's raises. resolve_path
  returns no path when a relative spelling
  needs the working directory and the
  working-directory read fails, in place of
  the substituted root spelling it returns
  today; absolute spellings never consult
  the working directory and never fail this
  way. The guard resolves in the Python's
  own order (the target, the before set, the
  after set, then the stated files), and a
  failed resolution takes the crash class:
  exit 1 with no message, this port's
  standing posture for the Python's uncaught
  raise. The named refusals and their order
  stand where every resolution succeeds.
- ST-44. The differential harness gains the
  family: the four convicted shapes as
  crash-class cells (exit, stdout, and files
  byte-compared; stderr compared by class, a
  traceback naming FileNotFoundError against
  silence), and the agreement shapes (the
  relative registration, no output with
  relative sets, absolute everything from
  the deleted directory, and the
  live-directory controls) byte-compared in
  full. A control run against the pre-round
  binary fails exactly the convicted cells.
- ST-45. The README comes to the fleet
  standard in this round: the first screen
  in the standard's order, the lede the
  repository description verbatim (the
  description carries one colon of its own;
  the copy bar yields to the
  description-identity rule there, and the
  exception is named here on the onrecord
  OR-39 precedent), one badge, the ci
  workflow's, verified against the
  workflow's live state before it ships, and
  one block of real captured output; then
  Install, Quickstart, Usage, Status, and
  License. The quickstart and usage outputs
  are captured from runs made for this round
  and reproduce from the repository's own
  fixtures. The package metadata in
  Cargo.toml carries a third description
  variant; this round's bound is the README,
  so the variant stands recorded here as a
  named open item, not edited.
- ST-46. README records 7 of 100 only in the
  closure commit written after this round's
  three reviews, fixes, and owner
  verification.
- ST-47. The CodeRabbit disposition owed
  from Round 7 is discharged here. PR 9's
  three surfaces, read to their ends
  through the list endpoints after the
  merge (5f61d67e, 2026-11-07T06:34:10Z),
  stand as follows: zero review comments
  and zero reviews; one summary shell
  (issue comment 6703379997), opened at
  06:29:21Z on the owner's manual
  invocation and finalized at 06:35:06Z,
  after the merge, in the Review failed
  form that records the pull request is
  closed, over a complete and accurate
  walkthrough of the round; and one later
  bot comment (6703451275, 06:37:11Z), the
  action notice that no review ran
  because the pull request is closed. No
  finding stands on PR 9, late or
  otherwise, so Round 7's merge-time zero
  stands confirmed as a count; the
  shell's final form is stated exactly
  here because a merge-time read cannot
  see it: a walkthrough under a Review
  failed header is a failed review with a
  walkthrough attached, not a completed
  one.
- ST-48. PR 8, read the same way for
  completeness, carries one finding,
  discharged here. Its summary shell
  (issue comment 6700626191) finalized at
  2026-11-07T01:56:36Z, before that merge
  (c8ecd01b, 01:58:08Z), in completed
  walkthrough form; its one inline
  comment (4499475165, Minor, on the CI
  workflow) and its one COMMENTED review
  (5809331939, actionable comments
  posted: 1) both predate the merge, and
  the comment stands unanswered. The
  finding asks that the queue branches
  (gh-readonly-queue/) be excluded from
  the workflow's push trigger, so that a
  merge group cannot start the gate
  twice, once by push and once by
  merge_group. It is recorded and not
  repaired, on two grounds. Merge queues
  are platform-blocked on user-owned
  repositories and cannot be switched on
  for this repository, so no merge group
  can form and the duplicate run cannot
  occur. And the workflow's trigger block
  is the fleet's uniform shaping step,
  the same in every repository on this
  line; a per-repository exclusion would
  diverge the fleet against a run that
  cannot happen, so the exclusion belongs
  to the fleet pass that lands when
  queues themselves land, not to one
  repository's round.
- ST-49. Where a guard spelling's
  resolution runs a symlink chain back
  into itself, the resolution fails (the
  loop arm). resolve_path keeps its shape
  (ST-43: canonicalize what exists, then
  the lexical remainder over the
  absolutized spelling) and gains one
  test on the absolutized spelling before
  the remainder is taken: a
  component-wise walk in the interpreter's
  own shape, substituting a link's target
  in place (an absolute target restarts
  the walk from the root) and holding
  each link in flight until its
  substitution completes; a link met
  again while its own substitution is in
  flight is the loop, a link met again
  after its substitution has completed is
  not, and a chain that never revisits a
  link is not, however deep it runs. A
  failed resolution takes the crash class
  at every guard call site (ST-43), exit
  1 with no message, matching the oracle:
  its resolve raises RuntimeError on the
  loop and its main catches only the
  named errors, so the run dies
  untrapped. The arm is scoped to genuine
  cycles. The measured 50-link acyclic
  chain still refuses at the loader on
  both sides, untouched, and every shape
  where the two resolutions already
  agreed stands unchanged. One cell of
  the family splits further than the owed
  item scoped: where the output spelling
  is itself the loop link, the unguarded
  port wrote the comparison over the link
  and exited 0; under the arm the run
  exits 1 and the link stands untouched.
  The probes behind this item ran eight
  convicted spellings (the output through
  two cycle shapes, the output as the
  loop link itself, one absolute output
  spelling, the before set, the after
  set, and the stated judge through a
  cycle and as the cycle) against seven
  agreement spellings, and every
  convicted spelling takes the crash
  class on both sides with exit, stdout,
  and trees identical.
- ST-50. The standing differential harness
  gains the symlink loop guard family:
  one special case laying a two-link
  cycle, a self-loop, and the agreement
  fixtures into both roots (fresh
  fixtures per cell, because the
  convicted cell whose output is the loop
  link rewrites that link on the
  pre-round binary and a shared fixture
  would carry the scar into later cells),
  then driving eight convicted cells
  (stderr by class, the oracle's
  traceback naming RuntimeError and the
  loop against the port's silence; exit,
  stdout, and files byte-compared) and
  seven agreement cells (byte-compared
  in full: the registration spelled
  through the cycle, an output inside a
  set and an output equal to the
  registration through links that resolve,
  an output through a dangling link, the
  loop before set with no output, the
  50-link chain as the before set, and a
  plain control). A control run against
  the pre-round binary reports 32 cases
  and 9 problems, every one inside this
  family and every one on a convicted
  cell, and passes the rest of the suite;
  the repaired run reports 32 cases and
  no problems. The crate suite gains
  the_write_guard_takes_the_crash_class_on_a_symlink_loop
  (tests/compare.rs), pinning the exit
  and the untouched link in process; run
  against the pre-round code it fails.
- ST-51. The description item carried
  open by ST-45 is closed. Cargo.toml's
  package description stood as a third
  variant of the text the repository
  description and the README's opening
  line already shared; it is set to that
  shared line byte for byte, and the
  three homes now carry one text.
- ST-52. README records 8 of 100 only in
  the closure commit written after this
  round's three reviews, fixes, and owner
  verification.
- ST-53. The round's three reviews all
  approve, and their findings converge
  in this amendment. (a) Two comments
  cited the loop arm to ST-47, the
  disposition item: the doc comment on
  resolve_path (src/cli.rs) and the
  comment on the crate pin
  (tests/compare.rs) now cite ST-49,
  the item that built the arm. (b)
  ST-49's law is scoped to the walk's
  defensive ceiling, commented in the
  source where it sits: a spelling
  whose resolution would follow more
  than 4096 links is reported as a
  loop, so no spelling can keep the
  walk substituting forever. The
  ceiling's observable reach was
  measured against the oracle over
  acyclic chains laid fresh per depth,
  the output spelled through the
  chain: at 900 links both sides exit
  1 on the named write error and
  nothing is written; at 1000 the
  oracle's own resolution dies first,
  a RecursionError traceback, while
  the port resolves the chain and
  reaches the named write error; the
  port still reaches it at 4096
  follows and takes the crash class,
  silence, from 4097, where the
  oracle still shows its
  RecursionError. The oracle's flip
  sits between 900 and 1000 links.
  Exit, empty stdout, and nothing
  written agree at every measured
  depth; only the stderr shape
  differs inside those bands. The
  band stands as a probe-pinned
  divergence, and whether the ceiling
  is ever aligned to the oracle's
  recursion boundary is a later
  source round's question. (c) The
  crate pin is renamed to state what
  it asserts:
  the_write_guard_exits_1_and_leaves_the_loop_link_untouched
  pins the exit and the untouched
  link in process, and the silence
  that completes the crash class
  rests on the harness cells of
  ST-50. (d) The harness gains one
  cell for ST-49's completed-link
  arm, out-through-completed-link-twice,
  spelling the output
  dlink/../dlink/out.json so a link
  is met again after its substitution
  has completed, the completed-link
  diamond: the full harness reports
  32 cases and no problems against
  the repaired binary, and against
  the pre-round control the same 9
  problems as ST-50, every one on a
  convicted cell, with this cell
  clean on both runs, the arm
  touching loop detection only.
- ST-54. Closure. The round closes
  on its three reviews (spec
  approves with no Critical and no
  Major findings, two Minors and one
  Note; standards approves with two
  Minors and two Notes; the red team
  approves with one Minor) and on
  the ST-53 repair commit, which
  answers every Minor the three
  raised. Under ST-52 the README's
  rounds line moves in this
  commit, to 8 of 100. The gate
  re-run on the repaired tree:
  cargo test 37 passed and none
  failed, clippy with warnings
  denied clean, fmt clean, and the
  codepoint scan over all 25 tracked
  files at zero for both dash
  characters; the harness stands as
  ST-53 records it, 32 cases and no
  problems against the repaired
  binary.
- ST-55. The CodeRabbit disposition on
  PR 10, three findings, each read in
  full and demonstrated against the
  oracle and the branch binary before
  it was disposed of. This item lands
  after the ST-54 closure commit; the
  README's rounds line is untouched by
  it and still reads 8 of 100, exact.
  (a) The description finding
  (4505417632) is demonstrated and not
  repaired. A registration stating
  min_pairs 3 against one matched pair
  ends in the proposal abstain, the
  judgment not consulted, the reason
  stated as insufficient pairs, 1 of 3,
  on the oracle and the port alike;
  the same tapes at the default floor
  propose keep. ST-14 contracts that
  outcome and ST-18 names abstain
  among the proposal field's four
  values, while the shared description
  names only keep, tune, and revert.
  The gap is real, and it stands
  flagged for David's word: the line
  is his text standing verbatim in
  three homes (the package
  description, the README's opening
  line, and the repository
  description), the repair belongs to
  the line itself, and no lane edits
  it inside a round. No edit is made.
  (b) The fallback finding
  (4505417635) is a real split on
  ST-43 ground, filed here as a named
  candidate for the next source round
  and not repaired. With the output
  spelled gone/../blink/out.json,
  where gone does not exist and blink
  is a link into the before set, the
  oracle refuses by name (writes
  nothing inside the tape sets it
  reads) while the port passes its
  guard, runs the comparison, and
  fails the write (cannot write the
  comparison, gone/../blink/out.json);
  the same split stands as uid 65534
  with the first component a directory
  of mode 444 in place of the missing
  one. The producer is the fallback's
  construction, not the loop arm: the
  fallback takes the longest existing
  ancestor and appends the remainder
  lexically, so a link sitting in the
  remainder is never followed, while
  the interpreter's resolution pops
  the .. off its resolved prefix and
  follows the link at its resolved
  position, landing inside the set.
  The candidate is named: links in
  the fallback's remainder resolve.
  (c) The completed-link finding
  (4505417638) is convicted against
  this round's own arm and repaired.
  On its fixture (hold/dlink a link
  to d/sub, hold/loop a self-loop,
  hold/d/loop a directory) the output
  spelled
  hold/dlink/../../dlink/../loop/../out.json
  resolves on the oracle, exit 0, the
  write landing in hold/d, while the
  shipped build took the crash class,
  exit 1 in silence, nothing written.
  The arm's completed-link case set
  the walk's prefix to the link's own
  spelling instead of following it
  again, so the components after the
  second meeting landed on hold/loop,
  the self-loop, where the resolution
  visits hold/d/loop, a directory.
  The repair follows a completed link
  again when a spelling meets it
  again, as the interpreter reuses
  the resolution it already made; the
  walk's ceiling counts a re-follow
  like any follow. Two pins guard it:
  the crate test
  a_completed_link_met_again_is_followed_again,
  which fails with the arm in its
  shipped form, and the harness cell
  out-past-a-loop-sibling-via-completed-link
  in ST-50's family. The harness on
  the final tree reports 32 cases and
  no problems; against the pre-round
  control it reports the same 9
  problems as ST-50, every one on a
  convicted cell, with the new cell
  clean there, the control's fallback
  reaching the spelling's ancestor
  through the kernel: the regression
  was the arm's own, and its
  conviction stands on the probe and
  the pin against the round's build.
  (d) The gate re-run on the final
  tree: cargo test 38 passed and none
  failed (the new pin is the one over
  ST-54's 37), clippy with warnings
  denied clean, fmt clean, and the
  codepoint scan over all 25 tracked
  files at zero for both dash
  characters.
- ST-56. The CodeRabbit disposition
  on PR 10, completed, and the reply
  standing on PR 8, confirmed: every
  surface read back to its end after
  the merge (2026-11-08T07:57:11Z,
  349e121b; 9 review comments, 7
  reviews, 4 issue comments).
  (a) The findings review 5817038584
  (COMMENTED, 2026-11-08T07:26:04Z,
  over 5f61d67e..8ff5ec30) carried,
  beside the three findings ST-55
  disposed of, a fourth item Round 8
  never disposed: a Nitpick (Trivial)
  in its body, at
  tests/compare.rs:586-592, asking
  that the loop pin capture stderr
  and assert the guard's silence for
  both cycle paths, a downstream
  write or judge-load error being
  able to exit 1 as well. It stands
  answered by the assignment ST-53(c)
  already made: the pin's seat is
  the exit and the untouched link,
  in process, and the crash class's
  silence rests on ST-50's harness
  cells, which compare stderr by
  class for both convicted cycle
  shapes. No edit is made; the item
  is recorded here so the
  disposition is complete.
  (b) The walkthrough shell
  6718669019 (created on the owner's
  trigger, 2026-11-08T07:19:47Z; last
  updated 07:56:45Z, before the
  merge) stands in its final form as
  the free-review rate-limit form:
  the banner (Review limit reached;
  the next included review available
  in 22 minutes) over a complete
  walkthrough of the full range,
  8ff5ec30..0da859c4, 3 files
  (CONTRACT.md, src/cli.rs,
  tests/compare.rs); not the Review
  failed form. The findings review
  states the included allowance
  exhausted, 0 remain after this
  review, so the ST-55 head drew no
  fresh findings review, and its
  delta stands on the Round 8 delta
  review. Issue comment 6718727154
  (07:26:32Z) is the standing
  Already reviewed notice. The
  owner's replies (4505490586,
  4505490721, 4505490787; 07:56:52Z
  to 07:56:58Z) all stand before
  the merge.
  (c) Three acknowledgment replies
  postdate the replies ST-55
  contracted: 4505491281 at
  07:57:11Z, the merge's own second,
  on the abstain thread, accepting
  ST-55(a) and offering a tracking
  issue; 4505491539 at 07:57:18Z,
  after the merge, on the
  completed-link thread, thanks,
  the thread resolved; 4505492309
  at 07:57:37Z, after the merge and
  after the landing comment
  6718994143 (07:57:29Z), on the
  fallback thread, accepting the
  ST-55(b) filing, recording that
  the finding remains open because
  its fix is deferred, offering an
  issue, and recording a learning
  in the finding's own shape. No
  issue was opened on any thread.
  ST-57's repair answers the
  fallback thread's substance.
  (d) On PR 8 the owner's reply
  4505402150 (2026-11-08T07:19:09Z)
  drew exactly one further item:
  the acknowledgment 4505402923 at
  07:19:29Z, under COMMENTED review
  5817022541, accepting the ST-48
  filing and asking whether a
  separate issue is wanted or
  ST-48 suffices. No new claim, no
  human activity, nothing after.
  No issue was opened, and ST-48
  stands.
- ST-57. The ST-55(b) candidate is
  worked: links in the fallback's
  remainder resolve. resolve_path
  keeps its shape (canonicalize what
  exists; the longest existing
  ancestor is the fallback's seat)
  and its lexical append is replaced
  by the interpreter-shaped component
  walk, factored into one walk both
  resolution seats share: the
  remainder's components are taken
  left to right against the
  canonicalized prefix, a dot-dot
  pops the resolved prefix (a file
  component included, as the
  interpreter pops it as a string),
  a link among them is followed at
  its resolved position with its
  target substituted in place, a
  component that cannot be read
  stands as its plain name, and a
  link met again while its own
  substitution is in flight fails
  the resolution, the ST-49 posture,
  under the same defensive ceiling
  of 4096 follows. The loop arm's
  adjudicated behavior is
  undisturbed: the walk's steps,
  order, and bookkeeping are the
  arm's own, and ST-50's family
  reports unchanged against the
  repaired binary (ST-58).
  Demonstrated against the oracle
  first (the round's probe, 16
  cells): ten convicted cells split
  on the pre-round binary and agree
  on the repaired one, the output
  spelled gone/../blink/out.json;
  the before set and the after set
  each spelled through the seam (on
  the pre-round binary each dies at
  its loader, not at the guard);
  the output equal to the
  registration through the seam; a
  remainder link whose target is
  itself reached through a link; an
  absolute target; a dangling
  target landing inside the set; a
  remainder link two dead components
  and two pops deep; a dot-dot
  popping a file component onto the
  seam; and the first shape again
  as uid 65534, once plain and once
  with its first component a
  directory of mode 444 in place of
  the missing one. Five agreement
  cells hold on both binaries: the
  resolution leaving the set again
  after the link, a link after a
  dead component with no pop never
  reached by either side, the file
  pop landing at the root, the
  plain output, and the plain
  output inside the set. A loop
  waiting in the remainder holds
  the crash-class pairing on both
  binaries, the oracle untrapped,
  the port silent at exit 1. The
  crate pin
  a_link_in_the_resolvers_remainder_is_followed
  runs the binary itself and reads
  its stderr, the guard's refusal
  and the write failure it replaces
  sharing an exit code and an
  untouched tree; it fails against
  the pre-round source on the
  cannot-write text and passes on
  the repaired source.
- ST-58. The standing differential
  gains the remainder family,
  remainder link guard, beside
  ST-50's loop family: nine
  convicted cells byte-compared in
  full (the ST-57 battery's same-uid
  shapes), four agreement cells,
  and a loop waiting in the
  remainder compared by crash class.
  The uid shapes stay where ST-57
  holds them, in the round's probe;
  the harness runs at one uid.
  Against the repaired binary the
  full harness reports 33 cases and
  no problems. Against the pre-round
  control it reports 33 cases and 9
  problems, every one a stderr
  difference on a convicted
  remainder cell; ST-50's family,
  every agreement cell, and the
  remainder loop cell are clean on
  both runs.
- ST-59. Closure. The round closes
  on its gate and stands for the
  owner's three reviews at landing.
  Under the standing rule the
  README's rounds line moves in
  this commit, to 9 of 100. The
  gate on the final tree: cargo
  test 39 passed and none failed
  (the new pin is the one over
  ST-55's 38), clippy with warnings
  denied clean, fmt clean, and the
  codepoint scan over all 25
  tracked files at zero for both
  dash characters; the harness
  stands as ST-58 records it, 33
  cases and no problems against
  the repaired binary.

- ST-60. Amendments on the round's
  three reviews, both of them
  repairs to this record and neither
  touching code. (a) ST-57's battery
  enumeration overclaims the probe
  by one shape. The probe runs 16
  cells and convicts ten: the output
  spelled gone/../blink/out.json;
  the before set and the after set
  each spelled through the seam; the
  output equal to the registration
  through the seam; a remainder link
  whose target is itself reached
  through a link; an absolute
  target; a dangling target landing
  inside the set; a dot-dot popping
  a file component onto the seam;
  and the first shape again as uid
  65534, once plain and once with
  its first component a directory
  of mode 444 in place of the
  missing one. The eleventh shape
  ST-57 names, a remainder link two
  dead components and two pops deep
  (gone/deeper/../../blink/out.json),
  is not among them: it stands only
  in ST-58's family, as the cell
  out-through-remainder-link-deep,
  where it does convict the control
  and agree on the repaired binary.
  ST-57's enumeration is read
  scoped to the probe's actual ten,
  and the deep shape is credited to
  ST-58's family. ST-58's
  parenthetical is restated with
  it: the family's nine convicted
  cells are the probe's eight
  same-uid convicted shapes plus
  the deep cell, the probe's two
  uid shapes staying in the probe
  where ST-57 holds them, the
  harness running at one uid.
  (b) The ceiling's band is stated
  for the re-meeting shape, which
  ST-53(b)'s chain language does
  not reach. Over chains the
  oracle's own resolution dies of
  recursion between 900 and 1000
  links, the band ST-53(b) records.
  Over re-meetings, where a
  completed link is met again and
  again (blink/../ repeated, blink
  a link into the before set), the
  oracle's resolution cache makes
  each re-meeting free, so it never
  approaches its own limit, and the
  measured band runs the other way:
  at 4096 follows both sides state
  the guard's named refusal, and at
  4097 the oracle still refuses by
  name while the port takes the
  crash class, silence. Exit, empty
  stdout, and an untouched tree
  agree at every measured depth;
  the split is in stderr class
  only, and the port's silence past
  the ceiling is the only death in
  this band. It stands as a
  probe-pinned divergence of the
  same ceiling, measured on the red
  team's fixtures; whether the
  ceiling is ever aligned stays
  where ST-53(b) left it, a later
  source round's question. This
  amendment lands after the round's
  closure commit (ST-59); the
  README's rounds line stands
  untouched and still exact at 9
  of 100.

ST-61. The CodeRabbit disposition
  for PR 11. The review (COMMENTED,
  5817485917) posted no inline
  findings; its body's only item is
  the Docstring Coverage pre-merge
  check, which is DISMISSED with
  this reason: it names no function,
  the repository carries no
  docstring configuration or
  threshold of its own, and the
  check applies the review tool's
  default profile, so there is no
  repo rule for it to enforce (the
  touchstone VL-51 basis). The
  functions this round touched
  carry their contract citations in
  their own comments. No reply
  thread exists for a review-body
  item; this item is the record.
- ST-62. Round 10 opens with the
  standing read-back of PR 11 in
  full. One review (5817485917,
  CodeRabbit, COMMENTED,
  2026-11-08T10:12:23Z, over commit
  d59ccbed), no review comments,
  and four issue comments: the
  owner's trigger 6720155478, the
  bot's summary shell 6720156498,
  the bot's acknowledgment
  6720232484 in the Action not
  completed / Already reviewed the
  last commit shape, carrying no
  findings, and the owner's landing
  note 6720299770. Every body was
  read to its end. The only item
  anywhere on the PR is the
  Docstring Coverage failed
  pre-merge check (66.67 percent
  against the tool's own 80.00
  percent default threshold),
  already disposed in ST-61 on
  ST-61's ground. Nothing on PR 11
  stands undisposed, and this round
  owes no new disposition from it.
- ST-63. The README's Install
  section is restored to the fleet
  standard's verbatim-runnable form
  (standalone audit fix 1, the
  audit's first priority). As it
  stood, the section gave a bare
  cargo install --path . with the
  clone and cd steps lost, so it
  could not be run from a lone
  clone. It now runs clone, cd, a
  scratch CARGO_HOME with its bin
  directory on PATH, and the
  install in one sh block, and it
  states plainly that the call
  script the JEV_CALL environment
  variable names is not shipped
  with this repository. The block's
  commands were run from a fresh
  directory against this round's
  tree and installed a working
  binary into the scratch home.
  What the tool does is unchanged
  and no code moved.
- ST-64. The judgment band's edges
  are measured fresh at the shipped
  CLI against a scripted fake judge
  in the established wire shape, its
  calls logged inside each run's own
  directory, every cell
  twin-compared byte for byte
  against the Python oracle over
  stdout, stderr, exit code, and
  every file, the log included.
  Fifteen cells, no problems, and
  no live model called. Agreement
  and two regressions never consult
  the judge (no call logged, the
  deterministic proposal standing),
  and below the pair floor the
  judge is likewise never consulted
  and the proposal abstains. In the
  band with no judge configured the
  tune stands. Consulted, the judge
  is asked exactly three questions
  (choice, severity, match): a keep
  at integer confidence 1 is
  judged, a tune at 0.6 is judged,
  a revert at 0.59 abstains with
  the reason naming 0.59, a revert
  at integer 0 abstains naming 0
  (that cell's input files verified
  byte-identical before and after),
  and an abstain choice abstains. A
  judge failing on the severity
  question stops at two calls, on
  the choice at one, and on the
  match at three, each leaving the
  tune standing as unavailable; a
  malformed choice answer stops at
  one call, and a missing call
  script is unavailable with no
  call made. The shipped fixture's
  recorded card is judged and its
  choice stands. Every edge
  acquits and no code moved.
- ST-65. The resolver bands are
  re-measured at this binary
  against the oracle, twenty-six
  cells, every cell twin-compared
  on exit, stdout, and every file,
  stderr byte-compared except where
  a band is recorded by class. A
  remainder link and a dot-dot
  popping a file component each
  draw the guard's named refusal on
  both sides; a link after a dead
  component splits neither (exit 1
  and byte-identical stderr); a
  link met again after its own
  substitution completed is
  followed again and the write
  lands; a loop waiting in the
  remainder holds the crash-class
  pairing, the oracle untrapped and
  the port silent at exit 1. Over
  chains, the split ST-53(b)
  recorded between 900 and 1000
  links is pinned at this seat: at
  991 links both sides state the
  named refusal, and at 992 the
  oracle takes the crash class
  while the port still refuses by
  name, the port naming the refusal
  at every measured length through
  1100. Over re-meetings (blink/../
  repeated, blink a link into the
  before set) both sides refuse by
  name at 4096 follows, and at 4097
  the oracle still refuses by name
  while the port takes the crash
  class, silence, reproducing
  ST-60(b) exactly. Every band
  acquits at its recorded edges and
  no code moved.
- ST-66. Closure. The round closes
  on its gate and stands for the
  owner's three reviews at landing.
  Under the standing rule the
  README's rounds line moves in
  this commit, to 10 of 100. The
  gate on the final tree: cargo
  test 39 passed and none failed
  (no count moved; the round
  changes no code), clippy with
  warnings denied clean, fmt clean,
  and the codepoint scan over all
  25 tracked files at zero for both
  dash characters; the standing
  differential reports 33 cases and
  no problems against the round's
  binary, and the round's own
  battery stands as ST-64 and ST-65
  record it, 41 cells and no
  problems.
- ST-68. Round 11 opens with the
  standing read-back of PR 12 in
  full. Five reviews: 5833871490
  (CodeRabbit, COMMENTED,
  2026-11-09T22:41:36Z, over the
  closure head 0a3eb77e) announcing
  three actionable comments;
  5833934086 and 5833934576 (the
  owner, COMMENTED, empty bodies,
  over the repair head 5dc37370),
  the carriers of the thread
  replies; and 5833938104 and
  5833939923 (CodeRabbit, COMMENTED,
  empty bodies, over 5dc37370).
  Six review comments: 4518849886
  (CodeRabbit, on README line 22)
  carrying two Minor findings, that
  the scratch Cargo home the Install
  block creates inside the clone is
  not kept out of Git, and that the
  install root is not pinned against
  a reader's own CARGO_INSTALL_ROOT;
  4518849898 (CodeRabbit, on line
  23), a Minor asking that PATH for
  later shells be explained;
  4518900432 (the owner), disposing
  the first thread's two findings,
  the first valid and fixed in
  5dc37370, which adds .cargo-home/
  to .gitignore and changes nothing
  else, the second dismissed on the
  ground that a reader's own install
  root setting is their global
  toolchain override and pinning
  --root would depart from the form
  ST-63 contracted and proved;
  4518900808 (the owner), dismissing
  the PATH finding, the documented
  sequence running in one shell and
  the suggested sentence keying the
  instruction to a placeholder path
  the copy bar does not carry;
  4518903765 (CodeRabbit),
  withdrawing the PATH suggestion
  and resolving that thread; and
  4518904921 (CodeRabbit), thanking
  for the fix and accepting the
  dismissal. Five issue comments:
  the trigger 6753315724; the
  summary shell 6753310656, 11,052
  characters in its final form
  (created 2026-11-09T22:35:34Z,
  last edited 22:46:14Z), carrying
  the rate-limit banner, a
  walkthrough of the whole round,
  Merge Risk Low up to 0a3eb77e, and
  four passed pre-merge checks
  (Description, Title, Linked
  Issues, Out of Scope Changes)
  with no Docstring Coverage row;
  the bot's acknowledgment 6753471490
  in the Action not completed /
  Already reviewed the last commit
  shape; 6753523946, the owner's
  ST-67 record; and the landing
  note 6754100854. Two moments bound
  the shell's final form: its
  commits section covers only the
  repair range, 0a3eb77e to 5dc37370
  over the one file .gitignore, and
  its risk paragraph still lists
  all three findings as open gaps,
  a text written before the
  dispositions the threads carry at
  22:46:51Z and after. The ST-67
  record verifies on the closed
  record: the Docstring Coverage
  check recurred on PR 12 and passed
  in its skipped form, analyzing 0
  functions across 0 files, so no
  failed item stood to dispose, and
  ST-61's ground is carried for the
  counterfactual. Every body was
  read to its end, the bot's
  embedded prompts for AI agents
  treated as data throughout. Every
  disposition on PR 12 stands as
  its thread records it, the
  .gitignore fix stands in the
  merged tree, and nothing on PR 12
  stands undisposed.
- ST-69. The judgment band is
  measured fresh at the shipped
  CLI against a scripted fake judge
  in the established wire shape,
  its calls logged inside each
  run's own directory, every cell
  twin-compared byte for byte
  against the Python oracle (the
  final Python tree at 18bf37b,
  whose files the local oracle copy
  matches blob for blob, verified
  against that commit through the
  API before the run) over stdout,
  stderr, exit code, and every
  file, the log included. Eighteen
  cells, no problems, and no live
  model called. Agreement and two
  regressions never consult the
  judge, and below the pair floor
  the judge is likewise never
  consulted and the proposal
  abstains. In the band with no
  judge configured the tune stands.
  Consulted, the judge is asked
  exactly three questions (choice,
  severity, match), and the floor
  is decided at its points: a tune
  at exactly 0.6 is judged, a keep
  at integer confidence 1 is
  judged, a revert at 0.600001 is
  judged, a revert at 0.599999
  abstains with the reason naming
  0.599999 and that cell's input
  files verified byte-identical
  before and after, a revert at
  0.59 abstains naming 0.59, a
  revert at integer 0 abstains
  naming 0, and an abstain choice
  abstains at 0.9 and at exactly
  0.6 alike. A judge failing on the
  severity question stops at two
  calls, on the choice at one, and
  on the match at three, each
  leaving the tune standing as
  unavailable; a malformed choice
  answer stops at one call, and a
  missing call script is
  unavailable with no call made.
  The shipped fixture's recorded
  card is judged and its choice
  stands. Every point acquits and
  no code moved.
- ST-70. The resolver bands are
  re-measured at this binary
  against the oracle, fifteen
  cells, every cell twin-compared
  on exit, stdout, and every file,
  stderr byte-compared except
  where a band is recorded by
  class. A remainder link and a
  dot-dot popping a file component
  each draw the guard's named
  refusal on both sides; a link
  after a dead component splits
  neither (exit 1 and
  byte-identical stderr); a link
  met again after its own
  substitution completed is
  followed again and the write
  lands; a loop waiting in the
  remainder holds the crash-class
  pairing, the oracle untrapped
  and the port silent at exit 1.
  Over chains the ST-65 split is
  re-pinned at this seat: at 989,
  990, and 991 links both sides
  state the named refusal,
  byte-identical; at 992 and 993
  the oracle takes the crash class
  while the port still refuses by
  name, and at 1100 the oracle
  stands in the crash class while
  the port names the refusal, the
  split standing at exactly 991
  and 992. Over re-meetings
  (blink/../ repeated, blink a
  link into the before set) both
  sides refuse by name at 4096
  follows, and at 4097 the oracle
  still refuses by name while the
  port takes the crash class,
  silence, reproducing ST-60(b)
  exactly. The flighttape pin is
  re-confirmed live: the pinned
  commit 74a83298 stands on tree
  afd4d7d1 with exactly 50 blobs
  in its recursive listing,
  untruncated, and the golden
  digest is executed at the pinned
  checkout, the converted
  synthetic session digesting to
  cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3
  and verify accepting it at exit
  0. Every band acquits at its
  recorded edges and no code
  moved.
- ST-71. The realistic pass. A
  corpus grid of synthetic tape
  pairs at realistic shapes (five
  token shapes, clean and erroring
  runs, redo counts from none to
  two, both outcomes, and two
  registration floors) is compared
  twin against twin, byte for
  byte, under a scripted judge
  whose plan rotates through six
  postures: keep at 0.9, tune at
  0.6, revert at 0.59, abstain at
  0.9, revert at 0.600001, and
  keep at integer 1. The pass
  stands at 132 steps: 124 compare
  events and 8 refusal steps, the
  refusals aiming the output
  inside the before set and
  refused by name on both arms.
  Of the 124 events the proposals
  stand at 77 keep, 33 abstain, 10
  revert, and 4 tune, and the
  judgments at 98 not consulted,
  24 judged, and 2 abstained; no
  step split the twins on exit,
  stdout, stderr, or any file. The
  standing differential harness
  also runs against the round's
  binary, 33 cases and no
  problems. No live model was
  called, and no code moved.
- ST-72. Closure. The round closes
  on its gate and stands for the
  owner's three reviews at landing.
  Under the standing rule the
  README's rounds line moves in
  this commit, to 11 of 100. The
  gate on the final tree: cargo
  test 39 passed and none failed
  (no count moved; the round
  changes no code), clippy with
  warnings denied clean, fmt clean,
  and the codepoint scan over all
  25 tracked files at zero for
  both dash characters; the
  standing differential reports 33
  cases and no problems against
  the round's binary, and the
  round's own measurements stand
  as ST-69 to ST-71 record them,
  33 battery cells and 132 corpus
  steps, no problems anywhere.

- ST-73. Round 12 opens with the
  standing read-back of PR 13 in
  full, on its closed record. The
  census stands at no reviews, no
  review comments, and four issue
  comments: the lane's trigger
  6830553637
  (2026-11-12T17:12:55Z), the
  CodeRabbit shell 6830557465, the
  bot's follow-up 6830597463 (the
  action-not-completed reply
  recording that the bot already
  reviewed the last commit), and
  the owner's landing comment
  6831097873. The shell is
  contracted in its true final
  form at this read: a completed
  CodeRabbit review and not a
  review skip, created
  2026-11-12T17:13:04Z and last
  edited 2026-11-12T17:33:36Z at
  11,523 characters, carrying a
  walkthrough, Merge Risk Minimal
  up to ea30d6c0, four passed
  pre-merge checks (Title, Linked
  Issues, Out of Scope Changes,
  Description), the word Docstring
  occurring zero times, the line
  that no actionable comments were
  generated in the recent review,
  and Run ID
  99158a60-5054-413b-b50c-5c0c3ad1f83c;
  no rate-limit banner stands in
  it. The Round 11 step 0 account
  is graded against it: the build
  lane's first form, a
  Markdown-only review skip at
  11,519 characters, was false and
  drew that round's Minors; the
  owner's corrected account in the
  body stands true of the shell as
  it stood at the owner's read
  (11,521 characters, last edited
  2026-11-12T17:17:42Z), and the
  shell then drifted two
  characters in a final edit of
  2026-11-12T17:33:36Z, which no
  disposition of that round rests
  on. No late CodeRabbit finding
  stands on PR 13: nothing was
  posted after the landing comment
  and no review comment exists, so
  nothing stands undisposed. Per
  the Round 11 charge, the ST-69
  enumeration is stated enumerably
  from its parts: its eighteen
  cells are the agreement cell
  (1), the two-regressions cell
  standing as one cell over two
  regression pairs (1), the
  below-floor cell (1), the band
  cell with no judge configured
  (1), the three judged cells (a
  tune at exactly 0.6, a keep at
  integer 1, a revert at
  0.600001), the five abstain
  cells (a revert at 0.599999, a
  revert at 0.59, a revert at
  integer 0, an abstain choice at
  0.9, an abstain choice at
  exactly 0.6), and the six
  failure and replay cells (a
  severity failure stopping at two
  calls, a choice failure stopping
  at one call, the shipped
  fixture's recorded card judged
  (the cell the Round 11 spec Note
  names as the battery's
  fifteenth), a match failure
  stopping at three calls, a
  malformed choice answer stopping
  at one call, and a missing call
  script with no call made),
  summing as 1 + 1 + 1 + 1 + 3 + 5
  + 6 = 18. The ST-70 enumeration
  likewise: its fifteen cells are
  the five structural cells its
  prose names (a remainder link, a
  dot-dot popping a file
  component, a link after a dead
  component, a link met again
  after its own substitution
  completed, and a loop waiting in
  the remainder), the six chain
  cells at 989, 990, 991, 992,
  993, and 1100 links, the two
  re-meeting cells its prose names
  at 4096 and 4097 follows, and
  the two re-meeting cells its
  battery carries and its prose
  does not name, at 11 and 1001
  follows, summing as 5 + 6 + 2 +
  2 = 15. No Round 11 disposition
  is falsified by the record as it
  stands.

- ST-74. The judgment band is
  measured fresh at the shipped
  CLI against a scripted fake
  judge in the established wire
  shape, its calls logged inside
  each run's own directory, every
  cell twin-compared byte for byte
  against the Python oracle (the
  final Python tree at 18bf37b,
  whose thirteen files the local
  oracle copy matches blob for
  blob, verified against that
  commit through the API before
  the run) over stdout, stderr,
  exit code, and every file, the
  log included. Nineteen cells,
  each one named, no problems, and
  no live model called. The
  never-consulted family stands at
  five cells: an agreement cell,
  an armed judge never called and
  the proposal keeping; a
  two-regressions cell over cost
  and time; a two-regressions cell
  over cost and rework (Round 11
  stated its two regression pairs
  inside a single cell; this round
  counts them as the two cells
  they are); a
  below-the-pair-floor cell, one
  pair against a registered floor
  of two, abstaining as
  insufficient pairs, 1 of 2, the
  judge never called; and the band
  cell with no judge configured,
  the tune standing as not
  consulted for that reason. The
  judged family stands at three
  cells, each decided after
  exactly the calls choice,
  severity, and match in that
  order: a tune at exactly 0.6 is
  judged, a keep at integer
  confidence 1 is judged, and a
  revert at 0.600001 is judged.
  The abstain family stands at
  five cells: a revert at 0.599999
  abstains with the reason naming
  0.599999 and that cell's input
  files verified byte-identical
  before and after; a revert at
  0.59 abstains naming 0.59; a
  revert at integer 0 abstains
  naming 0; an abstain choice
  abstains at 0.9; and an abstain
  choice abstains at exactly 0.6
  alike. The failure family stands
  at five cells: a judge failing
  on the choice question stops at
  one call, on the severity
  question at two calls, and on
  the match question at three
  calls, each leaving the tune
  standing as unavailable; a
  malformed choice answer, its
  probabilities summing to two,
  stops at one call as
  unavailable; and a missing call
  script is unavailable with no
  call made. The replay family
  stands at one cell: the shipped
  fixture's recorded card,
  produced by this round's own
  band set and replayed through
  the judgments file at
  tests/fixtures/judgments_synthetic.json,
  is judged and its choice stands,
  a tune at confidence 0.85. The
  families sum as 5 + 3 + 5 + 5 +
  1 = 19. Every point acquits and
  no code moved.

- ST-75. The resolver bands are
  re-measured at this binary
  against the oracle, fifteen
  cells, each one named, every
  cell twin-compared on exit and
  stdout, and on stderr byte for
  byte except where a band is
  recorded by class. The
  structural family stands at five
  cells: a remainder link, a
  directory link standing in the
  output's remainder and resolving
  inside the before set, draws the
  guard's named refusal on both
  sides; a dot-dot popping a file
  component draws the named
  refusal on both sides; a link
  after a dead component, a
  missing component popped by a
  dot-dot before the link is met,
  draws the named refusal on both
  sides; a link met again after
  its own substitution completed
  is followed again and the write
  lands on both sides; and a loop
  waiting in the remainder holds
  the crash-class pairing, the
  oracle untrapped and the port
  silent at exit 1. The chain
  family stands at six cells, a
  chain counted by its links from
  the output path to its target:
  at 989, 990, and 991 links both
  sides state the named refusal
  byte-identically; at 992 and 993
  links, and at 1100 links, the
  oracle takes the crash class
  while the port still refuses by
  name, the ST-65 split standing
  at exactly 991 and 992. The
  re-meeting family stands at four
  cells, the output spelled
  through a link standing in the
  before set and met again after
  each dot-dot: at 11 follows, at
  1001 follows, and at 4096
  follows both sides refuse by
  name (the two smaller counts
  stood in Round 11's battery
  unnamed in its prose; they are
  named here), and at 4097 follows
  the oracle still refuses by name
  while the port takes the crash
  class, silence, reproducing
  ST-60(b) exactly. The families
  sum as 5 + 6 + 4 = 15. The
  flighttape pin is re-confirmed
  live: the pinned commit 74a83298
  stands on tree afd4d7d1 with
  exactly 50 blobs in its
  recursive listing, untruncated,
  and the golden digest is
  executed at the pinned checkout,
  the synthetic session digesting
  to
  cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3
  and verify accepting it at exit
  0. Every band acquits at its
  recorded edges and no code
  moved.

- ST-76. The realistic pass. A
  corpus of synthetic tape pairs
  at realistic shapes (token
  totals at five levels, clean and
  erroring runs, redo counts from
  none to two, both outcomes, and
  two registration floors) is
  compared twin against twin, byte
  for byte, in five families. The
  agreement family stands at 40
  compare events, each proposing
  keep and none consulting the
  judge. The band family stands at
  60 compare events under a
  scripted judge rotating through
  six postures: keep at 0.9, tune
  at exactly 0.6, revert at 0.59,
  abstain at 0.9, revert at
  0.600001, and keep at integer 1.
  The two-regressions family
  stands at 24 events proposing
  revert without consulting. The
  below-floor family stands at 12
  events abstaining as
  insufficient pairs. The
  band-with-no-judge family stands
  at 12 events, the tune standing
  as not consulted. The pass
  stands at 160 steps: the 148
  compare events and 12 refusal
  steps, the refusals aiming the
  output inside the before set in
  four steps, inside the after set
  in four, over the registration
  file in two, and over the
  judgments file in two, each
  refused by name on both arms at
  exit 1. Of the 148 events the
  proposals stand at 60 keep, 22
  tune, 34 revert, and 32 abstain,
  and the judgments at 88 not
  consulted, 40 judged, and 20
  abstained; no step split the
  twins on exit, stdout, stderr,
  or any file. The standing
  differential harness also runs
  in full against the round's
  binary, 33 cases and no
  problems: help and parse
  surfaces at seven cases
  (top-level help, no arguments,
  an unknown subcommand, compare
  help, a missing registration
  argument, an extra positional,
  and a dash-dash separator), set
  and registration loading at ten
  (a missing before directory, a
  missing registration, a
  registration that is not JSON, a
  registration that is not an
  object, a registration missing
  its change, a registration
  missing its date, an unknown
  margin key, a zero pair floor, a
  margin of the wrong type, and a
  change carrying control
  characters), tape set shapes at
  six (a tape that is not JSON, a
  tape missing its totals, an
  ambiguous task key, an unmatched
  tape, an empty before set, and a
  tape of bytes that are not
  UTF-8), judge loading at four (a
  missing judgments file, a
  judgments file that is not JSON,
  a judgments file carrying no
  cards list, and live judging
  with no call script named),
  output and option forms at four
  (output to stdout, a glued
  output form, a glued judge form,
  and an abbreviated long option),
  and comparison shapes at two (a
  band control and a rescue rate
  regression). No live model was
  called, and no code moved.

- ST-77. Closure. The round closes
  on its gate and stands for the
  owner's three reviews at
  landing. Under the standing rule
  the README's rounds line moves
  in this commit, to 12 of 100.
  The gate on the final tree:
  cargo test 39 passed and none
  failed (22 in tests/compare.rs
  and 17 in tests/judge.rs; no
  count moved, the round changes
  no code), clippy with warnings
  denied clean, fmt clean, and the
  codepoint scan over all 25
  tracked files at zero for both
  dash characters; the standing
  differential reports 33 cases
  and no problems against the
  round's binary, and the round's
  own measurements stand as ST-74
  to ST-76 record them, 34 battery
  cells and 160 corpus steps, no
  problems anywhere. ST-55(a)
  stands open and untouched, as
  the repository owner's reserved
  question.

- ST-78. Round 13 opens with the
  standing read-back of PR 14 in
  full, on its closed record. The
  census stands at no reviews, no
  review comments, and four issue
  comments: the lane's trigger
  6847250851
  (2026-11-13T07:47:41Z), the
  CodeRabbit shell 6847252590, the
  bot's follow-up 6847326162 (the
  action-not-completed reply
  recording that the bot had
  already reviewed the last
  commit, created
  2026-11-13T07:52:11Z and last
  edited 2026-11-13T07:52:14Z,
  carrying no finding and no
  correction), and the owner's
  landing comment 6847785643
  (2026-11-13T08:17:40Z). The
  shell is contracted in its true
  final form at this read: a
  completed CodeRabbit review of
  head
  f9c0171fe7351dadadbac1dc4daf96a744e3ea91,
  the head its own change
  assessment names, created
  2026-11-13T07:47:48Z and last
  edited 2026-11-13T07:53:59Z at
  11,673 characters, carrying a
  walkthrough, the line that no
  actionable comments were
  generated in the recent review,
  Merge Risk Minimal up to f9c01,
  four passed pre-merge checks
  (Title, Linked Issues, Out of
  Scope Changes, Description), the
  word Docstring occurring zero
  times, no rate-limit banner
  standing in it, and Run ID
  d7ba933e-e2a1-41a8-b271-3e21bc527cca
  standing hyphenated in full. The
  Round 12 step 0 account in the
  body is graded against it: the
  form it stated, 11,671
  characters at a last edit of
  2026-11-13T07:49:46Z, was true
  at its read, and the shell then
  drifted two characters in its
  final edit at
  2026-11-13T07:53:59Z, a drift
  the landing comment already
  records; no disposition fact
  moved. The route charge from the
  Round 12 record is discharged
  here: ST-75's chain split is
  route-relative on the oracle's
  invocation route. At the round's
  own route, python3 -m
  secondtake.cli over chains
  spelled with an absolute target,
  the split stands at exactly 991
  and 992, and the Round 12 red
  team's first construction
  through a different route, a
  python3 -c import, resolved a
  992-link chain cleanly on the
  oracle. The figures ride their
  route; ST-75's text stands
  unamended, and the route is
  stated beside the figures again
  at ST-80. No late CodeRabbit
  finding stands on PR 14: nothing
  was posted after the landing
  comment, no review comment
  exists, and the follow-up reply
  carries no finding, so nothing
  stands undisposed. ST-55(a)
  stands open and untouched, as
  the repository owner's reserved
  question.

- ST-79. The judgment band is
  measured fresh at the shipped
  CLI against a scripted fake
  judge in the established wire
  shape, its calls logged inside
  each run's own directory, every
  cell twin-compared byte for byte
  against the Python oracle (the
  final Python tree at 18bf37b,
  whose thirteen files the local
  oracle copy matches blob for
  blob, verified against that
  commit through the API before
  the run) over stdout, stderr,
  exit code, and every file, the
  log included. Nineteen cells,
  each one named, no problems, and
  no live model called. The
  never-consulted family stands at
  five cells: an agreement cell,
  an armed judge never called and
  the proposal keeping; a
  two-regressions cell over cost
  and time; a two-regressions cell
  over cost and rework; a
  below-the-pair-floor cell, one
  pair against a registered floor
  of two, abstaining as
  insufficient pairs, 1 of 2, the
  judge never called; and the band
  cell with no judge configured,
  the tune standing as not
  consulted for that reason. The
  judged family stands at three
  cells, each decided after
  exactly the calls choice,
  severity, and match in that
  order: a tune at exactly 0.6 is
  judged, a keep at integer
  confidence 1 is judged, and a
  revert at 0.600001 is judged.
  The abstain family stands at
  five cells: a revert at 0.599999
  abstains with the reason naming
  0.599999 and that cell's input
  files verified byte-identical
  before and after; a revert at
  0.59 abstains naming 0.59; a
  revert at integer 0 abstains
  naming 0; an abstain choice
  abstains at 0.9; and an abstain
  choice abstains at exactly 0.6
  alike. The failure family stands
  at five cells: a judge failing
  on the choice question stops at
  one call, on the severity
  question at two calls, and on
  the match question at three
  calls, each leaving the tune
  standing as unavailable; a
  malformed choice answer, its
  probabilities summing to two,
  stops at one call as
  unavailable; and a missing call
  script is unavailable with no
  call made. The replay family
  stands at one cell: the shipped
  fixture's recorded card,
  produced by this round's own
  band set and replayed through
  the judgments file at
  tests/fixtures/judgments_synthetic.json,
  is judged and its choice stands,
  a tune at confidence 0.85. The
  families sum as 5 + 3 + 5 + 5 +
  1 = 19. Every point acquits and
  no code moved.

- ST-80. The resolver bands are
  re-measured at this binary
  against the oracle, fifteen
  cells, each one named, every
  cell twin-compared on exit and
  stdout, and on stderr byte for
  byte except where a band is
  recorded by class. The
  structural family stands at five
  cells: a remainder link, a
  directory link standing in the
  output's remainder and resolving
  inside the before set, draws the
  guard's named refusal on both
  sides; a dot-dot popping a file
  component draws the named
  refusal on both sides; a link
  after a dead component, a
  missing component popped by a
  dot-dot before the link is met,
  draws the named refusal on both
  sides; a link met again after
  its own substitution completed
  is followed again and the write
  lands on both sides; and a loop
  waiting in the remainder holds
  the crash-class pairing, the
  oracle dying untrapped with
  RuntimeError, Symlink loop,
  raised out of pathlib's resolve,
  and the port silent at exit 1.
  The chain family stands at six
  cells, a chain counted by its
  links from the output path to
  its target, the chains run at
  the round's route, python3 -m
  secondtake.cli with the output
  spelled as an absolute path
  through the chain: at 989, 990,
  and 991 links both sides state
  the named refusal
  byte-identically; at 992 and 993
  links, and at 1100 links, the
  oracle takes the crash class
  while the port still refuses by
  name, the ST-65 split standing
  at exactly 991 and 992 at this
  route. The re-meeting family
  stands at four cells, the output
  spelled through a link standing
  in the before set and met again
  after each dot-dot: at 11
  follows, at 1001 follows, and at
  4096 follows both sides refuse
  by name, and at 4097 follows the
  oracle still refuses by name
  while the port takes the crash
  class, silence, reproducing
  ST-60(b) exactly. The families
  sum as 5 + 6 + 4 = 15. The
  flighttape pin is re-confirmed
  live: the pinned commit 74a83298
  stands on tree afd4d7d1 with
  exactly 50 blobs in its
  recursive listing, untruncated,
  and the golden digest is
  executed at the pinned checkout,
  the cargo git checkout of the
  pinned dependency, the synthetic
  session digesting to
  cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3
  and verify accepting it at exit
  0. Every band acquits at its
  recorded edges and no code
  moved.
