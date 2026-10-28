//! Directed ties between persons: affinity, trust, respect, familiarity and labels (plan D4).
//!
//! `Ties` is a component rather than Bevy relationships because the edges are weighted, directed
//! and many-to-many. Entries stay sorted by the target's bits so lookups are binary searches and
//! every iteration is deterministic.

use bevy::prelude::*;
use bevy_emerge_rng::DetRng;
use serde::Deserialize;

use crate::core::require_core;
use crate::defs::SocietyDefs;
use crate::groups::{Memberships, shared_faction_bias};
use crate::model::{balance, recall};
use crate::{
    Perception, PersonIndex, SocietyClock, SocietyRng, SocietySettings, SocietySystems, Tuning,
};

/// The label a tie carries, recomputed each tick from affinity and familiarity (plan D4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Reflect, Deserialize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum TieLabel {
    /// Barely known: familiarity below 0.1.
    #[default]
    Stranger,
    /// Known but neither friend nor rival.
    Acquaintance,
    /// Affinity at least 0.4 and familiarity at least 0.3.
    Friend,
    /// Affinity at most -0.4 and familiarity at least 0.2.
    Rival,
}

impl TieLabel {
    /// Classifies a tie. Friend is checked first, then Rival, then Stranger; anything else is an
    /// Acquaintance.
    ///
    /// The four thresholds are design defaults (no source paper gives them); they sit on two
    /// mechanisms the literature does supply: familiarity from mere exposure (Zajonc 1968,
    /// DOI 10.1037/h0025848) and affinity from similarity (Byrne 1961, DOI 10.1037/h0044721).
    pub fn classify(affinity: f32, familiarity: f32, tuning: &crate::Tuning) -> Self {
        Self::classify_with_trust(affinity, familiarity, f32::MAX, tuning)
    }

    /// [`classify`](Self::classify) with the tie's trust, which `Friend` additionally requires.
    ///
    /// **Trust is what separates friends from everyone else**: measured on the village, friend-ties
    /// average trust 0.480 against 0.068 for the rest — sevenfold — while familiarity does not separate
    /// them at all (0.907 against 0.908). The `friend_affinity` bar alone therefore never distinguished
    /// anything; a clamped affinity plus this gate does.
    pub fn classify_with_trust(
        affinity: f32,
        familiarity: f32,
        trust: f32,
        tuning: &crate::Tuning,
    ) -> Self {
        if affinity >= tuning.friend_affinity
            && familiarity >= tuning.friend_familiarity
            && trust >= tuning.friend_trust
        {
            Self::Friend
        } else if affinity <= -tuning.rival_affinity && familiarity >= tuning.rival_familiarity {
            Self::Rival
        } else if familiarity < tuning.stranger_familiarity {
            Self::Stranger
        } else {
            Self::Acquaintance
        }
    }

