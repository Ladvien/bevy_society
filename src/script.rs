//! Rhai for authored scripts: compilation, the sandbox limits, and the `ctx` / `fx` runtime API.
//!
//! Compilation happens in [`ScriptCompiler`] when definitions are compiled; a syntax error there is
//! a [`DefsError::Script`], so a bad script is reported before the app runs. The runtime lives in
//! [`ScriptEngine`]: every evaluation gets a fresh scope with the bound constants, a [`Ctx`] over the
//! tick's frozen snapshot and a fresh [`Fx`] queue. Nothing in here applies an action; the caller
//! does that, at one application point per tick (plan D8).
//!
//! Person arguments are [`PersonId`]s, carried through in [`ScriptAction`] as `i64` and resolved to
//! entities by the caller. A script therefore never names an entity, and an id it invented is
//! reported by the applier rather than silently pointing somewhere.

use std::sync::{Arc, Mutex};

use bevy::ecs::entity::Entity;
use bevy::ecs::message::Message;
use bevy::ecs::resource::Resource;
use bevy::reflect::Reflect;
use rhai::{AST, Dynamic, Engine, EvalAltResult, ParseError, Scope};

use crate::core::PersonId;
use crate::defs::DefsError;
use crate::snapshot::SocietySnapshot;

/// Index of a compiled script inside `SocietyDefs`, in compile order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Reflect)]
pub struct ScriptId(pub u32);

/// A compiled script with the authored path that names it, so runtime errors can say where they
/// came from.
#[derive(Clone)]
pub(crate) struct CompiledScript {
    /// The authored path, for example `trigger 'greet'`.
    pub(crate) owner: String,
    /// The compiled syntax tree.
    pub(crate) ast: AST,
}

/// Applies the sandbox limits every script runs under (plan D2 and D8).
pub(crate) fn apply_limits(engine: &mut Engine) {
    engine.set_max_operations(10_000);
    engine.set_max_call_levels(16);
    engine.set_max_string_size(1024);
    engine.set_max_array_size(256);
    engine.set_max_map_size(64);
}

/// Compiles authored script text into `CompiledScript`s, addressed by `ScriptId`.
pub(crate) struct ScriptCompiler {
    engine: Engine,
    scripts: Vec<CompiledScript>,
}

impl ScriptCompiler {
    /// A compiler with no scripts yet and the sandbox limits applied.
    pub(crate) fn new() -> Self {
        let mut engine = Engine::new_raw();
        apply_limits(&mut engine);
        Self {
            engine,
            scripts: Vec::new(),
        }
    }

    /// Compiles a single expression, as used by conditions and considerations.
    pub(crate) fn expression(&mut self, owner: &str, source: &str) -> Result<ScriptId, DefsError> {
        let ast = self
            .engine
            .compile_expression(source)
            .map_err(|err| script_error(owner, &err))?;
        self.push(owner, ast)
    }

    /// Compiles a statement list, as used by trigger bodies, effects and interaction scripts.
    pub(crate) fn statements(&mut self, owner: &str, source: &str) -> Result<ScriptId, DefsError> {
        let ast = self
            .engine
            .compile(source)
            .map_err(|err| script_error(owner, &err))?;
        self.push(owner, ast)
    }

    fn push(&mut self, owner: &str, ast: AST) -> Result<ScriptId, DefsError> {
        let index = u32::try_from(self.scripts.len()).map_err(|_| DefsError::Invalid {
            owner: owner.to_owned(),
            message: "too many scripts (limit 2^32)".to_owned(),
        })?;
        self.scripts.push(CompiledScript {
            owner: owner.to_owned(),
            ast,
        });
        Ok(ScriptId(index))
    }

    /// The compiled scripts, in `ScriptId` order.
    pub(crate) fn into_scripts(self) -> Vec<CompiledScript> {
        self.scripts
    }
}

/// What a piece of authored script text must be, so [`check_script`] parses it the way the
/// definitions compiler will.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptKind {
    /// A single expression: a trigger `if`, a consideration, an interaction `condition`.
    Expression,
    /// A statement list: a trigger `then`, an effect, an interaction script.
    Statements,
}

