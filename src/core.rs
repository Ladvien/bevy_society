//! Core: persons, stable ids, the society clock, the one RNG, the system sets and perception.
//!
//! Every other plugin in the crate runs inside the [`SocietySystems`] sets configured here, and
//! every simulation loop walks [`PersonIndex::sorted`], so the order of persons is the order of
//! spawning and a deterministic spawn sequence gives deterministic ids.

use std::collections::HashMap;

use bevy::app::{App, FixedUpdate, Plugin, Startup};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::lifecycle::{Add, Remove};
use bevy::ecs::observer::On;
use bevy::ecs::query::Has;
use bevy::ecs::reflect::ReflectComponent;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::{IntoScheduleConfigs, SystemSet};
use bevy::ecs::system::{Commands, Local, Query, Res, ResMut};
use bevy::ecs::world::World;
use bevy::log::{error, warn};
use bevy::math::{Vec2, Vec3};
use bevy::reflect::Reflect;
use bevy::time::{Fixed, Time};
use bevy::transform::components::{GlobalTransform, Transform};
use rand_chacha::ChaCha8Rng;

use crate::beliefs::{Knowledge, Opinions};
use crate::defs::{SocietyDefs, size_person_state};
use crate::groups::{Memberships, Status};
use crate::mind::{Emotions, Memories, Mood, Needs, Personality, Signals};
use crate::relationships::Ties;
use crate::rumor::RumorBook;

/// Which two world axes the society lives on. Positions are projected onto these axes.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SocietyPlane {
    /// Ground plane of a 3D world: `(x, z)`.
    #[default]
    XZ,
    /// Side-on 2D world: `(x, y)`.
    XY,
}

impl SocietyPlane {
    /// Projects a world-space position onto the society plane.
    pub fn project(self, p: Vec3) -> Vec2 {
        match self {
            Self::XZ => Vec2::new(p.x, p.z),
            Self::XY => Vec2::new(p.x, p.y),
        }
    }

    /// Lifts a plane position back into world space at height zero (the other axis).
    pub fn lift(self, p: Vec2) -> Vec3 {
        match self {
            Self::XZ => Vec3::new(p.x, 0.0, p.y),
            Self::XY => Vec3::new(p.x, p.y, 0.0),
        }
    }
}

/// How witnesses judge a rule-breaking actor's trust (see the reputation note).
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AssessmentNorm {
    /// Image scoring (Nowak & Sigmund 1998, DOI 10.1038/31225): trust follows the deed alone.
    ImageScoring,
    /// Standing (Ohtsuki & Iwasa 2004, DOI 10.1016/j.jtbi.2004.06.005): a justified refusal of a
    /// distrusted recipient does not lower the refuser's standing.
    #[default]
    Standing,
}

