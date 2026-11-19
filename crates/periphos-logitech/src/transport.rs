//! HID++ over `hidapi`.
//!
//! On Windows every top-level HID collection is a separate handle: short
//! reports (0x10) go through the vendor collection with usage 0x0001 and long
//! reports (0x11) through usage 0x0002, and replies may arrive on either. On
//! Linux one hidraw node carries both. This transport hides the difference.

use std::time::{Duration, Instant};

use hidapi::{HidApi, HidDevice};

use crate::protocol::SHORT_REPORT;
use crate::session::Transport;

pub const HIDPP_USAGE_PAGE: u16 = 0xFF00;
pub const USAGE_SHORT: u16 = 0x0001;
pub const USAGE_LONG: u16 = 0x0002;

/// Time spent waiting on one handle before checking the next.
const POLL_SLICE_MS: i32 = 5;

pub struct HidTransport {
    /// (usage, handle) for each opened HID++ collection.
    handles: Vec<(u16, HidDevice)>,
}

impl HidTransport {
    /// Opens the HID++ collections at `paths` (usage, path). Fails if none can be opened.
    pub fn open(api: &HidApi, paths: &[(u16, std::ffi::CString)]) -> Result<Self, String> {
        let mut handles = Vec::new();
        let mut last_error = String::from("no HID++ interface");
        for (usage, path) in paths {
            match api.open_path(path) {
                Ok(device) => handles.push((*usage, device)),
                Err(e) => last_error = e.to_string(),
            }
        }
        if handles.is_empty() {
            return Err(last_error);
        }
        Ok(Self { handles })
    }

    fn handle_for(&self, report_id: u8) -> &HidDevice {
        let wanted = if report_id == SHORT_REPORT {
            USAGE_SHORT
        } else {
            USAGE_LONG
        };
        self.handles
            .iter()
            .find(|(usage, _)| *usage == wanted)
            .or_else(|| self.handles.first())
            .map(|(_, h)| h)
            .expect("transport has at least one handle")
    }
}

impl Transport for HidTransport {
    fn write(&mut self, report: &[u8]) -> Result<(), String> {
        self.handle_for(report[0])
            .write(report)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn read(&mut self, timeout: Duration) -> Result<Option<Vec<u8>>, String> {
        let deadline = Instant::now() + timeout;
        let mut buffer = [0u8; 64];
        loop {
            for (_, handle) in &self.handles {
                let size = handle
                    .read_timeout(&mut buffer, POLL_SLICE_MS)
                    .map_err(|e| e.to_string())?;
                if size > 0 {
                    return Ok(Some(buffer[..size].to_vec()));
                }
            }
            if Instant::now() >= deadline {
                return Ok(None);
            }
        }
    }
}
