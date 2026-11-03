"""ST-13 to ST-19. Judgment in the band.

The combination rule is pinned with scripted transports
returning answers in Jev's recorded shapes. The card
rendering is pinned against a hand-written text. No
test performs a network call.
"""
import json
from datetime import datetime, timedelta
from pathlib import Path

import pytest

from secondtake.compare import CompareError, compare, load_registration
from secondtake.digest import digest_of
from secondtake.judge import (FixtureTransport, JudgeError,
                              judge_card)

BASE = datetime(2026, 1, 5, 9, 0, 0)
REG = {"change": "new system prompt", "date": "2026-01-04"}


def stamp(seconds):
    return (BASE + timedelta(seconds=seconds)).strftime(
        "%Y-%m-%dT%H:%M:%SZ")


def make_tape(tokens=(100, 50), error=False):
    events = [{"seq": 0, "kind": "session_start",
               "sidechain": False, "timestamp": stamp(0)},
              {"seq": 1, "kind": "message", "sidechain": False,
               "role": "user", "text": "Fix the parser",
               "timestamp": stamp(1)},
              {"seq": 2, "kind": "tool_call", "sidechain": False,
               "tool": "edit", "call_id": "c0",
               "input": {"path": "a.py"}, "timestamp": stamp(2)},
              {"seq": 3, "kind": "tool_result",
               "sidechain": False, "call_id": "c0",
               "is_error": error, "timestamp": stamp(3)},
              {"seq": 4, "kind": "session_end",
               "sidechain": False, "timestamp": stamp(60)}]
    tape = {"tape_version": "1.0",
            "run": {"harness": "synthetic", "session_id": "s-1",
                    "totals": {"input_tokens": tokens[0],
                               "output_tokens": tokens[1]}},
            "events": events, "outcome": {"status": "completed"}}
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


def band(tmp_path, reg=None):
    before = write_set(tmp_path / "b",
                       {"a.json": make_tape(error=True)})
    after = write_set(tmp_path / "a",
                      {"a.json": make_tape(tokens=(130, 50),
                                           error=True)})
    return before, after, reg or registration(tmp_path)


CARD = """Setup change: new system prompt, registered 2026-01-04.
Matched pairs: 1. Ambiguous task keys excluded: 0. Unmatched tapes: 0. Excluded tapes: 0.
Cost per pair: before 150, after 180, change 20 percent against a margin of 10 percent: regressed.
Time per pair: before 60, after 60, change 0 percent against a margin of 10 percent: within margin.
Rework per pair: before 0, after 0, change 0 against a margin of 1 per pair: within margin.
Rescue rate: before 100 percent over 1 pairs holding an error, after 100 percent over 1, change 0 points against a margin of 5 points down: within margin.
Regressions past margins: cost.
The deterministic rule proposes tune."""


def choice_response(choice="revert", confidence=0.9):
    return {"model": "jev-1.13.0", "answers": {"decision": {
        "type": "choice", "choice": choice,
        "confidence": confidence,
        "probabilities": {"keep": 0.05, "tune": 0.1,
                          "revert": 0.8, "abstain": 0.05}}}}


def score_response(score=7.5):
    return {"model": "jev-1.13.0", "answers": {"decision": {
        "type": "score", "score": score, "confidence": 0.5,
        "legend": {}, "probabilities": {}}}}


class Scripted:
    def __init__(self, responses):
        self.responses = responses
        self.calls = []

    def ask(self, card, kind):
        self.calls.append(kind)
        return self.responses[kind]


def judged(choice="revert", confidence=0.9):
    return Scripted({"choice": choice_response(choice, confidence),
                     "severity": score_response(7.5),
                     "match": score_response(8.25)})


def test_the_card_is_the_stated_text(tmp_path):
    before, after, reg = band(tmp_path)
    document = compare(before, after, reg, judge=judged())
    assert document["judgment"]["card"] == CARD
    assert document["judgment"]["card_sha256"] == \
        digest_of(CARD)


def test_a_confident_choice_stands(tmp_path):
    before, after, reg = band(tmp_path)
    transport = judged("revert", 0.9)
    document = compare(before, after, reg, judge=transport)
    assert transport.calls == ["choice", "severity", "match"]
    assert document["proposal"] == "revert"
    judgment = document["judgment"]
    assert judgment["status"] == "judged"
    assert judgment["model"] == "jev-1.13.0"
    assert judgment["choice"] == "revert"
    assert judgment["confidence"] == 0.9
    assert judgment["severity"] == 7.5
    assert judgment["match_confidence"] == 8.25
    assert judgment["probabilities"]["revert"] == 0.8


