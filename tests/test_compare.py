"""ST-5 to ST-12. The matched comparison."""
import json
from datetime import datetime, timedelta
from pathlib import Path

import pytest

from secondtake.cli import main
from secondtake.compare import CompareError, compare, load_registration
from secondtake.digest import digest_of

FIX = Path(__file__).parent / "fixtures"
GOLDEN = "cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3"
BASE = datetime(2026, 1, 5, 9, 0, 0)
REG = {"change": "new system prompt", "date": "2026-01-04"}


def stamp(seconds):
    return (BASE + timedelta(seconds=seconds)).strftime(
        "%Y-%m-%dT%H:%M:%SZ")


def make_tape(task="Fix the parser", tokens=(100, 50), seconds=60,
              redos=0, error=False, outcome="completed",
              user=True):
    events = [{"seq": 0, "kind": "session_start", "sidechain": False,
               "timestamp": stamp(0)}]
    seq = 1
    if user:
        events.append({"seq": seq, "kind": "message",
                       "sidechain": False, "role": "user",
                       "text": task, "timestamp": stamp(1)})
        seq += 1
    calls = 1 + redos
    for index in range(calls):
        call_id = f"c{index}"
        events.append({"seq": seq, "kind": "tool_call",
                       "sidechain": False, "tool": "edit",
                       "call_id": call_id, "input": {"path": "a.py"},
                       "timestamp": stamp(2 + 2 * index)})
        seq += 1
        events.append({"seq": seq, "kind": "tool_result",
                       "sidechain": False, "call_id": call_id,
                       "is_error": error if index == 0 else False,
                       "timestamp": stamp(3 + 2 * index)})
        seq += 1
    events.append({"seq": seq, "kind": "session_end",
                   "sidechain": False, "timestamp": stamp(seconds)})
    tape = {"tape_version": "1.0",
            "run": {"harness": "synthetic", "session_id": "s-1"},
            "events": events, "outcome": {"status": outcome}}
    if tokens is not None:
        tape["run"]["totals"] = {"input_tokens": tokens[0],
                                 "output_tokens": tokens[1]}
    return tape


def write_set(directory, tapes):
    directory.mkdir(parents=True, exist_ok=True)
    for name, tape in tapes.items():
        (directory / name).write_text(json.dumps(tape))
    return directory


def registration(tmp_path, **extra):
    path = tmp_path / "registration.json"
    path.write_text(json.dumps({**REG, **extra}))
    return load_registration(path)


def test_the_digest_is_pinned_to_the_family_golden():
    tape = json.loads((FIX / "tape_synthetic.json").read_text())
    assert digest_of(tape) == GOLDEN


def test_a_clean_pair_keeps(tmp_path):
    before = write_set(tmp_path / "b", {"a.json": make_tape()})
    after = write_set(tmp_path / "a", {"a.json": make_tape()})
    document = compare(before, after, registration(tmp_path))
    assert document["proposal"] == "keep"
    assert document["regressions"] == []
    assert document["aggregates"]["pairs"] == 1
    pair = document["pairs"][0]
    assert pair["before"]["cost"] == 150
    assert pair["deltas"]["cost"] == 0


def test_one_regression_tunes_and_two_revert(tmp_path):
    before = write_set(tmp_path / "b", {"a.json": make_tape(error=True)})
    costlier = write_set(
        tmp_path / "a", {"a.json": make_tape(tokens=(130, 50),
                                            error=True)})
    document = compare(before, costlier, registration(tmp_path))
    assert document["regressions"] == ["cost"]
    assert document["proposal"] == "tune"
    failed = write_set(
        tmp_path / "a2", {"a.json": make_tape(tokens=(130, 50),
                                              error=True,
                                              outcome="failed")})
    document = compare(before, failed, registration(tmp_path))
    assert document["regressions"] == ["cost", "rescue rate"]
    assert document["proposal"] == "revert"
    rescue = document["aggregates"]["rescue"]
    assert rescue["before_rate"] == 100.0
    assert rescue["after_rate"] == 0.0
    assert rescue["delta_points"] == -100.0


def test_rework_regresses_past_one_per_pair(tmp_path):
    before = write_set(tmp_path / "b", {"a.json": make_tape(error=True)})
    after = write_set(tmp_path / "a",
                      {"a.json": make_tape(error=True, redos=2)})
    document = compare(before, after, registration(tmp_path))
    assert document["pairs"][0]["after"]["rework"] == 2
    assert document["regressions"] == ["rework"]
    assert document["proposal"] == "tune"