/// Where and why a script does not parse, from [`check_script`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptSyntaxError {
    /// The 1-based line of the error, when Rhai knows it.
    pub line: Option<usize>,
    /// The 1-based column of the error, when Rhai knows it.
    pub column: Option<usize>,
    /// Rhai's description of the error, without the position.
    pub message: String,
}

impl std::fmt::Display for ScriptSyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.line, self.column) {
            (Some(line), Some(column)) => {
                write!(f, "line {line}, column {column}: {}", self.message)
            }
            (Some(line), None) => write!(f, "line {line}: {}", self.message),
            _ => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for ScriptSyntaxError {}

/// Parses script text under the same sandbox limits as the definitions compiler and reports the
/// first syntax error with its line and column, without needing a [`DefsFile`](crate::DefsFile)
/// or a running app. An editor calls this on every edit and refuses to save text that fails.
/// Nothing is evaluated; a script that parses can still fail at run time (an unknown function is
/// found only when it runs).
pub fn check_script(source: &str, kind: ScriptKind) -> Result<(), ScriptSyntaxError> {
    let mut engine = Engine::new_raw();
    apply_limits(&mut engine);
    let parsed = match kind {
        ScriptKind::Expression => engine.compile_expression(source),
        ScriptKind::Statements => engine.compile(source),
    };
    parsed.map(|_| ()).map_err(|err| ScriptSyntaxError {
        line: err.1.line(),
        column: err.1.position(),
        message: err.0.to_string(),
    })
}

fn script_error(owner: &str, err: &ParseError) -> DefsError {
    DefsError::Script {
        owner: owner.to_owned(),
        message: err.to_string(),
    }
}

/// The values a script sees as bare constants for one evaluation (plan D8). Absent ids are −1 so a
/// script can test `actor >= 0` without an optional type; `accepted` is bound only when the event
/// carries it.
///
/// `Default` is hand-written because a derived one would bind every id to 0, which is the id of the
/// first person spawned: an event with no actor would silently read as "person 0".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Bindings {
    /// The actor's [`PersonId`], or −1.
    pub(crate) actor: i64,
    /// The target's [`PersonId`], or −1.
    pub(crate) target: i64,
    /// The rumor subject's [`PersonId`], or −1.
    pub(crate) subject: i64,
    /// The rumor id, or −1.
    pub(crate) rumor: i64,
    /// Whether an interaction was accepted, when the event carries it.
    pub(crate) accepted: Option<bool>,
}

impl Default for Bindings {
    fn default() -> Self {
        Self {
            actor: -1,
            target: -1,
            subject: -1,
            rumor: -1,
            accepted: None,
        }
    }
}

impl Bindings {
    /// The binding of one actor, with every other id absent.
    pub(crate) fn of_actor(id: Option<PersonId>) -> Self {
        Self {
            actor: id.map_or(-1, |p| i64::from(p.0)),
            ..Self::default()
        }
    }
}

/// A script that failed: the authored path that names it and what went wrong (plan D8).
///
/// The engine also logs the same text with `error!`, so a game that ignores this message still sees
/// the failure in its log.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct ScriptError {
    /// The authored path, for example `trigger 'greet'`.
    pub owner: String,
    /// The engine's message: a runtime error, a wrong return type or the operation limit.
    pub message: String,
}

/// A named event a script emitted with `fx.emit` (plan D8).
///
/// `actor` and `target` are the ids the evaluation was bound to, so a trigger can emit an event
/// about the people it was already about without repeating them.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct ScriptEvent {
    /// The event's name, the script's first argument to `fx.emit`.
    pub name: String,
    /// The person the event is about, when the evaluation was bound to one.
    pub actor: Option<Entity>,
    /// The person it is directed at, when the evaluation was bound to one.
    pub target: Option<Entity>,
    /// The payload, sorted by key as Rhai's map is.
    pub data: Vec<(String, ScriptValue)>,
}

/// One value an `fx.emit` call carries. Any other Rhai type is a runtime error at `emit` time.
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptValue {
    /// A boolean.
    Bool(bool),
    /// A 64-bit integer.
    Int(i64),
    /// A 32-bit float.
    Float(f32),
    /// A string.
    Text(String),
}

