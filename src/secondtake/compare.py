"""ST-5 to ST-9. The matched comparison."""
from __future__ import annotations

import json
from pathlib import Path

from .digest import DigestError, digest_of
from .measures import MATCHING_RULE, tape_measures, task_key
from .tapes import FAILED, load_tape_file

DEFAULT_MARGINS = {"cost_pct": 10, "time_pct": 10,
                   "rework_per_pair": 1, "rescue_points": 5}


class CompareError(ValueError):
    """A named refusal (family form, ST-12)."""


def load_registration(path: Path) -> dict:
    try:
        raw = path.read_bytes()
    except OSError:
        raise CompareError(
            f"no registration at {path}; a comparison is "
            f"refused without one") from None
    try:
        data = json.loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, ValueError, RecursionError):
        raise CompareError(
            f"the registration at {path} is not valid JSON") from None
    if not isinstance(data, dict):
        raise CompareError("the registration is not an object")
    change, date = data.get("change"), data.get("date")
    if not isinstance(change, str) or not change:
        raise CompareError(
            "the registration names no setup change")
    if not isinstance(date, str) or not date:
        raise CompareError("the registration names no date")
    margins = dict(DEFAULT_MARGINS)
    stated = data.get("margins")
    if stated is not None:
        if not isinstance(stated, dict):
            raise CompareError("the registration margins "
                               "are not an object")
        for key, value in stated.items():
            if key not in DEFAULT_MARGINS:
                raise CompareError(
                    f"the registration states an unknown "
                    f"margin, {key}")
            if (not isinstance(value, (int, float))
                    or isinstance(value, bool) or value < 0):
                raise CompareError(
                    f"the margin {key} is not a non-negative "
                    f"number")
            margins[key] = value
    return {"change": change, "date": date, "margins": margins}


def _load_set(directory: Path, side: str):
    if not directory.is_dir():
        raise CompareError(
            f"the {side} set at {directory} is not a directory")
    tapes = []
    excluded = []
    for name in sorted(p.name for p in directory.iterdir()):
        path = directory / name
        if not path.is_file():
            continue
        tape, reason, defects = load_tape_file(path)
        if tape is FAILED:
            excluded.append({"file": name, "reason": reason,
                             "defects": defects})
            continue
        try:
            digest_of(tape)
        except (DigestError, RecursionError):
            excluded.append({
                "file": name,
                "reason": "cannot be cited under canonical form 1",
                "defects": []})
            continue
        tapes.append({"file": name, "tape": tape})
    return tapes, excluded


def _index(tapes):
    by_key: dict[str, list[dict]] = {}
    unmatchable = []
    for entry in tapes:
        key = task_key(entry["tape"])
        if key is None:
            unmatchable.append({"file": entry["file"],
                                "reason": "no main-chain user "
                                          "message with a text"})
            continue
        entry = {**entry, "key": key,
                 "measures": tape_measures(entry["tape"])}
        by_key.setdefault(key, []).append(entry)
    return by_key, unmatchable


def _mean(values):
    return sum(values) / len(values) if values else None


def _pair_measure(pairs, name):
    counted = [(p["before"][name], p["after"][name])
               for p in pairs
               if p["before"][name] is not None
               and p["after"][name] is not None]
    before = _mean([b for b, _ in counted])
    after = _mean([a for _, a in counted])
    return before, after, len(counted)


def compare(before_dir: Path, after_dir: Path,
            registration: dict) -> dict:
    before_tapes, before_excluded = _load_set(before_dir, "before")
    after_tapes, after_excluded = _load_set(after_dir, "after")
    before_by_key, before_unmatchable = _index(before_tapes)
    after_by_key, after_unmatchable = _index(after_tapes)
    pairs = []
    ambiguous = []
    unmatched_before, unmatched_after = [], []
    for key in sorted(set(before_by_key) | set(after_by_key)):
        left = before_by_key.get(key, [])
        right = after_by_key.get(key, [])
        if len(left) == 1 and len(right) == 1:
            pairs.append({"key": key,
                          "before": left[0]["measures"],
                          "after": right[0]["measures"],
                          "before_file": left[0]["file"],
                          "after_file": right[0]["file"]})
        elif left and right:
            ambiguous.append({"key": key, "before": len(left),
                              "after": len(right)})
        elif left:
            unmatched_before.extend(
                {"file": e["file"], "key": key} for e in left)
        else:
            unmatched_after.extend(
                {"file": e["file"], "key": key} for e in right)
    for pair in pairs:
        pair["deltas"] = {
            name: (pair["after"][name] - pair["before"][name]
                   if pair["before"][name] is not None
                   and pair["after"][name] is not None else None)
            for name in ("cost", "time_seconds", "rework")}
    margins = registration["margins"]
    regressions = []
    aggregates: dict = {"pairs": len(pairs)}
    for name, margin_key, kind in (
            ("cost", "cost_pct", "relative"),
            ("time_seconds", "time_pct", "relative"),
            ("rework", "rework_per_pair", "absolute")):
        before_mean, after_mean, counted = _pair_measure(pairs, name)
        entry = {"before_mean": before_mean,
                 "after_mean": after_mean,
                 "pairs_counted": counted}
        if before_mean is not None and after_mean is not None:
            if kind == "relative":
                entry["delta_pct"] = (
                    (after_mean - before_mean) / before_mean * 100
                    if before_mean else None)
                regressed = (
                    after_mean > 0 if before_mean == 0 else
                    (after_mean - before_mean)
                    / before_mean * 100 > margins[margin_key])
            else:
                entry["delta"] = after_mean - before_mean
                regressed = entry["delta"] > margins[margin_key]
            if regressed:
                regressions.append(
                    "time" if name == "time_seconds" else name)
        aggregates["time" if name == "time_seconds" else name] = entry
    def rate(side):
        holders = [p for p in pairs if p[side]["has_error"]]
        if not holders:
            return None, 0
        rescued = sum(1 for p in holders if p[side]["rescued"])
        return rescued / len(holders) * 100, len(holders)
    before_rate, before_den = rate("before")
    after_rate, after_den = rate("after")
    rescue = {"before_rate": before_rate, "after_rate": after_rate,
              "before_denominator": before_den,
              "after_denominator": after_den}
    if before_rate is not None and after_rate is not None:
        rescue["delta_points"] = after_rate - before_rate
        if before_rate - after_rate > margins["rescue_points"]:
            regressions.append("rescue rate")
    aggregates["rescue"] = rescue
    proposal = ("revert" if len(regressions) >= 2 else
                "tune" if regressions else "keep")
    return {
        "registration": registration,
        "matching_rule": MATCHING_RULE,
        "before": {"tapes_read": len(before_tapes) + len(before_excluded),
                   "tapes_scored": len(before_tapes),
                   "excluded": before_excluded},
        "after": {"tapes_read": len(after_tapes) + len(after_excluded),
                  "tapes_scored": len(after_tapes),
                  "excluded": after_excluded},
        "pairs": pairs,
        "aggregates": aggregates,
        "unmatched": {"before": unmatched_before,
                      "after": unmatched_after},
        "ambiguous": ambiguous,
        "unmatchable": (
            [{"side": "before", **u} for u in before_unmatchable]
            + [{"side": "after", **u} for u in after_unmatchable]),
        "regressions": regressions,
        "proposal": proposal,
    }
