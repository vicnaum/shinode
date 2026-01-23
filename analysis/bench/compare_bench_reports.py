#!/usr/bin/env python3
"""
Compare two benchmark report JSONs produced by stateless-history-node.

Usage:
  python analysis/bench/compare_bench_reports.py \
    benchmarks/ingest_100k_v2__...json \
    benchmarks/ingest_100k_v4_shards__...json
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, List, Tuple


Number = int | float


def _is_number(v: Any) -> bool:
    return isinstance(v, (int, float)) and not isinstance(v, bool)


def _fmt_num(v: Any) -> str:
    if v is None:
        return ""
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, int):
        return f"{v:,}"
    if isinstance(v, float):
        if math.isfinite(v):
            # keep enough precision for rates, but avoid noise
            return f"{v:,.6g}"
        return str(v)
    return str(v)


def _fmt_delta(a: Any, b: Any) -> str:
    if not (_is_number(a) and _is_number(b)):
        return ""
    d = b - a
    if isinstance(d, float):
        if not math.isfinite(d):
            return str(d)
        return f"{d:,.6g}"
    return f"{d:,}"


def _fmt_delta_pct(a: Any, b: Any) -> str:
    if not (_is_number(a) and _is_number(b)):
        return ""
    if a == 0:
        return ""
    pct = (b - a) / a * 100.0
    if not math.isfinite(pct):
        return str(pct)
    return f"{pct:+.1f}%"


def flatten_numeric(obj: Any, prefix: str = "") -> Dict[str, Any]:
    """
    Flatten numeric leaves into a {path: value} dict.
    Lists are ignored by default (to avoid peer_health explosion), except for
    a few small structured lists (e.g. storage segments).
    """
    out: Dict[str, Any] = {}

    if isinstance(obj, dict):
        for k, v in obj.items():
            if k in ("peer_health_all", "peer_health_top", "peer_health_worst"):
                continue
            key = f"{prefix}.{k}" if prefix else str(k)
            out.update(flatten_numeric(v, key))
        return out

    if isinstance(obj, list):
        # Intentionally skip most lists to keep output stable/compact.
        # Include storage segment sizes (small + useful).
        if prefix.endswith("results.storage.segments") or prefix.endswith("storage.segments"):
            for item in obj:
                if not isinstance(item, dict):
                    continue
                name = item.get("name")
                bytes_ = item.get("bytes")
                if isinstance(name, str) and _is_number(bytes_):
                    out[f"{prefix}.{name}.bytes"] = bytes_
            return out
        return out

    if _is_number(obj) or obj is None:
        out[prefix] = obj
        return out

    return out


@dataclass(frozen=True)
class Report:
    label: str
    path: Path
    data: Dict[str, Any]

    @property
    def name(self) -> str:
        return self.data.get("meta", {}).get("benchmark_name", self.label)


def load_report(path: Path, label: str) -> Report:
    data = json.loads(path.read_text())
    if not isinstance(data, dict):
        raise ValueError(f"{path} is not a JSON object")
    return Report(label=label, path=path, data=data)


def _get(d: Dict[str, Any], path: str, default: Any = None) -> Any:
    cur: Any = d
    for part in path.split("."):
        if not isinstance(cur, dict):
            return default
        if part not in cur:
            return default
        cur = cur[part]
    return cur


def derived_metrics(report: Report) -> Dict[str, Any]:
    d = report.data
    elapsed_ms = _get(d, "results.performance.elapsed_ms")
    wall_s = (elapsed_ms / 1000.0) if _is_number(elapsed_ms) and elapsed_ms else None

    fetch_total_us = _get(d, "results.fetch.total_us")
    fetch_s = (fetch_total_us / 1_000_000.0) if _is_number(fetch_total_us) else None
    fetch_parallelism = (fetch_s / wall_s) if (fetch_s and wall_s) else None

    fetch_bytes = _get(d, "results.fetch.bytes_total")
    fetch_mib_wall = (
        (fetch_bytes / (1024.0 * 1024.0)) / wall_s
        if (_is_number(fetch_bytes) and wall_s and wall_s > 0)
        else None
    )

    db_total_us = _get(d, "results.db_write.total_us")
    db_s = (db_total_us / 1_000_000.0) if _is_number(db_total_us) else None
    db_parallelism = (db_s / wall_s) if (db_s and wall_s) else None

    db_bytes = _get(d, "results.db_write.bytes_total")
    db_mib_wall = (
        (db_bytes / (1024.0 * 1024.0)) / wall_s
        if (_is_number(db_bytes) and wall_s and wall_s > 0)
        else None
    )

    # Peer work concentration: how evenly blocks were distributed across peers.
    peers = d.get("peer_health_all", [])
    assigned_blocks: List[int] = []
    successes: List[int] = []
    if isinstance(peers, list):
        for p in peers:
            if not isinstance(p, dict):
                continue
            ab = p.get("assigned_blocks")
            if isinstance(ab, int) and ab >= 0:
                assigned_blocks.append(ab)
            s = p.get("successes")
            if isinstance(s, int) and s >= 0:
                successes.append(s)
    total_assigned = sum(assigned_blocks)
    assigned_sorted = sorted(assigned_blocks, reverse=True)
    top1_share = (assigned_sorted[0] / total_assigned) if total_assigned and assigned_sorted else None
    top3_share = (
        (sum(assigned_sorted[:3]) / total_assigned) if total_assigned and assigned_sorted else None
    )
    top5_share = (
        (sum(assigned_sorted[:5]) / total_assigned) if total_assigned and assigned_sorted else None
    )
    peers_with_successes = None
    if isinstance(peers, list):
        peers_with_successes = sum(
            1
            for p in peers
            if isinstance(p, dict)
            and isinstance(p.get("successes"), int)
            and p.get("successes") > 0
        )

    return {
        "derived.wall_s": wall_s,
        "derived.fetch_total_s": fetch_s,
        "derived.fetch_parallelism_x": fetch_parallelism,
        "derived.fetch_mib_per_sec_wall": fetch_mib_wall,
        "derived.db_total_s": db_s,
        "derived.db_parallelism_x": db_parallelism,
        "derived.db_mib_per_sec_wall": db_mib_wall,
        "derived.peers.peer_health_entries": len(peers) if isinstance(peers, list) else None,
        "derived.peers.peers_with_successes": peers_with_successes,
        "derived.peers.total_assigned_blocks": total_assigned,
        "derived.peers.top1_assigned_share": top1_share,
        "derived.peers.top3_assigned_share": top3_share,
        "derived.peers.top5_assigned_share": top5_share,
    }


def render_markdown_table(
    rows: List[Tuple[str, Any, Any]],
    left_label: str,
    right_label: str,
) -> str:
    lines = []
    lines.append(f"| Metric | {left_label} | {right_label} | Δ | Δ% |")
    lines.append("|---|---:|---:|---:|---:|")
    for key, a, b in rows:
        lines.append(
            "| "
            + key
            + " | "
            + _fmt_num(a)
            + " | "
            + _fmt_num(b)
            + " | "
            + _fmt_delta(a, b)
            + " | "
            + _fmt_delta_pct(a, b)
            + " |"
        )
    return "\n".join(lines)


def main(argv: List[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("left", type=Path, help="baseline report JSON (e.g. v2)")
    parser.add_argument("right", type=Path, help="comparison report JSON (e.g. v4)")
    parser.add_argument("--left-label", default="v2", help="label for left column")
    parser.add_argument("--right-label", default="v4", help="label for right column")
    parser.add_argument(
        "--filter-prefix",
        action="append",
        default=[],
        help="only include metrics starting with this prefix (repeatable)",
    )
    args = parser.parse_args(argv)

    left = load_report(args.left, args.left_label)
    right = load_report(args.right, args.right_label)

    left_flat = flatten_numeric(left.data)
    right_flat = flatten_numeric(right.data)
    left_flat.update(derived_metrics(left))
    right_flat.update(derived_metrics(right))

    # Focus on config/derived/results/events + storage.
    allow_prefixes = ("config.", "derived.", "results.", "events.")
    raw_keys = {k for k in set(left_flat) | set(right_flat) if k.startswith(allow_prefixes)}
    if args.filter_prefix:
        keys = sorted(k for k in raw_keys if any(k.startswith(p) for p in args.filter_prefix))
    else:
        keys = sorted(raw_keys)

    rows: List[Tuple[str, Any, Any]] = [(k, left_flat.get(k), right_flat.get(k)) for k in keys]

    title = f"{left.name} vs {right.name}"
    print(f"## {title}")
    print()
    print(f"- left: `{left.path}`")
    print(f"- right: `{right.path}`")
    print()
    print(render_markdown_table(rows, args.left_label, args.right_label))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

