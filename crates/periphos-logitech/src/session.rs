//! Request/reply exchange over an abstract transport.

use std::time::{Duration, Instant};

use crate::protocol::{
    feature, hidpp10_error, match_reply, Reply, Request, PAIRING_INFO_SUB_REGISTER,
    RECEIVER_INFO_REGISTER,
};

/// Something that carries HID++ reports: the HID handles of a receiver or of
/// a device connected by cable.
pub trait Transport {
    fn write(&mut self, report: &[u8]) -> Result<(), String>;
    /// Next input report, or `None` when nothing arrived within `timeout`.
    fn read(&mut self, timeout: Duration) -> Result<Option<Vec<u8>>, String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Io(String),
    Timeout,
    Feature(u8),
    Hidpp10(u8),
}

/// Result of pinging a device slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presence {
    /// HID++ 2.0 or newer device, online.
    Online { major: u8, minor: u8 },
    /// A HID++ 1.0 device is there; it does not support the 2.0 features used here.
    Legacy,
    /// No device is paired in this slot.
    Empty,
    /// A device may be paired but did not answer: switched off, asleep to save
    /// battery, out of range, or the reply was lost among other traffic.
    Unreachable,
}

pub struct Session<T: Transport> {
    transport: T,
    timeout: Duration,
}

impl<T: Transport> Session<T> {
    pub fn new(transport: T, timeout: Duration) -> Self {
        Self { transport, timeout }
    }

