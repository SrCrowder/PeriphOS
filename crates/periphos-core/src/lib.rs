//! # periphos-core
//!
//! The hardware-agnostic heart of PeriphOS. This crate defines the domain
//! model shared by every other part of the application:
//!
//! - [`device`]: the unified device abstraction, capability model and the
//!   [`device::DeviceDriver`] trait that manufacturer adapters implement.
//! - [`profile`]: configuration profiles and their validation rules.
//! - [`settings`]: application preferences.
//!
//! It deliberately has **no dependency on Tauri, SQLite or any OS API** so it
//! can be unit-tested in isolation and reused by a future background service
//! or command-line tool.

pub mod device;
pub mod error;
pub mod profile;
pub mod settings;

pub use error::{CoreError, CoreResult};