    /// [`classify_with_trust`](Self::classify_with_trust), except that a tie **already** in a category
    /// keeps it until the value falls past a lower bar.
    ///
    /// **A label is a latitude, not a line.** `classify_with_trust` gives *promotion* bars — what it
    /// takes to become a Friend. Leaving a category asks for a band below that, so a value resting near
    /// the bar does not flip the label every time the model nudges it.
    ///
    /// The shape is social judgment theory's **latitude of acceptance**: a judgement about a person is a
    /// band around an anchor, not a point, and a stimulus just past the anchor's end shifts the band
    /// rather than reversing the judgement — Fagan & Browne (1994: 219-220), reproducing Sherif &
    /// Hovland (1961), corpus doc `10.1007_0-306-47586-3_8`. The source supplies the shape and no
    /// numbers; both bands are **design defaults chosen by measurement**.
    ///
    /// # Two bands, and why they are different widths
    ///
    /// `Tuning::label_hysteresis` guards the **low end** (`stranger_familiarity`), where the noise is
    /// pure decay: measured, `Stranger -> Acquaintance` fired 828 times with a median margin of 0.0027
    /// and a one-tick step of 0.0002. `Tuning::friend_band` guards **friendship**, where the shelf is
    /// narrower and the traffic heavier.
    ///
    /// `village-evening`, seed 7, 16.4 game-days: **523** friendships made and **504** lost, held for a
    /// median of **0.36 game-days** (p10 0.02). The friend bar is 0.30 and the ceiling is 0.35, so a
    /// friendship lives in a **0.05-wide sliver** on a variable whose median tie swings 0.072 — and 7%
    /// of all positive tie-samples sit inside that sliver. **91% of the endings are the affinity gate**
    /// (461 of 504; 43 are trust, 0 familiarity), and the affinity at the ending clusters on exactly
    /// **0.25 and 0.30**, the signature of a single `-0.05` step — `rejection_affinity_penalty`, which
    /// every refused approach pays, at a 38% refusal rate.
    ///
    /// So a friend is held while its affinity has dipped no further than `friend_band` below the bar.
    /// Measured at 0.05, seed 7: total label messages **1 953 -> 1 546** with **rivals unchanged at 42**
    /// (seed 11: 2 080 -> 1 710, rivals 40) — where holding the friend bar *unconditionally* took rivals
    /// to 48, the failure that rejected the previous iteration's attempt. Friendships last **0.36 -> 1.01
    /// game-days** at the median and **504 -> 242** of them end at all.
    ///
    /// **What it does not fix, measured.** A refusal-driven ending survives if the drop is deeper than
    /// the band, and the endings with **no interaction behind them** only fall 56 -> 43: the band holds
    /// the shallow dips, not the causal ones. Fixing those needs a "last exchange" stamp on the tie,
    /// which is a save-format change and therefore a `needs-decision` item, not this function's job.
    ///
    /// With both bands at `0.0` this is exactly `classify_with_trust`.
    pub fn classify_from(
        current: Self,
        affinity: f32,
        familiarity: f32,
        trust: f32,
        tuning: &crate::Tuning,
    ) -> Self {
        // A band wider than the bar itself would make `stranger_familiarity - band` negative and hold a
        // tie as an Acquaintance at familiarity 0, which is a relationship the tie does not have.
        let band = tuning
            .label_hysteresis
            .clamp(0.0, tuning.stranger_familiarity.max(0.0));
        let promoted = Self::classify_with_trust(affinity, familiarity, trust, tuning);
        // **A friendship is held while the dip is shallow.** `friend_band` is the second band, and it is
        // narrower than `label_hysteresis` because the shelf it guards is narrower: see the doc above.
        if promoted != Self::Friend
            && current == Self::Friend
            && affinity >= tuning.friend_affinity - tuning.friend_band
            && trust >= tuning.friend_trust
            && familiarity >= tuning.friend_familiarity
        {
            return Self::Friend;
        }
        // **A rivalry is held the way a friendship is held.** The clause above guards the positive
        // category and this one guards the negative one, for the reason the pack already cites: "bad
        // impressions ... are quicker to form and more resistant to disconfirmation than good ones"
        // (Baumeister et al. 2001, DOI `10.1037/1089-2680.5.4.323`). Measured before this clause
        // existed, a Rival was released at **0.0001** above `-rival_affinity` — one relaxation step —
        // and `became_rival` ran level with `left_rival` at three seeds. The band is clamped at
        // `rival_affinity` so it can never exceed the bar itself, which would make a rivalry permanent.
        let rival_band = tuning.rival_band.clamp(0.0, tuning.rival_affinity.max(0.0));
        if promoted != Self::Rival
            && current == Self::Rival
            && affinity <= -(tuning.rival_affinity - rival_band)
            && familiarity >= tuning.rival_familiarity
        {
            return Self::Rival;
        }
        // **Promotion is never held.** Only leaving the low end is: a tie that climbs is an
        // Acquaintance in the same tick it always was.
        if promoted != Self::Stranger {
            return promoted;
        }
        match current {
            Self::Acquaintance | Self::Friend | Self::Rival
                if familiarity >= (tuning.stranger_familiarity - band).max(0.0) =>
            {
                current
            }
            _ => promoted,
        }
    }

