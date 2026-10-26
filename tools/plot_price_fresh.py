#!/usr/bin/env python3
"""Plot the price and the store of a single run — the pass-3 "fresh eyes" panel.

    tools/plot_price_fresh.py <series.csv> <out.png> <title>

`plot_price.py` draws before against after at one seed. This draws one *new* seed and population, because
pass 3 asks whether the shape survives a rerun rather than whether the change helped the run it was tuned
on. The reader is looking for the same three things: no clipping at either end of the band, a price that
falls when the store fills, and a day cycle rather than a random walk.
"""
import csv
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

# `seconds_per_day / tick_secs`; the harness fixes `tick_secs` at 1/64 and `village-evening` sets 64.0,
# so a game-day is 4096 ticks.
TICKS_PER_DAY = 4096.0


def main():
    series, out, title = sys.argv[1], sys.argv[2], sys.argv[3]
    rows = list(csv.DictReader(open(series)))
    days = [int(r["tick"]) / TICKS_PER_DAY for r in rows]
    fig, axes = plt.subplots(2, 1, figsize=(13, 7), sharex=True)
    axes[0].plot(days, [float(r["price"]) for r in rows], "#2060a0", linewidth=0.7)
    axes[0].axhline(1.0, color="grey", linestyle="--", linewidth=0.6, label="nominal")
    axes[0].set_ylabel("price of bread")
    axes[0].set_title(title)
    axes[0].legend(loc="upper right")
    axes[1].plot(days, [float(r["bread_mean"]) for r in rows], "#2060a0", linewidth=0.7)
    axes[1].set_ylabel("mean bread per person")
    axes[1].set_xlabel("game days")
    fig.tight_layout()
    fig.savefig(out, dpi=130)
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
