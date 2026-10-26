#!/usr/bin/env bash
# Runs the nine-run crowding grid: seeds 7 42 11 x people 24 36 48 into <side>/s<seed>_p<people>.
#
# Usage: run_crowding_grid.sh <binary> <run-dir> <side>
#
# The identical command that produced `evidence/after/`; only the binary differs, so the comparison is
# the model and not the invocation. `village-evening` builds the same four places (6+10+8+6 = 30 seats)
# for every `--people`, which is the over-capacity condition this change is about.
set -euo pipefail

binary=$1
run_dir=$2
side=$3

out="$run_dir/evidence/$side"
mkdir -p "$out"

for seed in 7 42 11; do
  for people in 24 36 48; do
    dir="$out/s${seed}_p${people}"
    mkdir -p "$dir"
    echo "=== $side seed=$seed people=$people -> $dir"
    "$binary" --scenario village-evening --seed "$seed" --people "$people" --out "$dir" 2>&1 | tail -3
  done
done
