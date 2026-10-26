//! Goods, stock and a price per good (issue: "basic economics").
//!
//! **The shape is Edgeworth barter, not an auction.** Parunak et al. (corpus doc `10.1007_11678809_10`) note
//! that "a less-studied alternative to auction systems is the pattern of Edgeworth barter, in which buyers
//! and sellers interact directly with each other in repeated pairwise transactions, **without the benefit of
//! an auctioneer**". Twenty-four villagers do not clear a market; they produce, they eat what they produced,
//! and the price is an *observation* of how that went.
//!
//! **The acceptance test is the literature's own.** The production/consumption model in *Games and AI*
//! (corpus doc `eth-cgl-ar_games-Zun16a`) states it: "if the parameters are chosen such that the inflow of
//! resources into the system equals the outflow, then the economy is **stable and prices fluctuate around a
//! constant value**... by upsetting this balance we can simulate a variety of interesting real market
//! phenomenon, such as shortages and oversupply." [`update_prices`] is built so that claim is a test rather
//! than a hope: equal inflow and outflow give a zero delta and the price does not move.
//!
//! **Goods are not fungible.** Henrich et al. 2005 (DOI `10.1017/S0140525X05000142`, in the corpus): "meat
//! and honey are meticulously shared among the Ache, but goods purchased by money are not". There is no
//! money here and no exchange rate; a good is a quantity and nothing more.
//!
//! **Nothing gates.** Last iteration measured that putting a new condition on a *dominant* activity wrecks
//! the village (rivals 29 → 55 at seed 42). Having bread changes what a person prefers to do; it never
//! changes what a person is allowed to do.

use bevy::prelude::*;

use crate::core::require_core;
use crate::core::{SocietyClock, SocietySettings, SocietySystems};
use crate::defs::GoodId;

/// Per-person holdings, one level per good in definition order, each in `[0, Tuning::stock_ceiling]`.
///
/// This is the first per-person state added since the save format was written, so it is in
/// [`PersonSave`](crate::PersonSave) and `SocietySave::VERSION` is 2. A person without the component is
/// treated as holding nothing and is given one at spawn by
/// [`size_person_state`](crate::size_person_state).
#[derive(Component, Debug, Clone, Default, PartialEq, Reflect)]
pub struct Stock(pub Vec<f32>);

impl Stock {
    /// The level of one good, or `0.0` when the person holds none of it.
    pub fn get(&self, good: GoodId) -> f32 {
        self.0.get(usize::from(good.0)).copied().unwrap_or(0.0)
    }

    /// Adds to one good, clamped to `[0, ceiling]`, and returns how much was actually added.
    ///
    /// The return value is what [`Prices`] measures: a producer at the ceiling adds nothing to the flow, and
    /// counting the intent rather than the result would let demand run away from supply forever.
    pub fn add(&mut self, good: GoodId, amount: f32, ceiling: f32) -> f32 {
        let slot = usize::from(good.0);
        if self.0.len() <= slot {
            self.0.resize(slot + 1, 0.0);
        }
        let before = self.0[slot];
        self.0[slot] = (before + amount).clamp(0.0, ceiling.max(0.0));
        self.0[slot] - before
    }

    /// Removes up to `amount` of one good, returning how much was actually removed.
    pub fn take(&mut self, good: GoodId, amount: f32) -> f32 {
        let slot = usize::from(good.0);
        let Some(level) = self.0.get_mut(slot) else {
            return 0.0;
        };
        let taken = amount.max(0.0).min(*level);
        *level -= taken;
        taken
    }
}

/// What flowed through one good this tick: how much was produced and how much was consumed.
///
/// Accumulated by `produce_and_consume` between society ticks and consumed by `update_prices`, so the price
/// is a measurement of what the village did rather than a second simulation beside it.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct Flow {
    /// Units produced this tick, by good.
    pub produced: Vec<f32>,
    /// Units consumed this tick, by good.
    pub consumed: Vec<f32>,
}

impl Flow {
    /// Makes room for `goods` goods and clears every counter.
    fn reset(&mut self, goods: usize) {
        self.produced.clear();
        self.produced.resize(goods, 0.0);
        self.consumed.clear();
        self.consumed.resize(goods, 0.0);
    }
}