/// Every model rate of the crate, one named field each. Rates are per game-day unless the name says
/// otherwise.
///
/// A default either comes from a paper, in which case the field's doc says so, or is a design
/// default chosen so a society of a few dozen people visibly moves within a game-day. The research
/// notes record which is which.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub struct Tuning {
    /// Half-life of an emotion's intensity, in game-days (ALMA-style decay; Gebhard 2005,
    /// DOI 10.1145/1082473.1082478).
    pub emotion_half_life_days: f32,
    /// How fast mood is pulled toward the emotion centre, per game-day, scaled by emotion intensity.
    /// Mechanism: ALMA's mood update (Gebhard 2005, DOI 10.1145/1082473.1082478); the baseline it
    /// returns to is the Mehrabian 1996 personality mapping (DOI 10.1080/00049539608259510).
    pub mood_pull_per_day: f32,
    /// How fast mood returns to its personality baseline, per game-day. Mechanism: ALMA's mood
    /// update (Gebhard 2005, DOI 10.1145/1082473.1082478).
    pub mood_return_per_day: f32,
    /// The **positivity offset**: pleasure's resting point above the personality baseline, added to it.
    ///
    /// Cacioppo, Cacioppo & Gollan 2013, DOI `10.1017/S0140525X13002537`: *"the intercept for the positive
    /// activation function (i.e., the approach motivation at zero input) is higher than the intercept for
    /// the negative activation function (producing the positivity offset), and the gain for the negative
    /// activation function is higher than for the positivity activation function (producing the negativity
    /// bias)."*
    ///
    /// The crate had the negativity **gain** — a landed insult gives Anger 0.5 against a compliment's Joy
    /// 0.3, and distress outnumbers joy 3:1 — and no positivity **intercept**. Without it a person with no
    /// emotions at all rests at the personality baseline, which is **zero** for a sampled personality, so
    /// pleasure's ceiling was zero: measured on `village-evening`, its maximum over every person and tick
    /// was exactly `+0.000` and nobody was ever pleased.
    ///
    /// 0.1 is a design default; the paper states the offset's existence and sign, not a magnitude.
    pub positivity_offset: f32,
    /// Strength of pleasure contagion from contacts, per game-day (Hatfield 1993,
    /// DOI 10.1111/1467-8721.ep10770953).
    pub contagion_per_day: f32,
    /// Power-law forgetting time scale of familiarity, in game-days (Anderson & Schooler 1991,
    /// DOI 10.1111/j.1467-9280.1991.tb00174.x; Murre & Dros 2015, DOI 10.1371/journal.pone.0120644).
    pub familiarity_tau_days: f32,
    /// Power-law forgetting exponent of familiarity. Design default (no source paper gives this
    /// number); mechanism: Murre & Dros 2015, DOI 10.1371/journal.pone.0120644.
    pub familiarity_decay_exp: f32,
    /// Familiarity gained by contact, per game-day (mere exposure; Zajonc 1968,
    /// DOI 10.1037/h0025848).
    pub exposure_per_day: f32,
    /// Affinity gained by contact, per game-day, scaled by `1 - |affinity|`.
    ///
    /// **Zajonc 1968 (DOI 10.1037/h0025848) supplies the mechanism, not this rate** — the finding is that
    /// repeated exposure increases liking, not by how much per game-day. The rate is a design default.
    ///
    /// It and [`Self::affinity_relax_per_day`] are two first-order processes on one variable, so affinity
    /// settles where they balance:
    ///
    /// ```text
    /// equilibrium = exposure / (exposure + relax) = 0.05 / (0.05 + 0.01) = 0.833
    /// ```
    ///
    /// `TieLabel::classify` calls a tie a **Friend** at affinity `0.4` — under half that equilibrium — so
    /// proximity alone is *sufficient* for friendship. Measured, affinity reaches **0.999 by tick 9 599** on
    /// `village-evening`.
    ///
    /// **But proximity is not what decides the village's friend count.** Lowering this rate to 0.02, 0.01 and
    /// 0.005 moved friends only **22 → 15 → 15 → 16**, and lowering `balance_rate_per_day` from 0.5 to 0.05
    /// moved it **22 → 25**. What moves it is the *number of exchanges*: measured, two people become a Friend
    /// after **45 interactions**, and affinity keeps climbing to 0.73 by 57 with no diminishing return — so a
    /// village with three times the contact has three times the friends. That is why two separate mechanisms
    /// which added contact each produced a clique. The missing term is not a rate but a **saturating one**;
    /// see `docs/loop/needs-decision/`.
    pub mere_exposure_per_day: f32,
    /// Relaxation of affinity toward zero, per game-day, never crossing zero. Design default (no
    /// source paper gives this number).
    pub affinity_relax_per_day: f32,
    /// Triads sampled per tick for structural balance (Heider 1946, DOI 10.1080/00223980.1946.9917275).
    pub balance_samples_per_tick: usize,
    /// Rate at which a balanced triad pulls the third edge (Cartwright & Harary 1956,
    /// DOI 10.1037/h0046049).
    pub balance_rate_per_day: f32,
    /// Rate at which two people's **directed** affinities are pulled toward their mean, per game-day.
    ///
    /// The **dyadic** half of Heider's balance theory: "in a dyadic relation, balance basically means
    /// reciprocity: feeling positive about a person who doesn't reciprocate this feeling is unbalanced"
    /// (Heider 1946, via Schweighofer & Schweitzer, DOI `10.18564/jasss.5323`, in the corpus). The crate
    /// has always implemented the triadic half (`balance_rate_per_day`) and not this one.
    ///
    /// **Measured before it was added** (`village-evening`, seed 7, 16.4 game-days): **50%** of directed
    /// friend ties unreciprocated, a mean directional disagreement of **0.145** against a friend band 0.05
    /// wide, and **38.9%** of approaches refused — because the decision reads the actor's tie and the
    /// acceptance draw reads the target's. See `docs/loop/runs/2026-10-25-04/eval.md` for the sweep.
    ///
    /// **Design default, chosen by measurement**: the source is qualitative and gives no rate.
    /// `0.0` disables the step exactly.
    pub dyad_balance_per_day: f32,
    /// The most of any one good a person can hold.
    ///
    /// **Design default, chosen by measurement.** No source paper gives a stock ceiling; the game-economy
    /// model in the corpus uses "a maximum capacity of 500 units" for its gnomes (`eth-cgl-ar_games-Zun16a`),
    /// which is a number for a different simulation. This one is sized so a completed task is a visible
    /// fraction of a full store rather than a rounding error.
    pub stock_ceiling: f32,
    /// How much of a good **per person** counts as a full store, for the price the village's stores imply.
    ///
    /// **A design default, and the one number in the price model that is not the paper's.** The corpus's
    /// model reads `r = S_cur / S_max` where `S_max` is "the storage size" of a producer that spends a whole
    /// run filling it (`eth-cgl-ar_games-Zun16a` §3.1.1), so `r` spans the unit interval there. A village's
    /// stores do not: `Tuning::stock_ceiling` is a hard clamp at six units a person and the village holds a
    /// quarter of one each. Measured at seed 7 over 17 game-days, pricing against the clamp put `r` below
    /// **0.12** always and pinned the price in **1.76 – 2.00** — a scarcity index that reads the same in a
    /// glut and a drought, which is precisely the defect the storage-ratio model replaced. At **1.0** a loaf
    /// a head is half price, and that same measure spans **1.0 – 2.0**.
    ///
    /// One unit is a meal, so a loaf a head is a village with a day in hand: the reading is "how many days
    /// does the village have?", which is the question a price is supposed to answer.
    pub price_full_store: f32,
    /// The lowest a price may fall.
    ///
    /// **Design default.** A clamp, not a model: without it a permanently oversupplied good would run to
    /// zero and stop carrying information. The corpus's own model clamps at its `v(r)` extremes, which are
    /// half and double the nominal unit (`eth-cgl-ar_games-Zun16a` §3.1.1), so the band is wider than the
    /// shape needs — a host can tighten it without touching the model.
    pub price_floor: f32,
    /// The highest a price may rise. **Design default**, for the same reason as [`Self::price_floor`].
    pub price_ceiling: f32,
    /// How much of a good a person keeps back: a giver offers a good only when it holds **more than this**.
    ///
    /// **Design default, chosen by reading the log of who gave what.** At 0, and at a gate of "at least one",
    /// a villager's last loaf went to a neighbour ("Vesna holds 0.0") — generous, and not what Henrich et al.
    /// report: the meat that is shared is the surplus. With a whole-loaf gift the reserve of 1.0 means a giver
    /// needs two loaves and is left with at least one. It also stops one unit changing hands back and forth
    /// between two nearly equal stores.
    ///
    /// It is read when a share is **chosen**, not when the pair meets, so a giver who eats on the way can still
    /// hand over its last unit (1 of 76 logged shares at 24 and 36 people).
    ///
    /// **It was sized when a meal was a whole loaf, and a meal is now a quarter** (`eat`'s `consumes`), so this
    /// keeps back **four** meals where it used to keep back one. Measured, it did not change what a village does
    /// at 24 people — shares rose 29 → 110 with the same friend count — but a host that wants the old meaning
    /// should set 0.25. Recorded rather than re-tuned, because the number is host-visible and moving it changes
    /// every village that set it. `docs/loop/runs/2026-10-26-12/review.md`.
    pub trade_reserve: f32,
    /// The smallest gap between two people's valuations at which a trade is worth making.
    ///
    /// **Design default.** Hall & Milgrom's condition is that the parties must have found a *joint surplus*
    /// before bargaining; this is how large it has to be for a village to bother. No paper gives a number.
    pub trade_margin: f32,
    /// Familiarity added to two people by a mutual close friend's introduction. Design default (no
    /// source paper gives this number); mechanism: triadic closure, Kossinets & Watts 2006,
    /// DOI 10.1126/science.1116869.
    pub introduction_familiarity: f32,
    /// Base bounded-confidence threshold. Design default (no source paper gives this number);
    /// mechanism: Deffuant et al. 2000, DOI 10.1142/S0219525900000078.
    pub confidence_base: f32,
    /// Extra confidence per unit of openness. Design default (no source paper gives this number);
    /// mechanism: Deffuant et al. 2000, DOI 10.1142/S0219525900000078.
    pub confidence_openness: f32,
    /// Convergence rate of opinions inside the confidence bound. Design default (no source paper
    /// gives this number); mechanism: Deffuant et al. 2000, DOI 10.1142/S0219525900000078.
    pub influence_mu: f32,
    /// Affinity gained from similar opinions (similarity attraction; Byrne 1961,
    /// DOI 10.1037/h0044721).
    pub similarity_gain: f32,
    /// Pull toward an anchored opinion (Friedkin & Johnsen 1990, DOI 10.1080/0022250X.1990.9990069).
    pub anchor_pull: f32,
    /// Conformity rate toward a strong group majority. Design default (no source paper gives this
    /// number); mechanism: Nowak, Szamrej & Latané 1990, DOI 10.1037/0033-295X.97.3.362; Asch 1956,
    /// DOI 10.1037/h0093718.
    pub conformity_per_day: f32,
    /// Trust gain per unit of deed harm from a witness. Design default (no source paper gives this
    /// number); mechanism: image scoring, Nowak & Sigmund 1998, DOI 10.1038/31225.
    pub witness_gain: f32,
    /// Affinity moved per unit of a deed's harm, by **both** the person it was done to and everyone who saw
    /// it. `0.15` until 2026-10-27-03, when it was a literal inside `witness_reaction` — a rate with no field,
    /// which is the one thing this crate's README says never happens.
    ///
    /// The witness's write is `witness_affinity · approval`, where `approval = norm · sign(harm)`: a witness
    /// *judges* the deed against its own opinion. The patient's write is `witness_affinity · severity`, where
    /// `severity` is the harm itself scaled by [`Self::negativity_ratio`] when it is negative: a patient
    /// *reciprocates* what was done to it, and reciprocation reads what happened rather than what one thinks
    /// of that kind of act. Those are the two halves of indirect reciprocity, and a patient that judged
    /// instead of reciprocating would be moved by nothing whenever it holds no opinion on the topic.
    ///
    /// **Design default** (no source paper gives the weight); mechanism: Baumeister et al. 2001,
    /// DOI `10.1037/1089-2680.5.4.323`, and the image-scoring literature above.
    pub witness_affinity: f32,
    /// How much more a **harmful** deed moves the person it was done to than a helpful one of the same size.
    ///
    /// Baumeister, Bratslavsky, Finkenauer & Vohs 2001, *Bad Is Stronger Than Good*, DOI
    /// `10.1037/1089-2680.5.4.323`, reporting Gottman 1994 (p. 33): *"positive and good interactions must
    /// outnumber the negative and bad ones by at least five to one. If the ratio falls below that, the
    /// relationship is likely to fail and breakup. … Gottman's index suggests that **bad events are on
    /// average five times as powerful as good ones**"*. The same review reports the reciprocity half at
    /// p. 22, citing Wills et al. 1974: *"negative, displeasurable behaviors were reciprocated to a
    /// significant degree, whereas the reciprocation of positive, pleasurable behaviors was weaker and not
    /// significant"*.
    ///
    /// The paper is already cited by the pack's `snub` and already used by [`Self::rival_band`]; this is the
    /// magnitude it supplies, which the crate had nowhere.
    ///
    /// **Applies to the patient only.** A witness already reads the deed through a norm, and scaling a
    /// bystander's reaction by five would multiply every insult seen by a crowd — the runaway
    /// `witness_dilution` exists to damp. Applies only when `harm < 0.0`: a benefit is unscaled, which is what
    /// makes the ratio a ratio.
    ///
    /// `1.0` makes harm and benefit symmetric, which recovers the pre-2026-10-27 behaviour at the patient.
    /// `0.0` switches the harm half off — a victim takes no affinity from an injury and still takes a
    /// benefit — and is pinned by a test, so a host that wants a society without resentment has one number
    /// to set.
    pub negativity_ratio: f32,
    /// How a deed's per-witness cost shrinks as the crowd watching it grows.
    ///
    /// `0.0` applies the full `witness_gain · harm` to every witness, so the deed's total cost grows
    /// linearly with the crowd — while a trust *gain* from an interaction reaches only the two parties. One
    /// `chat` is `+0.10` in total; one insult seen by three people is `-0.42`, and trust in a village can
    /// only fall. Measured, `village-evening`'s mean trust at the moment of an exchange was **0.261** against
    /// the 0.5 `AcceptDef` treats as neutral.
    ///
    /// `1.0` divides the cost by the number of witnesses, so a deed costs the same *in total* however many
    /// people saw it.
    ///
    /// The literature measures the transmission rather than the total: Sommerfeld, Krambeck, Semmann &
    /// Milinski 2007, PNAS, DOI `10.1073/pnas.0704598104` — *"transmission by gossip had a dampening effect
    /// on the cooperation levels compared with acting on direct observation. The more extreme a gossip was,
    /// the less impact it had on the reader"* — and, citing Hess & Hagen, *"an increase in believability
    /// with an increasing number of sources"*. Believed more, moving each recipient less. 0.5 is the
    /// midpoint: a squared-root dilution.
    pub witness_dilution: f32,
    /// Absolute harm above which a deed becomes a rumor. Design default (no source paper gives this
    /// number); mechanism: gossip as reputation, Sommerfeld et al. 2007, DOI 10.1073/pnas.0704598104.
    pub rumor_threshold: f32,
    /// Trust gained from gossip, per unit of valence times belief. Design default (no source paper
    /// gives this number); mechanism: Dunbar 2004, DOI 10.1037/1089-2680.8.2.100.
    pub gossip_trust_gain: f32,
    /// Affinity gained from gossip, per unit of valence times belief. Design default (no source
    /// paper gives this number); mechanism: Dunbar 2004, DOI 10.1037/1089-2680.8.2.100.
    pub gossip_affinity_gain: f32,
    /// Affinity gained between persons with no shared faction. Design default (no source paper gives
    /// this number); mechanism: intergroup contact, Pettigrew & Tropp 2006,
    /// DOI 10.1037/0022-3514.90.5.751.
    pub contact_gain: f32,
    /// Affinity at or above which, with [`Self::friend_familiarity`], a tie is a **friend**.
    ///
    /// All five label thresholds are **design defaults** — no source paper gives them. They were literals
    /// inside `TieLabel::classify` until 2026-12-12, when the README's promise that every rate is a named
    /// `Tuning` field was checked and found broken in two places.
    ///
    /// They are consequential and were invisible. Measured on `village-evening`: friend-ties average affinity
    /// **0.792** against **-0.223** for the rest, but `friend_familiarity` is satisfied by **0.907** against
    /// **0.908** — the familiarity term discriminates nothing at all. See
    /// `docs/loop/needs-decision/friendship-has-no-diminishing-return.md`.
    pub friend_affinity: f32,
    /// The ceiling of `|affinity|`. **Every affinity write clamps to `±` this**, through
    /// [`Tie::add_affinity`](crate::Tie::add_affinity), so no amount of contact can carry a tie past it.
    ///
    /// This is the answer to "everyone becomes everyone's friend": affinity used to be able to reach
    /// 1.0, so every label threshold below 1.0 was eventually passed and a 60-person town with daily
    /// contact became a clique. Capping the variable, rather than raising the threshold, bounds the
    /// network by construction — the plan `docs/loop/needs-decision/friendship-has-no-diminishing-return.md`
    /// measures five other mechanisms that did not work.
    ///
    /// 0.35 is a design default, chosen by measurement: it sits **below** the `0.4` friend threshold, so
    /// contact alone can never make a friend; a friend needs the trust gate below too.
    pub affinity_ceiling: f32,
    /// Trust a tie must reach before `Friend` is even considered, in `[0, 1]`.
    ///
    /// Measured on the village, trust separates friend-ties from the rest **sevenfold** (0.480 against
    /// 0.068) while familiarity does not separate them at all (0.907 against 0.908), so trust is the
    /// second thing a friend is. Design default.
    pub friend_trust: f32,
    /// Familiarity at or above which, with [`Self::friend_affinity`], a tie is a friend.
    pub friend_familiarity: f32,
    /// Affinity at or below `-rival_affinity`, with [`Self::rival_familiarity`], makes a tie a rival.
    pub rival_affinity: f32,
    /// Familiarity a rival needs — deliberately lower than a friend's, as the literal was.
    pub rival_familiarity: f32,
    /// Familiarity below which a tie is a stranger, whatever its affinity.
    pub stranger_familiarity: f32,
    /// Familiarity a tie must **fall** below, past [`Self::stranger_familiarity`], before it returns
    /// to being a stranger: a `Stranger` becomes an `Acquaintance` at `stranger_familiarity`, and an
    /// `Acquaintance` only falls back at `stranger_familiarity - label_hysteresis`.
    ///
    /// **A label is a latitude, not a line.** With one bar, a tie whose familiarity rests on it flips
    /// its label every time the model nudges it, and the host gets a `TieLabelChanged` that reports no
    /// change of relationship. Measured on `village-evening` seed 7 over 16.4 game-days: `Stranger ->
    /// Acquaintance` fired **828** times and back **96**, with a median margin from the bar of
    /// **0.0027** and the familiarity in the flipping tick having moved **0.0002** — a relaxation step.
    /// A weak tie is supposed to be the most stable thing in the graph, not the least ("stable but
    /// changeable": Igarashi, corpus doc `10.1017_cbo9780511894701.022`).
    ///
    /// **Only the low end has a band, and that is measured.** Half of all label messages in the village
    /// are events — one accepted `compliment` or one witnessed deed moving affinity by 0.01 or more in a
    /// single tick — and at those bars the label *should* change. Adding the band to the friend and
    /// rival bars as well cut messages 19% but took rivals from 42 to **48** and friends 18 to **20** at
    /// seed 7, because it holds a friendship after the event that ended it. At 0.02 the low-end band
    /// leaves friends and rivals **identical on all three seeds** and cuts the low-end churn to one
    /// acquaintance per ordered pair, ever.
    ///
    /// Social judgment theory is the mechanism — a judgement about a person has a latitude of
    /// acceptance, and a stimulus just past the anchor shifts the band rather than reversing the
    /// judgement (Sherif & Hovland 1961, via Fagan & Browne 1994: 219-220, corpus doc
    /// `10.1007_0-306-47586-3_8`). **The source gives the shape and no number**: this width is a **design
    /// default chosen by measurement** — see `docs/loop/runs/2026-10-25-02/eval.md`.
    ///
    /// `0.0` restores the single bar exactly, which is the old behaviour a host may want back.
    pub label_hysteresis: f32,
    /// How far below [`Self::friend_affinity`] a friendship's affinity may dip before the friend label
    /// is lost, in affinity units.
    ///
    /// **A friendship lives in a narrow shelf.** `friend_affinity` is 0.30 and
    /// [`Self::affinity_ceiling`] is 0.35, so the whole band a friend can occupy is **0.05 wide** while a
    /// median tie's affinity swings **0.072** over a run. Measured on `village-evening` seed 7 over 16.4
    /// game-days: **523** friendships made, **504** lost, held for a median of **0.36 game-days**; **91%**
    /// of the losses are the affinity gate, and the affinity at the moment of loss clusters on exactly
    /// **0.25 and 0.30** — one `-0.05` step, which is [`Self::rejection_affinity_penalty`] paid by the
    /// person whose approach was refused (38% of approaches are).
    ///
    /// Holding a friend while its affinity dipped into this band cut total label messages **1 953 ->
    /// 1 546** at seed 7 with **rivals unchanged at 42**, and raised the median held friendship from
    /// **0.36 to 1.01 game-days**. Holding the friend bar *unconditionally* instead took rivals to 48,
    /// which is why the band is a number and not an absence.
    ///
    /// **Design default, chosen by measurement** — `docs/loop/runs/2026-10-25-03/eval.md`. Altman &
    /// Taylor 1973 supply the mechanism — a friendship is *maintained*, so its loss should follow a
    /// failure to maintain rather than a small dip — and no number.
    ///
    /// `0.0` is the single bar, `classify_with_trust` exactly.
    pub friend_band: f32,
    /// How far above `-`[`Self::rival_affinity`] a rivalry's affinity may rise before the rival label
    /// is lost, in affinity units. The mirror of [`Self::friend_band`].
    ///
    /// **A rivalry with no band is a label, not a grudge.** Without this clause a Rival was released the
    /// instant its affinity rose above `-rival_affinity` — which affinity relaxation does on its own, at
    /// `affinity_relax_per_day` 0.01 a game-day, and one accepted `compliment` does ten times over.
    /// Measured on `village-evening`, three seeds, 2.93 game-days, pair level: rival spells lasted a
    /// median of **0.14 / 0.29 / 0.38 game-days** and the affinity at the moment of release was
    /// **0.0001-0.0002 above the bar** — one arithmetic step — while `became_rival` (47 / 43 / 19) ran
    /// almost level with `left_rival` (42 / 38 / 19). The village formed rivalries it could not keep, and
    /// no rival spell ended for want of familiarity (0 of 149 released below `familiarity_peak` 0.1).
    ///
    /// **The direction is the pack's own citation, and the crate had it inverted.** `snub`'s
    /// consideration cites Baumeister, Bratslavsky, Finkenauer & Vohs 2001 (DOI
    /// `10.1037/1089-2680.5.4.323`): "bad impressions and bad stereotypes are quicker to form and **more
    /// resistant to disconfirmation** than good ones". Quicker to form is implemented — one insult moves
    /// every witness — and more resistant to disconfirmation was not, because the positive side alone had
    /// a band. The paper's other half, "good may prevail over bad by superior force of numbers", is why
    /// this is a band and not a floor: good still wins on count, which the village already shows.
    ///
    /// The shape is the **latitude of acceptance** the other two bands use — a judgement about a person is
    /// a band around an anchor, and a stimulus just past the anchor's end shifts the band rather than
    /// reversing the judgement (Sherif & Hovland 1961, via Fagan & Browne 1994: 219-220, corpus doc
    /// `10.1007_0-306-47586-3_8`). **The source gives the shape and no number**: this width is a **design
    /// default chosen by measurement** — `docs/loop/runs/2026-10-27-02/eval.md`.
    ///
    /// A rivalry still ends: affinity must climb the whole band, so reconciliation needs a real change
    /// rather than rounding. `0.0` is the single bar, the behaviour before this field existed. The field
    /// is **clamped at [`Self::rival_affinity`]** inside `classify_from`, because a band as wide as the
    /// bar would hold a rivalry at any affinity below zero and one wider than it would hold a tie at any
    /// affinity under [`Self::affinity_ceiling`] — a grudge that no amount of good conduct releases.
    pub rival_band: f32,
    /// Affinity lost by the person whose approach is **refused**, per refusal.
    ///
    /// Design default (no source paper gives it); it was a literal `0.05` in `resolve_interactions` until
    /// 2026-12-12. A refusal is not an injury, so this is small, but it is the only thing that makes a
    /// persistent pursuer reconsider.
    pub rejection_affinity_penalty: f32,
    /// Steepness of the contest logistic. Design default (no source paper gives this number);
    /// mechanism: Bonabeau et al. 1995, DOI 10.1016/0378-4371(95)00064-e.
    pub contest_steepness: f32,
    /// Options below this fraction of the best score are dropped. Design default (no source paper
    /// gives this number); mechanism: dual-utility selection, Dill 2015, DOI 10.1201/b18373-5.
    pub tier_cutoff: f32,
    /// Distress a person feels when the someone it approached **refuses** the interaction, before any
    /// `on_reject` emotion of the definition's own.
    ///
    /// Design default (no source paper gives this number). Mechanism: rejection is an interpersonal
    /// offence and the OCC appraisal for it is distress (Ortony, Clore & Collins, via Gratsch & Marsella
    /// 2004, DOI 10.1016/j.cogsys.2004.02.002). It was a literal `0.3` in `exchange`, which is the crate's
    /// only unnamed emotion intensity — every other one is a deed's `harm` or a `groups.rs` constant.
    ///
    /// Worth measuring before raising: this is the largest single source of negative emotion in the
    /// village. Measured over 6 000 ticks and 8 people, **112 distress appraisals against 10 joy**, and
    /// the ratio, not the magnitude, is what leaves every mood's pleasure at or below zero.
    pub rejection_distress: f32,
    /// Familiarity with the dead that counts as close enough to grieve them, even without a `Friend`
    /// label. A design default: the papers state that grief follows the loss of a close relationship
    /// (Prigerson et al. 2009, DOI `10.1371/journal.pmed.1000121`) rather than what familiarity number
    /// that is, so this is the crate's own line between an acquaintance and someone you knew.
    ///
    /// It exists so a **family member** grieves: they are spawned with high familiarity and no time to
    /// reach the affinity a `Friend` label needs, and losing a parent should not be a stranger's death.
    pub bereavement_familiarity: f32,
    /// The most distress a holder feels at a close person's absence, at a closeness of 1: the ceiling of
    /// the *missing* a [`crate::Away`] person leaves behind (issue #35). `0.0` switches absence's social
    /// weight off entirely, reunion included, and leaves every society bit-identical to one that never
    /// had `Away`.
    ///
    /// Design default (no source paper gives this number); the mechanism is separation distress as a
    /// defining mark of an attachment bond (Hazan & Shaver 1987, DOI `10.1037/0022-3514.52.3.511`). 0.4
    /// is a mood the mood word shows without reaching the 0.6 a close friend's death gives.
    pub missing_cap: f32,
    /// Days of absence after which a holder has felt `1 - 1/e` of the ceiling: the time constant of how
    /// fast missing grows, `cap · closeness · (1 − e^(−days / this))`. Design default: a few days
    /// away is a felt absence, which is the length an expedition runs.
    pub missing_days: f32,
    /// The share of what a holder was missing that comes back as `Joy` when the person returns. Design
    /// default: a reunion is the missing turned over, so it can never exceed it.
    pub reunion_fraction: f32,
    /// Hours a person may travel toward an activity before giving up.
    /// Game hours a person may travel toward an activity before giving up, or `None` for a walk that
    /// never times out.
    ///
    /// `None` is what a host wants under [`Locomotion::Host`]: it owns the path, so only it can say a
    /// destination is unreachable, and it says so with [`TravelFailed`](crate::TravelFailed). A timeout
    /// left in place would end activities the host was still walking toward.
    pub travel_timeout_hours: Option<f32>,
    /// Game hours a travelling person may go **without getting closer** to its destination before the
    /// activity ends interrupted, or `None` for no such limit.
    ///
    /// **The timeout a `Locomotion::Host` wants** (issue #26). [`travel_timeout_hours`](Self::travel_timeout_hours)
    /// bounds the *total* walk, so a host that owns the path leaves it `None` — a walk across a town takes
    /// game-hours and a total limit would end walks still being made — and then a destination nobody can reach
    /// (a point inside a building's collision boxes, a ring of standing people the solver treats as a wall)
    /// never ends: `arrived_day` is never set, the duration never starts, the person aims at the same point
    /// for the rest of the run. This one measures the thing that distinguishes "slow" from "stuck": the crate
    /// tracks the **closest the person has been to its target**, and the game-day that distance last improved
    /// by more than [`stall_epsilon`](Self::stall_epsilon). A walker improves it every few ticks, however slowly;
    /// a person standing in a queue or against a wall does not. When the last improvement is older than this,
    /// the activity ends exactly as a travel timeout ends it (`ActivityEnded { completed: false }`, the need
    /// left unmet).
    ///
    /// A **design default**, `None`: no paper gives a stall limit, it is a host's judgement of how long a
    /// queue is allowed to be. A `Person` target moves, so progress is measured against its current point and
    /// a chase that closes counts. A save does not carry the tracker; a restored walker gets a fresh window.
    /// [`TravelFailed`](crate::TravelFailed) is unaffected.
    pub stall_timeout_hours: Option<f32>,
    /// How much closer to its destination, in the plane's units, a travelling person must get to count as
    /// progress for [`stall_timeout_hours`](Self::stall_timeout_hours). A **design default** of 0.25 m, a
    /// fraction of a body diameter (0.7 m): enough to ignore the solver's sideways jitter, small enough
    /// that any real walk beats it within a few ticks.
    pub stall_epsilon: f32,
    /// How long, in game-hours, a person avoids a **place** whose walk just ended before arrival
    /// (travel timeout, stall timeout or a host's [`TravelFailed`](crate::TravelFailed)). Without it a
    /// person whose nearest tavern is unreachable re-chooses that same tavern the moment the walk ends and
    /// spends the day walking at a wall, because a `Place` target is always the nearest tied place. With
    /// `Some(h)` the place is dropped from that person's candidate targets for `h` game-hours, so the next
    /// choice is the next-nearest place, or another activity if none is left.
    ///
    /// A **design default**, `None`: no paper gives a retry delay; it is a host's judgement of how long a
    /// closed route stays closed (reinforcement-learning agents likewise stop re-trying an action whose last
    /// outcome was a failure, but no corpus paper fixes the interval). Only a `Place` target is remembered;
    /// a `Person` target moves and is chosen afresh. Not saved: a restored person forgets.
    pub failed_place_memory_hours: Option<f32>,
    /// How long, in game-hours, a person who has just attempted a **harmful** interaction (one whose deed
    /// has `harm < 0`, such as an insult) will not choose another harmful activity against the same
    /// target, accepted or refused. Measured on the village, 73% of snubs answered an insult the snubber
    /// had *received*, and the reply gave the first insulter a fresh grievance: a standing exchange of
    /// insults, not one retaliation. With `Some(h)` the answer to a grievance is one blow per `h` hours
    /// per ordered pair, so the exchange cannot sustain itself faster than that.
    ///
    /// A **design default**, `None`: negative reciprocity is documented (Gouldner 1960, DOI
    /// 10.2307/2092623) but no paper fixes a cooling-off interval. Only the choice of a harmful activity
    /// is held back; a host's `Interact` is never refused by it. Not saved.
    pub retaliation_refractory_hours: Option<f32>,
    /// Walking speed in metres per **real** second, used by the core walker that moves a person
    /// toward its `Destination` when no `SocialMover` is steering. 1.4 m/s is an ordinary human
    /// walking speed. `SocialMover::speed` overrides it per person.
    pub walk_speed: f32,
    /// Game-days between two contact-gossips along one directed tie. A shared place is one
    /// conversation, not one per fixed step; the gate is the teller having news, this is only pacing.
    pub contact_gossip_cooldown_days: f32,
    /// How far a crowded `Place`'s arrival radius grows per `sqrt(person)` standing at it, in the plane's
    /// own units (the same units as `interaction_radius`; a host in feet sets two body radii in feet).
    /// "At it" is the larger of the bodies inside `interaction_radius` and the people who have arrived
    /// there, so those admitted beyond the disc keep widening the allowance.
    ///
    /// Design default (no paper). `SocialMover::radius` is 0.35 m, so `2 * 0.35 = 0.70 m` is the distance
    /// ORCA holds two people's centres apart — one body diameter is therefore the queue spacing, and a disc
    /// holding `n` people legitimately reaches `interaction_radius + this * sqrt(n)` before it is full.
    ///
    /// **Why it is needed.** Everyone goes to bed at the same hour, so every resident of a home aims at the
    /// same coordinate inside `interaction_radius`. Measured on `village-evening`: at every timed-out `sleep`
    /// the goal already held a median of **6** bodies, and a 1.5 m disc seats only about `(1.5 / 0.7)^2 =
    /// 4.6`. The queue behind the disc then expires its whole `travel_timeout_hours`.
    ///
    /// **Measured effect**, three seeds, same village, 17 game-days — interrupted endings and `sleep`s:
    ///
    /// | seed | interrupted | `sleep` | friends | rivals |
    /// |---|---|---|---|---|
    /// | 7 | 401 → **329** | 169 → **129** | 15 → 18 | 28 → 42 |
    /// | 42 | 417 → **369** | 176 → **139** | 19 → 14 | 25 → 35 |
    /// | 11 | 438 → **383** | 187 → **148** | 14 → 9 | 30 → 40 |
    ///
    /// Interruptions fall **12-18%** and `sleep` falls by 21-24% on every seed. Rivalries rise by 10-14,
    /// which `Tuning::affinity_ceiling` bounds — before the clamp the same change took 38 rivals to 61.
    /// Set this to `0.0` to restore plain `interaction_radius` arrival exactly.
    pub crowded_arrival_growth: f32,
    /// What a place with **no free seat** costs an option's score: the fraction of the score removed.
    ///
    /// **A price, not a prohibition.** Target expansion used to drop a place whose `used >= capacity`, so a full
    /// workshop made `work` an option that did not exist. Measured, a 30-seat village given 48 people took `work`
    /// from 10.3% to 3.6% of a life and rival density from 7.8% to 20.1% of directed pairs. Removing the veto
    /// fixes both — and raises interrupted arrivals 34% at 24 people, because more people now aim at a place that
    /// cannot seat them.
    ///
    /// The repair is the one Pentheny names in *Game AI Pro 2* ch. 17 (corpus
    /// `gameaipro2-ch17-advanced-techniques-robust-efficient-crowds`): "agents fight the rest of the crowd to follow
    /// their ideal path as closely as possible, **seemingly ignoring all other possible routes**", and with a
    /// congestion map "agents ... **reconsider their current path choice if a less-congested alternative
    /// exists**". A full place is *expensive*, so a person with an alternative takes it and a person with none
    /// still arrives.
    ///
    /// **The price is a step at "no room", not a ramp.** A proportional price (`1 - penalty · fill`) was measured
    /// first and rejected: it diverts people from a place that is merely 9/10 full, where nobody queues, and cost
    /// `work` **10.3% → 6.6%** at 48 people — reintroducing the defect it was meant to cure. The queue exists only
    /// at full, so the price is charged there.
    ///
    /// `capacity: 0` stays a **prohibition**: a place that seats nobody has no room at any price.
    ///
    /// **Design default, chosen by measurement** (`docs/loop/runs/2026-10-25-10/eval.md`). `0.0` restores the
    /// un-priced ranking; `1.0` makes a full place score nothing, which is close to the old veto but keeps it
    /// available when it is the only option.
    pub crowding_penalty: f32,
}