    /// Lowercase name used by scripts (`"stranger" | "acquaintance" | "friend" | "rival"`).
    pub fn script_name(self) -> &'static str {
        match self {
            Self::Stranger => "stranger",
            Self::Acquaintance => "acquaintance",
            Self::Friend => "friend",
            Self::Rival => "rival",
        }
    }
}

/// One directed edge from the owning person to `target` (plan D4).
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct Tie {
    /// The person this edge points at.
    pub target: Entity,
    /// Liking in `[-1, 1]`.
    pub affinity: f32,
    /// Reliance in `[0, 1]`; 0.5 for a new acquaintance.
    pub trust: f32,
    /// Conferred esteem in `[-1, 1]`.
    pub respect: f32,
    /// Familiarity peak in `[0, 1]`, decayed by power-law forgetting when read.
    pub familiarity_peak: f32,
    /// Game-day of the last contact.
    pub last_contact_day: f32,
    /// The label as of the last refresh.
    pub label: TieLabel,
}

impl Tie {
    /// A fresh tie with the plan's defaults: neutral affinity and respect, trust 0.5, no familiarity.
    pub fn new(target: Entity) -> Self {
        Self {
            target,
            affinity: 0.0,
            trust: 0.5,
            respect: 0.0,
            familiarity_peak: 0.0,
            last_contact_day: 0.0,
            label: TieLabel::Stranger,
        }
    }

    /// Adds `delta` to affinity, **clamped to `±Tuning::affinity_ceiling`**.
    ///
    /// The one place affinity moves. Routing every writer through it is what makes the clamp a property
    /// of the model rather than of whichever call site remembered: a new affinity effect cannot
    /// accidentally escape the ceiling, and `grep -n 'affinity =' src` finds no direct write outside
    /// [`Tie::new`], this function, and the two places that set a tie's *initial* value.
    pub fn add_affinity(&mut self, delta: f32, tuning: &Tuning) {
        let ceiling = tuning.affinity_ceiling.max(0.0);
        self.affinity = (self.affinity + delta).clamp(-ceiling, ceiling);
    }

    /// Familiarity at `now_days`, decayed from the peak by power-law forgetting (Anderson &
    /// Schooler 1991, DOI 10.1111/j.1467-9280.1991.tb00174.x; Murre & Dros 2015, DOI
    /// 10.1371/journal.pone.0120644).
    pub fn familiarity(&self, now_days: f32, tuning: &Tuning) -> f32 {
        recall::familiarity(
            self.familiarity_peak,
            now_days - self.last_contact_day,
            tuning.familiarity_tau_days,
            tuning.familiarity_decay_exp,
        )
    }

    /// Records one contact over `dt_days`. Familiarity is first read at `now_days`, then raised by
    /// mere exposure; affinity is nudged toward the target (Zajonc 1968, DOI 10.1037/h0025848).
    pub fn expose(&mut self, now_days: f32, dt_days: f32, bond_scale: f32, tuning: &Tuning) {
        let f = self.familiarity(now_days, tuning);
        self.familiarity_peak = recall::exposed_peak(f, tuning.exposure_per_day, dt_days);
        self.last_contact_day = now_days;
        // Exposure saturates on `(1 − |affinity|)` and now also honours the ceiling, through the one
        // writer. Before the ceiling, this term is what carried a village to affinity 0.999.
        // `bond_scale` is `Tuning::bond_scale`: the friend-capacity brake on a bond not yet formed.
        let step = tuning.mere_exposure_per_day * dt_days * (1.0 - self.affinity.abs()) * bond_scale;
        self.add_affinity(step, tuning);
    }

    /// Relaxes affinity toward zero by `affinity_relax_per_day · dt_days`, never crossing zero.
    ///
    /// Goes through [`add_affinity`](Self::add_affinity) like every other writer, so a ceiling lowered
    /// under an existing tie is respected on the next tick rather than only on the next write.
    pub fn relax(&mut self, dt_days: f32, tuning: &Tuning) {
        let relaxed =
            balance::relax_toward_zero(self.affinity, tuning.affinity_relax_per_day * dt_days);
        self.add_affinity(relaxed - self.affinity, tuning);
    }

