#!/usr/bin/env python3
"""Plot the food need and the bread store, before against after, at one seed.

    tools/plot_food.py <before/series.csv> <after/series.csv> <out.png>

The question this answers is narrow and it is the whole iteration: **does `food` change when the shelf is
empty, and is the village unchanged when it is not?** Three panels, one pair of axes each, red before and
blue after:

1. mean `food` over the run, with the zero-bread ticks shaded — the famine is drawn as a band behind the
   line, so a reader can see whether the need fell *during* it.
2. the bread store (`bread_mean`), identical on both sides unless the change fed back into consumption.
3. the difference in mean `food` over time, which is where a change that only matters in a famine shows up
   as a step and a change that shifted everything shows up as an offset everywhere.
"""
import csv
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

TICKS_PER_DAY = 4096.0


def load(path):
    rows = list(csv.DictReader(open(path)))
    days = [int(r["tick"]) / TICKS_PER_DAY for r in rows]
    return rows, days


def famine_spans(rows, days):
    """Game-day intervals where nobody held any bread."""
    spans, start = [], None
    for r, day in zip(rows, days):
        empty = float(r["bread_max"]) == 0.0
        if empty and start is None:
            start = day
        elif not empty and start is not None:
            spans.append((start, day))
            start = None
    if start is not None:
        spans.append((start, days[-1]))
    return spans


def main():
    before_path, after_path, out = sys.argv[1], sys.argv[2], sys.argv[3]
    b_rows, b_days = load(before_path)
    a_rows, a_days = load(after_path)
    b_food = [float(r["need2"]) for r in b_rows]
    a_food = [float(r["need2"]) for r in a_rows]

    fig, axes = plt.subplots(3, 1, figsize=(13, 9), sharex=True)

    for lo, hi in famine_spans(b_rows, b_days):
        for axis in axes:
            axis.axvspan(lo, hi, color="#f0c0c0", alpha=0.55, linewidth=0, zorder=0)

    axes[0].plot(b_days, b_food, "#a02020", linewidth=0.7, label="before: food paid from an empty shelf")
    axes[0].plot(a_days, a_food, "#2040a0", linewidth=0.7, label="after: food paid for what was there")
    axes[0].set_ylabel("mean food")
    axes[0].set_title("food need, before against after (shaded = no bread anywhere in the village)")
    axes[0].legend(loc="upper right", fontsize=8)

    axes[1].plot(b_days, [float(r["bread_mean"]) for r in b_rows], "#a02020", linewidth=0.7,
                 label="before")
    axes[1].plot(a_days, [float(r["bread_mean"]) for r in a_rows], "#2040a0", linewidth=0.7,
                 label="after")
    axes[1].set_ylabel("mean bread per person")
    axes[1].legend(loc="upper right", fontsize=8)

    axes[2].plot(b_days, [b - a for b, a in zip(b_food, a_food)], "#404040", linewidth=0.7)
    axes[2].axhline(0.0, color="grey", linestyle="--", linewidth=0.6)
    axes[2].set_ylabel("food before − after")
    axes[2].set_xlabel("game days")

    fig.tight_layout()
    fig.savefig(out, dpi=130)
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