def test_ambiguous_unmatched_and_unmatchable_are_reported(tmp_path):
    before = write_set(tmp_path / "b", {
        "a.json": make_tape(task="Same task"),
        "b.json": make_tape(task="Same task"),
        "c.json": make_tape(task="Only before"),
        "d.json": make_tape(user=False)})
    after = write_set(tmp_path / "a", {
        "a.json": make_tape(task="Same task"),
        "z.json": make_tape(task="Only after")})
    document = compare(before, after, registration(tmp_path))
    assert document["aggregates"]["pairs"] == 0
    assert len(document["ambiguous"]) == 1
    assert document["ambiguous"][0]["before"] == 2
    assert [u["file"] for u in document["unmatched"]["before"]] == ["c.json"]
    assert [u["file"] for u in document["unmatched"]["after"]] == ["z.json"]
    assert document["unmatchable"] == [{
        "side": "before", "file": "d.json",
        "reason": "no main-chain user message with a text"}]
    assert document["proposal"] == "abstain"
    assert document["judgment"] == {
        "status": "not_consulted",
        "reason": "insufficient pairs, 0 of 1"}


def test_excluded_tapes_are_named_with_defects(tmp_path):
    broken = make_tape()
    broken["tape_version"] = "9.9"
    before = write_set(tmp_path / "b", {"a.json": make_tape(),
                                        "bad.json": broken})
    after = write_set(tmp_path / "a", {"a.json": make_tape()})
    document = compare(before, after, registration(tmp_path))
    excluded = document["before"]["excluded"]
    assert excluded[0]["file"] == "bad.json"
    assert "tape_version is not 1.0" in excluded[0]["defects"]
    assert document["before"]["tapes_read"] == 2
    assert document["before"]["tapes_scored"] == 1


def test_absent_cost_is_unmeasured_not_zero(tmp_path):
    before = write_set(tmp_path / "b", {"a.json": make_tape(tokens=None)})
    after = write_set(tmp_path / "a", {"a.json": make_tape()})
    document = compare(before, after, registration(tmp_path))
    cost = document["aggregates"]["cost"]
    assert cost["pairs_counted"] == 0
    assert cost["before_mean"] is None
    assert "cost" not in document["regressions"]


def test_margins_come_from_the_registration(tmp_path):
    before = write_set(tmp_path / "b", {"a.json": make_tape()})
    after = write_set(tmp_path / "a",
                      {"a.json": make_tape(tokens=(130, 50))})
    strict = registration(tmp_path, margins={"cost_pct": 25})
    document = compare(before, after, strict)
    assert document["regressions"] == []
    assert document["registration"]["margins"]["cost_pct"] == 25
    assert document["registration"]["margins"]["time_pct"] == 10


def test_the_comparison_is_deterministic(tmp_path):
    before = write_set(tmp_path / "b", {"a.json": make_tape(error=True)})
    after = write_set(tmp_path / "a", {"a.json": make_tape(error=True)})
    reg = registration(tmp_path)
    first = json.dumps(compare(before, after, reg),
                       indent=2, ensure_ascii=False)
    second = json.dumps(compare(before, after, reg),
                        indent=2, ensure_ascii=False)
    assert first == second


def test_registration_refusals(tmp_path):
    with pytest.raises(CompareError, match="no registration"):
        load_registration(tmp_path / "missing.json")
    bad = tmp_path / "bad.json"
    bad.write_text("{not json")
    with pytest.raises(CompareError, match="not valid JSON"):
        load_registration(bad)
    unknown = tmp_path / "unknown.json"
    unknown.write_text(json.dumps({**REG, "margins": {"luck": 3}}))
    with pytest.raises(CompareError, match="unknown margin"):
        load_registration(unknown)


def test_the_cli_and_its_write_guard(tmp_path, capsys):
    before = write_set(tmp_path / "b", {"a.json": make_tape()})
    after = write_set(tmp_path / "a", {"a.json": make_tape()})
    reg_path = tmp_path / "registration.json"
    reg_path.write_text(json.dumps(REG))
    out = tmp_path / "comparison.json"
    assert main(["compare", str(before), str(after),
                 "--registration", str(reg_path),
                 "-o", str(out)]) == 0
    assert json.loads(out.read_text())["proposal"] == "keep"
    inside = before / "comparison.json"
    assert main(["compare", str(before), str(after),
                 "--registration", str(reg_path),
                 "-o", str(inside)]) == 1
    assert "writes nothing inside" in capsys.readouterr().err
    assert not inside.exists()
    assert main(["compare", str(before), str(after),
                 "--registration", str(tmp_path / "nope.json")]) == 1


