#!/usr/bin/env python3
"""Per game-day: how many villagers sit at the floor on `food`, before against after a starting larder.

    tools/plot_larder.py <run-dir> <out.png> <floor.md>

Reads `<run-dir>/evidence/{before,after}/<key>/{series.csv,summary.json}` for the keys it finds (`v7`, `v42`, `v11`,
`v99` for the 24-person village, `t7`, `t99` for the 60-person town), and writes

- a figure: one column per key, top row the share of villager-ticks with `food` below 0.05 on each game-day
  (red before, blue after), bottom row the mean `food` and the bread each person holds on average;
- a markdown table of the same numbers for the first six game-days and the mean of the rest, which is what a
  review quotes (the CSVs themselves are ignored by git and regenerate with the command in `eval.md`).

`series.csv` has one row per tick and a `floor2` column (villagers whose `food`, need 2, is below 0.05), written by
`examples/evidence`. A game-day is `summary.json`'s `ticks / game_days` rows.
"""
import csv
import json
import sys
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

KEYS = ["v7", "v42", "v11", "v99", "t7", "t99"]
NAMES = {
    "v7": "village s7",
    "v42": "village s42",
    "v11": "village s11",
    "v99": "village s99 (fresh eyes)",
    "t7": "town 60 s7",
    "t99": "town 60 s99 (fresh eyes)",
}


def load(side_dir):
    """Per game-day: (floor share of villager-ticks, mean food, mean bread a head)."""
    summary = json.loads((side_dir / "summary.json").read_text())
    people = summary["people"]
    per_day = round(summary["ticks"] / summary["game_days"])
    days = []
    acc = [0.0, 0.0, 0.0, 0]
    with open(side_dir / "series.csv") as handle:
        for row in csv.DictReader(handle):
            acc[0] += int(row["floor2"]) / people
            acc[1] += float(row["need2"])
            acc[2] += float(row["bread_mean"])
            acc[3] += 1
            if acc[3] == per_day:
                days.append((acc[0] / acc[3], acc[1] / acc[3], acc[2] / acc[3]))
                acc = [0.0, 0.0, 0.0, 0]
    return days


def main():
    run, out_png, out_md = Path(sys.argv[1]), sys.argv[2], sys.argv[3]
    keys = [k for k in KEYS if (run / "evidence" / "before" / k / "series.csv").exists()]
    data = {
        k: {
            side: load(run / "evidence" / side / k)
            for side in ("before", "after")
        }
        for k in keys
    }

    fig, axes = plt.subplots(3, len(keys), figsize=(3.3 * len(keys), 8.5), sharex="col", squeeze=False)
    for col, key in enumerate(keys):
        for side, colour in (("before", "#c0392b"), ("after", "#1f6fb2")):
            days = data[key][side]
            xs = [i + 1 for i in range(len(days))]
            axes[0][col].plot(xs, [100 * d[0] for d in days], color=colour, marker="o", ms=3, label=side)
            axes[1][col].plot(xs, [d[1] for d in days], color=colour, marker="o", ms=3)
            axes[2][col].plot(xs, [d[2] for d in days], color=colour, marker="o", ms=3)
        axes[0][col].set_title(NAMES[key], fontsize=9)
        axes[2][col].set_xlabel("game-day")
    axes[0][0].set_ylabel("% of villager-ticks\nwith food < 0.05")
    axes[1][0].set_ylabel("mean food")
    axes[2][0].set_ylabel("bread a head")
    axes[0][0].legend(fontsize=8)
    for row in axes:
        for axis in row:
            axis.grid(alpha=0.3)
    fig.suptitle("A starting larder: the floor on food, per game-day, same seeds", fontsize=11)
    fig.tight_layout()
    fig.savefig(out_png, dpi=110)

    lines = [
        "| run | side | day 1 | day 2 | day 3 | day 4 | day 5 | day 6 | mean of day 7 on |",
        "|---|---|---|---|---|---|---|---|---|",
    ]
    for key in keys:
        for side in ("before", "after"):
            days = data[key][side]
            cells = [f"{100 * d[0]:.1f}" for d in days[:6]]
            rest = days[6:]
            tail = f"{100 * sum(d[0] for d in rest) / len(rest):.1f}" if rest else ""
            lines.append(f"| {NAMES[key]} | {side} | " + " | ".join(cells) + f" | {tail} |")
    lines.append("")
    lines.append("Cells are the percentage of villager-ticks on that game-day with `food` below 0.05.")
    Path(out_md).write_text("\n".join(lines) + "\n")
    print("\n".join(lines))


if __name__ == "__main__":
    main()
