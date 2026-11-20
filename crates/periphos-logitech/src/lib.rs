//! # periphos-logitech
//!
//! Logitech HID++ driver (`logitech-hidpp`). It lists the devices connected
//! through Logitech receivers (Unifying, LIGHTSPEED, Nano, Bolt), reads their
//! battery, and changes sensitivity (DPI) and report rate. Every change is
//! validated against the values the device reports and read back afterwards.
//!
//! - [`protocol`]: report layout and decoders (pure, tested).
//! - [`session`]: request/reply matching over a [`session::Transport`], tested
//!   against simulated devices.
//! - [`probe`]: what is paired with a receiver.
//! - [`settings`]: DPI, report rate and onboard-profile mode.
//! - [`diagnostics`]: read-only technical report (features, controls, memory).
//! - `transport` / `driver`: real I/O through `hidapi`.
//!
//! The protocol was implemented from public descriptions of HID++ (Logitech's
//! feature documentation and the Solaar and libratbag projects). No code from
//! those projects is used.

pub mod diagnostics;
pub mod probe;
pub mod protocol;
pub mod receivers;
pub mod session;
pub mod settings;

mod driver;
mod transport;

pub use driver::LogitechDriver;
