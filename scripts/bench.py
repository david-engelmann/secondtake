#!/usr/bin/env python3
"""Reproducible scale benchmark harness for secondtake.

Usage:
    python3 scripts/bench.py [--bin PATH] [--pairs 100] [--json]
"""
import argparse
import json
import pathlib
import subprocess
import tempfile
import time


def make_tape(task_idx: int, out_tokens: int, calls: int = 20) -> dict:
    events = [{
        "seq": 0,
        "kind": "message",
        "role": "user",
        "sidechain": False,
        "timestamp": "2026-02-02T10:00:00Z",
        "text": f"Benchmark task {task_idx}: verify module invariants.",
    }]
    seq = 1
    for c in range(calls):
        events.append({
            "seq": seq,
            "kind": "tool_call",
            "sidechain": False,
            "timestamp": f"2026-02-02T10:00:{(c % 50) + 1:02d}Z",
            "call_id": f"c_{task_idx}_{c}",
            "tool": "Read",
            "input": {"path": f"src/file_{c}.rs"},
        })
        seq += 1
        events.append({
            "seq": seq,
            "kind": "tool_result",
            "sidechain": False,
            "timestamp": f"2026-02-02T10:00:{(c % 50) + 1:02d}Z",
            "call_id": f"c_{task_idx}_{c}",
            "is_error": False,
            "content": "ok",
        })
        seq += 1
    events.append({
        "seq": seq,
        "kind": "message",
        "role": "assistant",
        "sidechain": False,
        "timestamp": "2026-02-02T10:02:00Z",
        "text": "Verified.",
    })
    return {
        "tape_version": "1.0",
        "run": {
            "harness": "claude-code",
            "session_id": f"s-{task_idx}",
            "totals": {"input_tokens": 1000, "output_tokens": out_tokens},
        },
        "outcome": {"status": "completed", "basis": "final-assistant-message"},
        "events": events,
    }


def measure(cmd: list[str], runs: int = 5) -> float:
    samples = []
    for _ in range(runs):
        t0 = time.perf_counter()
        subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        samples.append((time.perf_counter() - t0) * 1000.0)
    samples.sort()
    return round(samples[len(samples) // 2], 2)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--bin", default="target/release/secondtake")
    ap.add_argument("--pairs", type=int, default=100)
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    bin_path = pathlib.Path(args.bin)
    if not bin_path.exists():
        subprocess.run(["cargo", "build", "--release", "--locked"], check=True)

    with tempfile.TemporaryDirectory() as tmp:
        tdir = pathlib.Path(tmp)
        bdir = tdir / "before"
        adir = tdir / "after"
        bdir.mkdir()
        adir.mkdir()
        for i in range(args.pairs):
            (bdir / f"t_{i:04d}.json").write_text(json.dumps(make_tape(i, 400)), encoding="utf-8")
            (adir / f"t_{i:04d}.json").write_text(json.dumps(make_tape(i, 320)), encoding="utf-8")
        reg = tdir / "reg.json"
        reg.write_text(json.dumps({
            "change": "bench-setup",
            "date": "2026-01-01",
            "min_pairs": 10,
            "margins": {"cost_pct": 5.0, "time_pct": 10.0, "rework_per_pair": 0.5, "rescue_points": 5.0},
        }), encoding="utf-8")
        out = tdir / "cmp.json"
        ms = measure([str(bin_path), "compare", str(bdir), str(adir), "--registration", str(reg), "-o", str(out)])
        results = {
            "matched_pairs": args.pairs,
            "compare_ms": ms,
            "pairs_per_sec": round(args.pairs / (ms / 1000.0)),
        }

    if args.json:
        print(json.dumps(results, indent=2))
    else:
        for k, v in results.items():
            print(f"{k}: {v}")


if __name__ == "__main__":
    main()
