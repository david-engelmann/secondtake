#!/usr/bin/env python3
"""Reference JEV_CALL connector for `secondtake compare --judge live`.

Matches `LiveTransport` in `src/judge.rs`:
- Invoked three times per single-regression comparison (`choice`, `severity`, `match`).
- Receives `--state <plain-text card>` and `--labels keep,tune,revert,abstain`,
  plus `--question-type score` on the `severity` and `match` calls.
- Writes a single JSON object to stdout carrying `"model": "jev-1.13.0"` and
  `"answers": {"decision": ...}`.

Modes:
1. Offline deterministic heuristic (default when JEV_ENDPOINT is unset):
   Answers locally so `--judge live` can be verified without network access.
2. Live HTTPS JSON endpoint (when JEV_ENDPOINT is set):
   POSTs `{state, labels, question_type}` over HTTPS with optional JEV_API_KEY
   bearer auth, refusing plain-HTTP egress and cross-origin redirects.
"""

import argparse
import json
import os
import sys
import urllib.error
import urllib.parse
import urllib.request

MODEL = "jev-1.13.0"


class _SameOriginHTTPSRedirectHandler(urllib.request.HTTPRedirectHandler):
    def __init__(self, allowed_netloc: str):
        super().__init__()
        self._allowed_netloc = allowed_netloc

    def redirect_request(self, req, fp, code, msg, headers, newurl):
        parsed = urllib.parse.urlparse(newurl)
        if parsed.scheme != "https" or parsed.netloc != self._allowed_netloc:
            raise urllib.error.HTTPError(
                newurl, code, "refusing cross-origin or non-HTTPS redirect", headers, fp
            )
        return super().redirect_request(req, fp, code, msg, headers, newurl)


def offline_heuristic(state: str, labels: list[str], question_type: str) -> dict:
    if question_type == "score":
        score = 4.0 if "severe" in state.lower() else 7.5
        return {
            "model": MODEL,
            "answers": {
                "decision": {
                    "type": "score",
                    "score": score,
                }
            },
        }
    probs = {label: 0.0 for label in labels}
    choice = "tune" if "tune" in probs else labels[0]
    if set(labels) == {"keep", "tune", "revert", "abstain"}:
        probs = {"keep": 0.15, "tune": 0.65, "revert": 0.15, "abstain": 0.05}
    else:
        share = round(1.0 / max(len(labels), 1), 4)
        probs = {label: share for label in labels}
    return {
        "model": MODEL,
        "answers": {
            "decision": {
                "type": "choice",
                "choice": choice,
                "confidence": 0.72,
                "probabilities": probs,
            }
        },
    }


def call_https_endpoint(endpoint: str, state: str, labels: list[str], question_type: str) -> dict:
    parsed = urllib.parse.urlparse(endpoint)
    if parsed.scheme != "https" or not parsed.netloc:
        raise ValueError("JEV_ENDPOINT must be an explicit https:// URL")
    payload = json.dumps({
        "model": MODEL,
        "state": state,
        "labels": labels,
        "question_type": question_type,
    }).encode("utf-8")
    headers = {"Content-Type": "application/json", "Accept": "application/json"}
    api_key = os.environ.get("JEV_API_KEY", "").strip()
    if api_key:
        headers["Authorization"] = f"Bearer {api_key}"
    req = urllib.request.Request(endpoint, data=payload, headers=headers, method="POST")
    opener = urllib.request.build_opener(_SameOriginHTTPSRedirectHandler(parsed.netloc))
    with opener.open(req, timeout=25) as resp:
        return json.loads(resp.read().decode("utf-8"))


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--state", required=True)
    ap.add_argument("--labels", default="keep,tune,revert,abstain")
    ap.add_argument("--question-type", default="choice")
    args = ap.parse_args()

    labels = [item.strip() for item in args.labels.split(",") if item.strip()]
    endpoint = os.environ.get("JEV_ENDPOINT", "").strip()
    if endpoint:
        result = call_https_endpoint(endpoint, args.state, labels, args.question_type)
    else:
        result = offline_heuristic(args.state, labels, args.question_type)
    sys.stdout.write(json.dumps(result, separators=(",", ":")) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
