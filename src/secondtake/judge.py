"""ST-13 to ST-19. The decision card, the judge's three
calls, and the two transports that answer them."""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

from .digest import digest_of

MODEL = "jev-1.13.0"
LABELS = ("keep", "tune", "revert", "abstain")
CONFIDENCE_FLOOR = 0.6

FRAMING = {
    "severity": (
        "\n\nRate the severity of the tradeoff between "
        "the measures that improved and the measure that "
        "regressed, on the scale given."),
    "match": (
        "\n\nRate the confidence that the matched pairs "
        "compare like with like, given the ambiguous, "
        "unmatched, and excluded counts, on the scale "
        "given."),
}


class JudgeError(ValueError):
    """A named refusal, or a judge that cannot answer."""


def _num(value) -> str:
    return f"{value:.2f}".rstrip("0").rstrip(".")


def _verdict(delta, regressed: bool) -> str:
    if regressed:
        return "regressed"
    if delta is not None and delta != 0:
        return "improved" if delta < 0 else "within margin"
    return "within margin"


def build_card(registration: dict, aggregates: dict,
               regressions: list, proposal: str,
               counts: dict) -> str:
    """The card text, deterministic in the comparison
    (ST-13). Rescue improvement runs the other way from
    the cost measures, so its verdict is stated from
    its own delta's sign."""
    margins = registration["margins"]
    lines = [
        f"Setup change: {registration['change']}, "
        f"registered {registration['date']}.",
        f"Matched pairs: {aggregates['pairs']}. "
        f"Ambiguous task keys excluded: "
        f"{counts['ambiguous']}. Unmatched tapes: "
        f"{counts['unmatched']}. Excluded tapes: "
        f"{counts['excluded']}.",
    ]
    for key, label, margin_key, unit in (
            ("cost", "Cost per pair", "cost_pct", "percent"),
            ("time", "Time per pair", "time_pct", "percent")):
        entry = aggregates[key]
        name = "time" if key == "time" else key
        if entry["pairs_counted"] == 0:
            lines.append(f"{label}: unmeasured, no pair "
                         "records it on both sides.")
        else:
            delta = entry["delta_pct"]
            lines.append(
                f"{label}: before {_num(entry['before_mean'])}, "
                f"after {_num(entry['after_mean'])}, change "
                f"{_num(delta) if delta is not None else 'n/a'} "
                f"{unit} against a margin of "
                f"{_num(margins[margin_key])} percent: "
                f"{_verdict(delta, name in regressions)}.")
    entry = aggregates["rework"]
    if entry["pairs_counted"] == 0:
        lines.append("Rework per pair: unmeasured, no pair "
                     "records it on both sides.")
    else:
        delta = entry["delta"]
        lines.append(
            f"Rework per pair: before "
            f"{_num(entry['before_mean'])}, after "
            f"{_num(entry['after_mean'])}, change "
            f"{_num(delta)} against a margin of "
            f"{_num(margins['rework_per_pair'])} per pair: "
            f"{_verdict(delta, 'rework' in regressions)}.")
    rescue = aggregates["rescue"]
    if rescue["before_rate"] is None \
            or rescue["after_rate"] is None:
        lines.append("Rescue rate: unmeasured, a side holds "
                     "no pair with an error.")
    else:
        delta = rescue["delta_points"]
        verdict = ("regressed" if "rescue rate" in regressions
                   else "improved" if delta > 0
                   else "within margin")
        lines.append(
            f"Rescue rate: before "
            f"{_num(rescue['before_rate'])} percent over "
            f"{rescue['before_denominator']} pairs holding "
            f"an error, after {_num(rescue['after_rate'])} "
            f"percent over {rescue['after_denominator']}, "
            f"change {_num(delta)} points against a margin "
            f"of {_num(margins['rescue_points'])} points "
            f"down: {verdict}.")
    lines.append(
        "Regressions past margins: "
        + (", ".join(regressions) if regressions else "none")
        + ".")
    lines.append(f"The deterministic rule proposes "
                 f"{proposal}.")
    return "\n".join(lines)


