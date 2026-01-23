#!/usr/bin/env python3
"""
Stream-analyze huge `tracing-chrome` JSON traces (10GB+).

The trace files produced by `tracing_chrome::ChromeLayerBuilder` are JSON arrays with
one event object per line. This script scans them line-by-line without loading the
whole file into memory.

Example:
  python3 analysis/bench/analyze_chrome_trace.py \
    benchmarks_hetzner/trace__20260122T230028Z__tmp.json
"""

from __future__ import annotations

import argparse
import os
import sys
import time
from collections import Counter
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Optional, Tuple


@dataclass
class CatStat:
    count: int = 0
    bytes: int = 0
    max_line_bytes: int = 0


def _extract_json_string_field(line: str, key: str) -> Optional[str]:
    """
    Extremely fast, best-effort extractor for `"key":"value"` fields.

    Assumptions (true for tracing-chrome output for the fields we use):
    - value does not contain an unescaped quote (`"`).
    - JSON is not pretty-printed inside the value.
    """
    needle = f'"{key}":"'
    start = line.find(needle)
    if start == -1:
        return None
    start += len(needle)
    end = line.find('"', start)
    if end == -1:
        return None
    return line[start:end]


def _human_bytes(n: int) -> str:
    if n < 1024:
        return f"{n} B"
    for unit in ("KiB", "MiB", "GiB", "TiB"):
        n_f = n / 1024.0
        if n_f < 1024.0:
            return f"{n_f:.2f} {unit}"
        n = int(n_f)
    return f"{n} B"


def _print_top(
    title: str,
    stats: Dict[str, CatStat],
    total_events: int,
    total_bytes: int,
    key: str,
    top_n: int,
) -> None:
    items: list[Tuple[str, CatStat]] = list(stats.items())
    if key == "bytes":
        items.sort(key=lambda kv: kv[1].bytes, reverse=True)
    elif key == "count":
        items.sort(key=lambda kv: kv[1].count, reverse=True)
    else:
        raise ValueError(f"unsupported sort key {key!r}")

    print()
    print(title)
    print("-" * len(title))
    print(
        f"{'cat':60} {'events':>12} {'events%':>8} {'bytes':>12} {'bytes%':>8} {'max_line':>10}"
    )
    shown = 0
    for cat, st in items:
        if shown >= top_n:
            break
        events_pct = (st.count / total_events * 100.0) if total_events else 0.0
        bytes_pct = (st.bytes / total_bytes * 100.0) if total_bytes else 0.0
        print(
            f"{cat[:60]:60} "
            f"{st.count:12,} "
            f"{events_pct:7.2f}% "
            f"{_human_bytes(st.bytes):>12} "
            f"{bytes_pct:7.2f}% "
            f"{_human_bytes(st.max_line_bytes):>10}"
        )
        shown += 1