    /// Recomputes the label from the current affinity and familiarity. Returns `Some((old, new))`
    /// only when it changed, so the caller can write one `TieLabelChanged` per transition.
    pub fn refresh_label(
        &mut self,
        now_days: f32,
        tuning: &Tuning,
    ) -> Option<(TieLabel, TieLabel)> {
        // **The tie's own label is the state.** A category is entered at its promotion bar and left at
        // the lower bar `Tuning::label_hysteresis` defines, so a value resting on the line does not
        // announce a relationship change every time relaxation crosses it.
        let next = TieLabel::classify_from(
            self.label,
            self.affinity,
            self.familiarity(now_days, tuning),
            self.trust,
            tuning,
        );
        if next == self.label {
            None
        } else {
            let old = self.label;
            self.label = next;
            Some((old, next))
        }
    }
}

/// A person's outgoing ties, sorted by `target.to_bits()` with at most one entry per target.
#[derive(Component, Debug, Clone, Default, PartialEq, Reflect)]
pub struct Ties(pub Vec<Tie>);

impl Ties {
    /// Binary-search result for `target`: `Ok(index)` when present, `Err(insert_at)` otherwise.
    fn search(&self, target: Entity) -> Result<usize, usize> {
        let bits = target.to_bits();
        self.0.binary_search_by_key(&bits, |t| t.target.to_bits())
    }

    /// The tie to `target`, if any.
    pub fn get(&self, target: Entity) -> Option<&Tie> {
        self.search(target).ok().map(|i| &self.0[i])
    }

    /// Mutable access to the tie to `target`, if any.
    pub fn get_mut(&mut self, target: Entity) -> Option<&mut Tie> {
        match self.search(target) {
            Ok(i) => Some(&mut self.0[i]),
            Err(_) => None,
        }
    }

    /// The tie to `target`, inserted with [`Tie::new`] in sorted position when absent.
    pub fn entry(&mut self, target: Entity) -> &mut Tie {
        self.entry_with(target, 0.0)
    }

    /// The tie to `target`. When absent it is inserted with [`Tie::new`] and the given affinity,
    /// in sorted position; an existing tie keeps its affinity. D7 uses this to start co-members at
    /// their faction's `ingroup_bias`.
    pub fn entry_with(&mut self, target: Entity, affinity: f32) -> &mut Tie {
        let index = match self.search(target) {
            Ok(i) => i,
            Err(i) => {
                let mut tie = Tie::new(target);
                // **An initial value, not a delta**, and one of the two direct writes left in the crate
                // (the other is `Tie::new`). `entry_with` has no `Tuning` to clamp against; the caller
                // sets a faction's `ingroup_bias`, which is already in `[-1, 1]`, and the next
                // `add_affinity` brings it under the ceiling.
                tie.affinity = affinity;
                self.0.insert(i, tie);
                i
            }
        };
        &mut self.0[index]
    }

    /// Removes and returns the tie to `target`, if any.
    pub fn remove(&mut self, target: Entity) -> Option<Tie> {
        match self.search(target) {
            Ok(i) => Some(self.0.remove(i)),
            Err(_) => None,
        }
    }

    /// Number of ties.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether there are no ties.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Drops the least-familiar ties until at most `max_ties` remain (Dunbar 1992, DOI
    /// 10.1016/0047-2484(92)90081-J). Ties of equal familiarity go first by lower target bits, so
    /// the result is deterministic.
    pub fn trim(&mut self, now_days: f32, tuning: &Tuning, max_ties: usize) {
        self.trim_keeping(now_days, tuning, max_ties, |_| false);
    }