/// One write a script asked for, in [`PersonId`] terms (plan D8). The caller resolves each id and
/// applies the action; unknown ids are a `warn!` and the action is skipped.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ScriptAction {
    /// `fx.adjust(a, b, field, delta)`; `field` is `affinity`, `trust`, `respect` or `familiarity`.
    Adjust {
        /// The person whose tie moves.
        person: i64,
        /// The tie's target.
        other: i64,
        /// Which of the four tie values.
        field: String,
        /// How much it moves.
        delta: f32,
    },
    /// `fx.emote(a, kind, intensity)`.
    Emote {
        /// Who feels it.
        person: i64,
        /// The emotion's lowercase name.
        kind: String,
        /// Its intensity.
        intensity: f32,
    },
    /// `fx.need(a, id, delta)`.
    Need {
        /// Whose need moves.
        person: i64,
        /// The need's name.
        need: String,
        /// How much it moves.
        delta: f32,
    },
    /// `fx.opinion(a, topic, delta)`.
    Opinion {
        /// Whose opinion moves.
        person: i64,
        /// The topic's name.
        topic: String,
        /// How much it moves.
        delta: f32,
    },
    /// `fx.join(a, faction)`.
    Join {
        /// Who joins.
        person: i64,
        /// The faction's name.
        faction: String,
    },
    /// `fx.leave(a, faction)`.
    Leave {
        /// Who leaves.
        person: i64,
        /// The faction's name.
        faction: String,
    },
    /// `fx.rumor(origin, subject, deed, truth)`.
    Rumor {
        /// Who starts it.
        origin: i64,
        /// Who it is about.
        subject: i64,
        /// The deed's name.
        deed: String,
        /// Whether it reports what really happened.
        truth: bool,
    },
    /// `fx.interact(a, b, interaction)`.
    Interact {
        /// Who starts the exchange.
        actor: i64,
        /// Who it is directed at.
        target: i64,
        /// The interaction's name.
        interaction: String,
    },
    /// `fx.bark(a, text)`.
    Bark {
        /// Who says it.
        speaker: i64,
        /// The line.
        text: String,
    },
    /// `fx.think(a, text)`.
    Think {
        /// Who thinks it.
        speaker: i64,
        /// The line.
        text: String,
    },
    /// `fx.remember(a, b, text, valence, importance)`; `b` is −1 when the memory is about nobody.
    Remember {
        /// Whose memory it is.
        person: i64,
        /// The other person, or −1.
        other: i64,
        /// The text, stored as `MemoryKind::Custom`.
        text: String,
        /// Good (+) or bad (−).
        valence: f32,
        /// How much it matters.
        importance: f32,
    },
    /// `fx.converse(npc, player)`.
    Converse {
        /// The non-player person.
        npc: i64,
        /// The player person.
        player: i64,
    },
    /// `fx.emit(name, data)`, with the actor and target taken from the event's bindings.
    Emit {
        /// The event's name.
        name: String,
        /// The event's actor binding.
        actor: i64,
        /// The event's target binding.
        target: i64,
        /// Its payload.
        data: Vec<(String, ScriptValue)>,
    },
}

/// The `ctx` argument of a script: read-only access to the tick's frozen society (plan D8).
#[derive(Clone)]
pub(crate) struct Ctx(pub(crate) Arc<SocietySnapshot>);

/// The `fx` argument of a script: the queue its writes go to, drained by the caller.
///
/// The queue is shared with the evaluation, so `emit` can carry the event's own bindings without
/// the script having to repeat them.
#[derive(Clone)]
pub(crate) struct Fx {
    queue: Arc<Mutex<Vec<ScriptAction>>>,
    actor: i64,
    target: i64,
}

/// Turns a Rhai error into the `String` the engine's callers report.
fn boxed(err: String) -> Box<EvalAltResult> {
    Box::new(EvalAltResult::ErrorRuntime(
        Dynamic::from(err),
        rhai::Position::NONE,
    ))
}

