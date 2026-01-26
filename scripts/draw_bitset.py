#!/usr/bin/env python3
"""Visualize a shard's bitset as ASCII art."""

import argparse
import sys
from pathlib import Path


def read_bitset(path: Path) -> list[bool]:
    """Read bitset file and return list of bools."""
    data = path.read_bytes()
    bits = []
    for byte in data:
        for i in range(8):
            bits.append(bool(byte & (1 << i)))
    return bits


def visualize(bits: list[bool], width: int = 100, shard: int = 0) -> str:
    """Create ASCII visualization of bitset.

    Uses Unicode block characters:
    - █ (full block) = fetched
    - ░ (light shade) = not fetched
    """
    total = len(bits)
    height = (total + width - 1) // width

    # Calculate padding for row labels (max block number width)
    max_block = shard + (height - 1) * width
    label_width = len(str(max_block))

    # Build column ticks (every 10 columns)
    tick_line = " " * label_width + " "
    for col in range(0, width, 10):
        tick_line += f"{col:<10}"
    tick_line = tick_line.rstrip()

    lines = [tick_line]

    for row in range(height):
        block_start = shard + row * width
        label = f"{block_start:>{label_width}} "
        line = label
        for col in range(width):
            idx = row * width + col
            if idx < total:
                line += "█" if bits[idx] else "░"
            else:
                line += " "
        lines.append(line)

    # Bottom ticks
    lines.append(tick_line)

    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description="Visualize shard bitset")
    parser.add_argument("shard", type=int, help="Shard number (e.g., 24300000)")
    parser.add_argument(
        "--data-dir", "-d",
        type=Path,
        default=Path("data"),
        help="Data directory (default: data)"
    )
    parser.add_argument(
        "--width", "-w",
        type=int,
        default=100,
        help="Display width in characters (default: 100)"
    )
    args = parser.parse_args()

    bitset_path = args.data_dir / "static" / "shards" / str(args.shard) / "present.bitset"

    if not bitset_path.exists():
        print(f"Error: Bitset not found at {bitset_path}", file=sys.stderr)
        sys.exit(1)

    bits = read_bitset(bitset_path)
    total = len(bits)
    fetched = sum(bits)
    pct = (fetched / total * 100) if total > 0 else 0

    print(f"Shard: {args.shard}")
    print(f"Blocks: {fetched}/{total} ({pct:.1f}%)")
    print(f"Path: {bitset_path}")
    print()
    print(visualize(bits, args.width, args.shard))


if __name__ == "__main__":
    main()