    /// [`trim`](Self::trim), except a tie for which `frozen(target)` holds is **excluded from the count
    /// and never removed**.
    ///
    /// That is what a dead person's ties need: a town that has lost half its people must not forget them
    /// because the Dunbar cap only counted the living. A cap below the frozen count keeps every frozen
    /// tie and trims the living ones to nothing, which is the honest answer — the frozen set is not the
    /// caller's to trade away.
    pub fn trim_keeping(
        &mut self,
        now_days: f32,
        tuning: &Tuning,
        max_ties: usize,
        frozen: impl Fn(Entity) -> bool,
    ) {
        let live = self.0.iter().filter(|t| !frozen(t.target)).count();
        if live <= max_ties {
            return;
        }
        let excess = live - max_ties;
        let mut ranked: Vec<(f32, u64)> = self
            .0
            .iter()
            .filter(|t| !frozen(t.target))
            .map(|t| (t.familiarity(now_days, tuning), t.target.to_bits()))
            .collect();
        ranked.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        let doomed: Vec<u64> = ranked.iter().take(excess).map(|&(_, bits)| bits).collect();
        self.0
            .retain(|t| frozen(t.target) || !doomed.contains(&t.target.to_bits()));
    }

    /// The `close_ties` targets with the highest `max(0, affinity) · familiarity`, best first.
    /// Ties of equal score go by lower target bits (Zhou et al. 2005, DOI
    /// 10.1098/rspb.2004.2970).
    pub fn close_targets(&self, now_days: f32, tuning: &Tuning, close_ties: usize) -> Vec<Entity> {
        let mut scored: Vec<(f32, u64, Entity)> = self
            .0
            .iter()
            .map(|t| {
                let score = t.affinity.max(0.0) * t.familiarity(now_days, tuning);
                (score, t.target.to_bits(), t.target)
            })
            .collect();
        scored.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        scored
            .into_iter()
            .take(close_ties)
            .map(|(_, _, e)| e)
            .collect()
    }
}

/// Written once per tie whose label changed (plan D4). `from` owns the tie, `to` is its target.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TieLabelChanged {
    /// The person who owns the tie.
    pub from: Entity,
    /// The person the tie points at.
    pub to: Entity,
    /// The label before the change.
    pub old: TieLabel,
    /// The label after the change.
    pub new: TieLabel,
}

/// Records one contact from `ties`' owner toward `target`. `shared_bias` is the faction
/// `ingroup_bias` for co-members, used only when the tie is new (plan D4/D6).
pub(crate) fn expose_toward(
    ties: &mut Ties,
    target: Entity,
    shared_bias: Option<f32>,
    now_days: f32,
    dt_days: f32,
    friends: usize,
    tuning: &Tuning,
) {
    let tie = match shared_bias {
        Some(bias) => ties.entry_with(target, bias),
        None => ties.entry(target),
    };
    let scale = tuning.bond_scale(friends, tie.label == TieLabel::Friend);
    tie.expose(now_days, dt_days, scale, tuning);
}

/// Triadic closure for one direction (Kossinets & Watts 2006, DOI 10.1126/science.1116869): when
/// the owner meets someone who knows `target`, familiarity toward `target` rises to
/// `f + introduction_familiarity · (1 − f)`, where `f` is the current familiarity.
pub(crate) fn introduce_toward(ties: &mut Ties, target: Entity, now_days: f32, tuning: &Tuning) {
    let tie = ties.entry(target);
    let f = tie.familiarity(now_days, tuning);
    tie.familiarity_peak = (f + tuning.introduction_familiarity * (1.0 - f)).clamp(0.0, 1.0);
    tie.last_contact_day = now_days;
}

