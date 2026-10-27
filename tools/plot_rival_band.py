#!/usr/bin/env python3
"""Plot rival pairs over a run, before and after the rival band.

    /tmp/plotenv/bin/python tools/plot_rival_band.py <run-dir> <out.png>

Reads `<run-dir>/before/s{7,11,42}/series.csv` and `<run-dir>/after/s{7,11,42}/series.csv` and draws one
panel per seed: rivals (solid) and friends (dashed) over game-days, before and after. Two questions the
panel must answer at a glance: does the rival line stop dropping to zero, and does the friend line stay
where it was — because the change guarded the negative category only, and a friend curve that moved would
mean the whole network moved.
"""
import csv
import pathlib
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

TICKS_PER_DAY = 64.0 / (1.0 / 64.0)
SEEDS = (7, 11, 42)


def series(path):
    rows = list(csv.DictReader(open(path)))
    days = [int(r["tick"]) / TICKS_PER_DAY for r in rows]
    rivals = [int(r["rivals"]) for r in rows]
    friends = [int(r["friends"]) for r in rows]
    return days, rivals, friends


def main():
    run = pathlib.Path(sys.argv[1])
    out = sys.argv[2]
    fig, axes = plt.subplots(3, 1, figsize=(13, 10), sharex=True)
    for ax, seed in zip(axes, SEEDS):
        for label, colour, sub in (
            ("before (rival_band 0.0)", "#c0392b", "before"),
            ("after (rival_band 0.10)", "#1f77b4", "after"),
        ):
            path = run / sub / f"s{seed}" / "series.csv"
            days, rivals, friends = series(path)
            ax.plot(days, rivals, color=colour, linewidth=1.2, label=f"rivals, {label}")
            ax.plot(
                days,
                friends,
                color=colour,
                linewidth=0.9,
                linestyle="--",
                alpha=0.65,
                label=f"friends, {label}",
            )
        ax.set_title(f"village-evening, seed {seed}")
        ax.set_ylabel("pairs")
        ax.grid(alpha=0.25)
        ax.legend(fontsize=7, ncol=2, loc="upper left")
    axes[-1].set_xlabel("game-day")
    fig.suptitle(
        "A rivalry held through a band: rivals stop collapsing to zero, friends do not move"
    )
    fig.tight_layout()
    fig.savefig(out, dpi=140)
    print(out)


if __name__ == "__main__":
    main()
