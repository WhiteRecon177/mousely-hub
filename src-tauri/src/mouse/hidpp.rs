use hidapi::HidDevice;
use std::fmt;
use std::time::{Duration, Instant};

const REPORT_LONG: u8 = 0x11;
const REPORT_LEN: usize = 20;
const SW_ID: u8 = 0x0B;

#[derive(Debug)]
pub enum HidppError {
    Timeout,
    Protocol(u8),
    Io(String),
}

impl fmt::Display for HidppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HidppError::Timeout => write!(f, "no answer (timeout)"),
            HidppError::Protocol(code) => write!(f, "device error 0x{:02X}", code),
            HidppError::Io(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

pub struct Hidpp {
    dev: HidDevice,
}

impl Hidpp {
    pub fn new(dev: HidDevice) -> Self {
        Hidpp { dev }
    }

    /// Throws away answers that arrived late for an earlier request.
    fn drain(&self) {
        let mut buf = [0u8; 64];
        for _ in 0..64 {
            match self.dev.read_timeout(&mut buf, 8) {
                Ok(n) if n > 0 => continue,
                _ => break,
            }
        }
    }

    /// Sends one HID++ 2.0 request and waits for the matching answer.
    /// Returns the 16 payload bytes that follow the 4 header bytes.
    pub fn request(
        &self,
        device: u8,
        feature: u8,
        function: u8,
        params: &[u8],
        timeout_ms: u64,
    ) -> Result<Vec<u8>, HidppError> {
        let mut out = [0u8; REPORT_LEN];
        out[0] = REPORT_LONG;
        out[1] = device;
        out[2] = feature;
        out[3] = (function << 4) | SW_ID;
        for (i, b) in params.iter().take(16).enumerate() {
            out[4 + i] = *b;
        }

        self.drain();

        self.dev
            .write(&out)
            .map_err(|e| HidppError::Io(e.to_string()))?;

        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        let mut buf = [0u8; 64];

        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return Err(HidppError::Timeout);
            }
            let wait = left.as_millis().min(1000) as i32;
            let n = self
                .dev
                .read_timeout(&mut buf, wait)
                .map_err(|e| HidppError::Io(e.to_string()))?;
            if n < 6 {
                continue;
            }

            // HID++ 2.0 answer
            if buf[0] == REPORT_LONG && buf[1] == device && buf[2] == feature && buf[3] == out[3] {
                let end = n.min(REPORT_LEN);
                return Ok(buf[4..end].to_vec());
            }
            // HID++ 2.0 error
            if buf[0] == REPORT_LONG && buf[1] == device && buf[2] == 0xFF && buf[3] == feature && buf[4] == out[3] {
                return Err(HidppError::Protocol(buf[5]));
            }
            // HID++ 1.0 error (old devices and receivers)
            if buf[0] == 0x10
                && buf[1] == device
                && buf[2] == 0x8F
                && buf[3] == feature
                && buf[4] == out[3]
            {
                return Err(HidppError::Protocol(buf[5]));
            }
            // anything else is a notification for someone else: ignore it
        }
    }

    /// Asks the root feature where a feature lives. None if the device doesn't have it.
    pub fn feature_index(&self, device: u8, feature_id: u16) -> Result<Option<u8>, HidppError> {
        let id = feature_id.to_be_bytes();
        let payload = self.request(device, 0x00, 0, &[id[0], id[1]], 300)?;
        match payload.first() {
            Some(0) | None => Ok(None),
            Some(index) => Ok(Some(*index)),
        }
    }
}