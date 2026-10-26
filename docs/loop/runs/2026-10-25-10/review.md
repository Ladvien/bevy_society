# Review: a full place costs something

## Pass 1 — Behavior: does the change do what the paper and the scenario say?

**Claim:** a place with no room should be *expensive*, not absent, so a village that grows past its seats keeps
working and does not turn hostile (Pentheny, *Game AI Pro 2* ch. 17; last iteration's measurement of the defect).

**Critique.**

- The **mechanism** is as claimed. `crowding_price` charges `1 - penalty` only when `used >= capacity`, leaves a
  place with room free, prices nothing else, and keeps `capacity: 0` a prohibition. Four unit tests pin each of
  those, and four integration tests pin the behaviour through real decisions.
- The **scenario observable** — density flat as the population grows — is met at 36 and 48 (6.10% and 3.94%
  against 23.08% and 20.11%), and only partly at 24: the shipped 2.90% is *below* the 7.83% baseline, not equal to
  it. I predicted "an uncrowded village is unchanged" and that prediction was **wrong** — see Pass 2.
- The **`work` observable** is met and better than the target: 12.9% / 11.3% / 9.6% against 10.4% / 5.8% / 3.7%.

**Verdict: GREAT** for the change. Two of three predictions held; the third is recorded as a finding, not smoothed
over.

## Pass 2 — Zoom: read the plots closely

`evidence/plots/capacity-price.png`, three panels, five seeds per point.

- **Left (rival density).** The red curve is the defect: 7.8% → 23.1% → 20.1%, and it *falls* from 36 to 48, which is
  not a scaling law and was the first hint last iteration that the cause was capacity. The green curve is 2.9% →
  6.1% → 3.9%: no explosion, and the shape now follows the population instead of saturating at a wall. It does not
  sit on the 24-person baseline (dotted), and that is the honest gap.
- **Middle (`work` share).** The red line falls 10.4% → 5.8% → 3.7%: the working day disappearing as seats run out.
  Green holds 12.9% → 11.3% → 9.6%. **Green starts above red at 24 people**, which I did not predict. The cause is
  in the mix: `craft` shares the workshop place with `work`, so a 24-person village already fills it part of the day,
  and the price shifts time from `craft` (5.1% → 3.6%) to `work`. That is the mechanism working on a competition
  nobody had measured, not a bug — and it means every earlier nine-seed baseline moved.
- **Right (interruptions).** Both curves rise with population; the green one is slightly *below* the red at 36 and
  48 and slightly **above** at 24. So the change does not fix timed-out arrivals, and at small populations the price
  costs a little: people who would have queued elsewhere now spread out. Stated, not claimed as a win.
- **Nothing clips, saturates at 0 or 1, or drifts.** The only flat region is the `capacity: 0` behaviour, which is
  the prohibition working.

**Verdict: GREAT.** The plot that mattered (left) shows the defect gone, and the panel that shows my wrong
prediction (middle, at 24) is included rather than dropped.

## Pass 3 — Fresh eyes: a different seed and population

The whole comparison is five seeds at three populations, and the population sweep *is* the fresh-eyes run for this
change: the same rule at 24, 36 and 48, with no per-population tuning.

**Can a host developer be told this without reading the code?** Yes: *"A place that is full is now a place that costs
something rather than a place that does not exist. Your villagers still go — the ones with no alternative — and the
ones who have another option take it, so a village that outgrows its seats keeps a working day and does not turn into
a feud. `capacity: 0` still means closed."*

**One thing a host must be told**, and it is in the changelog: this changes every village, including one with room to
spare wherever `craft` and `work` compete for the same place, so previously recorded numbers move.

**Verdict: GREAT.**

## The GIFs

No new feature and no new clip: the seven existing clips are unchanged, because none of them fills a place — the
capture's village has 24 people in 30 seats and the shop is shared only part of the day. `docs/media/goods.gif` and
the other six were re-inspected at frames 0 / 120 / 239 earlier in this run and are byte-identical to the previous
iteration's, since the pack did not change.
