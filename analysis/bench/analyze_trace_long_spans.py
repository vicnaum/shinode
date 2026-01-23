#!/usr/bin/env python3
"""
Find longest spans in huge `tracing-chrome` JSON traces.

`tracing-chrome` encodes spans as Chrome trace events with phases:
- "B": span begin
- "E": span end

This script reconstructs per-thread stacks and reports the longest observed
durations, optionally filtered by `cat` prefix.

Example:
  python3 analysis/bench/analyze_trace_long_spans.py \
    benchmarks_hetzner/trace__20260122T230028Z__tmp.json \
    --cat-prefix stateless_history_node --top 30
"""

from __future__ import annotations

import argparse
import os
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Optional


def _extract_json_string_field(line: str, key: str) -> Optional[str]:
    needle = f'"{key}":"'
    start = line.find(needle)
    if start == -1:
        return None
    start += len(needle)
    end = line.find('"', start)
    if end == -1:
        return None
    return line[start:end]


def _extract_json_number_field(line: str, key: str) -> Optional[float]:
    needle = f'"{key}":'
    start = line.find(needle)
    if start == -1:
        return None
    start += len(needle)
    end = start
    while end < len(line) and line[end] not in ",}":
        end += 1
    try:
        return float(line[start:end])
    except ValueError:
        return None


def _human_bytes(n: int) -> str:
    if n < 1024:
        return f"{n} B"
    for unit in ("KiB", "MiB", "GiB", "TiB"):
        n_f = n / 1024.0
        if n_f < 1024.0:
            return f"{n_f:.2f} {unit}"
        n = int(n_f)
    return f"{n} B"


@dataclass
class SpanStart:
    cat: str
    name: str
    ts_us: float


@dataclass
class SpanRecord:
    dur_s: float
    start_s: float
    end_s: float
    cat: str
    name: str
    tid: int


def analyze(path: Path, *, cat_prefix: Optional[str], top_n: int) -> int:
    if not path.exists():
        raise FileNotFoundError(path)

    total_size = os.path.getsize(path)
    scanned_bytes = 0

    stacks: Dict[int, list[SpanStart]] = {}
    top: list[SpanRecord] = []

    total_begins = 0
    total_ends = 0
    dropped_unbalanced = 0

    with path.open("r", encoding="utf-8", errors="replace") as f:
        for raw_line in f:
            scanned_bytes += len(raw_line)
            line = raw_line.strip()
            if not line or line == "[" or line == "]" or line == "],":
                continue
            if line.endswith(","):
                line = line[:-1]
            if not line.startswith("{"):
                continue

            ph = _extract_json_string_field(line, "ph")
            if ph not in ("B", "E"):
                continue

            tid_f = _extract_json_number_field(line, "tid")
            ts = _extract_json_number_field(line, "ts")
            name = _extract_json_string_field(line, "name")
            cat = _extract_json_string_field(line, "cat")
            if tid_f is None or ts is None or name is None:
                continue
            tid = int(tid_f)
            cat = cat or "(no cat)"

            # tracing-chrome uses microseconds.
            ts_us = ts
            if ph == "B":
                total_begins += 1
                stacks.setdefault(tid, []).append(SpanStart(cat=cat, name=name, ts_us=ts_us))
                continue

            # ph == "E"
            total_ends += 1
            stack = stacks.get(tid)
            if not stack:
                dropped_unbalanced += 1
                continue
            start = stack.pop()
            dur_us = ts_us - start.ts_us
            if dur_us < 0:
                continue
            if cat_prefix is not None and not start.cat.startswith(cat_prefix):
                continue

            start_s = start.ts_us / 1_000_000.0
            end_s = ts_us / 1_000_000.0
            dur_s = dur_us / 1_000_000.0
            rec = SpanRecord(
                dur_s=dur_s,
                start_s=start_s,
                end_s=end_s,
                cat=start.cat,
                name=start.name,
                tid=tid,
            )
            top.append(rec)
            top.sort(key=lambda r: r.dur_s, reverse=True)
            if len(top) > top_n:
                top.pop()

    print()
    print(f"File: {path}")
    print(f"Size: {_human_bytes(total_size)}")
    print(f"Scanned: {_human_bytes(scanned_bytes)} ({scanned_bytes / total_size * 100.0:.2f}%)")
    print(f"Filter: cat_prefix={cat_prefix!r}")
    print(f"Span begins: {total_begins:,}")
    print(f"Span ends: {total_ends:,}")
    print(f"Unbalanced ends: {dropped_unbalanced:,}")
    print()
    print(f"Top {len(top)} spans by duration:")
    for r in top:
        print(
            f"{r.dur_s:9.3f}s  "
            f"t={r.start_s:10.3f}s..{r.end_s:10.3f}s  "
            f"tid={r.tid:<4d}  "
            f"{r.cat} :: {r.name}"
        )

    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Find longest spans in tracing-chrome traces.")
    parser.add_argument("trace", type=Path, help="Path to trace JSON")
    parser.add_argument(
        "--cat-prefix",
        type=str,
        default=None,
        help="Only record spans whose cat starts with prefix",
    )
    parser.add_argument("--top", type=int, default=50, help="How many longest spans to print")
    args = parser.parse_args()

    try:
        return analyze(args.trace, cat_prefix=args.cat_prefix, top_n=args.top)
    except KeyboardInterrupt:
        return 130


if __name__ == "__main__":
    raise SystemExit(main())

