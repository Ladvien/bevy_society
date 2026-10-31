#!/usr/bin/env python3
"""Per game-day: the floor on `food`, the bread a head and the bakes, clock-driven `bake` against the provisioning overlay.

    tools/plot_provisioning.py <run-dir> <out.png> <table.md>

Reads `<run-dir>/evidence/{before,after}/<key>/{series.csv,summary.json,activities.txt}` for the keys it finds (`v7`, `v42`,
`v11`, `v99` for the 24-person village, `t7`, `t99` for the 60-person town at the harness's 64-second day, `t7d240` for the
town at the library's 240-second day) and writes

- a figure: one column per key; top row the share of villager-ticks with `food` below 0.05 on each game-day, middle the
  bread each person holds on average, bottom the bakes started a game-day (red before, blue after);
- a markdown table of the same numbers plus what else moved (friends, rivals, insults, shares) from `summary.json`, which is
  what a review quotes. The CSVs are ignored by git and regenerate with the command in `eval.md`.

`series.csv` has one row per tick (`floor2`, `need2`, `bread_mean`); `activities.txt` is the per-game-day activity ledger the
harness writes. A game-day is `summary.json`'s `ticks / game_days` rows. Reuses `tools/plot_larder.py`'s loader and encoder.
"""
import json
import re
import sys
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

sys.path.insert(0, str(Path(__file__).resolve().parent))
from plot_larder import load, save_small  # noqa: E402

KEYS = ["v7", "v42", "v11", "v99", "t7", "t99", "t40s3", "t7d240"]
NAMES = {
    "v7": "village s7",
    "v42": "village s42",
    "v11": "village s11",
    "v99": "village s99 (fresh eyes)",
    "t7": "town 60 s7",
    "t99": "town 60 s99 (fresh eyes)",
    "t40s3": "town 40 s3 (fresh eyes)",
    "t7d240": "town 60 s7, 240 s day",
}
PLOTTED = ["v7", "v42", "t7", "t99", "t40s3", "t7d240"]


def bakes(side_dir, days):
    """Bakes started on each game-day, from `activities.txt`."""
    text = (side_dir / "activities.txt").read_text()
    block = re.search(r"^bake: day started completed interrupted\n((?:  .*\n)+)", text, re.M)
    per_day = {}
    if block:
        for line in block.group(1).splitlines():
            day, started, *_ = (int(x) for x in line.split())
            per_day[day] = started
    return [per_day.get(day + 1, 0) for day in range(days)]


def main():
    run, out_png, out_md = Path(sys.argv[1]), sys.argv[2], sys.argv[3]
    keys = [k for k in KEYS if (run / "evidence" / "before" / k / "series.csv").exists()]
    data = {}
    for key in keys:
        data[key] = {}
        for side in ("before", "after"):
            side_dir = run / "evidence" / side / key
            days = load(side_dir)
            data[key][side] = (days, bakes(side_dir, len(days)), json.loads((side_dir / "summary.json").read_text()))

    plotted = [k for k in PLOTTED if k in keys]
    fig, axes = plt.subplots(3, len(plotted), figsize=(3.3 * len(plotted), 8.5), sharex="col", squeeze=False)
    for col, key in enumerate(plotted):
        for side, colour in (("before", "#c0392b"), ("after", "#1f6fb2")):
            days, baked, _ = data[key][side]
            xs = [i + 1 for i in range(len(days))]
            axes[0][col].plot(xs, [100 * d[0] for d in days], color=colour, marker="o", ms=3, label=side)
            axes[1][col].plot(xs, [d[2] for d in days], color=colour, marker="o", ms=3)
            axes[2][col].plot(xs, baked, color=colour, marker="o", ms=3)
        axes[0][col].set_title(NAMES[key], fontsize=9)
        axes[2][col].set_xlabel("game-day")
    axes[0][0].set_ylabel("% of villager-ticks\nwith food < 0.05")
    axes[1][0].set_ylabel("bread a head")
    axes[2][0].set_ylabel("bakes started")
    axes[0][0].legend(fontsize=8)
    for row in axes:
        for axis in row:
            axis.grid(alpha=0.3)
    fig.suptitle("Households that provision themselves: clock-driven bake (red) against the overlay (blue), same seeds", fontsize=10)
    fig.tight_layout()
    save_small(fig, out_png)

    lines = [
        "| run | side | floor %, days 2-6 | floor %, last 6 days | bread a head, last 6 days | bakes a day, last 6 days | friends / rivals | insults | shares |",
        "|---|---|---|---|---|---|---|---|---|",
    ]
    for key in keys:
        for side in ("before", "after"):
            days, baked, summary = data[key][side]
            early = days[1:6]
            late = days[-6:]
            floor_early = 100 * sum(d[0] for d in early) / len(early)
            floor_late = 100 * sum(d[0] for d in late) / len(late)
            bread_late = sum(d[2] for d in late) / len(late)
            bakes_late = sum(baked[-6:]) / len(baked[-6:])
            lines.append(
                f"| {NAMES[key]} | {side} | {floor_early:.1f} | {floor_late:.1f} | {bread_late:.2f} | {bakes_late:.1f} | "
                f"{summary['friends']} / {summary['rivals']} | {summary['insults']} | {summary['shares']} |"
            )
    lines.append("")
    lines.append("Floor % is the share of villager-ticks on a game-day with `food` below 0.05; the last six game-days of each run.")
    Path(out_md).write_text("\n".join(lines) + "\n")
    print("\n".join(lines))


if __name__ == "__main__":
    main()
