//! Glorious Model O / O- / D / D- Wireless (cable and dongle): battery and polling rate.
//!
//! Source (only facts are used, the code is written from scratch): the project "mxw" by Ola Naess
//! Kaldestad, licence MIT. Not tested on real hardware by us: the support level is "expected".
//!
//! The mouse is configured with 65 byte FEATURE reports (report id 0) on interface 2.
//! * battery: send `00 00 00 02 02 00 83` (feature report), wait 50 ms, read the feature report:
//!   byte 8 = percent (0 is shown as 1), byte 1 = state (A1 awake, A4 asleep, A0 waking up) and
//!   byte 6 must be 83, otherwise the answer is not a battery answer. On the cable the mouse
//!   charges; the answer has no separate charging flag;
//! * polling rate: `00 00 00 02 01 01 00 <ms>` with ms = 1, 2, 4, 8 (1000, 500, 250, 125 Hz).
//! DPI is NOT written: it belongs to numbered profiles and the active profile cannot be read, so a
//! write could change another profile than the one in use. Nothing else is ever sent.

use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

use hidapi::{HidApi, HidDevice};

use super::readonly::{self, Reading};
use super::MouseDriver;
use crate::mouse::MouseInfoDto;

const VID: u16 = 0x258A;
const INTERFACE: i32 = 2;
const POLLING_HZ: [u32; 4] = [125, 250, 500, 1000];

struct Model {
    name: &'static str,
    pid_wired: u16,
    pid_wireless: u16,
}

const MODELS: [Model; 4] = [
    Model { name: "Glorious Model O Wireless", pid_wired: 0x2011, pid_wireless: 0x2022 },
    Model { name: "Glorious Model O- Wireless", pid_wired: 0x2013, pid_wireless: 0x2024 },
    Model { name: "Glorious Model D Wireless", pid_wired: 0x2012, pid_wireless: 0x2023 },
    Model { name: "Glorious Model D- Wireless", pid_wired: 0x2014, pid_wireless: 0x2025 },
];

static LAST_POLLING: Mutex<Option<u32>> = Mutex::new(None);

/// (model, wired)
fn model_for(pid: u16) -> Option<(&'static Model, bool)> {
    MODELS.iter().find_map(|m| {
        if m.pid_wired == pid {
            Some((m, true))
        } else if m.pid_wireless == pid {
            Some((m, false))
        } else {
            None
        }
    })
}

pub fn battery_request() -> [u8; 65] {
    let mut data = [0u8; 65];
    data[3] = 0x02;
    data[4] = 0x02;
    data[6] = 0x83;
    data
}

pub fn polling_request(hz: u32) -> Option<[u8; 65]> {
    let ms = match hz {
        1000 => 1,
        500 => 2,
        250 => 4,
        125 => 8,
        _ => return None,
    };
    let mut data = [0u8; 65];
    data[3] = 0x02;
    data[4] = 0x01;
    data[5] = 0x01;
    data[7] = ms;
    Some(data)
}

#[derive(Debug, PartialEq, Eq)]
pub enum Battery {
    Level(u32),
    Asleep,
    WakingUp,
    Unknown,
}

pub fn parse_battery(answer: &[u8]) -> Battery {
    if answer.len() < 9 || answer[6] != 0x83 {
        return Battery::Unknown;
    }
    match answer[1] {
        0xA1 => Battery::Level(u32::from(answer[8].clamp(1, 100))),
        0xA4 => Battery::Asleep,
        0xA0 => Battery::WakingUp,
        _ => Battery::Unknown,
    }
}

fn read_battery(device: &HidDevice) -> Battery {
    if device.send_feature_report(&battery_request()).is_err() {
        return Battery::Unknown;
    }
    sleep(Duration::from_millis(50));
    let mut buffer = [0u8; 65];
    match device.get_feature_report(&mut buffer) {
        Ok(read) if read > 0 => parse_battery(&buffer),
        _ => Battery::Unknown,
    }
}

fn describe(model: &'static Model, wired: bool, battery: Option<u32>) -> MouseInfoDto {
    let mut info = readonly::build(Reading {
        brand: "Glorious",
        name: model.name.to_string(),
        connection: if wired { "wired" } else { "wireless" },
        battery,
        // On the cable the mouse charges (unless full)
        charging: wired && battery.map_or(false, |percent| percent < 100),
        dpi: None,
        polling: *LAST_POLLING.lock().unwrap_or_else(|error| error.into_inner()),
    });
    info.polling_options = POLLING_HZ.to_vec();
    info.polling_supported = POLLING_HZ.to_vec();
    info.capabilities.polling_write = true;
    info
}

