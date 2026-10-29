#!/usr/bin/env python3
"""Plot what the people who stayed felt while a party was away.

    tools/plot_absence.py <series.csv> <out.png>

`series.csv` is the output of `cargo run --release --example absence`. Two panels: the distress each holder
feels toward the first away person (Ada) and the joy at the reunion, and each holder's pleasure. The trip
is shaded.
"""
import csv
import sys
from collections import defaultdict

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

rows = list(csv.DictReader(open(sys.argv[1])))
by = defaultdict(list)
for r in rows:
    if r["away"] == "Ada":
        by[r["holder"]].append(r)
fig, axes = plt.subplots(2, 1, figsize=(9, 6), sharex=True)
for holder, rs in by.items():
    day = [float(r["day"]) for r in rs]
    axes[0].plot(day, [float(r["distress"]) for r in rs], label=f"{holder} distress")
    axes[0].plot(day, [float(r["joy"]) for r in rs], ls=":", label=f"{holder} joy")
    axes[1].plot(day, [float(r["pleasure"]) for r in rs], label=holder)
for ax in axes:
    ax.axvspan(0.5, 3.5, color="grey", alpha=0.15)
    ax.legend(fontsize=7, ncol=2)
axes[0].set_ylabel("feeling toward Ada")
axes[1].set_ylabel("pleasure")
axes[1].set_xlabel("game days (Ada and Bram away 0.5 to 3.5)")
fig.tight_layout()
fig.savefig(sys.argv[2], dpi=110)