impl Tuning {
    /// Affinity as an **influence weight in `[-1, 1]`**, scaled so the clamp ceiling reads as 1.0.
    ///
    /// The clamp bounds how much a person can like anyone, which is what keeps friendship scarce — but
    /// the opinion model reads affinity as *how much weight this person's view carries*, and if it read
    /// the raw clamped value it would read at most 0.35 where the model was tuned for 1.0, so opinions
    /// would barely move and the whole society would agree on nothing. Dividing by the ceiling restores
    /// the scale the model was designed against while keeping the network bounded.
    ///
    /// `0.0` when the ceiling is not positive: a caller that has disabled the clamp has no scale to
    /// restore, and multiplying by nothing is worse than multiplying by zero.
    pub fn affinity_weight(&self, affinity: f32) -> f32 {
        if self.affinity_ceiling <= 0.0 {
            return 0.0;
        }
        (affinity / self.affinity_ceiling).clamp(-1.0, 1.0)
    }
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            emotion_half_life_days: 0.1,
            mood_pull_per_day: 4.0,
            mood_return_per_day: 1.0,
            positivity_offset: 0.1,
            contagion_per_day: 0.3,
            familiarity_tau_days: 1.0,
            familiarity_decay_exp: 0.5,
            exposure_per_day: 2.0,
            mere_exposure_per_day: 0.05,
            affinity_relax_per_day: 0.01,
            balance_samples_per_tick: 8,
            balance_rate_per_day: 0.5,
            // **Design default, chosen by measurement.** 0.5 closes the directional affinity gap
            // 0.133 -> 0.048 over 16 game-days while leaving the village's shape recognisable; the
            // sweep and the trade it makes are in `docs/loop/runs/2026-10-25-04/eval.md`.
            dyad_balance_per_day: 0.5,
            // **Design defaults, chosen by measurement** — `docs/loop/runs/2026-10-25-07/eval.md`.
            stock_ceiling: 6.0,
            // **Design default, chosen by measurement** — `docs/loop/runs/2026-10-26-11/eval.md`.
            price_full_store: 1.0,
            price_floor: 0.25,
            price_ceiling: 4.0,
            // **Design defaults, chosen by measurement** — `docs/loop/runs/2026-10-25-08/eval.md`.
            trade_reserve: 1.0,
            trade_margin: 0.1,
            introduction_familiarity: 0.1,
            confidence_base: 0.2,
            confidence_openness: 0.6,
            influence_mu: 0.3,
            similarity_gain: 0.05,
            anchor_pull: 0.1,
            conformity_per_day: 0.2,
            witness_gain: 0.2,
            witness_affinity: 0.15,
            // Baumeister et al. 2001 (Gottman's 5:1). See the field's doc.
            negativity_ratio: 5.0,
            witness_dilution: 0.5,
            rumor_threshold: 0.3,
            gossip_trust_gain: 0.3,
            gossip_affinity_gain: 0.2,
            contact_gain: 0.02,
            friend_affinity: 0.3,
            affinity_ceiling: 0.35,
            friend_trust: 0.5,
            friend_familiarity: 0.3,
            // **Also under the clamp ceiling**, for the same reason `friend_affinity` is: `0.4` was
            // unreachable once affinity is bounded at `±affinity_ceiling`, so a rival could never be
            // labelled and the network was one-sided. Measured: at 0.4 a 60-person village over 30 days
            // produced **155 friend ties and 0 rivals**; at 0.30 it produces both.
            rival_affinity: 0.3,
            rival_familiarity: 0.2,
            stranger_familiarity: 0.1,
            // **Design default, chosen by measurement** — `docs/loop/runs/2026-10-25-02/eval.md`.
            // The sweep: 0.00 removes nothing, 0.02 removes the low-end churn and moves no friend or
            // rival count on any of three seeds, and 0.04 / 0.08 start holding real friendships.
            label_hysteresis: 0.02,
            // **Design default, chosen by measurement.** 0.05 holds a friendship through one refusal
            // without creating one: measured, rivals unchanged, friends 18 -> 22 at seed 7.
            friend_band: 0.05,
            // **Design default, chosen by measurement** — `docs/loop/runs/2026-10-27-02/eval.md`. The
            // release margin it replaces was 0.0001, and the sweep is below.
            rival_band: 0.1,
            rejection_affinity_penalty: 0.05,
            contest_steepness: 4.0,
            tier_cutoff: 0.25,
            rejection_distress: 0.3,
            bereavement_familiarity: 0.3,
            missing_cap: 0.4,
            missing_days: 2.0,
            reunion_fraction: 0.75,
            travel_timeout_hours: Some(2.0),
            stall_timeout_hours: None,
            stall_epsilon: 0.25,
            failed_place_memory_hours: None,
            retaliation_refractory_hours: Some(6.0),
            walk_speed: 1.4,
            // **A design default, set by measurement.** Not from a paper; the *gate* is the paper. Every
            // contact-tell that finds an informed neighbour stifles the teller (Daley–Kendall), so pacing
            // is what protects the spreader pool. Measured on the village, same seed:
            //
            //   cooldown | friends | rivals | rumor reach
            //   0.5 d    |      23 |     45 |       0.375   <- burns spreaders, reach *falls* 35%
            //   2.0 d    |      25 |     28 |       0.875   <- better than baseline on both axes
            //   6.0 d    |      22 |     33 |       0.583   <- identical to no contact gossip at all
            //   none     |      22 |     33 |       0.583
            //
            // Two days is the setting where contact gossip *adds* something: reach 50% above the shipped
            // 0.583 while rivals fall from 33 to 28. At six days the system costs compute and changes nothing.
            contact_gossip_cooldown_days: 2.0,
            crowded_arrival_growth: 0.7,
            crowding_penalty: 0.6,
        }
    }
}

