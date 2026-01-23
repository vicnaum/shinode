#!/usr/bin/env python3
"""
Find "stall gaps" in huge `tracing-chrome` JSON traces.

This is intentionally streaming and dependency-free (works on 10GB+ files).

Example:
  python3 analysis/bench/analyze_trace_gaps.py \
    benchmarks_hetzner/trace__20260122T235403Z__tmp.json \
    --cat-prefix stateless_history_node::sync::historical \
    --name process_block
"""

from __future__ import annotations

import argparse
import os
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Optional, Tuple


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
class Gap:
    gap_s: float
    prev_t_s: float
    cur_t_s: float


def analyze(
    path: Path,
    *,
    cat_prefix: Optional[str],
    name: Optional[str],
    bucket_s: int,
    top_gaps: int,
) -> int:
    if not path.exists():
        raise FileNotFoundError(path)

    total_size = os.path.getsize(path)
    scanned_bytes = 0

    matched = 0
    prev_ts_s: Optional[float] = None
    max_gap: Optional[Gap] = None
    top: list[Gap] = []

    gap_gt_0_5 = 0
    gap_gt_1 = 0
    gap_gt_2 = 0
    gap_gt_5 = 0

    buckets: Dict[int, int] = {}
    min_bucket: Optional[int] = None
    max_bucket: Optional[int] = None

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

            if cat_prefix is not None:
                cat = _extract_json_string_field(line, "cat")
                if cat is None or not cat.startswith(cat_prefix):
                    continue

            if name is not None:
                # Try to get the top-level name quickly: look for `,"name":"..."` first.
                marker = ',"name":"'
                idx = line.find(marker)
                if idx != -1:
                    start = idx + len(marker)
                    end = line.find('"', start)
                    if end == -1:
                        continue
                    ev_name = line[start:end]
                else:
                    # Fallback for lines that start with the name field.
                    ev_name = _extract_json_string_field(line, "name")
                    if ev_name is None:
                        continue
                if ev_name != name:
                    continue

            ts = _extract_json_number_field(line, "ts")
            if ts is None:
                continue

            # tracing-chrome uses microseconds; keep everything in seconds.
            ts_s = ts / 1_000_000.0
            matched += 1

            b = int(ts_s // bucket_s)
            buckets[b] = buckets.get(b, 0) + 1
            min_bucket = b if min_bucket is None else min(min_bucket, b)
            max_bucket = b if max_bucket is None else max(max_bucket, b)

            if prev_ts_s is not None:
                gap_s = ts_s - prev_ts_s
                if gap_s > 0.5:
                    gap_gt_0_5 += 1
                if gap_s > 1.0:
                    gap_gt_1 += 1
                if gap_s > 2.0:
                    gap_gt_2 += 1
                if gap_s > 5.0:
                    gap_gt_5 += 1

                gap = Gap(gap_s=gap_s, prev_t_s=prev_ts_s, cur_t_s=ts_s)
                if max_gap is None or gap_s > max_gap.gap_s:
                    max_gap = gap
                if gap_s > 0:
                    top.append(gap)
                    top.sort(key=lambda g: g.gap_s, reverse=True)
                    if len(top) > top_gaps:
                        top.pop()

            prev_ts_s = ts_s

    print()
    print(f"File: {path}")
    print(f"Size: {_human_bytes(total_size)}")
    print(f"Scanned: {_human_bytes(scanned_bytes)} ({scanned_bytes / total_size * 100.0:.2f}%)")
    print(f"Filter: cat_prefix={cat_prefix!r} name={name!r}")
    print(f"Matched events: {matched:,}")
    if max_gap is not None:
        print(
            f"Max gap: {max_gap.gap_s:.3f}s "
            f"(t={max_gap.prev_t_s:.3f}s → {max_gap.cur_t_s:.3f}s)"
        )
    else:
        print("Max gap: (n/a)")

    print("Gap counts:")
    print(f"  >0.5s: {gap_gt_0_5:,}")
    print(f"  >1.0s: {gap_gt_1:,}")
    print(f"  >2.0s: {gap_gt_2:,}")
    print(f"  >5.0s: {gap_gt_5:,}")

    if top:
        print()
        print(f"Top {len(top)} gaps:")
        for g in top:
            print(f"  {g.gap_s:8.3f}s  (t={g.prev_t_s:10.3f}s → {g.cur_t_s:10.3f}s)")

    if min_bucket is not None and max_bucket is not None:
        print()
        print(f"Event counts per {bucket_s}s bucket (range {min_bucket}..{max_bucket}):")
        # Print only non-empty buckets to keep output compact.
        for b in range(min_bucket, max_bucket + 1):
            c = buckets.get(b, 0)
            if c == 0:
                continue
            print(f"  t={b * bucket_s:5d}s..{(b + 1) * bucket_s:5d}s: {c:,}")

    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Find timing gaps in tracing-chrome traces.")
    parser.add_argument("trace", type=Path, help="Path to trace JSON")
    parser.add_argument(
        "--cat-prefix",
        type=str,
        default=None,
        help="Only consider events with cat starting with prefix",
    )
    parser.add_argument(
        "--name",
        type=str,
        default=None,
        help="Only consider events with this exact top-level name",
    )
    parser.add_argument("--bucket-s", type=int, default=1, help="Bucket size in seconds for counts output")
    parser.add_argument("--top-gaps", type=int, default=20, help="How many largest gaps to print")
    args = parser.parse_args()

    try:
        return analyze(
            args.trace,
            cat_prefix=args.cat_prefix,
            name=args.name,
            bucket_s=args.bucket_s,
            top_gaps=args.top_gaps,
        )
    except KeyboardInterrupt:
        return 130


if __name__ == "__main__":
    raise SystemExit(main())

