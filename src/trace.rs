//! **Why a person chose what it chose.** A bounded record of the scores behind every decision, so a
//! host can explain a behaviour instead of guessing at it.
//!
//! Game AI Pro 3 ch. 6 ("Debugging AI with Instant In-Game Scrubbing", corpus doc
//! `gameaipro3-ch06-debugging-ai-instant-game-scrubbing`) states the three constraints this module
//! is built to, all of them measured in that chapter's own recorder:
//!
//! > "The recording of object data must have minimal to no impact on the speed of the game
//! > simulation, the memory requirements of the system must be **fixed**, and the recording of object
//! > data must have **no impact on the outcome of simulating objects**."
//!
//! and the AI-specific use:
//!
//! > "recording an AI's entire behavior tree in game […] displayed which branches within the tree
//! > were currently executing as well as where within other possible decision branches, execution
//! > terminated prematurely."
//!
//! The last sentence is the one this crate was missing. [`Doing`](crate::Doing) says what a person
//! is doing; nothing said what else it was weighing, or by how much it lost.

use bevy::prelude::*;

/// One option a decision weighed: the activity, the score it reached, and why it lost or won.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoredOption {
    /// The activity's name, as authored.
    pub activity: String,
    /// `weight · Π compensated consideration`, in `[0, 1]`.
    pub score: f32,
    /// Whether this option was the one taken.
    pub chosen: bool,
}

/// Why one person chose the activity it chose, at one tick.
#[derive(Debug, Clone, PartialEq)]
pub struct Decision {
    /// The tick the decision ran on.
    pub tick: u64,
    /// The person.
    pub person: Entity,
    /// The person's name, when it had one. Copied here because the entity may despawn before a host
    /// reads the trace, and a name outlives the entity.
    pub name: Option<String>,
    /// The host-written signal values the decision scored with, indexed by
    /// [`SignalId`](crate::SignalId). Empty when the person had no `Signals` to copy.
    pub signals: Vec<f32>,
    /// Every option that scored above zero, best first. The idle fallback is included when nothing
    /// else scored.
    pub options: Vec<ScoredOption>,
}

impl Decision {
    /// The activity taken, or `None` when the trace recorded no options.
    pub fn chosen(&self) -> Option<&ScoredOption> {
        self.options.iter().find(|option| option.chosen)
    }

    /// The highest-scoring option, which is **not** necessarily the chosen one: selection is a
    /// weighted random draw over the viable tier, so a lower scorer can win.
    pub fn best(&self) -> Option<&ScoredOption> {
        self.options.first()
    }

    /// A one-line account of this decision, for a log or a HUD.
    ///
    /// `names` is whatever the caller has: the trace stores names when it can, and a host reading a
    /// ring whose entities have since despawned can pass an empty list to fall back to "someone".
    pub fn describe(&self, names: &[(Entity, String)]) -> String {
        let who = self
            .name
            .clone()
            .or_else(|| {
                names
                    .iter()
                    .find(|(entity, _)| *entity == self.person)
                    .map(|(_, name)| name.clone())
            })
            .unwrap_or_else(|| "someone".to_owned());
        match self.chosen() {
            Some(chosen) if self.took_the_best() => format!(
                "tick {}: {who} chose {} ({}), the best option",
                self.tick,
                chosen.activity,
                format_args!("{:.3}", chosen.score)
            ),
            Some(chosen) => format!(
                "tick {}: {who} chose {} ({:.3}), {:.0}% below {} ({:.3})",
                self.tick,
                chosen.activity,
                chosen.score,
                self.shortfall() * 100.0,
                self.best().map_or("the best", |b| b.activity.as_str()),
                self.best().map_or(0.0, |b| b.score)
            ),
            None => format!("tick {}: {who} weighed nothing", self.tick),
        }
    }

    /// How far the chosen option fell short of the best available, as a fraction of that best score.
    /// `0.0` means the chosen option was the best one; `1.0` means it scored nothing at all.
    ///
    /// This is the quantity that says whether a decision was close, and it is deliberately not
    /// "chosen minus runner-up": a weighted draw can pick the fifth-best option legitimately, which
    /// made the first version of this report a negative "margin" — a fraction of a score that had
    /// nothing to do with the choice.
    pub fn shortfall(&self) -> f32 {
        match (self.chosen(), self.best()) {
            (Some(chosen), Some(best)) if best.score > 0.0 => {
                (best.score - chosen.score) / best.score
            }
            _ => 0.0,
        }
    }

    /// Whether the chosen option was also the best-scoring one.
    pub fn took_the_best(&self) -> bool {
        self.best().is_some_and(|option| option.chosen)
    }
}