/// A skill's level for a person, in `[0, 1]`: the trait its group reads, as it stands.
///
/// **A task is not a skill.** Acemoglu & Autor (2010, DOI `10.3386/w16082`): "a task is a unit of work
/// activity that produces output ... a skill is a worker's endowment of capabilities for performing various
/// tasks", and the allocation of one to the other is what a labour market settles. The crate models the
/// task as an [`ActivityDef`](crate::ActivityDef) and the endowment as this number, read from a Big Five
/// trait so that it needs **no stored state**: two people with the same personality are equally good at the
/// same trade, and nobody carries a skill vector to save.
///
/// The trait-to-group mapping is the crate's own choice (a **design default**; no paper in the corpus
/// assigns traits to trades) and lives in [`SkillGroup::trait_of`](crate::defs::SkillGroup).
///
/// No gating: a low-skill person may still take a task. Comparative advantage, not assignment.
pub fn skill_level(traits: &crate::Personality, group: crate::defs::SkillGroup) -> f32 {
    traits.value(group.trait_of()).clamp(0.0, 1.0)
}

/// Where the society clock's time comes from.
#[derive(Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ClockSource {
    /// The crate's own: each tick advances `now_days` by `tick_secs / seconds_per_day`.
    #[default]
    Fixed,
    /// The host's. Each tick reads [`HostClock`] and takes the change since the last tick, so a day of
    /// unequal-length phases (a dawn of 5 s for one game hour, a night of 180 s for ten) stays in step
    /// with the game's own hour.
    Host,
}

