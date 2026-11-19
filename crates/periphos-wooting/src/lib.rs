//! # periphos-wooting
//!
//! Wooting keyboard driver (`wooting`). It lists Wooting keyboards and
//! switches between the profiles stored on the keyboard. Each of those
//! profiles holds its own actuation points, rapid trigger, key bindings and
//! lighting, all configured in Wootility; PeriphOS only chooses which one is
//! active, so every PeriphOS profile (and game) can use a different one.
//!
//! Wootility must not be running at the same time: it talks to the keyboard
//! through the same interface and switches profiles itself.

pub mod protocol;

mod driver;

pub use driver::{WootingDriver, WOOTING_VID};
