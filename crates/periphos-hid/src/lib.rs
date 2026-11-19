//! # periphos-hid
//!
//! Generic USB/Bluetooth HID detection for PeriphOS (the `hid-generic`
//! driver). It identifies mice, keyboards, controllers and known receivers
//! and reports what each model can do, using the device database in
//! `devices.json`.
//!
//! This driver only **identifies** devices. It never opens them or sends
//! reports, so every hardware feature it lists is `vendorProtocol` until a
//! manufacturer driver implements it.
//!
//! The grouping and classification logic lives in [`classify`] and works on
//! plain data, so it is tested without hardware; [`HidDriver`] only feeds it
//! with what `hidapi` enumerates.

pub mod classify;
pub mod database;
mod driver;

pub use driver::HidDriver;
