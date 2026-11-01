//! Lamzu Maya X 8K (dongle or cable). READ ONLY: battery and charging state only.
//!
//! Source: the public notes in docs/PROTOCOLLI.md (MIT projects `Sheroune/lamzu-battery-monitory`,
//! HaloBattery). Facts only, no code copied. The notes say the battery frame was used on Windows
//! by other projects. DPI and polling are not documented anywhere, so they are not shown.
//!
//! The request is a feature report of 65 bytes (report id 0): `00 00 00 02 02 00 83 00 ..`.
//! The answer must have `A1` at byte 1 and `83` at byte 6; byte 7 is the charging flag and
//! byte 8 the percentage. Only the interface 2 with usage page 0xFFFF is used.

use std::thread;
use std::time::Duration;

use hidapi::{HidApi, HidDevice};

use super::readonly::{self, Reading};
use super::MouseDriver;
use crate::mouse::MouseInfoDto;

const LAMZU_VID: u16 = 0x373E;
const PID_DONGLE: u16 = 0x001E;
const PID_CABLE: u16 = 0x001C;
const INTERFACE: i32 = 2;
const USAGE_PAGE: u16 = 0xFFFF;

pub fn request() -> [u8; 65] {
    let mut out = [0u8; 65];
    out[3] = 0x02;
    out[4] = 0x02;
    out[6] = 0x83;
    out
}

/// Returns (percentage, charging) when the answer has the expected markers.
pub fn parse_answer(answer: &[u8]) -> Option<(u32, bool)> {
    if answer.len() < 9 || answer[1] != 0xA1 || answer[6] != 0x83 {
        return None;
    }
    let percent = u32::from(answer[8]);
    if percent > 100 {
        return None;
    }
    Some((percent, answer[7] != 0))
}

fn ask(device: &HidDevice) -> Option<(u32, bool)> {
    device.send_feature_report(&request()).ok()?;
    thread::sleep(Duration::from_millis(100));

    // The notes read the answer up to 10 times, every 50 ms.
    for _ in 0..10 {
        let mut answer = [0u8; 65];
        if let Ok(read) = device.get_feature_report(&mut answer) {
            if let Some(found) = parse_answer(&answer[..read.min(65)]) {
                return Some(found);
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    None
}

pub struct LamzuMayaDriver;

impl MouseDriver for LamzuMayaDriver {
    fn id(&self) -> &'static str {
        "lamzu-maya"
    }

    fn brand(&self) -> &'static str {
        "Lamzu"
    }

    fn present(&self, api: &HidApi) -> bool {
        api.device_list().any(|device| {
            device.vendor_id() == LAMZU_VID
                && (device.product_id() == PID_DONGLE || device.product_id() == PID_CABLE)
        })
    }

    fn probe(&self, api: &HidApi, log: &mut Vec<String>) -> Option<MouseInfoDto> {
        for info in api.device_list() {
            if info.vendor_id() != LAMZU_VID
                || (info.product_id() != PID_DONGLE && info.product_id() != PID_CABLE)
                || info.interface_number() != INTERFACE
                || info.usage_page() != USAGE_PAGE
            {
                continue;
            }

            log.push(format!(
                "Lamzu {:04X}:{:04X} interface {}: asking the battery (read only)",
                info.vendor_id(),
                info.product_id(),
                info.interface_number()
            ));

            let device = match info.open_device(api) {
                Ok(device) => device,
                Err(error) => {
                    log.push(format!("  cannot open: {}", error));
                    continue;
                }
            };

            let Some((percent, charging)) = ask(&device) else {
                log.push("  no valid answer (the mouse may be asleep)".to_string());
                continue;
            };

            return Some(readonly::build(Reading {
                brand: "Lamzu",
                name: "Lamzu Maya X 8K".to_string(),
                connection: if info.product_id() == PID_CABLE { "wired" } else { "wireless" },
                battery: Some(percent),
                charging,
                dpi: None,
                polling: None,
            }));
        }
        None
    }

    fn set_polling(&self, _hz: u32) -> Result<u32, String> {
        Err(readonly::READ_ONLY.to_string())
    }

    fn set_dpi(&self, _value: u32) -> Result<u32, String> {
        Err(readonly::READ_ONLY.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_matches_the_notes() {
        let out = request();
        assert_eq!(&out[..7], &[0x00, 0x00, 0x00, 0x02, 0x02, 0x00, 0x83]);
        assert!(out[7..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn answer_parsing() {
        let mut answer = [0u8; 65];
        answer[1] = 0xA1;
        answer[6] = 0x83;
        answer[7] = 1;
        answer[8] = 87;
        assert_eq!(parse_answer(&answer), Some((87, true)));

        answer[1] = 0x00;
        assert_eq!(parse_answer(&answer), None);

        answer[1] = 0xA1;
        answer[8] = 200;
        assert_eq!(parse_answer(&answer), None);

        assert_eq!(parse_answer(&[0u8; 4]), None);
    }
}
