//! Rhai compilation for authored scripts: conditions, considerations, trigger and effect bodies.
//! The runtime engine with the `ctx`/`fx` API arrives with `RulesPlugin` (plan D8). This file only
//! compiles and stores syntax trees, under the same sandbox limits the runtime will use.

use bevy::reflect::Reflect;
use rhai::{Engine, ParseError};

use crate::defs::DefsError;

/// Index of a compiled script inside `SocietyDefs`, in compile order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Reflect)]
pub struct ScriptId(pub u32);

/// A compiled script with the authored path that names it, so runtime errors can say where they
/// came from.
pub(crate) struct CompiledScript {
    /// The authored path, for example `trigger 'greet'`.
    pub(crate) owner: String,
    /// The compiled syntax tree.
    pub(crate) ast: rhai::AST,
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

    fn push(&mut self, owner: &str, ast: rhai::AST) -> Result<ScriptId, DefsError> {
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

fn script_error(owner: &str, err: &ParseError) -> DefsError {
    DefsError::Script {
        owner: owner.to_owned(),
        message: err.to_string(),
    }
}