/// The Rhai engine every runtime script runs on, with the `ctx` / `fx` API registered (plan D8).
///
/// A resource so the ASTs in `SocietyDefs` can be evaluated without re-registering the API each
/// time. The engine holds no state between evaluations: scopes, snapshots and action queues are all
/// per call.
#[derive(Resource)]
pub struct ScriptEngine {
    engine: Engine,
}

impl Default for ScriptEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ScriptEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScriptEngine").finish_non_exhaustive()
    }
}

impl ScriptEngine {
    /// A fresh engine with the sandbox limits and the whole `ctx` / `fx` API registered.
    pub fn new() -> Self {
        let mut engine = Engine::new();
        apply_limits(&mut engine);
        register_ctx(&mut engine);
        register_fx(&mut engine);
        Self { engine }
    }

    /// Evaluates a condition: a script that must yield a boolean. A wrong return type is an error,
    /// which callers treat as "did not hold" (plan D8).
    pub(crate) fn eval_bool(
        &self,
        scripts: &[CompiledScript],
        id: ScriptId,
        snapshot: &Arc<SocietySnapshot>,
        bindings: Bindings,
    ) -> Result<bool, String> {
        let mut scope = scope_with(snapshot, bindings);
        let value: Dynamic = self
            .engine
            .eval_ast_with_scope(&mut scope, ast(scripts, id)?)
            .map_err(|err| err.to_string())?;
        let type_name = value.type_name();
        value
            .try_cast::<bool>()
            .ok_or_else(|| format!("a condition returned {type_name}, not a bool"))
    }

    /// Evaluates a consideration: a script that must yield a number. Callers clamp the result to
    /// `[0, 1]`; a wrong return type is an error, which callers score as 0 (plan D8).
    pub(crate) fn eval_f32(
        &self,
        scripts: &[CompiledScript],
        id: ScriptId,
        snapshot: &Arc<SocietySnapshot>,
        bindings: Bindings,
    ) -> Result<f32, String> {
        let mut scope = scope_with(snapshot, bindings);
        let value: Dynamic = self
            .engine
            .eval_ast_with_scope(&mut scope, ast(scripts, id)?)
            .map_err(|err| err.to_string())?;
        if let Some(n) = value.clone().try_cast::<f32>() {
            return Ok(n);
        }
        if let Some(n) = value.clone().try_cast::<i64>() {
            return Ok(n as f32);
        }
        Err(format!(
            "a consideration returned {}, not a number",
            value.type_name()
        ))
    }

    /// Runs a statement script and returns the actions it queued, in queue order (plan D8).
    ///
    /// On an error the queued actions are dropped: a failed effect applies none of its actions, so
    /// a script cannot half-apply.
    pub(crate) fn run(
        &self,
        scripts: &[CompiledScript],
        id: ScriptId,
        snapshot: &Arc<SocietySnapshot>,
        bindings: Bindings,
    ) -> Result<Vec<ScriptAction>, String> {
        let queue = Arc::new(Mutex::new(Vec::new()));
        let mut scope = scope_with(snapshot, bindings);
        scope.push(
            "fx",
            Fx {
                queue: Arc::clone(&queue),
                actor: bindings.actor,
                target: bindings.target,
            },
        );
        self.engine
            .run_ast_with_scope(&mut scope, ast(scripts, id)?)
            .map_err(|err| err.to_string())?;
        let actions = match queue.lock() {
            Ok(mut q) => std::mem::take(&mut *q),
            Err(poisoned) => std::mem::take(&mut *poisoned.into_inner()),
        };
        Ok(actions)
    }

    /// The authored path of a script, for error messages.
    pub(crate) fn owner(scripts: &[CompiledScript], id: ScriptId) -> &str {
        scripts
            .get(id.0 as usize)
            .map_or("<unknown script>", |s| s.owner.as_str())
    }
}

fn ast(scripts: &[CompiledScript], id: ScriptId) -> Result<&AST, String> {
    scripts
        .get(id.0 as usize)
        .map(|s| &s.ast)
        .ok_or_else(|| format!("no script with id {}", id.0))
}

