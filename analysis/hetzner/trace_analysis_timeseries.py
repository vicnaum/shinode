#!/usr/bin/env python3
"""
Hetzner trace analysis (streaming).

This script parses huge `tracing-chrome` JSON traces (10–14GB) **without** loading them into memory.
It builds a 1-second time series (cached as `*.timeseries.csv`) and writes two quick SVG plots:
- `*.throughput.svg`: process_block begins/s + fetch_batch begins/s (with stall shading)
- `*.trace_rates.svg`: total trace events/s + selected high-volume categories
"""

from __future__ import annotations

import argparse
import csv
import os
import sys
import time
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import DefaultDict, Dict, List, Optional, Tuple


def _extract_ts_us(line: str) -> Optional[float]:
    k = '"ts":'
    i = line.find(k)
    if i == -1:
        return None
    i += len(k)
    j = i
    while j < len(line) and line[j] not in ",}":
        j += 1
    try:
        return float(line[i:j])
    except ValueError:
        return None


def _is_event_line(line: str) -> bool:
    if not line or line == "[" or line == "]" or line == "],":
        return False
    if not line.startswith("{"):
        return False
    if '"ts":' not in line:
        return False
    return True


@dataclass
class TimeseriesRow:
    t_s: int
    total_events: int
    net_ecies: int
    net_session: int
    node_hist: int
    fetch_batch_begins: int
    process_block_begins: int


