#!/usr/bin/env python3
"""Reference JEV_CALL connector for `secondtake compare --judge live`.

Reads a jev-1.13.0 JSON decision card from stdin and writes a single-line JSON
judgment response to stdout.

Modes:
1. Offline deterministic heuristic (default when JEV_ENDPOINT is unset):
   Evaluates the card locally so you can test `--judge live` offline.
2. Live HTTPS JSON endpoint (when JEV_ENDPOINT is set):
   POSTs the card to an HTTPS endpoint with optional JEV_API_KEY bearer auth,
   blocking plain-HTTP credential egress and cross-origin redirects.
"""

import json
import os
import sys
import urllib.error
import urllib.parse
import urllib.request


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


def offline_heuristic(card: dict) -> dict:
    regressions = card.get("regressions", [])
    if len(regressions) == 0:
        choice, conf = "keep", 0.90
    elif len(regressions) == 1:
        choice, conf = "tune", 0.72
    else:
        choice, conf = "revert", 0.90
    return {
        "schema_version": "jev-1.13.0",
        "decision_type": "categorical",
        "choice": choice,
        "confidence": conf,
        "probabilities": {
            "keep": 0.15 if choice != "keep" else 0.75,
            "tune": 0.72 if choice == "tune" else 0.15,
            "revert": 0.13 if choice != "revert" else 0.75,
        },
    }


def call_https_endpoint(endpoint: str, card: dict) -> dict:
    parsed = urllib.parse.urlparse(endpoint)
    if parsed.scheme != "https" or not parsed.netloc:
        raise ValueError("JEV_ENDPOINT must be an explicit https:// URL")
    payload = json.dumps(card).encode("utf-8")
    headers = {"Content-Type": "application/json", "Accept": "application/json"}
    api_key = os.environ.get("JEV_API_KEY", "").strip()
    if api_key:
        headers["Authorization"] = f"Bearer {api_key}"
    req = urllib.request.Request(endpoint, data=payload, headers=headers, method="POST")
    opener = urllib.request.build_opener(_SameOriginHTTPSRedirectHandler(parsed.netloc))
    with opener.open(req, timeout=25) as resp:
        return json.loads(resp.read().decode("utf-8"))


def main() -> int:
    card = json.load(sys.stdin)
    endpoint = os.environ.get("JEV_ENDPOINT", "").strip()
    result = call_https_endpoint(endpoint, card) if endpoint else offline_heuristic(card)
    sys.stdout.write(json.dumps(result, separators=(",", ":")) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