/// Records every contact pair in `perception.contact` once per direction (plan D4). Mere exposure
/// (Zajonc 1968, DOI 10.1037/h0025848) applies to both sides of each pair.
fn expose_contacts(
    clock: Res<SocietyClock>,
    settings: Res<SocietySettings>,
    perception: Res<Perception>,
    defs: Res<SocietyDefs>,
    memberships: Query<&Memberships>,
    mut ties: Query<&mut Ties>,
) {
    let tuning = &settings.tuning;
    let now = clock.now_days as f32;
    let dt = clock.dt_days;
    for (i, &entity) in perception.persons.iter().enumerate() {
        let Ok(mine) = memberships.get(entity) else {
            continue;
        };
        for &j in &perception.contact[i] {
            let Some(&other) = perception.persons.get(j as usize) else {
                continue;
            };
            let bias = match memberships.get(other) {
                Ok(theirs) => shared_faction_bias(mine, theirs, &defs),
                Err(_) => None,
            };
            if let Ok(mut owned) = ties.get_mut(entity) {
                expose_toward(&mut owned, other, bias, now, dt, tuning);
            }
        }
    }
}

/// Relaxes every affinity toward zero (plan D4), in sorted person order.
fn relax_affinity(
    clock: Res<SocietyClock>,
    settings: Res<SocietySettings>,
    index: Res<PersonIndex>,
    deceased: Query<(), With<crate::Deceased>>,
    mut ties: Query<&mut Ties>,
) {
    let tuning = &settings.tuning;
    for &(_, entity) in &index.sorted {
        // **A tie to or from the dead is frozen**: it stays at what it was when they died. That is what
        // lets the living grieve someone the social graph still remembers, and it is why a dead person's
        // own ties stop moving too — nothing decides for them.
        if deceased.get(entity).is_ok() {
            continue;
        }
        if let Ok(mut owned) = ties.get_mut(entity) {
            for tie in owned.0.iter_mut() {
                if deceased.get(tie.target).is_ok() {
                    continue;
                }
                tie.relax(clock.dt_days, tuning);
            }
        }
    }
}

/// Structural balance (Heider 1946, DOI 10.1080/00223980.1946.9917275; Cartwright & Harary 1956,
/// DOI 10.1037/h0046049). Each sample draws a person A and two of A's ties B and C; when B→C
/// exists, `aff(A→C) += balance_rate · dt · aff(A→B) · aff(B→C)`. Failed draws are skipped.
fn balance_triads(
    clock: Res<SocietyClock>,
    settings: Res<SocietySettings>,
    index: Res<PersonIndex>,
    deceased: Query<(), With<crate::Deceased>>,
    mut rng: ResMut<SocietyRng>,
    mut ties: Query<&mut Ties>,
) {
    let n = index.sorted.len();
    if n == 0 {
        return;
    }
    let tuning = &settings.tuning;
    let dt = clock.dt_days;
    for _ in 0..tuning.balance_samples_per_tick {
        let a = index.sorted[rng.0.below(n)].1;
        let picked = {
            let Ok(a_ties) = ties.get(a) else {
                continue;
            };
            let len = a_ties.len();
            if len < 2 {
                continue;
            }
            let b = a_ties.0[rng.0.below(len)];
            let c = a_ties.0[rng.0.below(len)];
            if b.target == c.target {
                continue;
            }
            let Ok(b_ties) = ties.get(b.target) else {
                continue;
            };
            let Some(bc) = b_ties.get(c.target) else {
                continue;
            };
            (b.affinity, c.target, bc.affinity)
        };
        let (ab, c_target, bc) = picked;
        let delta = balance::triad_delta(tuning.balance_rate_per_day, dt, ab, bc);
        // A frozen tie takes no balance step, in either direction: the dead person's relations are fixed
        // at death, and so is everyone's feeling toward them.
        if deceased.get(a).is_ok() || deceased.get(c_target).is_ok() {
            continue;
        }
        if let Ok(mut owned) = ties.get_mut(a)
            && let Some(ac) = owned.get_mut(c_target)
        {
            ac.add_affinity(delta, tuning);
        }
    }
}

