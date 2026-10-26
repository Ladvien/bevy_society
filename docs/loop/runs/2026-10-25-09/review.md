# Review: the diagnosis, and a reverted fix

Three passes. This iteration's verdicts are about its **claims**, and the central claim is a negative one.

## Pass 1 — Behavior: does the change do what the paper says?

There is no change. The pass is applied to the diagnosis.

**Claim under test:** "rivalries explode with population because a deed's total hostility grows linearly with the
crowd (`src/reputation.rs` dilutes trust but not affinity, respect or memory importance), and Latané's power law is
the missing law."

**Critique.**

- The **inconsistency is real**, and the source is right. Latané 1981 (*American Psychologist* 36(4): 343-356, DOI
  `10.1037/0003-066X.36.4.343`), verified against Crossref and quoted from its abstract: *"impact should take the
  form of a power function, with the marginal effect of the Nth other person being less than that of the (N−1)th."*
  Diluting one of four effects is not a defensible reading of that.
- The **causal claim is false.** The control in `eval.md` (places scaled to the population) takes rival density
  from 23.1% / 20.1% to **0.00% / 0.02%** with the dilution left exactly as it was. The crowd multiplier is not what
  made the village hostile; a village that cannot seat its people is.
- The dilution change also **overshot on its own terms**: it flattened density by emptying the graph (21.6 rival
  pairs → 1.0 at 24 people), which the plan had named a regression before the measurement was taken.

**Verdict: FAIL for the change, and it is reverted.** The *diagnosis* is GREAT: the mechanism is identified, quoted,
and reproduced across five seeds and three populations, and it is in `docs/loop/needs-decision/a-village-with-no-room.md`.

## Pass 2 — Zoom: read the plots closely

`evidence/plots/capacity.png`, three panels.

- **Left (density vs population).** The red curve rises 7.8% → 23.1% → 20.1% and *falls* from 36 to 48, which is not
  the shape of a monotone scaling law and was the first hint the hypothesis was wrong: if the crowd per deed is the
  driver, 48 people (30 witnesses) must be worse than 36 (21.5). It is not. The green dashed curve (seated) sits on
  the floor at 0.00-0.02%.
- **Middle (activity mix).** `work` 10.3% → 5.5% → 3.6% over capacity and back to 10.3% / 7.9% when seated; `snub`
  essentially flat at 1.2-1.3% over capacity and 0.8-0.9% seated. So the *rate* of hostile acts barely moves while
  the *contact graph* changes: more `compliment` (4.6% → 7.4%) and `gossip` (4.2% → 7.2%) in a crowd, because nobody
  is at work.
- **Right (interruptions).** 318 → 711 → 1 325 over capacity, and 318 → 815 → 1 313 seated: **seating does not
  reduce interrupted arrivals.** I expected it to and the plot says no; that is a separate defect (travel-time
  constants) and is stated as such rather than folded into this story.
- **No clipping, no drift, no dead agents** in any panel. Every population does something; nothing saturates at 0 or
  1 except the seated rival density, which is the finding.

**Verdict: GREAT.** The plot that killed the hypothesis is the most valuable artefact of the iteration.

## Pass 3 — Fresh eyes: a different seed and population

The whole comparison is five seeds at three populations, which is this iteration's version of a fresh-eyes run;
`evidence/before/` and `evidence/seated/` hold every `summary.json`. Independently: seed 99 at 36 people was the
observation that started the iteration (183 rival pairs), and it reproduces under the sweep (`before/36_*`:
137-170 rival pairs).

**Can a host developer be told this without reading the code?** Yes, in three sentences: *"Your village has thirty
seats for work and sleep. Give it forty people and the workshop is always full, so nobody works, so everyone spends
the day standing near each other, and a fifth of all pairs end up hating each other. Give it forty seats instead and
there are no rivalries at all — which is also wrong, so the fix is to make a full place cost something rather than
vanish."*

**Verdict: GREAT** for the diagnosis. The open question is in `needs-decision/`, with three options and a
recommendation, as a structural choice should be.