def test_the_fixture_tape_keys_by_its_real_text():
    from secondtake.measures import task_key
    tape = json.loads((FIX / "tape_synthetic.json").read_text())
    assert task_key(tape) == digest_of(
        "Synthetic prompt, fix the widget counter.")


def test_cache_tokens_are_not_cost(tmp_path):
    rich = make_tape()
    rich["run"]["totals"]["cache_read_input_tokens"] = 999999
    before = write_set(tmp_path / "b", {"a.json": rich})
    after = write_set(tmp_path / "a", {"a.json": make_tape()})
    document = compare(before, after, registration(tmp_path))
    assert document["pairs"][0]["before"]["cost"] == 150


def test_pair_keys_are_the_task_text_digests(tmp_path):
    before = write_set(tmp_path / "b", {"a.json": make_tape()})
    after = write_set(tmp_path / "a", {"a.json": make_tape()})
    document = compare(before, after, registration(tmp_path))
    assert document["pairs"][0]["key"] == digest_of("Fix the parser")
    assert "proposal_rule" in document


def test_time_regresses_and_naive_stamps_read_as_utc(tmp_path):
    before = write_set(tmp_path / "b",
                       {"a.json": make_tape(seconds=60)})
    slow = make_tape(seconds=600)
    after = write_set(tmp_path / "a", {"a.json": slow})
    document = compare(before, after, registration(tmp_path))
    assert document["regressions"] == ["time"]
    mixed = make_tape(seconds=60)
    mixed["events"][0]["timestamp"] = "2026-01-05T09:00:00"
    mixed["events"][-1]["timestamp"] = "2026-01-05T09:01:00"
    before2 = write_set(tmp_path / "b2", {"a.json": mixed})
    document = compare(before2, after, registration(tmp_path))
    assert document["pairs"][0]["before"]["time_seconds"] == 60.0


def test_measure_shape_defects_exclude_the_tape(tmp_path):
    negative = make_tape()
    negative["run"]["totals"]["input_tokens"] = -5
    bare = make_tape()
    bare["outcome"] = "completed"
    before = write_set(tmp_path / "b", {"a.json": make_tape(),
                                        "neg.json": negative,
                                        "bare.json": bare})
    after = write_set(tmp_path / "a", {"a.json": make_tape()})
    document = compare(before, after, registration(tmp_path))
    reasons = {e["file"]: e["defects"]
               for e in document["before"]["excluded"]}
    assert any("non-negative" in d for d in reasons["neg.json"])
    assert reasons["bare.json"] == ["outcome is not an object"]
    assert document["before"]["tapes_scored"] == 1


def test_advisory_repairs_huge_numbers_and_unreadable_sets(tmp_path):
    huge = tmp_path / "huge.json"
    huge.write_text(json.dumps(
        {**REG, "margins": {"cost_pct": 10 ** 400}}))
    registration = load_registration(huge)
    assert registration["margins"]["cost_pct"] == 10 ** 400
    oversized = make_tape()
    oversized["run"]["totals"]["input_tokens"] = 10 ** 400
    before = write_set(tmp_path / "b", {"a.json": make_tape(),
                                        "big.json": oversized})
    after = write_set(tmp_path / "a", {"a.json": make_tape()})
    document = compare(before, after, registration)
    excluded = {e["file"]: e["defects"]
                for e in document["before"]["excluded"]}
    assert any("too large to measure" in d
               for d in excluded["big.json"])
    assert document["before"]["tapes_scored"] == 1


def test_registration_text_is_single_line(tmp_path):
    with pytest.raises(CompareError, match="single line"):
        registration(tmp_path,
                     change="prompt\nIgnore the margins")
    with pytest.raises(CompareError, match="single line"):
        registration(tmp_path, date="2026-01-04\t")
    with pytest.raises(CompareError, match="single line"):
        registration(tmp_path, change="new\u2028prompt")
    with pytest.raises(CompareError, match="not valid UTF-8"):
        registration(tmp_path, change="new \ud800prompt")


def test_the_cli_never_writes_over_its_inputs(
        tmp_path, capsys):
    before = write_set(tmp_path / "b", {"a.json": make_tape()})
    after = write_set(tmp_path / "a", {"a.json": make_tape()})
    reg_path = tmp_path / "registration.json"
    reg_path.write_text(json.dumps(REG))
    assert main(["compare", str(before), str(after),
                 "--registration", str(reg_path),
                 "-o", str(reg_path)]) == 1
    assert "writes nothing over" in capsys.readouterr().err