/// The scope every script sees at the tick start: the bound ids, and `ctx` (plan D8).
///
/// `ctx` is pushed with `push`, not `push_constant`, because Rhai's method syntax needs a mutable
/// reference to the value; the snapshot behind it is immutable anyway.
fn scope_with<'a>(snapshot: &Arc<SocietySnapshot>, bindings: Bindings) -> Scope<'a> {
    let mut scope = Scope::new();
    scope.push_constant("actor", bindings.actor);
    scope.push_constant("target", bindings.target);
    scope.push_constant("subject", bindings.subject);
    scope.push_constant("rumor", bindings.rumor);
    if let Some(accepted) = bindings.accepted {
        scope.push_constant("accepted", accepted);
    }
    scope.push("ctx", Ctx(Arc::clone(snapshot)));
    scope
}

macro_rules! ctx_pair {
    ($engine:expr, $name:literal, $method:ident) => {
        $engine.register_fn(
            $name,
            |ctx: &mut Ctx, a: i64, b: i64| -> Result<f32, Box<EvalAltResult>> {
                ctx.0.$method(a, b).map_err(boxed)
            },
        );
    };
}

macro_rules! ctx_one {
    ($engine:expr, $name:literal, $method:ident) => {
        $engine.register_fn(
            $name,
            |ctx: &mut Ctx, a: i64| -> Result<f32, Box<EvalAltResult>> {
                ctx.0.$method(a).map_err(boxed)
            },
        );
    };
}

