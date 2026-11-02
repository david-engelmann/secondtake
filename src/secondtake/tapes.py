"""Reading flighttape tapes (ST-5a).

Secondtake reads tapes at flighttape format version
1.0 only, never raw harness session files. The checks
here are the format rules the comparison stands on,
the same subset the family reads everywhere: the value
is an object carrying tape_version 1.0, a run object,
and an events array whose seq values run consecutively
from 0 and whose kinds are in the format's enum, and
the tool call shapes the flighttape validator enforces
under its FT-4 pairing rule. A file that fails any
check is excluded with its defects named, and the
comparison proceeds on the tapes that remain.
"""

from __future__ import annotations

import json

EVENT_KINDS = {
    "session_start", "message", "tool_call", "tool_result",
    "diff", "spend", "note", "session_end",
}


class _Failed:
    def __repr__(self):
        return "FAILED"


FAILED = _Failed()


def load_tape_file(path):
    """Read one tape file.

    Returns (tape, None, []) on success, or (FAILED, reason,
    defects) on failure, where defects carries the format
    defects when the file parsed but is not a valid tape.
    """
    try:
        raw = path.read_bytes()
    except OSError:
        return FAILED, "cannot read the tape file", []
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError:
        return FAILED, "the file is not valid UTF-8", []
    try:
        tape = json.loads(text)
    except RecursionError:
        return FAILED, "the file is not valid JSON", []
    except ValueError as error:
        if "integer string conversion" in str(error):
            return FAILED, "the file holds a number too large to read", []
        return FAILED, "the file is not valid JSON", []
    defects = tape_defects(tape)
    if defects:
        return FAILED, "not a valid tape", defects
    return tape, None, []


def tape_defects(tape):
    """The format checks the tool's evidence stands on."""
    if not isinstance(tape, dict):
        return ["the tape is not an object"]
    defects = []
    if tape.get("tape_version") != "1.0":
        defects.append("tape_version is not 1.0")
    if not isinstance(tape.get("run"), dict):
        defects.append("run is not an object")
    events = tape.get("events")
    if not isinstance(events, list) or not events:
        defects.append("events must hold at least one event")
        return defects
    seen_calls = set()
    for index, event in enumerate(events):
        where = f"events[{index}]"
        if not isinstance(event, dict):
            defects.append(f"{where} is not an object")
            continue
        seq = event.get("seq")
        if not isinstance(seq, int) or isinstance(seq, bool) \
                or seq != index:
            defects.append(f"{where}.seq is not {index}")
        kind = event.get("kind")
        if not isinstance(kind, str) or kind not in EVENT_KINDS:
            defects.append(f"{where}.kind is not in the format enum")
            continue
        call_id = event.get("call_id")
        if kind == "tool_call":
            if not isinstance(event.get("tool"), str) \
                    or not event.get("tool"):
                defects.append(
                    f"{where}.tool must be a non-empty string")
            if not isinstance(call_id, str) or not call_id:
                defects.append(
                    f"{where}.call_id must be a non-empty string")
            elif call_id in seen_calls:
                defects.append(
                    f"{where}.call_id {call_id!r} is used twice")
            else:
                seen_calls.add(call_id)
        if kind == "tool_result":
            if not isinstance(call_id, str) \
                    or call_id not in seen_calls:
                defects.append(
                    f"{where}.call_id {call_id!r} answers no "
                    f"earlier tool_call")
            if "is_error" in event \
                    and not isinstance(event["is_error"], bool):
                defects.append(f"{where}.is_error must be a boolean")
    return defects


OUTCOME_STATUSES = {"completed", "failed", "aborted", "unknown"}


def measure_shape_defects(tape):
    """ST-5b. The shapes secondtake's own measures stand
    on, beyond the family subset above: totals values the
    format bounds at zero, and an outcome object whose
    status is in the format enum. A tape breaking either
    is excluded with the defect named, never crashed on."""
    defects = []
    run = tape.get("run")
    if isinstance(run, dict) and "totals" in run:
        totals = run["totals"]
        if not isinstance(totals, dict):
            defects.append("run.totals is not an object")
        else:
            for key, value in totals.items():
                if (not isinstance(value, int)
                        or isinstance(value, bool) or value < 0):
                    defects.append(
                        f"run.totals.{key} is not a non-negative "
                        f"integer")
                elif value > 10 ** 18:
                    defects.append(
                        f"run.totals.{key} is too large to "
                        f"measure")
    if "outcome" in tape:
        outcome = tape["outcome"]
        if not isinstance(outcome, dict):
            defects.append("outcome is not an object")
        elif outcome.get("status") not in OUTCOME_STATUSES:
            defects.append("outcome.status is not in the "
                           "format enum")
    return defects