/// The host's game time, in days. Write it every frame; the crate never inserts it.
///
/// Only read when [`SocietySettings::clock`] is [`ClockSource::Host`]. Ticks still fire every
/// `tick_secs` of fixed-step time, so a host that jumps its clock forward (skipping a phase) applies the
/// whole jump in one tick rather than catching up tick by tick.
#[derive(Resource, Reflect, Debug, Clone, Copy, Default, PartialEq)]
pub struct HostClock {
    /// Game time in days. The hour of the day is its fractional part times 24.
    pub now_days: f64,
}

/// Who moves a person's `Transform` toward its [`Destination`](crate::Destination).
#[derive(Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Locomotion {
    /// The crate's own: every person with a `Destination` is walked at
    /// [`Tuning::walk_speed`](crate::Tuning) metres per real second, straight at the target.
    #[default]
    Builtin,
    /// The host's. The society decides *where* a person goes and the host decides *how* they get
    /// there — along its street grid, on terrain height, around its houses, and avoiding figures the
    /// society does not own. **No crate system writes a person's `Transform`**; `Destination` is the
    /// request to read, and the arrival check still reads the `Transform` the host moved.
    Host,
}

/// Global settings of the society. Insert before adding [`SocietyCorePlugin`] to override defaults.
#[derive(Resource, Reflect, Debug, Clone)]
pub struct SocietySettings {
    /// Seed of the one [`SocietyRng`] stream.
    pub seed: u64,
    /// Seconds of fixed-step time per society tick.
    pub tick_secs: f32,
    /// Seconds of real time that make one game-day.
    pub seconds_per_day: f32,
    /// Hour of the game-day at which the clock starts.
    pub start_hour: f32,
    /// Where the clock's time comes from.
    pub clock: ClockSource,
    /// Who moves a person toward its `Destination`.
    pub locomotion: Locomotion,
    /// Plane on which positions are projected.
    pub plane: SocietyPlane,
    /// Radius within which a person perceives others (metres on the plane).
    pub perception_radius: f32,
    /// Radius within which two persons are in contact (conversation, conformity, exposure).
    pub contact_radius: f32,
    /// Distance at which a person counts as arrived at an interaction target.
    pub interaction_radius: f32,
    /// Maximum ties a person keeps (Dunbar 1992, DOI 10.1016/0047-2484(92)90081-J).
    pub max_ties: usize,
    /// Size of the close-tie layer (Zhou et al. 2005, DOI 10.1098/rspb.2004.2970).
    pub close_ties: usize,
    /// Memories kept per person before eviction. Design default (no source paper gives this
    /// number).
    pub memory_capacity: usize,
    /// Rumors one person keeps before the least useful is forgotten. Design default, on the same
    /// footing as [`Self::memory_capacity`]: a person cannot hold every rumor the world ever
    /// planted, and `Knowledge::best_telling` scans all of them on every gossip, so an unbounded
    /// list made gossip cost grow with the length of the run. Measured: 344 entries per person after
    /// 4 000 ticks on 12 people. Dunbar 2004 (DOI 10.1037/1089-2680.8.2.100) is the mechanism for
    /// bounded social memory generally.
    pub knowledge_capacity: usize,
    /// Assessment norm used when a witness judges a deed.
    pub assessment: AssessmentNorm,
    /// Probability that an interaction or activity produces a bark.
    pub bark_chance: f32,
    /// Every model rate.
    pub tuning: Tuning,
}

