#!/usr/bin/env python3
"""Plot the price and the village's store from a `series.csv`, before and after.

    tools/plot_price.py <run-dir> <out.png>

Reads `<run-dir>/{before,after}/s7_p24/series.csv` and draws two panels: the price over time (top) and the
mean bread per person (bottom). The price's old shape was a flow integral and its new one is a function of
the store, so the two are drawn on the same axes against the same store series: a plot that cannot show
that is not evidence.
"""
import csv
import pathlib
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt


def series(path):
    rows = list(csv.DictReader(open(path)))
    ticks = [int(r["tick"]) for r in rows]
    return (
        ticks,
        [float(r["price"]) for r in rows],
        [float(r["bread_mean"]) for r in rows],
    )


# A game-day is `seconds_per_day / tick_secs`. The harness fixes `tick_secs` at 1/64 for every
# scenario and `village-evening` sets `seconds_per_day: 64.0`, so a day is **4096** ticks — not 256,
# which is what this said when the axis was first drawn and which put every day number off by 16x.
TICKS_PER_DAY = 64.0 / (1.0 / 64.0)


def main():
    run = pathlib.Path(sys.argv[1])
    out = sys.argv[2]
    fig, axes = plt.subplots(2, 1, figsize=(13, 7), sharex=True)
    colours = {"before": "#b03030", "after": "#2060a0"}
    for side in ("before", "after"):
        path = run / "evidence" / side / "s7_p24" / "series.csv"
        if not path.exists():
            continue
        ticks, price, bread = series(path)
        days = [t / TICKS_PER_DAY for t in ticks]
        axes[0].plot(days, price, colours[side], label=side, linewidth=0.7)
        axes[1].plot(days, bread, colours[side], label=side, linewidth=0.7)
    axes[0].axhline(1.0, color="grey", linestyle="--", linewidth=0.6, label="nominal")
    axes[0].set_ylabel("price of bread")
    axes[0].set_title("village-evening, seed 7, 24 people: the price and the store it is read from")
    axes[0].legend(loc="upper right")
    axes[1].set_ylabel("mean bread per person")
    axes[1].set_xlabel("game days")
    axes[1].legend(loc="upper right")
    fig.tight_layout()
    fig.savefig(out, dpi=130)
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
