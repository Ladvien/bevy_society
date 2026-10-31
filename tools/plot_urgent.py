#!/usr/bin/env python3
"""The floor on `food` and the spells of hunger, with and without the `urgent` rank on the provisioning overlay.

    tools/plot_urgent.py <run-dir> <out.png> <table.md>     # both paths relative to <run-dir>

Reads `<run-dir>/evidence/{before,after}/<key>/{floor.txt,summary.json,series.csv}` for the keys it finds (`v7`, `v42`, `v11`,
`v99` for the 24-person village, `t7`, `t99` for the 60-person town, `t40s3` for a 40-person town, `t7d240` for the town at the
library's own 240-second day) and writes

- a figure: one column per key; top row the share of villager-ticks with `food` below 0.05 on each game-day (red before, blue
  after), middle row what the people at the floor were doing (stacked share of the floor, before then after), bottom row the
  spells of hunger (mean, median, p90 in game-hours);
- a markdown table of the same numbers plus what else moved (friends, rivals, insults, shares, bread a head), which is what a
  review quotes. `series.csv` is ignored by git and regenerates with the command in `eval.md`; `floor.txt` and
  `summary.json` are committed.

`floor.txt` is the harness's ledger of who is at the floor (from game-day 2) and what they were doing, plus the spells.
Reuses `tools/plot_larder.py`'s loader and encoder.
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
# What a person at the floor was doing, in the order they are stacked.
GROUPS = [
    ("asleep or walking to bed", ("sleep", "travelling:sleep"), "#6b7a8f"),
    ("walking to the table", ("travelling:eat",), "#e0a030"),
    ("at the table", ("eat",), "#3a9d5d"),
    ("making bread", ("bake", "travelling:bake"), "#8c564b"),
    ("work and craft", ("work", "travelling:work", "craft", "travelling:craft"), "#c0392b"),
    ("company and leisure", None, "#9467bd"),
]


def read_floor(side_dir):
    """`(floor share, {activity label: share of the floor}, share with a meal in hand, spells)` from `floor.txt`."""
    text = (side_dir / "floor.txt").read_text()
    share = float(re.search(r"at the floor on food \(< [\d.]+\): \d+ \(([\d.]+)%\)", text).group(1))
    by, total, meal = {}, 0, 0
    for m in re.finditer(r"^\s+(\d+)\s+[\d.]+%\s+(\S+)\s+\[(holds a meal|no meal)\]", text, re.M):
        n = int(m.group(1))
        by[m.group(2)] = by.get(m.group(2), 0) + n
        total += n
        meal += n if m.group(3) == "holds a meal" else 0
    spells = re.search(
        r"(\d+) finished, mean ([\d.]+), median ([\d.]+), p90 ([\d.]+), max ([\d.]+)", text
    )
    n, mean, median, p90, longest = int(spells.group(1)), *(float(x) for x in spells.groups()[1:])
    return share, {k: v / max(total, 1) for k, v in by.items()}, meal / max(total, 1), (n, mean, median, p90, longest)


def grouped(shares):
    """Stacks the shares into [`GROUPS`], the last being whatever is left."""
    out, used = [], set()
    for _, labels, _ in GROUPS[:-1]:
        out.append(sum(shares.get(label, 0.0) for label in labels))
        used.update(labels)
    out.append(sum(v for k, v in shares.items() if k not in used))
    return out


def main():
    run, out_png, out_md = Path(sys.argv[1]), sys.argv[2], sys.argv[3]
    keys = [k for k in KEYS if (run / "evidence" / "before" / k / "floor.txt").exists()]
    data = {}
    for key in keys:
        data[key] = {}
        for side in ("before", "after"):
            side_dir = run / "evidence" / side / key
            has_series = (side_dir / "series.csv").exists()
            data[key][side] = {
                "days": load(side_dir) if has_series else None,
                "floor": read_floor(side_dir),
                "summary": json.loads((side_dir / "summary.json").read_text()),
            }

    plotted = [k for k in PLOTTED if k in keys]
    fig, axes = plt.subplots(3, len(plotted), figsize=(3.3 * len(plotted), 9.5), squeeze=False)
    for col, key in enumerate(plotted):
        ax = axes[0][col]
        for side, color in (("before", "#c0392b"), ("after", "#2468b8")):
            days = data[key][side]["days"]
            if days:
                ax.plot(range(1, len(days) + 1), [100 * d[0] for d in days], color=color, lw=1.3, label=side)
        ax.set_title(NAMES[key], fontsize=9)
        ax.set_ylim(0, None)
        ax.set_xlabel("game-day", fontsize=8)
        if col == 0:
            ax.set_ylabel("% of person-ticks at food < 0.05")
            ax.legend(fontsize=8)

        ax = axes[1][col]
        for x, side in enumerate(("before", "after")):
            bottom = 0.0
            for (label, _, color), value in zip(GROUPS, grouped(data[key][side]["floor"][1])):
                ax.bar(x, 100 * value, bottom=bottom, color=color, label=label if col == 0 and x == 0 else None, width=0.6)
                bottom += 100 * value
        ax.set_xticks([0, 1], ["before", "after"])
        ax.set_ylim(0, 100)
        if col == 0:
            ax.set_ylabel("what the floor was doing, %")
            ax.legend(fontsize=6, loc="lower left")

        ax = axes[2][col]
        for x, side in enumerate(("before", "after")):
            _, mean, median, p90, _ = data[key][side]["floor"][3]
            for k, (v, name) in enumerate(((mean, "mean"), (median, "median"), (p90, "p90"))):
                ax.bar(x * 4 + k, v, color=("#c0392b" if side == "before" else "#2468b8"), alpha=(0.55, 0.8, 1.0)[k],
                       label=name if col == 0 and x == 0 else None, width=0.9)
        ax.set_xticks([1, 5], ["before", "after"])
        if col == 0:
            ax.set_ylabel("spell of hunger, game-hours\n(mean, median, p90)")
    fig.tight_layout()
    (run / out_png).parent.mkdir(parents=True, exist_ok=True)
    save_small(fig, run / out_png)

    last = lambda d: 100 * sum(x[0] for x in d[-6:]) / max(len(d[-6:]), 1)
    bread = lambda d: sum(x[2] for x in d[-6:]) / max(len(d[-6:]), 1)
    lines = [
        "| run | side | floor % (all) | floor %, last 6 days | spells: n / mean / median / p90 / max (h) | held a meal | bread a head, last 6 days | friends / rivals | insults | shares |",
        "|---|---|---|---|---|---|---|---|---|---|",
    ]
    for key in keys:
        for side in ("before", "after"):
            cell = data[key][side]
            floor, _, meal, (n, mean, median, p90, longest) = cell["floor"]
            summary = cell["summary"]
            days = cell["days"]
            lines.append(
                f"| {NAMES[key]} | {side} | {floor:.1f} | {last(days):.1f} | {n} / {mean:.1f} / {median:.1f} / {p90:.1f} / {longest:.1f}"
                f" | {100 * meal:.0f} % | {bread(days):.2f} | {summary['friends']} / {summary['rivals']} | {summary['insults']} | {summary['shares']} |"
                if days
                else f"| {NAMES[key]} | {side} | {floor:.1f} | n/a | {n} / {mean:.1f} / {median:.1f} / {p90:.1f} / {longest:.1f}"
                f" | {100 * meal:.0f} % | {summary['bread_mean']:.2f} | {summary['friends']} / {summary['rivals']} | {summary['insults']} | {summary['shares']} |"
            )
    (run / out_md).write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    main()