impl Default for SocietySettings {
    fn default() -> Self {
        Self {
            seed: 0x5EED_5EED,
            tick_secs: 0.25,
            seconds_per_day: 240.0,
            start_hour: 8.0,
            clock: ClockSource::Fixed,
            locomotion: Locomotion::Builtin,
            plane: SocietyPlane::XZ,
            perception_radius: 8.0,
            contact_radius: 4.0,
            interaction_radius: 1.5,
            max_ties: 150,
            close_ties: 15,
            memory_capacity: 64,
            knowledge_capacity: 64,
            assessment: AssessmentNorm::Standing,
            bark_chance: 0.35,
            tuning: Tuning::default(),
        }
    }
}

/// The society clock. `now_days` is the continuous game time in days; `dt_days` is the length of
/// one society tick in days.
#[derive(Resource, Reflect, Debug, Clone, Copy, Default, PartialEq)]
pub struct SocietyClock {
    /// Number of society ticks run so far.
    pub tick: u64,
    /// Game time in days.
    pub now_days: f64,
    /// Length of one society tick in game-days.
    pub dt_days: f32,
}

impl SocietyClock {
    /// Hour of the current game-day, in `[0, 24)`.
    pub fn hour(&self) -> f32 {
        (self.now_days.rem_euclid(1.0) * 24.0) as f32
    }

    /// Whole game-days elapsed.
    pub fn day(&self) -> u32 {
        self.now_days.floor() as u32
    }
}

/// Accumulates fixed-step time and says whether a society tick is due this step.
#[derive(Resource, Reflect, Debug, Clone, Copy, Default, PartialEq)]
pub struct SocietyTick {
    /// Fixed-step time not yet spent on a tick, in seconds.
    pub accumulator: f32,
    /// True during the fixed steps in which a society tick runs.
    pub due: bool,
}

/// The only randomness in the crate: a ChaCha8 stream seeded from [`SocietySettings::seed`].
#[derive(Resource, Clone, Debug)]
pub struct SocietyRng(pub ChaCha8Rng);

/// Next [`PersonId`] to hand out. Ids follow spawn order.
#[derive(Resource, Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NextPersonId(pub u32);

/// Stable, deterministic id of a person, assigned in spawn order.
#[derive(Component, Reflect, Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PersonId(pub u32);

/// The host's own stable identity for a person, written by the host before a save is taken.
///
/// A [`PersonId`] is an index into one session's spawn order, so it cannot address anybody across a
/// reload; a `PersonKey` is Sanctuary's roster id, which the host already has and which outlives the
/// session. Everything the crate saves as a reference to a person — a tie's target, an emotion's or
/// memory's subject, a rumor's people, the person a `Doing` is directed at — is stored as this key.
///
/// The crate never writes one. A person spawned without a key is simply not saved: see
/// [`capture`](crate::capture).
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[reflect(Component)]
pub struct PersonKey(pub u64);

/// The host's own stable identity for a [`Place`], written by the host before a save is taken.
///
/// A `Doing` directed at a place is saved as this key, because a place's `Entity` is as
/// session-local as a person's. Like [`PersonKey`], the crate never writes one.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[reflect(Component)]
pub struct PlaceKey(pub u64);

/// Marker of a simulated person. Adding it gives the person every per-person component.
#[derive(Component, Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[require(
    Personality,
    Mood,
    Emotions,
    Needs,
    Signals,
    Ties,
    Opinions,
    Knowledge,
    Memories,
    Memberships,
    Status,
    Transform,
    GlobalTransform
)]
pub struct Person;

/// Marker: this person chooses its own activities. A player-controlled person is a [`Person`]
/// without this marker.
#[derive(Component, Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Autonomous;

/// Marker: this person is out of its daily life for now — posted on a wall, sheltering indoors, away on
/// an expedition, fleeing, or in a conversation with the player.
///
/// A dormant person **keeps who they are**: ties, memories, opinions, knowledge and memberships are
/// untouched, and needs keep decaying and emotions keep relaxing, because time still passes for them.
/// What stops is the social machinery around them:
///
/// - the crate never chooses an activity for them;
/// - they are in nobody's [`Perception::near`] or [`Perception::contact`], so no familiarity accrues
///   with them, they are never a `Person` target, and they are never a witness or a mood source. They
///   keep their slot in [`Perception::persons`] and [`Perception::pos`], because scripts and the
///   snapshot index by slot and a shifted list would point every `ctx` call at the wrong person;
/// - an activity they were running ends with `ActivityEnded { completed: false }`, and so does anyone
///   else's activity that targeted them;
/// - no social exchange happens with them, including one the host asked for directly with
///   [`crate::Interact`].
///
/// Removing the marker lets them re-decide on the next tick. Removing `Person` instead destroys them:
/// `on_person_removed` deletes every tie to and from them.
///
/// Compare [`Deceased`], which is this plus forever.
#[derive(Component, Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[reflect(Component)]
pub struct Dormant;

/// Marker: this person is dead, and stays dead.
///
/// It implies everything [`Dormant`] does — out of its daily life, out of everyone's perception, no
/// exchanges — and additionally **freezes the ties that point at them and the ties they hold**: they are
/// kept, never relaxed, never trimmed, and stay readable through [`Ties::get`](crate::Ties::get). That is
/// what lets a game keep the town roster intact (Sanctuary's dead come back as the walking dead under
/// their own names) and what lets the living grieve someone the social graph still remembers.
///
/// Inserting it writes [`crate::PersonDied`] once and makes everyone close to them grieve. Removing it is
/// not required to restore anything: a death is not an undo.
///
/// Prigerson et al. 2009 (DOI `10.1371/journal.pmed.1000121`) is the mechanism for *why* the reaction is
/// proportional to closeness: prolonged grief disorder's own criteria are keyed to the loss of a close
/// relationship, not to any death.
#[derive(Component, Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[reflect(Component)]
#[require(Dormant)]
pub struct Deceased;

/// Marker: this person is out of the town and expected back — on an expedition, a voyage, a posting.
///
/// It implies everything [`Dormant`] does, and adds absence's social weight (issue #35): while it
/// lasts, every living holder who is **close** to the away person (a `Friend` tie, or a
/// `familiarity_peak` at [`Tuning::bereavement_familiarity`], the same line grief uses) *misses* them. The
/// crate keeps a [`Distress`](crate::EmotionKind::Distress) toward the away person at a level that
/// grows with the days away and the closeness of the tie and is capped at [`Tuning::missing_cap`]; it is
/// mood only, never the cause of an activity. Removing the marker writes [`crate::PersonReturned`] once
/// and gives each holder who missed them a reunion lift. A whole party that is `Away` together does not
/// miss each other: holders who are themselves `Away` miss nobody.
///
/// The crate records when the absence began in [`AwaySince`] by itself, so a host inserts this marker
/// and nothing else. `Deceased` replaces `Away`: inserting it removes the marker and writes no
/// `PersonReturned`.
///
/// Attachment theory takes proximity-seeking and **separation distress** to be what a bond is (Hazan &
/// Shaver 1987, DOI `10.1037/0022-3514.52.3.511`): the people one is close to feel an absence in
/// proportion to how close they are, and the feeling ends with the return.
#[derive(Component, Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[reflect(Component)]
#[require(Dormant)]
pub struct Away;

