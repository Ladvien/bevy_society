//! Shared example code: the village scene both examples build, the visuals on top of it, and the
//! headless recorder.
//!
//! Cargo compiles this module into each example separately, so an item only one example uses is dead
//! code in the other. The allow is for that, not for leaving genuine dead weight around.
#![allow(dead_code)]

pub mod recorder;
pub mod scene;
pub mod scenes;
pub mod visuals;