pub struct GloriousDriver;

impl MouseDriver for GloriousDriver {
    fn id(&self) -> &'static str {
        "glorious-model-o"
    }

    fn brand(&self) -> &'static str {
        "Glorious"
    }

    fn present(&self, api: &HidApi) -> bool {
        api.device_list().any(|d| d.vendor_id() == VID && model_for(d.product_id()).is_some())
    }

    fn probe(&self, api: &HidApi, log: &mut Vec<String>) -> Option<MouseInfoDto> {
        let mut first: Option<(&'static Model, bool)> = None;

        // Prefer the cable when both are present
        let mut candidates: Vec<_> = api
            .device_list()
            .filter(|d| d.vendor_id() == VID && model_for(d.product_id()).is_some())
            .collect();
        candidates.sort_by_key(|d| !model_for(d.product_id()).map_or(false, |(_, wired)| wired));

        for info in candidates {
            let (model, wired) = model_for(info.product_id())?;
            first.get_or_insert((model, wired));
            if info.interface_number() != INTERFACE {
                continue;
            }
            log.push(format!("Glorious {:04X}:{:04X}: asking the battery", VID, info.product_id()));
            let device = match info.open_device(api) {
                Ok(device) => device,
                Err(error) => {
                    log.push(format!("  cannot open: {}", error));
                    continue;
                }
            };
            let battery = read_battery(&device);
            log.push(format!("  battery answer: {:?}", battery));
            let percent = if let Battery::Level(percent) = battery { Some(percent) } else { None };
            return Some(describe(model, wired, percent));
        }

        first.map(|(model, wired)| describe(model, wired, None))
    }

    fn set_polling(&self, hz: u32) -> Result<u32, String> {
        let request = polling_request(hz)
            .ok_or_else(|| format!("{} Hz is not a valid polling rate for this mouse (125, 250, 500, 1000)", hz))?;
        let api = HidApi::new().map_err(|error| error.to_string())?;
        for info in api.device_list() {
            if info.vendor_id() != VID || model_for(info.product_id()).is_none() || info.interface_number() != INTERFACE {
                continue;
            }
            let device = match info.open_device(&api) {
                Ok(device) => device,
                Err(_) => continue,
            };
            device.send_feature_report(&request).map_err(|error| error.to_string())?;
            *LAST_POLLING.lock().unwrap_or_else(|error| error.into_inner()) = Some(hz);
            return Ok(hz);
        }
        Err("Glorious mouse not found".to_string())
    }

    fn set_dpi(&self, _value: u32) -> Result<u32, String> {
        Err("DPI cannot be changed on this mouse yet: it depends on the active profile, which is not readable".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests() {
        let b = battery_request();
        assert_eq!(&b[..8], &[0, 0, 0, 2, 2, 0, 0x83, 0]);
        let p = polling_request(1000).unwrap();
        assert_eq!(&p[..8], &[0, 0, 0, 2, 1, 1, 0, 1]);
        assert_eq!(polling_request(125).unwrap()[7], 8);
        assert_eq!(polling_request(250).unwrap()[7], 4);
        assert_eq!(polling_request(500).unwrap()[7], 2);
        assert!(polling_request(2000).is_none());
    }

    #[test]
    fn battery_answers() {
        let mut a = [0u8; 65];
        a[1] = 0xA1;
        a[6] = 0x83;
        a[8] = 72;
        assert_eq!(parse_battery(&a), Battery::Level(72));
        a[8] = 0;
        assert_eq!(parse_battery(&a), Battery::Level(1));
        a[1] = 0xA4;
        assert_eq!(parse_battery(&a), Battery::Asleep);
        a[1] = 0xA0;
        assert_eq!(parse_battery(&a), Battery::WakingUp);
        a[1] = 0xA3;
        assert_eq!(parse_battery(&a), Battery::Unknown);
        a[1] = 0xA1;
        a[6] = 0x00;
        assert_eq!(parse_battery(&a), Battery::Unknown);
        assert_eq!(parse_battery(&[]), Battery::Unknown);
    }

    #[test]
    fn ids() {
        assert_eq!(model_for(0x2011).map(|(_, w)| w), Some(true));
        assert_eq!(model_for(0x2022).map(|(_, w)| w), Some(false));
        assert!(model_for(0x2000).is_none());
    }
}
