"""ST-12. A thin command line: compare. Failures are
named errors with exit 1."""
from __future__ import annotations

import argparse
import json
import os
import sys
import tempfile
from pathlib import Path

from .compare import CompareError, compare, load_registration
from .judge import FixtureTransport, JudgeError, LiveTransport


def _write_atomically(path: Path, text: str) -> None:
    fd, tmp = tempfile.mkstemp(dir=str(path.parent),
                               prefix=path.name + ".")
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as handle:
            handle.write(text)
        os.replace(tmp, path)
    except BaseException:
        try:
            os.unlink(tmp)
        except OSError:
            pass
        raise


def _emit(document, out) -> int:
    text = json.dumps(document, indent=2, ensure_ascii=False) + "\n"
    if out is not None:
        try:
            _write_atomically(Path(out), text)
        except (OSError, UnicodeEncodeError):
            print(f"error, cannot write the comparison, {out}",
                  file=sys.stderr)
            return 1
        return 0
    try:
        sys.stdout.write(text)
        sys.stdout.flush()
    except (OSError, UnicodeEncodeError):
        print("error, cannot write to stdout", file=sys.stderr)
        try:
            sys.stdout.close()
        except OSError:
            pass
        sys.stdout = open(os.devnull, "w", encoding="utf-8")
        return 1
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="secondtake")
    sub = parser.add_subparsers(dest="command", required=True)
    cp = sub.add_parser("compare", help="the matched comparison")
    cp.add_argument("before")
    cp.add_argument("after")
    cp.add_argument("--registration", required=True)
    cp.add_argument("--judge",
                    help="a judgments file to replay, or live")
    cp.add_argument("-o", "--output")
    args = parser.parse_args(argv)
    try:
        registration = load_registration(Path(args.registration))
        if args.output is not None:
            target = Path(args.output).resolve()
            for directory in (Path(args.before), Path(args.after)):
                root = directory.resolve()
                if target == root or root in target.parents:
                    print("error, secondtake writes nothing inside "
                          "the tape sets it reads", file=sys.stderr)
                    return 1
        judge = None
        if args.judge == "live":
            script = os.environ.get("JEV_CALL")
            if not script:
                raise CompareError(
                    "live judging needs the JEV_CALL "
                    "environment variable naming the "
                    "connector call script")
            judge = LiveTransport(script)
        elif args.judge is not None:
            judge = FixtureTransport(Path(args.judge))
        document = compare(Path(args.before), Path(args.after),
                           registration, judge=judge)
    except (CompareError, JudgeError) as exc:
        print(f"error, {exc}", file=sys.stderr)
        return 1
    return _emit(document, args.output)


if __name__ == "__main__":
    raise SystemExit(main())
