#!/usr/bin/env python3
"""Plot rival and friend pairs over a run, before and after `Tuning::forgiveness`, plus the gain sweep.

    /tmp/plotenv/bin/python tools/plot_forgiveness.py <run-dir> <out.png> [<sweep-dir> <sweep.png>]

Reads `<run-dir>/evidence/{before,after}/{v7,v42,v11,t7}/series.csv` and draws one panel per run: rivals
(solid) and friends (dashed) over game-days. The questions a panel answers at a glance: do rivals stop
climbing without falling to zero, do friends rise rather than fall, and does either line oscillate.

With a sweep directory (`<sweep-dir>/{v7,t7}-f<gain>/summary.json`, gain 0 read from `before/`), draws
insults, rivals and friends against the gain, to show whether the response is smooth or a cliff.
"""
import csv
import json
import pathlib
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

# The village scenario runs at 64 s a game-day and 64 ticks a second.
TICKS_PER_DAY = 64.0 * 64.0
RUNS = (("v7", "village, seed 7"), ("v42", "village, seed 42"), ("v11", "village, seed 11"), ("t7", "town of 60, seed 7"))


def series(path):
    days, rivals, friends = [], [], []
    with open(path) as f:
        for i, r in enumerate(csv.DictReader(f)):
            if i % 64:
                continue
            days.append(int(r["tick"]) / TICKS_PER_DAY)
            rivals.append(int(r["rivals"]))
            friends.append(int(r["friends"]))
    return days, rivals, friends


def over_time(run, out):
    fig, axes = plt.subplots(len(RUNS), 1, figsize=(13, 12), sharex=False)
    for ax, (key, title) in zip(axes, RUNS):
        for label, colour, sub in (("before (HEAD)", "#c0392b", "before"), ("after (forgiveness)", "#1f77b4", "after")):
            path = run / "evidence" / sub / key / "series.csv"
            if not path.exists():
                continue
            d, r, f = series(path)
            ax.plot(d, r, color=colour, lw=1.4, label=f"rivals, {label}")
            ax.plot(d, f, color=colour, lw=1.4, ls="--", label=f"friends, {label}")
        ax.set_title(title)
        ax.set_ylabel("pairs")
        ax.grid(alpha=0.3)
        ax.legend(fontsize=8, loc="upper left")
    axes[-1].set_xlabel("game-days")
    fig.tight_layout()
    fig.savefig(out, dpi=110)


def sweep(run, sweep_dir, out):
    fig, axes = plt.subplots(1, 2, figsize=(13, 4.5))
    for ax, key, title in ((axes[0], "v7", "village, seed 7, 17 days"), (axes[1], "t7", "town of 60, seed 7, 30 days")):
        points = []
        base = run / "evidence" / "before" / key / "summary.json"
        if base.exists():
            points.append((0.0, json.load(open(base))))
        for path in sorted(sweep_dir.glob(f"{key}-f*/summary.json")):
            gain = path.parent.name.split("-f", 1)[1]
            if gain == "off":
                continue
            points.append((float(gain), json.load(open(path))))
        points.sort(key=lambda p: p[0])
        gains = [g for g, _ in points]
        for field, colour in (("insults", "#7f7f7f"), ("rivals", "#c0392b"), ("friends", "#1f77b4")):
            values = [s[field] for _, s in points]
            if field == "insults":
                values = [v / 10.0 for v in values]
                field = "insults / 10"
            ax.plot(gains, values, "o-", color=colour, label=field)
        ax.set_xscale("symlog", linthresh=1.0)
        ax.set_xticks(gains)
        ax.set_xticklabels([f"{g:g}" for g in gains])
        ax.set_xlabel("Tuning::forgiveness")
        ax.set_title(title)
        ax.grid(alpha=0.3)
        ax.legend(fontsize=8)
    fig.tight_layout()
    fig.savefig(out, dpi=110)


def main():
    run = pathlib.Path(sys.argv[1])
    over_time(run, sys.argv[2])
    if len(sys.argv) > 4:
        sweep(run, pathlib.Path(sys.argv[3]), sys.argv[4])


if __name__ == "__main__":
    main()