    pub fn call(&mut self, request: &Request) -> Result<Vec<u8>, Error> {
        self.transport.write(&request.encode()).map_err(Error::Io)?;
        let deadline = Instant::now() + self.timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(Error::Timeout);
            }
            let Some(report) = self.transport.read(remaining).map_err(Error::Io)? else {
                continue;
            };
            match match_reply(request, &report) {
                Some(Reply::Ok(data)) => return Ok(data),
                Some(Reply::FeatureError(code)) => return Err(Error::Feature(code)),
                Some(Reply::Hidpp10Error(code)) => return Err(Error::Hidpp10(code)),
                None => {} // notification or someone else's reply
            }
        }
    }

    /// Pings a slot, retrying once if no answer arrives in time.
    pub fn ping(&mut self, device: u8) -> Result<Presence, Error> {
        match self.ping_once(device)? {
            Presence::Unreachable => self.ping_once(device),
            presence => Ok(presence),
        }
    }

    fn ping_once(&mut self, device: u8) -> Result<Presence, Error> {
        const MARK: u8 = 0x5A;
        match self.call(&Request::new(device, 0x00, 0x1, &[0, 0, MARK])) {
            Ok(data) if data.len() >= 3 && data[2] == MARK => Ok(Presence::Online {
                major: data[0],
                minor: data[1],
            }),
            Ok(_) => Err(Error::Io("unexpected ping reply".into())),
            Err(Error::Hidpp10(hidpp10_error::INVALID_SUB_ID)) => Ok(Presence::Legacy),
            Err(Error::Hidpp10(hidpp10_error::UNKNOWN_DEVICE)) => Ok(Presence::Empty),
            Err(Error::Hidpp10(
                hidpp10_error::RESOURCE_ERROR | hidpp10_error::CONNECTION_REQUEST_FAILED,
            ))
            | Err(Error::Timeout) => Ok(Presence::Unreachable),
            Err(other) => Err(other),
        }
    }

    /// Index of `feature_id` on `device`, or `None` if unsupported.
    pub fn feature_index(&mut self, device: u8, feature_id: u16) -> Result<Option<u8>, Error> {
        let [hi, lo] = feature_id.to_be_bytes();
        let data = self.call(&Request::new(device, 0x00, 0x0, &[hi, lo]))?;
        Ok(data.first().copied().filter(|&i| i != 0))
    }

    pub fn call_feature(
        &mut self,
        device: u8,
        index: u8,
        function: u8,
        params: &[u8],
    ) -> Result<Vec<u8>, Error> {
        self.call(&Request::new(device, index, function, params))
    }

    /// Wireless product ID of the device paired in `slot` (1-based), read from the
    /// receiver's pairing register (Unifying, LIGHTSPEED and Nano receivers).
    pub fn wireless_product_id(&mut self, slot: u8) -> Result<Option<u16>, Error> {
        let sub_register = PAIRING_INFO_SUB_REGISTER + slot - 1;
        let data = self.call(&Request::read_receiver_register(
            RECEIVER_INFO_REGISTER,
            sub_register,
        ))?;
        // data[0] echoes the sub-register; the wireless PID is big-endian at 3..5.
        Ok(data
            .get(3..5)
            .map(|b| u16::from_be_bytes([b[0], b[1]]))
            .filter(|&p| p != 0))
    }

    /// Marketing name and type code from the DEVICE_NAME feature.
    pub fn name_and_type(&mut self, device: u8) -> Result<Option<(String, Option<u8>)>, Error> {
        let Some(index) = self.feature_index(device, feature::DEVICE_NAME)? else {
            return Ok(None);
        };
        let length = *self
            .call_feature(device, index, 0, &[])?
            .first()
            .unwrap_or(&0) as usize;
        let mut name = Vec::with_capacity(length);
        while name.len() < length {
            let chunk = self.call_feature(device, index, 1, &[name.len() as u8])?;
            if chunk.is_empty() {
                break;
            }
            let take = (length - name.len()).min(chunk.len());
            name.extend_from_slice(&chunk[..take]);
        }
        let name = String::from_utf8_lossy(&name)
            .trim_end_matches('\0')
            .trim()
            .to_owned();
        let kind = self
            .call_feature(device, index, 2, &[])
            .ok()
            .and_then(|d| d.first().copied());
        Ok((!name.is_empty()).then_some((name, kind)))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::VecDeque;

    use super::*;
    use crate::protocol::{LONG_LEN, SOFTWARE_ID};

    /// Simulated HID++ endpoint: replies are computed from each written request.
    type Responder = Box<dyn FnMut(&[u8]) -> Vec<Vec<u8>>>;

    pub struct FakeTransport {
        pub respond: Responder,
        pub queue: VecDeque<Vec<u8>>,
        pub written: Vec<Vec<u8>>,
    }

    impl FakeTransport {
        pub fn new(respond: impl FnMut(&[u8]) -> Vec<Vec<u8>> + 'static) -> Self {
            Self {
                respond: Box::new(respond),
                queue: VecDeque::new(),
                written: vec![],
            }
        }
    }

    impl Transport for FakeTransport {
        fn write(&mut self, report: &[u8]) -> Result<(), String> {
            self.written.push(report.to_vec());
            let replies = (self.respond)(report);
            self.queue.extend(replies);
            Ok(())
        }
        fn read(&mut self, _timeout: Duration) -> Result<Option<Vec<u8>>, String> {
            Ok(self.queue.pop_front())
        }
    }

    pub fn long(device: u8, index: u8, address: u8, payload: &[u8]) -> Vec<u8> {
        let mut r = vec![0x11, device, index, address];
        r.extend_from_slice(payload);
        r.resize(LONG_LEN, 0);
        r
    }

    fn session(t: FakeTransport) -> Session<FakeTransport> {
        Session::new(t, Duration::from_millis(50))
    }

    #[test]
    fn ping_reports_protocol_version() {
        let t = FakeTransport::new(|req| vec![long(req[1], 0, req[3], &[4, 2, req[6]])]);
        assert_eq!(
            session(t).ping(1).unwrap(),
            Presence::Online { major: 4, minor: 2 }
        );
    }

    #[test]
    fn ping_on_empty_slot_is_absent() {
        let t = FakeTransport::new(|req| vec![vec![0x10, req[1], 0x8F, req[2], req[3], 0x08, 0]]);
        assert_eq!(session(t).ping(3).unwrap(), Presence::Empty);
    }

    #[test]
    fn sleeping_device_is_unreachable_not_empty() {
        let t = FakeTransport::new(|req| vec![vec![0x10, req[1], 0x8F, req[2], req[3], 0x09, 0]]);
        assert_eq!(session(t).ping(1).unwrap(), Presence::Unreachable);
    }

    #[test]
    fn ping_is_retried_once_when_the_reply_is_lost() {
        let mut first = true;
        let t = FakeTransport::new(move |req| {
            if std::mem::take(&mut first) {
                return vec![]; // reply lost
            }
            vec![long(req[1], 0, req[3], &[4, 2, req[6]])]
        });
        assert!(matches!(
            session(t).ping(1).unwrap(),
            Presence::Online { .. }
        ));
    }

    #[test]
    fn silent_slot_times_out_as_unreachable() {
        let t = FakeTransport::new(|_| vec![]);
        assert_eq!(session(t).ping(5).unwrap(), Presence::Unreachable);
    }

    #[test]
    fn notifications_and_foreign_replies_are_skipped() {
        let t = FakeTransport::new(|req| {
            vec![
                long(req[1], 0x04, 0x00, &[1, 2, 3]),   // notification
                long(req[1], req[2], 0x1D, &[9, 9, 9]), // G HUB reply (software ID 0x0D)
                long(req[1], req[2], req[3], &[4, 2, req[6]]),
            ]
        });
        assert!(matches!(
            session(t).ping(1).unwrap(),
            Presence::Online { .. }
        ));
    }

    #[test]
    fn reads_name_in_chunks_and_type() {
        const NAME: &[u8] = b"G502 X LIGHTSPEED";
        let t = FakeTransport::new(|req| {
            let (dev, index, address) = (req[1], req[2], req[3]);
            assert_eq!(address & 0x0F, SOFTWARE_ID);
            let function = address >> 4;
            let payload: Vec<u8> = match (index, function) {
                (0, 0) => vec![if req[4..6] == [0x00, 0x05] { 0x03 } else { 0 }],
                (3, 0) => vec![NAME.len() as u8],
                (3, 1) => {
                    let start = req[4] as usize;
                    NAME[start..(start + 16).min(NAME.len())].to_vec()
                }
                (3, 2) => vec![0x03],
                _ => vec![],
            };
            vec![long(dev, index, address, &payload)]
        });
        let (name, kind) = session(t).name_and_type(1).unwrap().unwrap();
        assert_eq!(name, "G502 X LIGHTSPEED");
        assert_eq!(kind, Some(0x03));
    }
}