/// When an [`Away`] person left: the game-day, written by the crate when `Away` is inserted (unless the
/// host or a restore already inserted it). Read it for how long someone has been gone; do not insert it
/// without `Away`.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component)]
pub struct AwaySince(pub f64);

/// The home of a person, used by the `Home` activity target.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Home(pub Entity);

/// The places bound to one person by name, for `ActivityTarget::Bound`.
///
/// `Home` is one place every person has; this is the general case. Sanctuary assigns each townsperson
/// their own work site each morning — the woodlot, the quarry, a stretch of wall — and daily life must
/// send them to *their* site, not the nearest site of that kind. A named binding is also how a
/// night-watch post, a bed or a favourite bench gets attached to one person.
///
/// ```ignore
/// commands.entity(person).insert(BoundPlaces(vec![("work".to_owned(), woodlot)]));
/// ```
///
/// A person with **no** binding under the name an activity asks for simply gets no candidate for that
/// activity: they do not do it, and they are not warned about, because a host that binds some people
/// and not others (a night watch has no work place) would otherwise be told once per person for
/// nothing. A binding to an entity that is not a [`Place`], or to one at capacity, gets no candidate
/// too, and that *is* warned about once per person and name, because it is a wiring mistake.
#[derive(Component, Reflect, Debug, Clone, Default, PartialEq)]
#[reflect(Component)]
pub struct BoundPlaces(pub Vec<(String, Entity)>);

impl BoundPlaces {
    /// Binds `name` to `place`, replacing an existing binding under that name.
    pub fn bind(&mut self, name: impl Into<String>, place: Entity) {
        let name = name.into();
        match self.0.iter_mut().find(|(existing, _)| *existing == name) {
            Some(slot) => slot.1 = place,
            None => self.0.push((name, place)),
        }
    }

    /// The place bound under `name`, when there is one.
    pub fn get(&self, name: &str) -> Option<Entity> {
        self.0
            .iter()
            .find(|(existing, _)| existing == name)
            .map(|(_, entity)| *entity)
    }
}

/// A place people can go to. `capacity` limits how many persons may be doing an activity there.
#[derive(Component, Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Place {
    /// Maximum number of persons whose activity targets this place at once.
    pub capacity: u32,
}

/// Tags that let activities find a place (`ActivityTarget::Place(tag)`).
#[derive(Component, Reflect, Debug, Clone, Default, PartialEq, Eq)]
pub struct PlaceTags(pub Vec<String>);

/// Persons in spawn order, with lookup by id. Maintained by the add and remove observers only.
#[derive(Resource, Reflect, Debug, Clone, Default)]
pub struct PersonIndex {
    /// Entity of each live person by id (lookup only; never iterated).
    pub by_id: HashMap<PersonId, Entity>,
    /// Live persons in ascending id order. Every simulation loop iterates this.
    pub sorted: Vec<(PersonId, Entity)>,
}

/// Per-tick perception: positions and neighbourhoods, indexed by position in
/// [`PersonIndex::sorted`]. Neighbour lists are ascending by index and exclude the person itself.
#[derive(Resource, Reflect, Debug, Clone, Default)]
pub struct Perception {
    /// Entity at each index.
    pub persons: Vec<Entity>,
    /// Plane position at each index.
    pub pos: Vec<Vec2>,
    /// Persons within `perception_radius`, with their distance.
    pub near: Vec<Vec<(u32, f32)>>,
    /// Persons within `contact_radius`.
    pub contact: Vec<Vec<u32>>,
}

/// System sets of one society tick, chained in this order inside `FixedUpdate`.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SocietySystems {
    /// Advances the clock. Runs every fixed step.
    Clock,
    /// Builds [`Perception`]. Runs on society ticks only.
    Perceive,
    /// Emotions, mood, needs and memories.
    Mind,
    /// Daily-life decisions.
    Decide,
    /// Social exchanges.
    Interact,
    /// Opinion and rumor propagation.
    Propagate,
    /// Ties, balance, labels, groups and status.
    Relate,
    /// Triggers and scripted effects.
    Rules,
}

/// Run condition: true on fixed steps in which a society tick is due.
pub fn society_tick_due(tick: Res<SocietyTick>) -> bool {
    tick.due
}

/// Advances [`SocietyClock`] by one tick each time enough fixed-step time has accumulated.
///
/// The **tick gate is the same for both clock sources**: a tick still costs one `tick_secs` of
/// fixed-step time, so a host cannot make the simulation run faster by speeding its own clock up. Only
/// how much game time that tick represents changes.
fn advance_clock(
    settings: Res<SocietySettings>,
    fixed: Res<Time<Fixed>>,
    host: Option<Res<HostClock>>,
    mut warned: Local<bool>,
    mut tick: ResMut<SocietyTick>,
    mut clock: ResMut<SocietyClock>,
) {
    tick.accumulator += fixed.delta_secs();
    if tick.accumulator + 1e-6 < settings.tick_secs {
        tick.due = false;
        return;
    }
    tick.accumulator -= settings.tick_secs;
    tick.due = true;
    clock.tick += 1;

    match settings.clock {
        ClockSource::Fixed => {
            clock.dt_days = settings.tick_secs / settings.seconds_per_day;
            clock.now_days += f64::from(clock.dt_days);
        }
        ClockSource::Host => {
            let Some(host) = host.as_deref() else {
                // No host clock at all is a wiring mistake, not a per-tick event: freeze and say so
                // once. Freezing is the safe answer, because inventing time would decay needs and
                // time out walks against a clock nobody is actually running.
                if !*warned {
                    *warned = true;
                    warn!(
                        "bevy_society: ClockSource::Host but no HostClock resource; the society clock \
                         is frozen. Insert a HostClock and write now_days every frame."
                    );
                }
                clock.dt_days = 0.0;
                return;
            };
            // A host may stand still (a pause, a cutscene) or go backward (a reload). Neither is an
            // error: the first decays nothing, the second takes the host's time and applies no
            // negative dt, so nothing fires twice for hours already crossed.
            clock.dt_days = (host.now_days - clock.now_days).max(0.0) as f32;
            clock.now_days = host.now_days;
        }
    }
}

/// Seeds the RNG and sets the clock to the configured start hour — or, when the host owns the clock, to
/// the host's own time, so the first tick does not look like a jump from `start_hour`.
fn setup_society(
    mut commands: Commands,
    settings: Res<SocietySettings>,
    host: Option<Res<HostClock>>,
    mut clock: ResMut<SocietyClock>,
) {
    commands.insert_resource(SocietyRng(bevy_emerge_rng::seeded(settings.seed)));
    clock.now_days = match (settings.clock, host.as_deref()) {
        (ClockSource::Host, Some(host)) => host.now_days,
        _ => f64::from(settings.start_hour) / 24.0,
    };
    // **The hour trigger's baseline is the clock's start instant, not its first tick.** The first
    // tick arrives one `tick_secs` of fixed-step time later, by which point a host that jumped its
    // clock (or a fixed clock that started just before an hour boundary) has already crossed one — and
    // taking the baseline there would swallow that crossing silently.
    commands.insert_resource(crate::rules::LastDays(Some(clock.now_days)));
}

/// Gives each new [`Person`] the next id and records it in the index.
fn on_person_added(
    add: On<Add<Person>>,
    mut commands: Commands,
    mut next: ResMut<NextPersonId>,
    mut index: ResMut<PersonIndex>,
) {
    let entity = add.entity;
    let id = PersonId(next.0);
    match next.0.checked_add(1) {
        Some(n) => next.0 = n,
        None => {
            error!("bevy_society: PersonId overflow; person {entity:?} gets no id");
            return;
        }
    }
    commands.entity(entity).insert(id);
    index.by_id.insert(id, entity);
    index.sorted.push((id, entity));
}

/// Removes a despawned person from the index, from every other person's ties and from the rumor
/// book, and drops every rumor about it from every other person's knowledge.
///
/// A rumor whose subject leaves the world stops circulating, so nothing can ever spread it again;
/// leaving it in other persons' `Knowledge` would keep a dead rumor selectable by
/// `Knowledge::best_telling`.
fn on_person_removed(
    remove: On<Remove<Person>>,
    mut index: ResMut<PersonIndex>,
    mut ties: Query<&mut Ties>,
    rumors: Option<ResMut<RumorBook>>,
    mut knowledge: Query<&mut Knowledge>,
) {
    let entity = remove.entity;
    if let Some(pos) = index.sorted.iter().position(|(_, e)| *e == entity) {
        let (id, _) = index.sorted.remove(pos);
        index.by_id.remove(&id);
    }
    for mut own in &mut ties {
        own.0.retain(|tie| tie.target != entity);
    }
    if let Some(mut book) = rumors {
        for retired in book.retire_about(entity) {
            for mut owned in &mut knowledge {
                owned.forget(retired);
            }
        }
    }
}

