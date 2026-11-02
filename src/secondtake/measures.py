"""ST-7, ST-7a. The four measures of one tape."""
from __future__ import annotations

from datetime import datetime, timezone

from .digest import DigestError, digest_of
from .moments import redo_moments

MATCHING_RULE = (
    "tapes match by task key: the canonical form 1 digest "
    "of the text of the first main-chain user message")


def task_key(tape) -> str | None:
    """The ST-6a task key, or None when unmatchable."""
    for event in tape["events"]:
        if (event.get("kind") == "message"
                and event.get("role") == "user"
                and event.get("sidechain") is False):
            text = event.get("text")
            if not isinstance(text, str):
                return None
            try:
                return digest_of(text)
            except DigestError:
                return None
    return None


def _parse_timestamp(value):
    """Flighttape's parse_timestamp, ported exactly:
    naive stamps are read as UTC (FT-32)."""
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed


def _duration_seconds(tape):
    """FT-32: first to last parseable event timestamp."""
    stamps = []
    for event in tape["events"]:
        stamp = event.get("timestamp")
        if isinstance(stamp, str):
            parsed = _parse_timestamp(stamp)
            if parsed is not None:
                stamps.append(parsed)
    if len(stamps) < 2:
        return None
    return (stamps[-1] - stamps[0]).total_seconds()


def tape_measures(tape) -> dict:
    totals = (tape.get("run") or {}).get("totals") or {}
    cost = None
    inputs, outputs = totals.get("input_tokens"), totals.get("output_tokens")
    if (isinstance(inputs, int) and not isinstance(inputs, bool)
            and isinstance(outputs, int)
            and not isinstance(outputs, bool)):
        cost = inputs + outputs
    has_error = any(
        event.get("kind") == "tool_result"
        and event.get("is_error") is True
        for event in tape["events"])
    outcome = (tape.get("outcome") or {}).get("status")
    return {
        "cost": cost,
        "time_seconds": _duration_seconds(tape),
        "rework": len(redo_moments(tape)),
        "rescued": bool(has_error and outcome == "completed"),
        "has_error": has_error,
    }