def _answer(response, kind: str) -> dict:
    if not isinstance(response, dict):
        raise JudgeError("the judge's answer is not an object")
    answers = response.get("answers")
    if not isinstance(answers, dict):
        raise JudgeError("the judge's answer carries no answers")
    decision = answers.get("decision")
    if not isinstance(decision, dict):
        raise JudgeError("the judge's answer carries no decision")
    return decision


def _number(value) -> bool:
    return isinstance(value, (int, float)) \
        and not isinstance(value, bool)


def judge_card(card: str, transport) -> dict:
    """The three calls and their parsed answers (ST-16).
    Any malformed answer is unavailable, never repaired
    in flight (ST-19)."""
    choice_response = transport.ask(card, "choice")
    model = choice_response.get("model") \
        if isinstance(choice_response, dict) else None
    if not isinstance(model, str) or not model:
        raise JudgeError("the judge's answer names no model")
    decision = _answer(choice_response, "choice")
    choice = decision.get("choice")
    confidence = decision.get("confidence")
    probabilities = decision.get("probabilities")
    if decision.get("type") != "choice" \
            or choice not in LABELS \
            or not _number(confidence) \
            or not isinstance(probabilities, dict) \
            or any(label not in probabilities
                   or not _number(probabilities[label])
                   for label in LABELS):
        raise JudgeError("the judge's choice answer is "
                         "malformed")
    scores = {}
    for kind in ("severity", "match"):
        decision = _answer(transport.ask(card, kind), kind)
        if decision.get("type") != "score" \
                or not _number(decision.get("score")):
            raise JudgeError(f"the judge's {kind} answer is "
                             "malformed")
        scores[kind] = decision["score"]
    return {"model": model, "choice": choice,
            "confidence": confidence,
            "probabilities": probabilities,
            "severity": scores["severity"],
            "match_confidence": scores["match"]}


class FixtureTransport:
    """Replays recorded responses, keyed by the card's
    digest (ST-17, ST-19)."""

    def __init__(self, path: Path):
        try:
            raw = Path(path).read_bytes()
        except OSError:
            raise JudgeError(
                f"no judgments file at {path}") from None
        try:
            data = json.loads(raw.decode("utf-8"))
        except (UnicodeDecodeError, ValueError,
                RecursionError):
            raise JudgeError(
                f"the judgments file at {path} is not valid "
                "JSON") from None
        cards = data.get("cards") if isinstance(data, dict) \
            else None
        if not isinstance(cards, list):
            raise JudgeError(
                "the judgments file carries no cards list")
        self._by_digest = {}
        for entry in cards:
            if isinstance(entry, dict) \
                    and isinstance(entry.get("card_sha256"), str) \
                    and isinstance(entry.get("responses"), dict):
                self._by_digest[entry["card_sha256"]] = \
                    entry["responses"]

    def ask(self, card: str, kind: str) -> dict:
        responses = self._by_digest.get(digest_of(card))
        if responses is None:
            raise JudgeError(
                "the card is not recorded in the judgments "
                "file")
        response = responses.get(kind)
        if not isinstance(response, dict):
            raise JudgeError(
                f"the recorded {kind} answer is missing")
        return response


class LiveTransport:
    """Calls Jev through the connector's call script
    (ST-17)."""

    def __init__(self, script: str):
        self._script = script

    def ask(self, card: str, kind: str) -> dict:
        state = card + FRAMING.get(kind, "")
        command = [sys.executable, self._script,
                   "--state", state,
                   "--labels", ",".join(LABELS)]
        if kind != "choice":
            command += ["--question-type", "score"]
        try:
            result = subprocess.run(
                command, capture_output=True, text=True,
                timeout=180)
        except (OSError, subprocess.TimeoutExpired):
            raise JudgeError(
                "the judge call could not be made") from None
        if result.returncode != 0:
            raise JudgeError("the judge call failed")
        try:
            return json.loads(result.stdout)
        except ValueError:
            raise JudgeError(
                "the judge's answer is not JSON") from None