/// Rebuilds [`Perception`] through a uniform grid over [`PersonIndex::sorted`].
///
/// **Why a grid.** Brute force tested every pair. Measured on the project's benchmark, whose people sit on a
/// 40 m disc with an 8 m perception radius: **499 500** pairs tested at 1 000 persons to find an average of
/// **36** neighbours each, and **12 497 500** at 5 000 to find **182**. Cost per tick went 14.55 ms ->
/// 53.96 -> 254.5 ms for 1 000 -> 2 000 -> 5 000 people: exactly N², with the per-pair cost flat at
/// 27-29 ms per million pairs. The pair count was the only variable.
///
/// **Cell size is the query radius.** Ericson 2005, *Real-Time Collision Detection*, §7.1.1: *"cell size is
/// generally adjusted to be large enough (but not much larger) to fit the largest object at any rotation...
/// the maximum number of objects in a cell is bounded by a small constant."* For points the largest object
/// has no extent, so the direct reading is one cell per query radius — a neighbour within `perception_radius`
/// is then always in the cell itself or one of the eight around it. The grid is only as wide as the
/// population's own bounds, so an empty map costs nothing.
///
/// **The result is bit-identical to brute force, and that is the point.** Each cell's list is in ascending
/// index order because the grid is filled by walking `PersonIndex::sorted` once, so the eight neighbour
/// cells are merged by that same ascending index. A person's neighbours therefore come out in the same order
/// with the same distances as the all-pairs scan produced, and determinism is untouched.
fn perceive(
    settings: Res<SocietySettings>,
    index: Res<PersonIndex>,
    transforms: Query<(&GlobalTransform, Has<Dormant>)>,
    mut perception: ResMut<Perception>,
) {
    let n = index.sorted.len();
    let Perception {
        persons,
        pos,
        near,
        contact,
    } = &mut *perception;
    persons.clear();
    pos.clear();
    // **A dormant person keeps their slot and position but is left out of the grid**, so nobody can be
    // near or in contact with them while every slot-indexed reader still points at the right person.
    let mut dormant: Vec<bool> = Vec::with_capacity(n);
    for (_, entity) in &index.sorted {
        persons.push(*entity);
        let (at, is_dormant) = transforms
            .get(*entity)
            .map(|(t, d)| (settings.plane.project(t.translation()), d))
            // A person without a transform yet sits at the origin for this tick.
            .unwrap_or((Vec2::ZERO, false));
        pos.push(at);
        dormant.push(is_dormant);
    }
    near.resize_with(n, Vec::new);
    contact.resize_with(n, Vec::new);
    for list in near.iter_mut() {
        list.clear();
    }
    for list in contact.iter_mut() {
        list.clear();
    }
    let perception_radius = settings.perception_radius;
    let contact_radius = settings.contact_radius;
    if perception_radius <= 0.0 || pos.is_empty() {
        return;
    }

    // The grid spans the population's own bounding box, with one cell per perception radius. `cols` and
    // `rows` are at least 1 so a population that has not moved yet still gets a usable grid.
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for point in pos.iter() {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }
    let cell = perception_radius;
    let cols = (((max_x - min_x) / cell).floor() as i64 + 1).max(1) as usize;
    let rows = (((max_y - min_y) / cell).floor() as i64 + 1).max(1) as usize;
    let column = |x: f32| (((x - min_x) / cell).floor() as i64).clamp(0, cols as i64 - 1) as usize;
    let row = |y: f32| (((y - min_y) / cell).floor() as i64).clamp(0, rows as i64 - 1) as usize;

    // Cell contents, in ascending index order because `pos` is walked once, in order.
    let mut cells: Vec<Vec<u32>> = vec![Vec::new(); cols * rows];
    for (index, point) in pos.iter().enumerate() {
        if dormant.get(index).copied().unwrap_or(false) {
            continue;
        }
        cells[row(point.y) * cols + column(point.x)].push(index as u32);
    }

    for (i, pi) in pos.iter().enumerate() {
        // A dormant person has no neighbours either: their `near` and `contact` stay empty, so nothing
        // they would accumulate from the world happens while they are away from it.
        if dormant.get(i).copied().unwrap_or(false) {
            continue;
        }
        let cx = column(pi.x);
        let cy = row(pi.y);
        // All nine cells around this one. The person's own cell is included, and so are the cells behind
        // it: a neighbour one cell away in *any* direction can be within the radius, so skipping the
        // already-decided half would drop pairs the all-pairs scan found.
        let mut candidates: Vec<u32> = Vec::new();
        for ny in cy.saturating_sub(1)..=(cy + 1).min(rows - 1) {
            for nx in cx.saturating_sub(1)..=(cx + 1).min(cols - 1) {
                candidates.extend_from_slice(&cells[ny * cols + nx]);
            }
        }
        // A cell list and the eight around it are each ascending, so sorting the merged list restores the
        // all-pairs order exactly. Duplicates cannot occur: every index sits in one cell.
        candidates.sort_unstable();
        for j in candidates {
            let j = j as usize;
            if j == i {
                continue;
            }
            let d = pi.distance(pos[j]);
            if d <= perception_radius {
                near[i].push((j as u32, d));
            }
            if d <= contact_radius {
                contact[i].push(j as u32);
            }
        }
    }
}

/// Fails loudly when a society plugin is added before [`SocietyCorePlugin`]. This is one of the
/// only sanctioned panics in the crate; the check runs in `finish()`, after every plugin is built.
pub(crate) fn require_core(app: &App, plugin: &str) {
    if !app.is_plugin_added::<SocietyCorePlugin>() {
        panic!("bevy_society: add SocietyCorePlugin (or SocietyPlugins) before {plugin}");
    }
}

/// Script writes waiting to be applied (plan D8).
///
/// Every evaluation queues its actions here and one exclusive system drains the queue once per
/// tick, after [`SocietySystems::Rules`]. One application point is what keeps a script from
/// changing the world another trigger in the same tick reads, and what makes the order of
/// application independent of which system ran first.
#[derive(Resource, Debug, Clone, Default)]
pub(crate) struct PendingActions(pub(crate) Vec<crate::script::ScriptAction>);

/// Queues script actions for the next application point. Plugins outside this module use it, so a
/// script effect queued in `Update` (a dialogue node) lands in the same place as one queued in
/// `FixedUpdate`.
pub(crate) fn push_script_actions(
    pending: &mut PendingActions,
    actions: impl IntoIterator<Item = crate::script::ScriptAction>,
) {
    pending.0.extend(actions);
}

/// Applies every queued script action, in queue order (plan D8).
///
/// This is the crate's one exclusive system: `fx` can write to any person component, so it needs the
/// whole world. It runs after [`SocietySystems::Rules`] on society ticks, and it resolves every
/// [`PersonId`] through [`PersonIndex::by_id`], skipping an action whose person is unknown with a
/// `warn!`.
fn apply_script_actions(world: &mut World) {
    let actions = {
        let Some(mut pending) = world.get_resource_mut::<PendingActions>() else {
            return;
        };
        std::mem::take(&mut pending.0)
    };
    if actions.is_empty() {
        return;
    }
    let now = world
        .get_resource::<SocietyClock>()
        .map_or(0.0, |clock| clock.now_days);
    for action in actions {
        crate::script_apply::apply(world, action, now);
    }
}

/// Persons, ids, clock, RNG, sets and perception. Add it before every other society plugin, or use
/// [`crate::SocietyPlugins`], which does.
pub struct SocietyCorePlugin;

impl Plugin for SocietyCorePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SocietyPlane>()
            .register_type::<AssessmentNorm>()
            .register_type::<Tuning>()
            .register_type::<SocietySettings>()
            .register_type::<SocietyClock>()
            .register_type::<SocietyTick>()
            .register_type::<NextPersonId>()
            .register_type::<PersonId>()
            .register_type::<PersonKey>()
            .register_type::<PlaceKey>()
            .register_type::<Person>()
            .register_type::<Autonomous>()
            .register_type::<Dormant>()
            .register_type::<Deceased>()
            .register_type::<Away>()
            .register_type::<AwaySince>()
            .register_type::<Home>()
            .register_type::<BoundPlaces>()
            .register_type::<Place>()
            .register_type::<PlaceTags>()
            .register_type::<PersonIndex>()
            .register_type::<Perception>()
            .init_resource::<SocietySettings>()
            .init_resource::<SocietyClock>()
            .init_resource::<SocietyTick>()
            .init_resource::<NextPersonId>()
            .init_resource::<PersonIndex>()
            .init_resource::<Perception>()
            .init_resource::<SocietyDefs>()
            .init_resource::<crate::script::ScriptEngine>()
            .init_resource::<crate::snapshot::SnapshotCell>()
            .init_resource::<PendingActions>()
            .add_message::<crate::interact::Interact>()
            .add_message::<crate::interact::SocietyBark>()
            .add_message::<crate::script::ScriptEvent>()
            .add_message::<crate::script::ScriptError>()
            .add_observer(on_person_added)
            .add_observer(on_person_removed)
            .add_systems(Startup, setup_society)
            .configure_sets(
                FixedUpdate,
                (
                    SocietySystems::Clock,
                    SocietySystems::Perceive,
                    SocietySystems::Mind,
                    SocietySystems::Decide,
                    SocietySystems::Interact,
                    SocietySystems::Propagate,
                    SocietySystems::Relate,
                    SocietySystems::Rules,
                )
                    .chain(),
            )
            .configure_sets(
                FixedUpdate,
                (
                    SocietySystems::Perceive,
                    SocietySystems::Mind,
                    SocietySystems::Decide,
                    SocietySystems::Interact,
                    SocietySystems::Propagate,
                    SocietySystems::Relate,
                    SocietySystems::Rules,
                )
                    .run_if(society_tick_due),
            )
            .add_systems(
                FixedUpdate,
                (advance_clock, size_person_state)
                    .chain()
                    .in_set(SocietySystems::Clock),
            )
            .add_systems(
                FixedUpdate,
                (perceive, crate::snapshot::build_snapshot)
                    .chain()
                    .in_set(SocietySystems::Perceive),
            )
            // One application point for every script write, after the rules have run.
            .add_systems(
                FixedUpdate,
                apply_script_actions
                    .after(SocietySystems::Rules)
                    .run_if(society_tick_due),
            );
    }
}