fn register_ctx(engine: &mut Engine) {
    engine.register_type_with_name::<Ctx>("Ctx");

    // Tie values. `ctx.affinity(a, b)` and friends are global functions whose first parameter is
    // the receiver, which is how Rhai's method-call syntax finds them.
    ctx_pair!(engine, "affinity", affinity);
    ctx_pair!(engine, "trust", trust);
    ctx_pair!(engine, "respect", respect);
    ctx_pair!(engine, "familiarity", familiarity);
    ctx_pair!(engine, "grudge", grudge);
    ctx_one!(engine, "standing", standing);
    engine.register_fn(
        "label",
        |ctx: &mut Ctx, a: i64, b: i64| -> Result<String, Box<EvalAltResult>> {
            ctx.0.label(a, b).map_err(boxed)
        },
    );

    // Mind.
    engine.register_fn(
        "mood",
        |ctx: &mut Ctx, a: i64| -> Result<rhai::Map, Box<EvalAltResult>> {
            let mood = ctx.0.mood(a).map_err(boxed)?;
            let mut map = rhai::Map::new();
            map.insert("p".into(), Dynamic::from(mood[0]));
            map.insert("a".into(), Dynamic::from(mood[1]));
            map.insert("d".into(), Dynamic::from(mood[2]));
            Ok(map)
        },
    );
    engine.register_fn(
        "emotion",
        |ctx: &mut Ctx, a: i64, kind: &str| -> Result<f32, Box<EvalAltResult>> {
            ctx.0.emotion(a, kind).map_err(boxed)
        },
    );
    engine.register_fn(
        "need",
        |ctx: &mut Ctx, a: i64, need: &str| -> Result<f32, Box<EvalAltResult>> {
            ctx.0.need(a, need).map_err(boxed)
        },
    );
    engine.register_fn(
        "signal",
        |ctx: &mut Ctx, a: i64, signal: &str| -> Result<f32, Box<EvalAltResult>> {
            ctx.0.signal(a, signal).map_err(boxed)
        },
    );
    // The goods half of the same interface: what somebody holds and what it is worth. A rule that wants to
    // react to a shortage reads `ctx.price("bread")`; a rule that wants to react to one villager's hunger
    // reads `ctx.stock(actor, "bread")`.
    engine.register_fn(
        "stock",
        |ctx: &mut Ctx, a: i64, good: &str| -> Result<f32, Box<EvalAltResult>> {
            ctx.0.stock(a, good).map_err(boxed)
        },
    );
    engine.register_fn(
        "price",
        |ctx: &mut Ctx, good: &str| -> Result<f32, Box<EvalAltResult>> {
            ctx.0.price(good).map_err(boxed)
        },
    );
    engine.register_fn(
        "opinion",
        |ctx: &mut Ctx, a: i64, topic: &str| -> Result<f32, Box<EvalAltResult>> {
            ctx.0.opinion(a, topic).map_err(boxed)
        },
    );
    engine.register_fn(
        "trait",
        |ctx: &mut Ctx, a: i64, name: &str| -> Result<f32, Box<EvalAltResult>> {
            ctx.0.trait_value(a, name).map_err(boxed)
        },
    );

    // Society.
    engine.register_fn(
        "knows",
        |ctx: &mut Ctx, a: i64, rumor: i64| -> Result<bool, Box<EvalAltResult>> {
            ctx.0.knows(a, rumor).map_err(boxed)
        },
    );
    engine.register_fn(
        "tellable",
        |ctx: &mut Ctx, a: i64| -> Result<bool, Box<EvalAltResult>> {
            ctx.0.tellable(a).map_err(boxed)
        },
    );
    engine.register_fn(
        "member",
        |ctx: &mut Ctx, a: i64, faction: &str| -> Result<bool, Box<EvalAltResult>> {
            ctx.0.member(a, faction).map_err(boxed)
        },
    );
    engine.register_fn(
        "name",
        |ctx: &mut Ctx, a: i64| -> Result<String, Box<EvalAltResult>> {
            ctx.0.name(a).map_err(boxed)
        },
    );
    engine.register_fn(
        "doing",
        |ctx: &mut Ctx, a: i64| -> Result<String, Box<EvalAltResult>> {
            ctx.0.doing(a).map_err(boxed)
        },
    );
    engine.register_fn(
        "position",
        |ctx: &mut Ctx, a: i64| -> Result<rhai::Map, Box<EvalAltResult>> {
            let [x, y] = ctx.0.position(a).map_err(boxed)?;
            let mut map = rhai::Map::new();
            map.insert("x".into(), Dynamic::from(x));
            map.insert("y".into(), Dynamic::from(y));
            Ok(map)
        },
    );
    ctx_pair!(engine, "distance", distance);
    // `Array`, not `Vec<i64>`: Rhai's indexer is registered for its own array type, so a plain
    // `Vec<i64>` would arrive in the script without being indexable (`ctx.persons()[0]` would fail).
    engine.register_fn("persons", |ctx: &mut Ctx| -> rhai::Array {
        ctx.0.person_ids().into_iter().map(Dynamic::from).collect()
    });
    engine.register_fn(
        "faction_size",
        |ctx: &mut Ctx, faction: &str| -> Result<i64, Box<EvalAltResult>> {
            ctx.0.faction_size(faction).map_err(boxed)
        },
    );
    engine.register_fn("hour", |ctx: &mut Ctx| -> f32 { ctx.0.hour() });
    engine.register_fn("day", |ctx: &mut Ctx| -> i64 { ctx.0.day() });
    engine.register_fn("weekday", |ctx: &mut Ctx| -> i64 { ctx.0.weekday() });
    engine.register_fn("rand", |ctx: &mut Ctx| -> f32 { ctx.0.unit() });
}

macro_rules! fx_push {
    ($engine:expr, $name:literal, |$fx:ident $(, $arg:ident : $ty:ty)*| $make:expr) => {
        $engine.register_fn(
            $name,
            |$fx: &mut Fx $(, $arg: $ty)*| -> Result<(), Box<EvalAltResult>> {
                let action = $make;
                let mut queue = match $fx.queue.lock() {
                    Ok(queue) => queue,
                    Err(poisoned) => poisoned.into_inner(),
                };
                queue.push(action);
                Ok(())
            },
        );
    };
}

