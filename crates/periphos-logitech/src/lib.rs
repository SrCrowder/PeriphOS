//! # periphos-logitech
//!
//! Logitech HID++ driver (`logitech-hidpp`). For now it lists the devices
//! connected through Logitech receivers (Unifying, LIGHTSPEED, Nano, Bolt)
//! and reads their battery. It only sends read requests.
//!
//! - [`protocol`]: report layout and decoders (pure, tested).
//! - [`session`]: request/reply matching over a [`session::Transport`], tested
//!   against simulated devices.
//! - [`probe`]: what is paired with a receiver.
//! - `transport` / `driver`: real I/O through `hidapi`.
//!
//! The protocol was implemented from public descriptions of HID++ (Logitech's
//! feature documentation and the Solaar and libratbag projects). No code from
//! those projects is used.

pub mod probe;
pub mod protocol;
pub mod receivers;
pub mod session;

mod driver;
mod transport;

pub use driver::LogitechDriver;