def analyze_trace(
    path: Path,
    *,
    top_n: int,
    max_events: Optional[int],
    cat_prefix: Optional[str],
    top_names_cat_prefix: Optional[str],
    top_names_n: int,
    progress_every_s: float,
) -> int:
    if not path.exists():
        raise FileNotFoundError(path)

    total_size = os.path.getsize(path)
    stats: Dict[str, CatStat] = {}
    name_counts: Optional[Counter[str]] = Counter() if top_names_cat_prefix else None
    total_events = 0
    total_bytes = 0

    started = time.time()
    last_report = started
    last_bytes = 0

    with path.open("r", encoding="utf-8", errors="replace") as f:
        for raw_line in f:
            total_bytes += len(raw_line)
            line = raw_line.strip()
            if not line or line == "[" or line == "]" or line == "],":
                continue
            if line.endswith(","):
                line = line[:-1]
            if not line.startswith("{"):
                continue

            cat = _extract_json_string_field(line, "cat") or "(no cat)"
            if cat_prefix is not None and not cat.startswith(cat_prefix):
                # still count bytes read for progress, but skip category aggregation
                if max_events is not None and total_events >= max_events:
                    break
                now = time.time()
                if (now - last_report) >= progress_every_s:
                    dt = now - last_report
                    dbytes = total_bytes - last_bytes
                    rate = (dbytes / dt) if dt > 0 else 0.0
                    print(
                        f"[{path.name}] scanned {_human_bytes(total_bytes)} / {_human_bytes(total_size)} "
                        f"({_human_bytes(int(rate))}/s)",
                        file=sys.stderr,
                    )
                    last_report = now
                    last_bytes = total_bytes
                continue

            st = stats.get(cat)
            if st is None:
                st = CatStat()
                stats[cat] = st

            total_events += 1
            line_len = len(line)
            st.count += 1
            st.bytes += line_len
            if line_len > st.max_line_bytes:
                st.max_line_bytes = line_len
            if name_counts is not None and cat.startswith(top_names_cat_prefix or ""):
                # Prefer the top-level `name` field (usually appears after `args`).
                name = (
                    _extract_json_string_field(line, "name")
                    if line.startswith('{"name":"')
                    else None
                )
                if name is None:
                    # Common layout: {"args":...,"name":"..."}
                    marker = ',"name":"'
                    idx = line.find(marker)
                    if idx != -1:
                        start = idx + len(marker)
                        end = line.find('"', start)
                        if end != -1:
                            name = line[start:end]
                if name is not None:
                    name_counts[name] += 1

            if max_events is not None and total_events >= max_events:
                break

            now = time.time()
            if (now - last_report) >= progress_every_s:
                dt = now - last_report
                dbytes = total_bytes - last_bytes
                rate = (dbytes / dt) if dt > 0 else 0.0
                print(
                    f"[{path.name}] scanned {_human_bytes(total_bytes)} / {_human_bytes(total_size)} "
                    f"({_human_bytes(int(rate))}/s) | events={total_events:,} cats={len(stats):,}",
                    file=sys.stderr,
                )
                last_report = now
                last_bytes = total_bytes

    elapsed_s = time.time() - started
    print()
    print(f"File: {path}")
    print(f"Size: {_human_bytes(total_size)}")
    print(f"Scanned: {_human_bytes(total_bytes)} ({total_bytes / total_size * 100.0:.2f}%)")
    print(f"Events (filtered): {total_events:,}")
    print(f"Categories: {len(stats):,}")
    print(f"Elapsed: {elapsed_s:.1f}s")

    _print_top(
        "Top categories by bytes",
        stats,
        total_events=total_events,
        total_bytes=sum(st.bytes for st in stats.values()),
        key="bytes",
        top_n=top_n,
    )
    _print_top(
        "Top categories by event count",
        stats,
        total_events=total_events,
        total_bytes=sum(st.bytes for st in stats.values()),
        key="count",
        top_n=top_n,
    )

    if name_counts is not None:
        print()
        print(f"Top names for categories starting with {top_names_cat_prefix!r}")
        print("-" * (len("Top names for categories starting with ") + len(repr(top_names_cat_prefix))))
        for name, cnt in name_counts.most_common(top_names_n):
            print(f"{cnt:12,}  {name}")

    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Stream-analyze huge tracing-chrome JSON traces (no full JSON parse)."
    )
    parser.add_argument("trace", nargs="+", type=Path, help="Path(s) to trace JSON file(s)")
    parser.add_argument("--top", type=int, default=25, help="How many categories to show")
    parser.add_argument(
        "--max-events",
        type=int,
        default=None,
        help="Stop after counting this many events (useful for quick sampling).",
    )
    parser.add_argument(
        "--cat-prefix",
        type=str,
        default=None,
        help="Only aggregate categories starting with this prefix (still scans full file).",
    )
    parser.add_argument(
        "--top-names-cat-prefix",
        type=str,
        default=None,
        help="Additionally show top event 'name's for categories starting with this prefix.",
    )
    parser.add_argument(
        "--top-names",
        type=int,
        default=30,
        help="How many top names to show (requires --top-names-cat-prefix).",
    )
    parser.add_argument(
        "--progress-every-s",
        type=float,
        default=5.0,
        help="Progress print interval in seconds.",
    )
    args = parser.parse_args()

    exit_code = 0
    for p in args.trace:
        try:
            exit_code = max(
                exit_code,
                analyze_trace(
                    p,
                    top_n=args.top,
                    max_events=args.max_events,
                    cat_prefix=args.cat_prefix,
                    top_names_cat_prefix=args.top_names_cat_prefix,
                    top_names_n=args.top_names,
                    progress_every_s=args.progress_every_s,
                ),
            )
        except KeyboardInterrupt:
            return 130
        except Exception as e:
            print(f"error: failed to analyze {p}: {e}", file=sys.stderr)
            exit_code = 2
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())

