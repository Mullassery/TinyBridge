#!/usr/bin/env python3
"""Graphify adapter -- optional, isolated code knowledge-graph backend.

Graphify (https://github.com/Graphify-Labs/graphify) turns a codebase into a
queryable knowledge graph so coding agents (OpenCode, Claude Code, Codex, ...)
can answer structural questions from the graph instead of scanning the whole
repository file by file.

This adapter is the *only* integration point. It shells out to the ``graphify``
CLI and never imports Graphify's Python internals, so Graphify stays an
**optional analysis backend** rather than a runtime dependency: if the CLI is
not installed, the repository's build, tests, and runtime are unaffected.

It exposes one stable interface -- a few subcommands that emit JSON on stdout --
so other tooling can consume graph results without depending on Graphify's
internals or version-specific flags.

Usage:
    python tools/graphify/graphify_adapter.py check
    python tools/graphify/graphify_adapter.py build --root . --out graphify-out
    python tools/graphify/graphify_adapter.py query "how does auth work?"

Install the backend (optional, per developer):
    uv tool install graphifyy      # recommended
    pipx install graphifyy         # alternative

Exit codes:
    0  success, or backend unavailable and --require was not passed
    1  the backend failed, or is unavailable and --require was passed
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

BACKEND = "graphify"
CLI = "graphify"

INSTALL_HINT = (
    f"{CLI!r} is not installed. Graphify is an optional analysis backend; "
    "install it with `uv tool install graphifyy` (or `pipx install graphifyy`)."
)


def _emit(payload: dict, code: int) -> int:
    print(json.dumps(payload, indent=2, sort_keys=True))
    return code


def _unavailable(args: argparse.Namespace) -> int:
    print(INSTALL_HINT, file=sys.stderr)
    return _emit(
        {"backend": BACKEND, "available": False, "message": INSTALL_HINT},
        1 if getattr(args, "require", False) else 0,
    )


def _run(argv, env, timeout):
    return subprocess.run(
        argv,
        env=env,
        timeout=timeout,
        capture_output=True,
        text=True,
        check=False,
    )


def _graphify_out(args: argparse.Namespace) -> Path:
    root = Path(args.root).resolve()
    out = Path(args.out)
    return out if out.is_absolute() else (root / out)


def cmd_check(args: argparse.Namespace) -> int:
    path = shutil.which(CLI)
    if not path:
        return _unavailable(args)
    version = None
    try:
        proc = _run([path, "--version"], os.environ.copy(), 30)
        version = (proc.stdout or proc.stderr).strip() or None
    except Exception:  # noqa: BLE001 - a version probe must never be fatal
        version = None
    return _emit(
        {"backend": BACKEND, "available": True, "path": path, "version": version},
        0,
    )


def cmd_build(args: argparse.Namespace) -> int:
    if not shutil.which(CLI):
        return _unavailable(args)

    root = Path(args.root).resolve()
    out = _graphify_out(args)
    env = os.environ.copy()
    env["GRAPHIFY_OUT"] = str(out)

    argv = [CLI, "extract", str(root), "--code-only"]
    if args.timing:
        argv.append("--timing")

    try:
        proc = _run(argv, env, args.timeout)
    except subprocess.TimeoutExpired:
        return _emit({"backend": BACKEND, "available": True, "error": "timed out"}, 1)

    graph = out / "graph.json"
    report = out / "GRAPH_REPORT.md"
    payload = {
        "backend": BACKEND,
        "available": True,
        "output": str(out),
        "graph": str(graph) if graph.exists() else None,
        "report": str(report) if report.exists() else None,
        "returncode": proc.returncode,
    }
    if proc.stdout:
        payload["stdout"] = proc.stdout.strip()
    if proc.returncode != 0:
        payload["stderr"] = (proc.stderr or "").strip()
        return _emit(payload, 1)
    return _emit(payload, 0)


def _passthrough(args: argparse.Namespace) -> int:
    if not shutil.which(CLI):
        return _unavailable(args)

    if args.command == "query":
        argv = [CLI, "query", args.text]
    elif args.command == "path":
        argv = [CLI, "path", args.start, args.end]
    elif args.command == "explain":
        argv = [CLI, "explain", args.node]
    elif args.command == "affected":
        argv = [CLI, "affected", args.node]
    elif args.command == "god-nodes":
        argv = [CLI, "god-nodes"]
    elif args.command == "diagnose":
        argv = [CLI, "diagnose"]
    else:  # pragma: no cover - argparse rejects anything else
        return _emit({"backend": BACKEND, "available": True, "error": "bad command"}, 1)

    env = os.environ.copy()
    env["GRAPHIFY_OUT"] = str(_graphify_out(args))

    try:
        proc = _run(argv, env, args.timeout)
    except subprocess.TimeoutExpired:
        return _emit({"backend": BACKEND, "available": True, "error": "timed out"}, 1)

    return _emit(
        {
            "backend": BACKEND,
            "available": True,
            "command": args.command,
            "returncode": proc.returncode,
            "stdout": (proc.stdout or "").strip(),
            "stderr": (proc.stderr or "").strip(),
        },
        0 if proc.returncode == 0 else 1,
    )


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="graphify_adapter",
        description="Optional, isolated adapter around the Graphify CLI.",
    )
    sub = parser.add_subparsers(dest="command")

    def common(p):
        p.add_argument("--root", default=".", help="Project root (default: .).")
        p.add_argument("--out", default="graphify-out", help="Output directory.")
        p.add_argument(
            "--timeout", type=int, default=1800, help="Seconds before aborting."
        )
        p.add_argument(
            "--require",
            action="store_true",
            help="Exit non-zero if the Graphify backend is unavailable.",
        )

    p = sub.add_parser("check", help="Report whether the backend is available.")
    common(p)
    p.set_defaults(func=cmd_check)

    p = sub.add_parser("build", help="Run code-only extraction (no model needed).")
    common(p)
    p.add_argument("--timing", action="store_true", help="Print stage timings.")
    p.set_defaults(func=cmd_build)

    p = sub.add_parser("query", help="Query the graph.")
    common(p)
    p.add_argument("text")
    p.set_defaults(func=_passthrough)

    p = sub.add_parser("path", help="Follow a path between two concepts.")
    common(p)
    p.add_argument("start")
    p.add_argument("end")
    p.set_defaults(func=_passthrough)

    p = sub.add_parser("explain", help="Explain a single node.")
    common(p)
    p.add_argument("node")
    p.set_defaults(func=_passthrough)

    p = sub.add_parser("affected", help="Show the dependent impact of a node.")
    common(p)
    p.add_argument("node")
    p.set_defaults(func=_passthrough)

    p = sub.add_parser("god-nodes", help="List highly connected nodes.")
    common(p)
    p.set_defaults(func=_passthrough)

    p = sub.add_parser("diagnose", help="Inspect graph quality and diagnostics.")
    common(p)
    p.set_defaults(func=_passthrough)

    return parser


def main(argv=None) -> int:
    args = build_parser().parse_args(argv)
    if not getattr(args, "func", None):
        build_parser().print_help(sys.stderr)
        return 2
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