def build_timeseries(trace_path: Path) -> Tuple[List[TimeseriesRow], Dict[str, int]]:
    # Stream the trace once and build per-second counters.
    total: DefaultDict[int, int] = defaultdict(int)
    ecies: DefaultDict[int, int] = defaultdict(int)
    session: DefaultDict[int, int] = defaultdict(int)
    hist: DefaultDict[int, int] = defaultdict(int)
    pb: DefaultDict[int, int] = defaultdict(int)
    fb: DefaultDict[int, int] = defaultdict(int)

    pb_first: Optional[int] = None
    pb_last: Optional[int] = None

    size_bytes = os.path.getsize(trace_path)
    scanned_bytes = 0
    started = time.time()
    last_report = started
    report_every_s = 10.0

    with trace_path.open("r", encoding="utf-8", errors="replace") as f:
        for raw in f:
            scanned_bytes += len(raw)
            line = raw.strip()
            if not _is_event_line(line):
                continue
            if line.endswith(","):
                line = line[:-1]

            ts_us = _extract_ts_us(line)
            if ts_us is None:
                continue
            s = int(ts_us // 1_000_000)
            total[s] += 1

            if '"cat":"net::ecies"' in line:
                ecies[s] += 1
            elif '"cat":"net::session"' in line:
                session[s] += 1
            elif '"cat":"stateless_history_node::sync::historical"' in line:
                hist[s] += 1
                if '"name":"process_block"' in line and '"ph":"B"' in line:
                    pb[s] += 1
                    pb_first = s if pb_first is None else min(pb_first, s)
                    pb_last = s if pb_last is None else max(pb_last, s)
                elif '"name":"fetch_batch"' in line and '"ph":"B"' in line:
                    fb[s] += 1

            now = time.time()
            if (now - last_report) >= report_every_s:
                pct = (scanned_bytes / size_bytes * 100.0) if size_bytes else 0.0
                rate_mib = (scanned_bytes / (1024 * 1024)) / max(now - started, 1e-9)
                print(f"[{trace_path.name}] {pct:5.1f}% scanned | {rate_mib:6.1f} MiB/s")
                last_report = now

    if pb_first is None or pb_last is None:
        raise RuntimeError("No process_block span begins found in trace")

    rows: List[TimeseriesRow] = []
    for abs_s in range(pb_first, pb_last + 1):
        t_s = abs_s - pb_first
        rows.append(
            TimeseriesRow(
                t_s=t_s,
                total_events=total.get(abs_s, 0),
                net_ecies=ecies.get(abs_s, 0),
                net_session=session.get(abs_s, 0),
                node_hist=hist.get(abs_s, 0),
                fetch_batch_begins=fb.get(abs_s, 0),
                process_block_begins=pb.get(abs_s, 0),
            )
        )

    meta = {
        "pb_first_abs_s": pb_first,
        "pb_last_abs_s": pb_last,
        "pb_window_s": (pb_last - pb_first + 1),
        "blocks_total": sum(r.process_block_begins for r in rows),
        "max_bps": max((r.process_block_begins for r in rows), default=0),
    }
    return rows, meta


def cache_path_for_trace(trace_path: Path) -> Path:
    return trace_path.with_suffix(trace_path.suffix + ".timeseries.csv")


def load_or_build_timeseries(trace_path: Path, *, use_cache: bool) -> Tuple[List[TimeseriesRow], Dict[str, int]]:
    cache = cache_path_for_trace(trace_path)
    if use_cache and cache.exists():
        rows: List[TimeseriesRow] = []
        with cache.open("r", newline="") as f:
            reader = csv.DictReader(f)
            for r in reader:
                rows.append(
                    TimeseriesRow(
                        t_s=int(r["t_s"]),
                        total_events=int(r["total_events"]),
                        net_ecies=int(r["net_ecies"]),
                        net_session=int(r["net_session"]),
                        node_hist=int(r["node_hist"]),
                        fetch_batch_begins=int(r["fetch_batch_begins"]),
                        process_block_begins=int(r["process_block_begins"]),
                    )
                )

        meta = {
            "pb_window_s": len(rows),
            "blocks_total": sum(r.process_block_begins for r in rows),
            "max_bps": max((r.process_block_begins for r in rows), default=0),
        }
        return rows, meta

    rows, meta = build_timeseries(trace_path)
    if use_cache:
        with cache.open("w", newline="") as f:
            writer = csv.writer(f)
            writer.writerow(
                [
                    "t_s",
                    "total_events",
                    "net_ecies",
                    "net_session",
                    "node_hist",
                    "fetch_batch_begins",
                    "process_block_begins",
                ]
            )
            for r in rows:
                writer.writerow(
                    [
                        r.t_s,
                        r.total_events,
                        r.net_ecies,
                        r.net_session,
                        r.node_hist,
                        r.fetch_batch_begins,
                        r.process_block_begins,
                    ]
                )
        print(f"wrote cache: {cache} ({cache.stat().st_size / (1024 * 1024):.1f} MiB)")
    return rows, meta


@dataclass
class StallInterval:
    start_s: int
    end_s: int
    len_s: int


def stall_intervals(rows: List[TimeseriesRow], *, min_len_s: int = 2) -> List[StallInterval]:
    out: List[StallInterval] = []
    i = 0
    while i < len(rows):
        if rows[i].process_block_begins != 0:
            i += 1
            continue
        start = rows[i].t_s
        j = i
        while j < len(rows) and rows[j].process_block_begins == 0:
            j += 1
        end = rows[j - 1].t_s
        length = end - start + 1
        if length >= min_len_s:
            out.append(StallInterval(start_s=start, end_s=end, len_s=length))
        i = j
    out.sort(key=lambda x: x.len_s, reverse=True)
    return out


def _svg_polyline(points, stroke, stroke_width=1.5, opacity=1.0):
    pts = " ".join(f"{x:.2f},{y:.2f}" for x, y in points)
    return (
        f'<polyline fill="none" stroke="{stroke}" stroke-width="{stroke_width}" '
        f'opacity="{opacity}" points="{pts}" />'
    )


def render_svg_timeseries(
    rows: List[TimeseriesRow],
    series,
    *,
    title: str,
    width: int = 1100,
    height: int = 260,
    y_max: Optional[float] = None,
    shade_stalls: bool = True,
    stall_min_len_s: int = 2,
) -> str:
    m_left, m_right, m_top, m_bottom = 60, 20, 30, 30
    w = width - m_left - m_right
    h = height - m_top - m_bottom
    n = len(rows)
    if n == 0:
        return "<svg />"

    x_min, x_max = rows[0].t_s, rows[-1].t_s
    x_span = max(x_max - x_min, 1)

    ys_all = []
    for _label, _color, fn in series:
        ys_all.extend(float(fn(r)) for r in rows)
    y_hi = max(ys_all) if ys_all else 1.0
    if y_max is None:
        y_max = y_hi if y_hi > 0 else 1.0

    def sx(t: int) -> float:
        return m_left + (t - x_min) / x_span * w

    def sy(v: float) -> float:
        v = max(min(v, y_max), 0.0)
        return m_top + (1.0 - (v / y_max if y_max else 0.0)) * h

    parts = []
    parts.append(f'<rect x="0" y="0" width="{width}" height="{height}" fill="#0b1020" />')
    parts.append(
        f'<text x="{m_left}" y="{m_top - 10}" fill="#e5e7eb" font-family="monospace" font-size="14">{title}</text>'
    )
    parts.append(f'<line x1="{m_left}" y1="{m_top}" x2="{m_left}" y2="{m_top + h}" stroke="#334155" />')
    parts.append(
        f'<line x1="{m_left}" y1="{m_top + h}" x2="{m_left + w}" y2="{m_top + h}" stroke="#334155" />'
    )

    if shade_stalls:
        stalls = stall_intervals(rows, min_len_s=stall_min_len_s)
        for s in stalls:
            x0 = sx(s.start_s)
            x1 = sx(s.end_s + 1)
            parts.append(
                f'<rect x="{x0:.2f}" y="{m_top}" width="{(x1 - x0):.2f}" height="{h}" fill="#ef4444" opacity="0.10" />'
            )

    for frac in [0.0, 0.25, 0.5, 0.75, 1.0]:
        y = m_top + (1.0 - frac) * h
        parts.append(f'<line x1="{m_left}" y1="{y:.2f}" x2="{m_left + w}" y2="{y:.2f}" stroke="#1f2937" />')
        parts.append(
            f'<text x="10" y="{(y + 4):.2f}" fill="#94a3b8" font-family="monospace" font-size="12">{(y_max * frac):.0f}</text>'
        )

    for _label, color, fn in series:
        pts = [(sx(r.t_s), sy(float(fn(r)))) for r in rows]
        parts.append(_svg_polyline(pts, stroke=color, stroke_width=1.5, opacity=0.95))

    lx = m_left
    ly = m_top + h + 20
    for label, color, _fn in series:
        parts.append(f'<rect x="{lx}" y="{ly - 10}" width="12" height="12" fill="{color}" />')
        parts.append(
            f'<text x="{lx + 18}" y="{ly}" fill="#e5e7eb" font-family="monospace" font-size="12">{label}</text>'
        )
        lx += 18 + 8 * len(label)
        if lx > (m_left + w - 200):
            lx = m_left
            ly += 18

    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}">'
        + "".join(parts)
        + "</svg>"
    )