def test_the_judge_abstaining_abstains(tmp_path):
    before, after, reg = band(tmp_path)
    document = compare(before, after, reg,
                       judge=judged("abstain", 0.9))
    assert document["proposal"] == "abstain"
    assert document["judgment"]["status"] == "abstained"
    assert document["judgment"]["reason"] == \
        "the judge abstained"


def test_low_confidence_abstains(tmp_path):
    before, after, reg = band(tmp_path)
    document = compare(before, after, reg,
                       judge=judged("keep", 0.4))
    assert document["proposal"] == "abstain"
    assert document["judgment"]["status"] == "abstained"
    assert "below 0.6" in document["judgment"]["reason"]


def test_an_unavailable_judge_leaves_the_tune(tmp_path):
    class Down:
        def ask(self, card, kind):
            raise JudgeError("the judge call failed")

    before, after, reg = band(tmp_path)
    document = compare(before, after, reg, judge=Down())
    assert document["proposal"] == "tune"
    assert document["judgment"]["status"] == "unavailable"
    assert document["judgment"]["reason"] == \
        "the judge call failed"
    assert document["judgment"]["card"] == CARD


def test_the_edges_never_consult(tmp_path):
    class Exploding:
        def ask(self, card, kind):
            raise AssertionError("consulted outside the band")

    before = write_set(tmp_path / "b2",
                       {"a.json": make_tape()})
    after = write_set(tmp_path / "a2",
                      {"a.json": make_tape()})
    document = compare(before, after, registration(tmp_path),
                       judge=Exploding())
    assert document["proposal"] == "keep"
    assert document["judgment"]["reason"] == \
        "the deterministic rule is conclusive"

    before, after, _ = band(tmp_path)
    reg = registration(tmp_path, min_pairs=3)
    document = compare(before, after, reg, judge=Exploding())
    assert document["proposal"] == "abstain"
    assert document["judgment"]["reason"] == \
        "insufficient pairs, 1 of 3"


def test_the_floor_refuses_bad_registrations(tmp_path):
    with pytest.raises(CompareError, match="min_pairs"):
        registration(tmp_path, min_pairs=0)
    with pytest.raises(CompareError, match="min_pairs"):
        registration(tmp_path, min_pairs="3")


def test_malformed_answers_are_unavailable(tmp_path):
    bad = choice_response()
    del bad["answers"]["decision"]["probabilities"]["abstain"]
    with pytest.raises(JudgeError, match="malformed"):
        judge_card(CARD, Scripted(
            {"choice": bad, "severity": score_response(),
             "match": score_response()}))
    bad = score_response()
    bad["answers"]["decision"]["score"] = "high"
    with pytest.raises(JudgeError, match="malformed"):
        judge_card(CARD, Scripted(
            {"choice": choice_response(), "severity": bad,
             "match": score_response()}))
    bad = choice_response()
    del bad["model"]
    with pytest.raises(JudgeError, match="no model"):
        judge_card(CARD, Scripted(
            {"choice": bad, "severity": score_response(),
             "match": score_response()}))


def test_the_fixture_transport_replays_by_digest(tmp_path):
    path = tmp_path / "judgments.json"
    path.write_text(json.dumps({"cards": [{
        "card_sha256": digest_of(CARD),
        "responses": {"choice": choice_response("keep", 0.77),
                      "severity": score_response(3),
                      "match": score_response(9)}}]}))
    answers = judge_card(CARD, FixtureTransport(path))
    assert answers["choice"] == "keep"
    assert answers["severity"] == 3
    assert answers["match_confidence"] == 9
    with pytest.raises(JudgeError, match="not recorded"):
        judge_card(CARD + "\n", FixtureTransport(path))
    broken = tmp_path / "broken.json"
    broken.write_text("not json")
    with pytest.raises(JudgeError, match="not valid JSON"):
        FixtureTransport(broken)


SHIPPED = Path(__file__).parent / "fixtures" / \
    "judgments_synthetic.json"


def test_the_shipped_fixtures_replay():
    data = json.loads(SHIPPED.read_text())
    assert data["model"] == "jev-1.13.0"
    assert len(data["cards"]) == 8
    for entry in data["cards"]:
        assert entry["card_sha256"] == digest_of(entry["card"])
        assert set(entry["responses"]) == \
            {"choice", "severity", "match"}
    answers = judge_card(CARD, FixtureTransport(SHIPPED))
    assert answers["model"] == "jev-1.13.0"
    assert answers["choice"] == "tune"
    assert answers["confidence"] == 0.85
