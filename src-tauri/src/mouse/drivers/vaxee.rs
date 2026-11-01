//! VAXEE mice (dongle or cable). READ ONLY: battery, charging state and polling rate.
//!
//! Source: the public notes in docs/PROTOCOLLI.md (MIT projects `stuffz/mouse-battery-monitor` and
//! `ryanlewis/mousebatt`). Facts only, no code copied. The notes say it was checked on one model
//! (XE-S-L with its dongle); the others are "expected". The DPI cannot be read (not documented).
//!
//! Every request is a feature report of 64 bytes: `0E A5 <command> 01 01 00 ...` (01 = read).
//! The answer has the same shape: byte 2 repeats the command, the value is at byte 5.

use std::thread;
use std::time::Duration;

use hidapi::{HidApi, HidDevice};

use super::readonly::{self, Reading};
use super::MouseDriver;
use crate::mouse::MouseInfoDto;

const VAXEE_VID: u16 = 0x3057;
const USAGE_PAGE: u16 = 0xFF05;
const USAGE: u16 = 0x0001;
const REPORT_ID: u8 = 0x0E;

const CMD_POLLING: u8 = 0x07;
const CMD_BATTERY: u8 = 0x0B;
const CMD_CHARGING: u8 = 0x10;

/// Receivers (shared by several models). Everything else with this vendor id is a mouse on cable.
const DONGLE_PIDS: [u16; 4] = [0x1001, 0x1002, 0x0005, 0x2001];

pub fn request(command: u8) -> [u8; 64] {
    let mut out = [0u8; 64];
    out[0] = REPORT_ID;
    out[1] = 0xA5;
    out[2] = command;
    out[3] = 0x01; // 01 = read (02 would write: never used here)
    out[4] = 0x01;
    out
}

pub fn polling_from_index(index: u8) -> Option<u32> {
    match index {
        1 => Some(500),
        2 => Some(1000),
        3 => Some(2000),
        4 => Some(4000),
        5 => Some(8000),
        _ => None,
    }
}

pub fn battery_from_raw(raw: u8) -> Option<u32> {
    // 0..=20 in steps of 5 percent
    if raw <= 20 {
        Some(u32::from(raw) * 5)
    } else {
        None
    }
}

/// Asks one question. The mouse needs about 100 ms; a few tries are made.
fn ask(device: &HidDevice, command: u8) -> Option<u8> {
    let out = request(command);

    for _ in 0..5 {
        if device.send_feature_report(&out).is_err() {
            return None;
        }
        thread::sleep(Duration::from_millis(100));

        let mut answer = [0u8; 64];
        answer[0] = REPORT_ID;
        match device.get_feature_report(&mut answer) {
            Ok(read) if read >= 6 && answer[2] == command => return Some(answer[5]),
            _ => {}
        }
    }
    None
}

pub struct VaxeeDriver;

impl MouseDriver for VaxeeDriver {
    fn id(&self) -> &'static str {
        "vaxee"
    }

    fn brand(&self) -> &'static str {
        "VAXEE"
    }

    fn present(&self, api: &HidApi) -> bool {
        api.device_list().any(|device| device.vendor_id() == VAXEE_VID)
    }

    fn probe(&self, api: &HidApi, log: &mut Vec<String>) -> Option<MouseInfoDto> {
        for info in api.device_list() {
            if info.vendor_id() != VAXEE_VID
                || info.usage_page() != USAGE_PAGE
                || info.usage() != USAGE
            {
                continue;
            }

            log.push(format!(
                "VAXEE {:04X}:{:04X} usage_page={:04X}: asking (read only)",
                info.vendor_id(),
                info.product_id(),
                info.usage_page()
            ));

            let device = match info.open_device(api) {
                Ok(device) => device,
                Err(error) => {
                    log.push(format!("  cannot open: {}", error));
                    continue;
                }
            };

            let battery = ask(&device, CMD_BATTERY).and_then(battery_from_raw);
            let charging = ask(&device, CMD_CHARGING).map(|value| value != 0).unwrap_or(false);
            let polling = ask(&device, CMD_POLLING).and_then(polling_from_index);

            log.push(format!(
                "  battery={:?} charging={} polling={:?}",
                battery, charging, polling
            ));

            if battery.is_none() && polling.is_none() {
                // The mouse did not answer (asleep, or not the right interface).
                continue;
            }

            let wireless = DONGLE_PIDS.contains(&info.product_id());
            let name = info
                .product_string()
                .filter(|text| !text.trim().is_empty())
                .map(|text| text.to_string())
                .unwrap_or_else(|| "VAXEE mouse".to_string());

            return Some(readonly::build(Reading {
                brand: "VAXEE",
                name,
                connection: if wireless { "wireless" } else { "wired" },
                battery,
                charging,
                dpi: None,
                polling,
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
    fn requests_only_read() {
        let out = request(CMD_BATTERY);
        assert_eq!(&out[..5], &[0x0E, 0xA5, 0x0B, 0x01, 0x01]);
        assert!(out[5..].iter().all(|byte| *byte == 0));
        // byte 3 is the read/write flag: it must be 1 (read) for every command we use
        for command in [CMD_POLLING, CMD_BATTERY, CMD_CHARGING] {
            assert_eq!(request(command)[3], 0x01);
        }
    }

    #[test]
    fn conversions() {
        assert_eq!(battery_from_raw(20), Some(100));
        assert_eq!(battery_from_raw(10), Some(50));
        assert_eq!(battery_from_raw(21), None);
        assert_eq!(polling_from_index(2), Some(1000));
        assert_eq!(polling_from_index(5), Some(8000));
        assert_eq!(polling_from_index(0), None);
    }
}