def write_svg(path: Path, svg: str) -> None:
    path.write_text(svg, encoding="utf-8")
    print("wrote", path)


def _default_base() -> Path:
    base = Path("benchmarks_hetzner")
    if base.exists():
        return base
    return Path(".")


def main(argv: List[str]) -> int:
    p = argparse.ArgumentParser()
    p.add_argument(
        "--base",
        type=Path,
        default=_default_base(),
        help="Base directory for default trace names and for output (default: benchmarks_hetzner/ if present).",
    )
    p.add_argument(
        "--trace",
        action="append",
        default=[],
        help="Trace JSON path (repeatable). If omitted, uses two default Hetzner trace filenames under --base.",
    )
    p.add_argument("--no-cache", action="store_true", help="Disable *.timeseries.csv cache read/write")
    p.add_argument("--no-svg", action="store_true", help="Do not write SVG plots")
    p.add_argument("--stall-min-len-s", type=int, default=2, help="Minimum stall interval length to report/shade")
    p.add_argument("--top-stalls", type=int, default=15, help="How many stall intervals to print")
    args = p.parse_args(argv)

    traces: List[Path] = []
    if args.trace:
        traces = [Path(t) for t in args.trace]
    else:
        default_names = [
            "trace__20260122T230028Z__tmp.json",
            "trace__20260122T235403Z__tmp.json",
        ]
        for name in default_names:
            cand = args.base / name
            if cand.exists():
                traces.append(cand)
        if not traces:
            raise SystemExit("No traces provided and default trace names not found under --base")

    for trace in traces:
        rows, meta = load_or_build_timeseries(trace, use_cache=(not args.no_cache))

        stall_secs = sum(1 for r in rows if r.process_block_begins == 0)
        active_secs = len(rows) - stall_secs

        print("---")
        print(trace)
        print("duration_s", meta["pb_window_s"])
        print("blocks_total", meta["blocks_total"])
        print("max_bps", meta["max_bps"])
        print("stall_secs", stall_secs, "active_secs", active_secs)
        if active_secs:
            print("avg_bps_over_active_seconds", (meta["blocks_total"] / active_secs))

        stalls = stall_intervals(rows, min_len_s=args.stall_min_len_s)
        print("top_stalls:")
        for s in stalls[: args.top_stalls]:
            print(f"  {s.len_s:3d}s  t={s.start_s:5d}s..{s.end_s:5d}s")

        if args.no_svg:
            continue

        out_dir = trace.parent
        svg1 = render_svg_timeseries(
            rows,
            series=[
                ("blocks/s (process_block begins)", "#22c55e", lambda r: r.process_block_begins),
                ("fetch_batch begins/s", "#60a5fa", lambda r: r.fetch_batch_begins),
            ],
            title=f"{trace.name} — throughput (stall shading = blocks/s == 0)",
            stall_min_len_s=args.stall_min_len_s,
        )
        write_svg(out_dir / f"{trace.name}.throughput.svg", svg1)

        svg2 = render_svg_timeseries(
            rows,
            series=[
                ("total_events/s ÷1000", "#eab308", lambda r: r.total_events / 1000.0),
                ("net::ecies/s ÷1000", "#f97316", lambda r: r.net_ecies / 1000.0),
                ("net::session/s ÷1000", "#a855f7", lambda r: r.net_session / 1000.0),
                ("node_hist/s ÷1000", "#38bdf8", lambda r: r.node_hist / 1000.0),
            ],
            title=f"{trace.name} — trace activity rates (per second)",
            stall_min_len_s=args.stall_min_len_s,
        )
        write_svg(out_dir / f"{trace.name}.trace_rates.svg", svg2)

    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