/// **The dyadic half of balance**: each relationship is pulled toward a mutual view.
///
/// Heider's balance theory is usually quoted for triads, and it is just as much a statement about a dyad:
/// "in a dyadic relation, balance basically means reciprocity: feeling positive about a person who doesn't
/// reciprocate this feeling is unbalanced" (Heider 1946, as summarised in Schweighofer & Schweitzer, DOI
/// `10.18564/jasss.5323`, in the corpus). [`balance_triads`] implements the triadic case; this is the
/// simpler one it was missing.
///
/// **Why it matters to behaviour, not just to bookkeeping.** The decision model reads the *actor's* tie
/// (`Signal::Affinity` is `decision.ties`) and the acceptance draw reads the *target's*
/// (`toward_actor.affinity`). Measured before this system, on `village-evening` seed 7 over 16.4
/// game-days: **27%** of directed friend ties were unreciprocated and the two directions of a pair
/// disagreed by **0.133** on average, against a friend band 0.05 wide. It closed the gap to **0.048** on
/// seed 7 and to 0.046 / 0.050 on seeds 42 and 11.
///
/// **The pairs come from each person's tie list, and that is what makes it exact and cheap.** Two earlier
/// versions were measured and rejected. Walking every unordered pair is `O(n^2)` and took the 5 000-person
/// benchmark from **495.7 to 942.0 ms** a tick. Walking `Perception::contact` instead was affordable but
/// **wrong**: only people within `contact_radius` were balanced, so the gap closed to 0.069 rather than
/// 0.048 and rivals fell 42 → 17 instead of 42 → 24, because two people who never stand near each other
/// still disagree about each other. A tie list is capped by `max_ties` (150), so iterating it is bounded
/// per person and visits exactly the pairs that have a relationship to balance — which is the set that
/// matters, because a pair with no tie in either direction cannot be one friend and one rival.
///
/// Each pair is handled once (`a.to_bits() < b.to_bits()`), both directions move by the same amount in
/// opposite signs so the mean is conserved and the step cannot overshoot, and there is no draw and no hash
/// iteration.
fn balance_dyads(
    clock: Res<SocietyClock>,
    settings: Res<SocietySettings>,
    index: Res<PersonIndex>,
    deceased: Query<(), With<crate::Deceased>>,
    mut ties: Query<&mut Ties>,
) {
    let tuning = &settings.tuning;
    if tuning.dyad_balance_per_day <= 0.0 {
        return;
    }
    let dt = clock.dt_days;
    for &(_, a) in &index.sorted {
        if deceased.get(a).is_ok() {
            continue;
        }
        let Ok(a_ties) = ties.get(a) else {
            continue;
        };
        // Every edge from `a`, read into a local list so no borrow is held across the writes.
        let edges: Vec<(Entity, f32)> = a_ties
            .0
            .iter()
            .filter(|tie| a.to_bits() < tie.target.to_bits())
            .map(|tie| (tie.target, tie.affinity))
            .collect();
        for (b, ab) in edges {
            if deceased.get(b).is_ok() {
                continue;
            }
            let Some(ba) = ties.get(b).ok().and_then(|t| t.get(a)).map(|t| t.affinity) else {
                continue;
            };
            let delta = balance::dyad_delta(tuning.dyad_balance_per_day, dt, ab, ba);
            if delta == 0.0 {
                continue;
            }
            if let Ok(mut owned) = ties.get_mut(a)
                && let Some(tie) = owned.get_mut(b)
            {
                tie.add_affinity(delta, tuning);
            }
            if let Ok(mut owned) = ties.get_mut(b)
                && let Some(tie) = owned.get_mut(a)
            {
                tie.add_affinity(-delta, tuning);
            }
        }
    }
}

/// Drops each person's least-familiar ties beyond `max_ties` (plan D4, Dunbar 1992).
fn trim_ties(
    clock: Res<SocietyClock>,
    settings: Res<SocietySettings>,
    index: Res<PersonIndex>,
    deceased: Query<(), With<crate::Deceased>>,
    mut ties: Query<&mut Ties>,
) {
    let now = clock.now_days as f32;
    for &(_, entity) in &index.sorted {
        if let Ok(mut owned) = ties.get_mut(entity) {
            // A frozen tie is excluded from the count and never removed, so a person who has lost many
            // people still keeps every one of them.
            owned.trim_keeping(now, &settings.tuning, settings.max_ties, |target| {
                deceased.get(entity).is_ok() || deceased.get(target).is_ok()
            });
        }
    }
}