fn register_fx(engine: &mut Engine) {
    engine.register_type_with_name::<Fx>("Fx");

    fx_push!(
        engine,
        "adjust",
        |fx, person: i64, other: i64, field: String, delta: f32| ScriptAction::Adjust {
            person,
            other,
            field,
            delta,
        }
    );
    fx_push!(
        engine,
        "emote",
        |fx, person: i64, kind: String, intensity: f32| ScriptAction::Emote {
            person,
            kind,
            intensity,
        }
    );
    fx_push!(engine, "need", |fx,
                              person: i64,
                              need: String,
                              delta: f32| {
        ScriptAction::Need {
            person,
            need,
            delta,
        }
    });
    fx_push!(
        engine,
        "opinion",
        |fx, person: i64, topic: String, delta: f32| ScriptAction::Opinion {
            person,
            topic,
            delta,
        }
    );
    fx_push!(engine, "join", |fx, person: i64, faction: String| {
        ScriptAction::Join { person, faction }
    });
    fx_push!(engine, "leave", |fx, person: i64, faction: String| {
        ScriptAction::Leave { person, faction }
    });
    fx_push!(
        engine,
        "rumor",
        |fx, origin: i64, subject: i64, deed: String, truth: bool| ScriptAction::Rumor {
            origin,
            subject,
            deed,
            truth,
        }
    );
    fx_push!(
        engine,
        "interact",
        |fx, actor: i64, target: i64, interaction: String| ScriptAction::Interact {
            actor,
            target,
            interaction,
        }
    );
    fx_push!(engine, "bark", |fx, speaker: i64, text: String| {
        ScriptAction::Bark { speaker, text }
    });
    fx_push!(engine, "think", |fx, speaker: i64, text: String| {
        ScriptAction::Think { speaker, text }
    });
    fx_push!(
        engine,
        "remember",
        |fx, person: i64, other: i64, text: String, valence: f32, importance: f32| {
            ScriptAction::Remember {
                person,
                other,
                text,
                valence,
                importance,
            }
        }
    );

    register_converse(engine);

    // `emit` is separate: its actor and target come from the evaluation's bindings, and its payload
    // has to be converted from a Rhai map, which can fail.
    engine.register_fn(
        "emit",
        |fx: &mut Fx, name: String, data: rhai::Map| -> Result<(), Box<EvalAltResult>> {
            let mut values = Vec::with_capacity(data.len());
            for (key, value) in data {
                let value = if let Some(b) = value.clone().try_cast::<bool>() {
                    ScriptValue::Bool(b)
                } else if let Some(n) = value.clone().try_cast::<i64>() {
                    ScriptValue::Int(n)
                } else if let Some(n) = value.clone().try_cast::<f32>() {
                    ScriptValue::Float(n)
                } else if let Some(text) = value.clone().try_cast::<rhai::ImmutableString>() {
                    ScriptValue::Text(text.to_string())
                } else {
                    return Err(boxed(format!(
                        "fx.emit value '{key}' is {}, which a ScriptValue cannot carry",
                        value.type_name()
                    )));
                };
                values.push((key.to_string(), value));
            }
            let mut queue = match fx.queue.lock() {
                Ok(queue) => queue,
                Err(poisoned) => poisoned.into_inner(),
            };
            queue.push(ScriptAction::Emit {
                name,
                actor: fx.actor,
                target: fx.target,
                data: values,
            });
            Ok(())
        },
    );
}

/// `fx.converse` queues a conversation. The plugin that runs conversations is `dialogue`; without
/// it the call is a runtime error, so a script never looks like it worked.
#[cfg(feature = "dialogue")]
fn register_converse(engine: &mut Engine) {
    fx_push!(engine, "converse", |fx, npc: i64, player: i64| {
        ScriptAction::Converse { npc, player }
    });
}

#[cfg(not(feature = "dialogue"))]
fn register_converse(engine: &mut Engine) {
    engine.register_fn(
        "converse",
        |_fx: &mut Fx, _npc: i64, _player: i64| -> Result<(), Box<EvalAltResult>> {
            Err(boxed("fx.converse needs the `dialogue` feature".to_owned()))
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The registered default is the whole API, not an empty engine: `init_resource` uses this, so
    /// a derive that skipped the registration would make every `fx` call fail at runtime.
    #[test]
    fn the_default_engine_has_the_whole_api() {
        let engine = ScriptEngine::default();
        let mut scope = Scope::new();
        scope.push(
            "fx",
            Fx {
                queue: Arc::new(Mutex::new(Vec::new())),
                actor: 0,
                target: -1,
            },
        );
        let result = engine
            .engine
            .run_with_scope(&mut scope, "fx.emit(\"met\", #{ n: 1 })");
        assert!(result.is_ok(), "{result:?}");
    }
}