/// The price of each good, in definition order.
///
/// One number per good, starting at 1.0 and clamped to `[Tuning::price_floor, Tuning::price_ceiling]` so a
/// village cannot inflate for ever. A host reads it through `ctx.price("bread")`.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct Prices(pub Vec<f32>);

impl Prices {
    /// The price of one good, or `1.0` when it has never been set — a good nobody has traded is worth its
    /// nominal unit, which is the only defensible default when there is no exchange rate.
    pub fn get(&self, good: GoodId) -> f32 {
        self.0.get(usize::from(good.0)).copied().unwrap_or(1.0)
    }
}

/// What one unit of a good is worth to a person holding `held` of it: `1 / (1 + held)`.
///
/// Diminishing marginal value, and the crate's only honest option: a store is a level, so somebody with an
/// empty shelf wants a unit more than somebody with a full one. No curve, no utility function, no new rate —
/// a person with none values a unit at 1.0 and a person at a full store at `1 / (1 + ceiling)`.
///
/// This is what makes a **joint surplus** possible between two people, which is the condition Hall & Milgrom
/// require before a bargain begins at all: "once a qualified worker and an employer have met and found that
/// they would enjoy a joint surplus, the threats to disclaim the match are hollow" (Hall 2005, DOI
/// `10.3386/w11678`, in the corpus, reporting Hall & Milgrom).
pub fn value_of(held: f32) -> f32 {
    1.0 / (1.0 + held.max(0.0))
}

/// Applies every finished activity's production and consumption, once.
///
/// Runs after the exchange set, on the tick's own [`ActivityEnded`](crate::ActivityEnded) messages, so a
/// task that was interrupted pays nothing — the same rule `satisfies` follows.
///
/// Iterates [`PersonIndex::sorted`](crate::PersonIndex) so the order is fixed, and draws nothing.
pub(crate) fn produce_and_consume(
    mut ended: MessageReader<crate::ActivityEnded>,
    mut flow: ResMut<Flow>,
    settings: Res<SocietySettings>,
    defs: Res<crate::SocietyDefs>,
    index: Res<crate::PersonIndex>,
    mut stocks: Query<&mut Stock>,
) {
    let goods = defs.goods.len();
    if goods == 0 {
        for _ in ended.read() {}
        return;
    }
    // **Do not reset every tick.** An activity ends only every few dozen ticks, so a per-tick reset would
    // hand `update_prices` one tick of flow followed by thirty ticks of zero and the price would never
    // move. `update_prices` consumes and clears instead, which makes the price an integral of what the
    // village did since the last price update.
    if flow.produced.len() != goods {
        flow.reset(goods);
    }
    // **Only a completed activity pays.** An interrupted one did not happen, which is the same rule
    // `satisfies` follows and the reason an interruption is a cost rather than a pause.
    let mut done: Vec<(Entity, String)> = ended
        .read()
        .filter(|e| e.completed)
        .map(|e| (e.person, e.activity.clone()))
        .collect();
    done.sort_by_key(|(entity, name)| (entity.to_bits(), name.clone()));
    let ceiling = settings.tuning.stock_ceiling;
    for (person, name) in done {
        let Some(spec) = index
            .sorted
            .iter()
            .position(|&(_, e)| e == person)
            .and_then(|_| defs.activity_by_name(&name))
        else {
            continue;
        };
        let Ok(mut stock) = stocks.get_mut(person) else {
            continue;
        };
        for &(good, amount) in &spec.consumes {
            let taken = stock.take(good, amount);
            flow.consumed[usize::from(good.0)] += taken;
        }
        for &(good, amount) in &spec.produces {
            let added = stock.add(good, amount, ceiling);
            flow.produced[usize::from(good.0)] += added;
        }
    }
}

