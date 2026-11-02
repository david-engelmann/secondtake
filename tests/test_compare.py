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
                       "content": task, "timestamp": stamp(1)})
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
    assert document["proposal"] == "keep"


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