/// The decision trace: a fixed-capacity ring of the most recent decisions.
///
/// Fixed capacity is a requirement, not a nicety — see this module's header. When the ring is full
/// the **oldest** decision is dropped, which is what a host that only ever looks at "the last few
/// minutes" wants, and what keeps memory flat over a long run.
#[derive(Resource, Debug, Clone)]
pub struct DecisionTrace {
    decisions: std::collections::VecDeque<Decision>,
    capacity: usize,
    /// How many options to keep per decision, best first. A definition file with many activities
    /// would otherwise store a long tail nobody reads.
    options_per_decision: usize,
}

impl Default for DecisionTrace {
    fn default() -> Self {
        Self {
            decisions: std::collections::VecDeque::new(),
            // Design default: no source paper gives these numbers. 64 decisions is a few minutes of
            // a village's life, and eight options is more than the starter pack can field.
            capacity: 64,
            options_per_decision: 8,
        }
    }
}

impl DecisionTrace {
    /// A trace that keeps `capacity` decisions of `options_per_decision` options each.
    pub fn with_capacity(capacity: usize, options_per_decision: usize) -> Self {
        Self {
            decisions: std::collections::VecDeque::new(),
            capacity,
            options_per_decision: options_per_decision.max(1),
        }
    }

    /// The most recent decisions, newest first.
    pub fn recent(&self) -> impl DoubleEndedIterator<Item = &Decision> {
        self.decisions.iter().rev()
    }

    /// The most recent decision for `person`, when the trace still holds one.
    pub fn last_for(&self, person: Entity) -> Option<&Decision> {
        self.decisions.iter().rev().find(|d| d.person == person)
    }

    /// How many decisions the trace holds.
    pub fn len(&self) -> usize {
        self.decisions.len()
    }

    /// Whether the trace holds nothing.
    pub fn is_empty(&self) -> bool {
        self.decisions.is_empty()
    }

    /// Records one decision, dropping the oldest when full and cutting the option list.
    pub(crate) fn push(&mut self, mut decision: Decision) {
        if self.capacity == 0 {
            return;
        }
        decision.options.truncate(self.options_per_decision);
        while self.decisions.len() >= self.capacity {
            self.decisions.pop_front();
        }
        self.decisions.push_back(decision);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn person() -> Entity {
        Entity::from_raw_u32(1).expect("a valid entity")
    }

    fn decision(tick: u64, who: Entity) -> Decision {
        Decision {
            tick,
            person: who,
            name: None,
            signals: Vec::new(),
            options: vec![
                ScoredOption {
                    activity: "eat".to_owned(),
                    score: 0.8,
                    chosen: true,
                },
                ScoredOption {
                    activity: "chat".to_owned(),
                    score: 0.2,
                    chosen: false,
                },
            ],
        }
    }

    #[test]
    fn the_ring_drops_the_oldest_and_keeps_memory_flat() {
        let mut trace = DecisionTrace::with_capacity(2, 8);
        for tick in 0..5 {
            trace.push(decision(tick, person()));
        }
        assert_eq!(trace.len(), 2, "capacity is a hard bound");
        let ticks: Vec<u64> = trace.recent().map(|d| d.tick).collect();
        assert_eq!(ticks, vec![4, 3], "newest first");
    }

    #[test]
    fn taking_the_best_option_has_no_shortfall() {
        let mut trace = DecisionTrace::default();
        trace.push(decision(0, person()));
        let record = trace.last_for(person()).expect("just pushed");
        assert!(record.took_the_best(), "eat at 0.8 was the best option");
        assert_eq!(record.shortfall(), 0.0);
    }

    #[test]
    fn a_weighted_draw_that_picked_a_lower_scorer_reports_a_shortfall() {
        // The draw is weighted, so a lower-scoring option can win. This is what produced a negative
        // "margin" in the first version: the report compared the chosen option with the next in a
        // sorted list rather than with the best on offer.
        let mut trace = DecisionTrace::default();
        trace.push(Decision {
            tick: 0,
            person: person(),
            name: None,
            signals: Vec::new(),
            options: vec![
                ScoredOption {
                    activity: "snub".to_owned(),
                    score: 1.0,
                    chosen: false,
                },
                ScoredOption {
                    activity: "compliment".to_owned(),
                    score: 0.75,
                    chosen: true,
                },
            ],
        });
        let record = trace.last_for(person()).expect("just pushed");
        assert!(!record.took_the_best());
        assert!(
            (record.shortfall() - 0.25).abs() < 1e-6,
            "0.75 against a best of 1.0 is a 25% shortfall: {}",
            record.shortfall()
        );
    }

    #[test]
    fn a_lone_option_is_the_best_option() {
        let mut trace = DecisionTrace::default();
        trace.push(Decision {
            tick: 0,
            person: person(),
            name: None,
            signals: Vec::new(),
            options: vec![ScoredOption {
                activity: "sleep".to_owned(),
                score: 1.0,
                chosen: true,
            }],
        });
        let only = trace.last_for(person()).expect("just pushed");
        assert_eq!(only.shortfall(), 0.0);
        assert!(only.took_the_best());
    }
}