/// Recomputes every tie's label and writes one [`TieLabelChanged`] per transition (plan D4).
pub(crate) fn refresh_labels(
    clock: Res<SocietyClock>,
    settings: Res<SocietySettings>,
    index: Res<PersonIndex>,
    deceased: Query<(), With<crate::Deceased>>,
    mut ties: Query<&mut Ties>,
    mut changed: MessageWriter<TieLabelChanged>,
) {
    let now = clock.now_days as f32;
    for &(_, entity) in &index.sorted {
        // A dead person's labels stay as they were at death, and so does the label of a tie pointing at
        // them: the living still think of them as the friend they lost.
        if deceased.get(entity).is_ok() {
            continue;
        }
        let Ok(mut owned) = ties.get_mut(entity) else {
            continue;
        };
        for tie in owned.0.iter_mut() {
            if deceased.get(tie.target).is_ok() {
                continue;
            }
            if let Some((old, new)) = tie.refresh_label(now, &settings.tuning) {
                changed.write(TieLabelChanged {
                    from: entity,
                    to: tie.target,
                    old,
                    new,
                });
            }
        }
    }
}

/// Plugin for the directed-tie model (plan D4). Runs in `SocietySystems::Relate`, after
/// `SocietyCorePlugin`.
pub struct RelationshipsPlugin;

impl Plugin for RelationshipsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TieLabel>()
            .register_type::<Tie>()
            .register_type::<Ties>()
            .add_message::<TieLabelChanged>()
            .add_systems(
                FixedUpdate,
                (
                    expose_contacts,
                    relax_affinity,
                    balance_triads,
                    balance_dyads,
                    trim_ties,
                    refresh_labels,
                )
                    .chain()
                    .in_set(SocietySystems::Relate),
            );
    }

    fn finish(&self, app: &mut App) {
        require_core(app, "RelationshipsPlugin");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(bits: u64) -> Entity {
        Entity::from_bits(bits)
    }

    #[test]
    fn labels_follow_the_d4_thresholds() {
        let tuning = Tuning::default();
        assert_eq!(TieLabel::classify(0.5, 0.5, &tuning), TieLabel::Friend);
        assert_eq!(TieLabel::classify(-0.5, 0.5, &tuning), TieLabel::Rival);
        assert_eq!(TieLabel::classify(0.0, 0.05, &tuning), TieLabel::Stranger);
        assert_eq!(
            TieLabel::classify(0.2, 0.5, &tuning),
            TieLabel::Acquaintance
        );
        assert_eq!(
            TieLabel::classify(0.5, 0.2, &tuning),
            TieLabel::Acquaintance
        );
    }

    #[test]
    fn entries_stay_sorted_by_target_bits() {
        let mut ties = Ties::default();
        for bits in [30_u64, 10, 20] {
            ties.entry(e(bits));
        }
        let order: Vec<u64> = ties.0.iter().map(|t| t.target.to_bits()).collect();
        assert_eq!(order, vec![10, 20, 30]);
    }

    #[test]
    fn trim_drops_least_familiar_first() {
        let tuning = Tuning::default();
        let mut ties = Ties::default();
        ties.entry(e(1)).familiarity_peak = 0.9;
        ties.entry(e(2)).familiarity_peak = 0.1;
        ties.entry(e(3)).familiarity_peak = 0.5;
        ties.trim(0.0, &tuning, 2);
        let kept: Vec<u64> = ties.0.iter().map(|t| t.target.to_bits()).collect();
        assert_eq!(kept, vec![1, 3]);
    }

    #[test]
    fn label_change_is_reported_once() {
        let tuning = Tuning::default();
        let mut tie = Tie::new(e(1));
        tie.affinity = 0.5;
        tie.familiarity_peak = 0.5;
        assert_eq!(
            tie.refresh_label(0.0, &tuning),
            Some((TieLabel::Stranger, TieLabel::Friend))
        );
        assert_eq!(tie.refresh_label(0.0, &tuning), None);
    }
}
