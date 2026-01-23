#!/usr/bin/env python3
"""
Extract a block-number slice from a large bench *.events.jsonl file.

This is useful because IDE viewers/tools may not handle hundreds of thousands of lines well.

Usage:
  python analysis/bench/slice_events_by_block.py \
    benchmarks/ingest_100k_v4_shards__...events.jsonl \
    --start-block 24282000 \
    --end-block 24283451 \
    --out /tmp/v4_tail.events.jsonl
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Dict, Optional, Tuple


def _block_range(event: Dict[str, Any]) -> Optional[Tuple[int, int]]:
    """
    Return (start, end) block range that the event refers to, if any.

    - process/db events usually have a single `block`
    - fetch events usually have `range_start`/`range_end`
    """
    if isinstance(event.get("block"), int):
        b = int(event["block"])
        return (b, b)
    if isinstance(event.get("range_start"), int) and isinstance(event.get("range_end"), int):
        return (int(event["range_start"]), int(event["range_end"]))
    return None


def _overlaps(a: Tuple[int, int], b: Tuple[int, int]) -> bool:
    return not (a[1] < b[0] or b[1] < a[0])


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser()
    p.add_argument("events_jsonl", type=Path)
    p.add_argument("--start-block", type=int, required=True)
    p.add_argument("--end-block", type=int, required=True)
    p.add_argument("--out", type=Path, required=True)
    args = p.parse_args(argv)

    start = int(args.start_block)
    end = int(args.end_block)
    if start > end:
        raise SystemExit("--start-block must be <= --end-block")
    wanted = (start, end)

    out_path: Path = args.out
    out_path.parent.mkdir(parents=True, exist_ok=True)

    kept = 0
    scanned = 0
    with args.events_jsonl.open("r", encoding="utf-8") as fin, out_path.open(
        "w", encoding="utf-8"
    ) as fout:
        for line in fin:
            scanned += 1
            line = line.strip()
            if not line:
                continue
            try:
                obj = json.loads(line)
            except json.JSONDecodeError:
                continue
            rng = _block_range(obj)
            if rng is None:
                continue
            if _overlaps(rng, wanted):
                fout.write(line)
                fout.write("\n")
                kept += 1

    print(f"scanned_lines={scanned} kept_lines={kept} out={out_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

