//! **Log capture for tests.** A `LogPlugin` layer that records every event into a channel a test can
//! read, so a warning can be asserted instead of read by hand.
//!
//! Two shipped warnings — the degenerate-duration one and the unscored-field one — are currently
//! pinned only by the *facts* they key on, because `headless` has no logger and a test that builds an
//! app never sees a `warn!`. Deleting either `warn!` call would leave every test green. This fixes
//! that.
//!
//! The pattern is Bevy's own `examples/app/log_layers_ecs.rs`: a `LogPlugin::custom_layer` receives
//! `&mut App`, so it can build a channel, keep the sender in the layer and insert the receiver as a
//! resource.
//!
//! Only compiled for tests: `tests/common` is the test harness, and nothing in the crate's published
//! surface depends on it.

use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};

use bevy::log::{BoxedLayer, tracing, tracing_subscriber::Layer};
use bevy::prelude::*;

/// One captured log event: its level and its rendered message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedLog {
    /// `"WARN"`, `"ERROR"`, `"INFO"`, …
    pub level: String,
    /// The event's message, as `warn!`/`error!`/`info!` produced it.
    pub message: String,
}

/// The receiving end. `mpsc::Receiver` is not `Sync`, and Bevy 0.20 has no non-send resources, so it
/// is wrapped in a `Mutex` to make the resource legal. Only the test thread reads it.
#[derive(Resource)]
pub struct LogCapture(Mutex<Receiver<CapturedLog>>);

impl LogCapture {
    /// Every event captured so far, oldest first.
    pub fn drain(&self) -> Vec<CapturedLog> {
        let receiver = match self.0.lock() {
            Ok(receiver) => receiver,
            Err(poisoned) => poisoned.into_inner(),
        };
        receiver.try_iter().collect()
    }

    /// Every captured message whose text contains `needle`.
    pub fn containing(&self, needle: &str) -> Vec<String> {
        self.drain()
            .into_iter()
            .filter(|event| event.message.contains(needle))
            .map(|event| event.message)
            .collect()
    }
}

/// The sending end, held by the layer.
struct LogSender(Sender<CapturedLog>);

/// Forwards every event into the channel.
impl<S: tracing::Subscriber> Layer<S> for LogSender {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _context: bevy::log::tracing_subscriber::layer::Context<'_, S>,
    ) {
        // The message is a `tracing` field named `message`; a visitor is the only way to read one.
        let mut message = String::new();
        event.record(&mut MessageVisitor(&mut message));
        let captured = CapturedLog {
            level: event.metadata().level().to_string(),
            message,
        };
        // A closed receiver means the test finished; dropping the event is correct then.
        let _ = self.0.send(captured);
    }
}

/// Collects the `message` field of one event.
struct MessageVisitor<'a>(&'a mut String);

impl tracing::field::Visit for MessageVisitor<'_> {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.0.push_str(value);
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            use std::fmt::Write as _;
            let _ = write!(self.0, "{value:?}");
        }
    }
}

/// A `LogPlugin` with the capture layer attached, for a test app.
///
/// ```ignore
/// app.add_plugins(
///     MinimalPlugins,
///     // …
/// );
/// app.add_plugins(capture::log_plugin());
/// ```
pub fn log_plugin() -> bevy::log::LogPlugin {
    bevy::log::LogPlugin {
        custom_layer,
        ..default()
    }
}

/// Builds the channel, keeps the sender, and inserts the receiver.
fn custom_layer(app: &mut App) -> Option<BoxedLayer> {
    let (sender, receiver) = channel();
    app.insert_resource(LogCapture(Mutex::new(receiver)));
    Some(Box::new(LogSender(sender)))
}