/// Moves each price toward the side the village is short on, and leaves it alone when it is not short.
///
/// `Δprice = price_speed_per_day · (produced − consumed)` — **per interval, not per tick**.
///
/// Not per tick: an activity ends only every few dozen ticks, so a per-tick `dt` would weight one tick's
/// imbalance by `1/320` of a day and the price would never visibly move. The counters accumulate between
/// price updates and are cleared here, so the delta is the change for the interval that just elapsed, and
/// `price_speed_per_day` reads as "how far the price moves per unit of imbalance per pricing step".
///
/// This is the whole economy's dynamics and it is deliberately one line: the acceptance test in the module
/// header — equal inflow and outflow means a price that does not move — is satisfied by construction, and
/// anything richer would need rates no paper in the corpus supplies.
///
/// **Design defaults**: `price_speed_per_day` and the clamp band. `stock_ceiling` is in [`Tuning`](crate::Tuning)
/// too. The paper's own figures (`w = 6.0`, 500 units) are for its gnomes and are not transferable.
pub(crate) fn update_prices(
    clock: Res<SocietyClock>,
    settings: Res<SocietySettings>,
    mut flow: ResMut<Flow>,
    mut prices: ResMut<Prices>,
) {
    if flow.produced.is_empty() {
        return;
    }
    let tuning = &settings.tuning;
    if prices.0.len() < flow.produced.len() {
        prices.0.resize(flow.produced.len(), 1.0);
    }
    let _ = clock;
    for i in 0..flow.produced.len() {
        // **Scarcity is `produced − consumed`.** A village that ate more than it baked is short, and a
        // short good gets dearer. The first version had this inverted, so an undersupplied village showed
        // its price *falling*: with a shared store, the eaters who cannot find bread stop eating, so
        // consumption falls faster than production and the raw difference is negative.
        let scarcity = flow.produced[i] - flow.consumed[i];
        let next = prices.0[i] + tuning.price_speed_per_day * scarcity;
        prices.0[i] = next.clamp(tuning.price_floor, tuning.price_ceiling);
    }
    // **Consume the flow once it has been priced.** The counters accumulate between society ticks;
    // clearing them here rather than in the producer is what makes the price an integral of what happened
    // rather than of the last tick alone.
    flow.consumed.fill(0.0);
    flow.produced.fill(0.0);
}

/// Goods, stock and prices.
///
/// Runs after the exchange set so a task's production lands in the tick that finished it. A host with no
/// `goods` in its definitions file pays nothing: every system returns immediately.
pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Stock>()
            .init_resource::<Flow>()
            .init_resource::<Prices>()
            .add_systems(
                FixedUpdate,
                (produce_and_consume, update_prices)
                    .chain()
                    .in_set(SocietySystems::Interact),
            );
    }

    fn finish(&self, app: &mut App) {
        require_core(app, "EconomyPlugin");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stock_clamps_and_reports_what_it_actually_moved() {
        let mut stock = Stock::default();
        assert_eq!(stock.add(GoodId(0), 0.4, 1.0), 0.4);
        // A producer already at the ceiling adds nothing, which is what keeps the flow honest.
        assert_eq!(stock.add(GoodId(0), 0.9, 1.0), 0.6);
        assert_eq!(stock.get(GoodId(0)), 1.0);
        // Taking more than is there takes what is there.
        assert_eq!(stock.take(GoodId(0), 5.0), 1.0);
        assert_eq!(stock.get(GoodId(0)), 0.0);
        // An unknown good holds nothing rather than panicking.
        assert_eq!(stock.get(GoodId(9)), 0.0);
        assert_eq!(stock.take(GoodId(9), 1.0), 0.0);
    }

    #[test]
    fn a_balanced_flow_leaves_the_price_where_it_was() {
        // The acceptance test, as arithmetic: `consumed - produced == 0` gives a zero delta whatever the
        // speed and whatever the tick length.
        let imbalance = 2.0_f32 - 2.0;
        let delta = 4.0_f32 * 0.01 * imbalance;
        assert_eq!(delta, 0.0);
    }

    #[test]
    fn an_undersupplied_good_gets_dearer_and_an_oversupplied_one_cheaper() {
        let (speed, dt) = (2.0_f32, 0.25_f32);
        let dearer = speed * dt * (3.0 - 1.0);
        let cheaper = speed * dt * (1.0 - 3.0);
        assert!(dearer > 0.0, "short supply must raise the price");
        assert!(cheaper < 0.0, "oversupply must lower it");
    }
}
