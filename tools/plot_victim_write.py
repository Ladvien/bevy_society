#!/usr/bin/env python3
"""Plot the victim's-write run: friends and rivals over time, at three seeds, before and after.

The question the plot answers: does a village where being harmed costs the relationship lurch into hostility,
or does it settle somewhere a host would believe? Two panels, same axes, so the two seeds are directly
comparable rather than described in prose.
"""
import csv
import json
import pathlib
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

RUN = pathlib.Path(__file__).resolve().parents[1] / "docs" / "loop" / "runs" / "2026-10-27-03"
SEEDS = [7, 11, 42]
COLOURS = {7: "#1f77b4", 11: "#2ca02c", 42: "#d62728"}


def series(tag, seed):
    path = RUN / "evidence" / tag / f"s{seed}" / "series.csv"
    if not path.exists():
        return None
    ticks, friends, rivals = [], [], []
    with path.open() as handle:
        for row in csv.DictReader(handle):
            ticks.append(int(row["tick"]) / 64.0 / 64.0)  # 64 s/day, 64 ticks/s -> game-days
            friends.append(int(row["friends"]))
            rivals.append(int(row["rivals"]))
    return ticks, friends, rivals


def main():
    fig, axes = plt.subplots(1, 2, figsize=(13, 5), sharex=True, sharey=True)
    for axis, tag, title in (
        (axes[0], "before", "before: the victim keeps liking its insulter"),
        (axes[1], "after", "after: harm costs the relationship"),
    ):
        for seed in SEEDS:
            got = series(tag, seed)
            if got is None:
                print(f"missing {tag}/s{seed}", file=sys.stderr)
                continue
            ticks, friends, rivals = got
            axis.plot(ticks, friends, color=COLOURS[seed], lw=1.3, label=f"seed {seed}")
            axis.plot(
                ticks,
                rivals,
                color=COLOURS[seed],
                lw=1.3,
                ls="--",
                alpha=0.85,
            )
        axis.set_title(title)
        axis.set_xlabel("game-day")
        axis.grid(alpha=0.25)
    axes[0].set_ylabel("directed tie edges (solid = friend, dashed = rival)")
    axes[0].legend(loc="upper left", fontsize=8)
    fig.suptitle(
        "village-evening, 24 people, 17.6 game-days: friend and rival edges over time",
        fontsize=11,
    )
    fig.tight_layout()
    out = RUN / "evidence" / "plots" / "victim-write.png"
    out.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(out, dpi=140)
    print(out)

    # The one number the change is about, beside the balance it has to be judged against.
    rows = []
    for tag in ("before", "after"):
        for seed in SEEDS:
            path = RUN / "evidence" / tag / f"s{seed}" / "summary.json"
            if not path.exists():
                continue
            data = json.loads(path.read_text())
            rows.append((tag, seed, data))
    fig, axes = plt.subplots(1, 2, figsize=(12, 4.5))
    labels = [f"{tag}\nseed {seed}" for tag, seed, _ in rows]
    axes[0].bar(
        range(len(rows)),
        [data.get("victims_who_dislike_their_insulter", 0) for _, _, data in rows],
        color=[COLOURS[seed] for _, seed, _ in rows],
    )
    axes[0].set_ylabel("victims holding a negative tie")
    axes[0].set_title("victims who dislike their insulter")
    axes[1].bar(
        range(len(rows)),
        [data.get("insults", 0) for _, _, data in rows],
        color=[COLOURS[seed] for _, seed, _ in rows],
    )
    axes[1].set_ylabel("accepted insults")
    axes[1].set_title("all accepted insults, the denominator")
    for axis in axes:
        axis.set_xticks(range(len(rows)))
        axis.set_xticklabels(labels, fontsize=8)
        axis.grid(alpha=0.25, axis="y")
    fig.tight_layout()
    out = RUN / "evidence" / "plots" / "victims.png"
    fig.savefig(out, dpi=140)
    print(out)

    for tag, seed, data in rows:
        print(
            tag,
            seed,
            "friends",
            data.get("friends"),
            "rivals",
            data.get("rivals"),
            "insults",
            data.get("insults"),
            "victims",
            data.get("victims_who_dislike_their_insulter"),
        )


if __name__ == "__main__":
    main()
