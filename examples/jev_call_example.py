#!/usr/bin/env python3
"""Reference JEV_CALL connector script for secondtake `--judge live`.

Protocol summary (matching `src/judge.rs`):
- Invoked by `secondtake` via `python3 $JEV_CALL` up to three times when a
  comparison lands in the single-regression tradeoff band (`regressions.len() == 1`
  and `pairs >= min_pairs`):
    1. Choice query:
       python3 $JEV_CALL --state <card> --labels keep,tune,revert,abstain
    2. Severity score query:
       python3 $JEV_CALL --state <card + severity_framing> \
         --labels keep,tune,revert,abstain --question-type score
    3. Match-confidence score query:
       python3 $JEV_CALL --state <card + match_framing> \
         --labels keep,tune,revert,abstain --question-type score
- Each subprocess call has a 180-second wall-clock deadline.
- Both stdout and stderr must be valid UTF-8, and the process must exit 0.
- Stdout must be a single JSON object carrying `"model": "jev-1.13.0"` and
  `"answers": {"decision": ...}`.

If `JEV_ENDPOINT` is set in the environment, this script forwards the request
over HTTPS POST (with optional `JEV_API_KEY` Bearer authentication) and prints
the `jev-1.13.0` response. When `JEV_ENDPOINT` is unset, it emits a valid
deterministic template response so operators can test `--judge live` locally.
"""

import argparse
import json
import os
import sys
import urllib.request

PINNED_MODEL = "jev-1.13.0"
REQUIRED_LABELS = ["keep", "tune", "revert", "abstain"]


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="JEV_CALL connector adapter for secondtake --judge live."
    )
    parser.add_argument(
        "--state",
        required=True,
        help="Decision card text (with optional score framing appended).",
    )
    parser.add_argument(
        "--labels",
        required=True,
        help="Comma-separated choice labels (keep,tune,revert,abstain).",
    )
    parser.add_argument(
        "--question-type",
        choices=["choice", "score"],
        default="choice",
        help="Question kind requested by secondtake (default: choice).",
    )
    return parser.parse_args(argv)


def call_remote(endpoint: str, state: str, labels: list[str], question_type: str) -> dict:
    payload = json.dumps(
        {
            "model": PINNED_MODEL,
            "state": state,
            "labels": labels,
            "question_type": question_type,
        }
    ).encode("utf-8")
    headers = {"Content-Type": "application/json", "Accept": "application/json"}
    api_key = os.environ.get("JEV_API_KEY")
    if api_key:
        headers["Authorization"] = f"Bearer {api_key}"
    request = urllib.request.Request(endpoint, data=payload, headers=headers, method="POST")
    with urllib.request.urlopen(request, timeout=170) as response:
        body = response.read().decode("utf-8")
    return json.loads(body)


def template_choice_response(state: str) -> dict:
    improved = state.count(": improved.")
    if improved >= 2:
        probs = {"keep": 0.10, "tune": 0.75, "revert": 0.10, "abstain": 0.05}
        choice = "tune"
        confidence = 0.75
    else:
        probs = {"keep": 0.05, "tune": 0.80, "revert": 0.10, "abstain": 0.05}
        choice = "tune"
        confidence = 0.80
    return {
        "model": PINNED_MODEL,
        "answers": {
            "decision": {
                "type": "choice",
                "choice": choice,
                "confidence": confidence,
                "probabilities": probs,
            }
        },
        "usage": {
            "input_tokens": len(state.split()),
            "output_tokens": 32,
        },
    }


def template_score_response(state: str) -> dict:
    is_match = "compare like with like" in state
    score = 8.0 if is_match else 4.5
    legend = {str(i): f"Level {i + 1}" for i in range(10)}
    probabilities = {str(i): 0.1 for i in range(10)}
    return {
        "model": PINNED_MODEL,
        "answers": {
            "decision": {
                "type": "score",
                "score": score,
                "confidence": 0.7,
                "legend": legend,
                "probabilities": probabilities,
            }
        },
        "usage": {
            "input_tokens": len(state.split()),
            "output_tokens": 24,
        },
    }


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    labels = [item.strip() for item in args.labels.split(",") if item.strip()]
    if labels != REQUIRED_LABELS:
        print(f"unexpected labels: {labels!r}", file=sys.stderr)
        return 2

    endpoint = os.environ.get("JEV_ENDPOINT", "").strip()
    if endpoint:
        result = call_remote(endpoint, args.state, labels, args.question_type)
    elif args.question_type == "score":
        result = template_score_response(args.state)
    else:
        result = template_choice_response(args.state)

    json.dump(result, sys.stdout)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
