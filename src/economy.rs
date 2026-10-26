//! Goods, stock and a price per good (issue: "basic economics").
//!
//! **The shape is Edgeworth barter, not an auction.** Parunak et al. (corpus doc `10.1007_11678809_10`) note
//! that "a less-studied alternative to auction systems is the pattern of Edgeworth barter, in which buyers
//! and sellers interact directly with each other in repeated pairwise transactions, **without the benefit of
//! an auctioneer**". Twenty-four villagers do not clear a market; they produce, they eat what they produced,
//! and the price is an *observation* of how that went.
//!
//! **The price is read off how full the stores are, not integrated from the flow.** The production/consumption
//! model in *Games and AI* (corpus doc `eth-cgl-ar_games-Zun16a`) states its acceptance test — "if the
//! parameters are chosen such that the inflow of resources into the system equals the outflow, then the
//! economy is **stable and prices fluctuate around a constant value** ... **shortages and oversupply**" move
//! it — and its own Model A satisfies it by pricing from the **storage ratio** rather than from a flow:
//! `p = w · v(r)` with `r = S_cur / S_max`, and *"the price modifier term `v(r)` doubles the price if the
//! storage is empty and halves the price if the storage is full"* (§3.1.1). [`update_prices`] applies
//! exactly that `v`, over village-wide stores and **per head**. The earlier `Δprice ∝ (produced − consumed)`
//! is gone: it integrated net stock, so a village that baked more than it ate saw its price *rise*, and it
//! counted units, so the same six loaves made a glut of one village and a famine of thirty. Both defects are
//! in `docs/research/economics.md` "Open contradictions" and both are repaired by
//! [`storage_price`](crate::model::price::storage_price).
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
use crate::core::{SocietySettings, SocietySystems};
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
    settings: Res<SocietySettings>,
    defs: Res<crate::SocietyDefs>,
    index: Res<crate::PersonIndex>,
    mut stocks: Query<&mut Stock>,
) {
    if defs.goods.is_empty() {
        for _ in ended.read() {}
        return;
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
            stock.take(good, amount);
        }
        for &(good, amount) in &spec.produces {
            stock.add(good, amount, ceiling);
        }
    }
}

/// Sets each good's price from how full the village's stores are, **per head**.
///
/// `price = storage_price(total_held, people, price_full_store, 1.0)`, clamped to
/// `[Tuning::price_floor, Tuning::price_ceiling]`. The function is the corpus's own `v(r)` — the storage
/// ratio, worth `2.0` at an empty store, `1.0` at half and `0.5` at a full one — and it is in
/// [`crate::model::price`] with its quotes and its test against the paper's three named points.
///
/// **Per head is the whole point.** The ratio is over the **mean** held per person against
/// `Tuning::price_full_store` — a day's supply a head — so the same six loaves price differently in a village
/// of three and a village of thirty. A unit count cannot make that distinction and the previous
/// `Δprice ∝ (produced − consumed)` counted units.
///
/// **Nothing to integrate, so nothing to get wrong.** The old price accumulated a flow counter between
/// updates, which is how it came to price *net stock*: a village that baked more than it ate watched its
/// price rise, and an eater with an empty shelf — who takes nothing, so consumes nothing — made a real
/// shortage invisible. Here the price is a function of the world, so a village whose inflow equals its
/// outflow holds a constant store and therefore a constant price, which is the acceptance test in the
/// module header satisfied by construction rather than by a rate.
///
/// Iterates [`PersonIndex::sorted`](crate::PersonIndex), so the sum is over a fixed order, and draws
/// nothing.
pub(crate) fn update_prices(
    settings: Res<SocietySettings>,
    defs: Res<crate::SocietyDefs>,
    index: Res<crate::PersonIndex>,
    stocks: Query<&Stock>,
    mut prices: ResMut<Prices>,
) {
    let goods = defs.goods.len();
    if goods == 0 {
        return;
    }
    if prices.0.len() < goods {
        prices.0.resize(goods, 1.0);
    }
    let mut held = vec![0.0_f32; goods];
    for &(_, person) in &index.sorted {
        let Ok(stock) = stocks.get(person) else {
            continue;
        };
        for (good, total) in held.iter_mut().enumerate() {
            *total += stock.get(GoodId(good as u16));
        }
    }
    let people = index.sorted.len();
    let full_store = settings.tuning.price_full_store;
    for (good, total) in held.into_iter().enumerate() {
        let raw = crate::model::price::storage_price(total, people, full_store, 1.0);
        prices.0[good] = raw.clamp(settings.tuning.price_floor, settings.tuning.price_ceiling);
    }
}

/// Goods, stock and prices.
///
/// Runs after the exchange set so a task's production lands in the tick that finished it. A host with no
/// `goods` in its definitions file pays nothing: every system returns immediately.
pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Stock>()
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
        // A producer already at the ceiling adds nothing.
        assert_eq!(stock.add(GoodId(0), 0.9, 1.0), 0.6);
        assert_eq!(stock.get(GoodId(0)), 1.0);
        // Taking more than is there takes what is there.
        assert_eq!(stock.take(GoodId(0), 5.0), 1.0);
        assert_eq!(stock.get(GoodId(0)), 0.0);
        // An unknown good holds nothing rather than panicking.
        assert_eq!(stock.get(GoodId(9)), 0.0);
        assert_eq!(stock.take(GoodId(9), 1.0), 0.0);
    }
}
