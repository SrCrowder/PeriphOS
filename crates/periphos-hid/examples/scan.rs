//! Prints the devices the generic HID driver detects.
//!
//! Run with: `cargo run -p periphos-hid --example scan`
//! Useful when reporting a device: include this output in the issue.

use periphos_core::device::DeviceDriver;
use periphos_hid::HidDriver;

fn main() {
    let driver = HidDriver::new();
    match driver.enumerate() {
        Ok(devices) if devices.is_empty() => println!("No mice, keyboards or controllers found."),
        Ok(devices) => {
            for d in devices {
                println!(
                    "{:04X}:{:04X}  {} {}  ({:?}, {:?}, {} interfaces, {})",
                    d.vendor_id,
                    d.product_id,
                    d.manufacturer,
                    d.name,
                    d.kind,
                    d.connection,
                    d.interface_count,
                    if d.known_model {
                        "known model"
                    } else {
                        "generic"
                    }
                );
            }
        }
        Err(err) => eprintln!("Scan failed: {err}"),
    }
}
