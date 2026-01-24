#!/usr/bin/env python3
"""
Compaction vs memory correlation for benchmark ingest artifacts.

This script reads the benchmark `events__*.jsonl` file and correlates:
  - compaction_start/compaction_end windows
  - resources_sample RSS / swap samples (from /proc/self/status on Linux)

It prints a short summary showing whether memory grows during compaction and
whether it returns after compaction.
"""

from __future__ import annotations

import argparse
import json
from bisect import bisect_left
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Tuple


GiB = 1024 * 1024  # KiB -> GiB


@dataclass(frozen=True)
class ResourceSample:
    t_ms: int
    rss_kb: int
    swap_kb: int


@dataclass(frozen=True)
class CompactionEnd:
    t_ms: int
    duration_ms: int


def _nearest_resource(samples: List[ResourceSample], t_ms: int) -> ResourceSample:
    ts = [s.t_ms for s in samples]
    i = bisect_left(ts, t_ms)
    if i <= 0:
        return samples[0]
    if i >= len(samples):
        return samples[-1]
    a = samples[i - 1]
    b = samples[i]
    return b if abs(b.t_ms - t_ms) < abs(a.t_ms - t_ms) else a


def _in_any_compaction(
    ranges: List[Tuple[int, int, int]],
    t_ms: int,
) -> Optional[int]:
    for start_ms, end_ms, shard_start in ranges:
        if start_ms <= t_ms <= end_ms:
            return shard_start
    return None


def parse_events(events_path: Path) -> Tuple[List[ResourceSample], Dict[int, int], Dict[int, CompactionEnd]]:
    resources: List[ResourceSample] = []
    comp_start: Dict[int, int] = {}
    comp_end: Dict[int, CompactionEnd] = {}

    with events_path.open("r", encoding="utf-8", errors="replace") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                o = json.loads(line)
            except Exception:
                continue

            ev = o.get("event")
            if ev == "resources_sample":
                resources.append(
                    ResourceSample(
                        t_ms=int(o["t_ms"]),
                        rss_kb=int(o["rss_kb"]),
                        swap_kb=int(o["swap_kb"]),
                    )
                )
            elif ev == "compaction_start":
                comp_start[int(o["shard_start"])] = int(o["t_ms"])
            elif ev == "compaction_end":
                comp_end[int(o["shard_start"])] = CompactionEnd(
                    t_ms=int(o["t_ms"]),
                    duration_ms=int(o["duration_ms"]),
                )

    resources.sort(key=lambda r: r.t_ms)
    return resources, comp_start, comp_end


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("events_jsonl", type=Path)
    p.add_argument("--post-window-s", type=int, default=60, help="window after compaction end to check RSS drop")
    p.add_argument("--top-n", type=int, default=12, help="how many entries to print in each section")
    args = p.parse_args()

    resources, comp_start, comp_end = parse_events(args.events_jsonl)
    if not resources:
        raise SystemExit("no resources_sample events found")

    comp_ranges: List[Tuple[int, int, int]] = []
    for shard_start, end in comp_end.items():
        start_ms = comp_start.get(shard_start, end.t_ms - end.duration_ms)
        comp_ranges.append((start_ms, end.t_ms, shard_start))
    comp_ranges.sort()

    # Per-compaction deltas.
    rows = []
    for start_ms, end_ms, shard_start in comp_ranges:
        rs0 = _nearest_resource(resources, start_ms)
        rs1 = _nearest_resource(resources, end_ms)
        rss0 = rs0.rss_kb / GiB
        swap0 = rs0.swap_kb / GiB
        rss1 = rs1.rss_kb / GiB
        swap1 = rs1.swap_kb / GiB

        # Look at min RSS after end within window.
        ts = [s.t_ms for s in resources]
        i0 = bisect_left(ts, end_ms)
        i1 = bisect_left(ts, end_ms + args.post_window_s * 1000)
        window = resources[i0 : min(i1 + 1, len(resources))]
        min_rss_next = min((s.rss_kb for s in window), default=rs1.rss_kb) / GiB

        rows.append(
            (
                start_ms / 1000.0,
                end_ms / 1000.0,
                shard_start,
                (end_ms - start_ms) / 1000.0,
                rss0,
                rss1,
                rss1 - rss0,
                swap0,
                swap1,
                swap1 - swap0,
                min_rss_next,
                min_rss_next - rss0,
            )
        )

    rows.sort(key=lambda r: r[0])
    print(f"resources_sample: {len(resources)}")
    print(f"compactions:      {len(rows)} (start={len(comp_start)} end={len(comp_end)})")

    print("\nFirst compactions (showing RSS jump during compaction and lack of post-drop):")
    for r in rows[: args.top_n]:
        ts, te, shard, dur, rss0, rss1, drss, swap0, swap1, dswap, min_next, dmin = r
        print(
            f"{ts:7.1f}-{te:7.1f}s shard {shard} dur {dur:5.1f}s | "
            f"rss {rss0:5.1f}->{rss1:5.1f} ({drss:+5.1f}) GiB | "
            f"swap {swap0:5.1f}->{swap1:5.1f} ({dswap:+5.1f}) GiB | "
            f"min_rss_next{args.post_window_s}s {min_next:5.1f} (delta {dmin:+5.1f})"
        )

    # Biggest per-sample jumps and whether they're inside compaction windows.
    deltas = []
    for prev, cur in zip(resources, resources[1:]):
        drss = (cur.rss_kb - prev.rss_kb) / GiB
        dswap = (cur.swap_kb - prev.swap_kb) / GiB
        dtotal = drss + dswap
        deltas.append((dtotal, drss, dswap, cur.t_ms))
    deltas.sort(reverse=True)

    print(f"\nTop {args.top_n} per-sample total (rss+swap) increases:")
    for dtotal, drss, dswap, t_ms in deltas[: args.top_n]:
        shard = _in_any_compaction(comp_ranges, t_ms)
        where = f"compaction shard {shard}" if shard is not None else "no_compaction"
        print(
            f"t={t_ms/1000:7.1f}s total {dtotal:+5.1f} GiB "
            f"(rss {drss:+5.1f}, swap {dswap:+5.1f}) {where}"
        )

    last = resources[-1]
    print(
        f"\nLast resource_sample: t={last.t_ms/1000:.1f}s rss={last.rss_kb/GiB:.1f}GiB swap={last.swap_kb/GiB:.1f}GiB"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

