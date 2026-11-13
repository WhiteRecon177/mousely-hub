//! SteelSeries Aerox 3 (cable) and Aerox 3 / 5 / 9 Wireless: battery (wireless), DPI and polling rate.
//!
//! Sources (only facts are used, the code is written from scratch):
//! * battery question: the MIT-licensed project "OpenSeries" (crate `openseries-gg` 1.0.0);
//! * DPI and polling commands, DPI code tables (TrueMove Air and TrueMove Core), Aerox ids:
//!   the project "rivalcfg" by Fabien Loison (flozz), licence WTFPL, version 4.17.0.
//! Tested on real hardware: SteelSeries Aerox 3 wired (`1038:1836`); wireless models are "expected".
//!
//! Every message is one output report: report id `00` followed by the command. Through the
//! 2.4 GHz receiver the command byte gets bit 0x40 set (`92` -> `D2`, `2D` -> `6D`, `2B` -> `6B`)
//! and the receiver answers every write with one 64 byte report, which is read and dropped.
//!
//! * battery (wireless models only): `92`; answer: bit 7 of the 2nd byte = charging, bits 0..6 = step 1..21
//!   (percent = (step - 1) * 5). A step outside 1..21 means "mouse not connected".
//! * DPI: `2D, count, selected, code...` (`2D 01 00 code`, 0-based active preset index, with
//!   TrueMove Air codes on wireless models and TrueMove Core codes on wired Aerox 3).
//! * polling: `2B, code` with `[3, 2, 1, 0]` for `[125, 250, 500, 1000]` Hz on wireless models,
//!   and `[4, 3, 2, 1]` on wired Aerox 3.
//!
//! Never sent: the "save to the mouse memory" command, lighting, buttons, sleep timer, pairing,
//! firmware or reset commands. The settings are therefore lost when the mouse is switched off,
//! which also makes a wrong byte harmless.

use hidapi::{HidApi, HidDevice};

use std::sync::Mutex;

use super::readonly::{self, Reading};
use super::MouseDriver;
use crate::mouse::MouseInfoDto;

const STEELSERIES_VID: u16 = 0x1038;
const CMD_BATTERY: u8 = 0x92;
const RECEIVER_FLAG: u8 = 0x40;

struct SensorProfile {
    dpi_min: u32,
    dpi_max: u32,
    dpi_codes: &'static [u8],
    first_preset: u8,
    /// Polling codes for [125, 250, 500, 1000] Hz.
    polling_codes: [u8; 4],
}

struct Model {
    pid: u16,
    name: &'static str,
    /// 2.4 GHz receiver (the question must use the receiver form of the command)
    receiver: bool,
    has_battery: bool,
    profile: &'static SensorProfile,
}

const CMD_DPI: u8 = 0x2D;
const CMD_POLLING: u8 = 0x2B;
const DPI_STEP: u32 = 100;
/// Shown before the user picks a DPI (the mouse cannot tell its real value).
const START_DPI: u32 = 800;
const POLLING_HZ: [u32; 4] = [125, 250, 500, 1000];

/// Sensor code of every DPI value from 100 to 18000 in steps of 100 (180 values, TrueMove Air).
const DPI_CODES: [u8; 180] = [
    0x00, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
    0x0E, 0x10, 0x11, 0x12, 0x13, 0x14, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B,
    0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x23, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A,
    0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x32, 0x33, 0x34, 0x35, 0x36, 0x38, 0x39,
    0x3A, 0x3B, 0x3C, 0x3E, 0x3F, 0x40, 0x41, 0x42, 0x44, 0x45, 0x46, 0x47,
    0x48, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x50, 0x51, 0x52, 0x53, 0x54, 0x56,
    0x57, 0x58, 0x59, 0x5A, 0x5C, 0x5D, 0x5E, 0x5F, 0x60, 0x62, 0x63, 0x64,
    0x65, 0x66, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6E, 0x6F, 0x70, 0x71, 0x72,
    0x74, 0x75, 0x76, 0x77, 0x78, 0x7A, 0x7B, 0x7C, 0x7D, 0x7E, 0x80, 0x81,
    0x82, 0x83, 0x84, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x8C, 0x8D, 0x8E, 0x8F,
    0x90, 0x92, 0x93, 0x94, 0x95, 0x96, 0x98, 0x99, 0x9A, 0x9B, 0x9C, 0x9E,
    0x9F, 0xA0, 0xA1, 0xA2, 0xA4, 0xA5, 0xA6, 0xA7, 0xA8, 0xAA, 0xAB, 0xAC,
    0xAD, 0xAE, 0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9,
    0xBA, 0xBB, 0xBC, 0xBD, 0xBF, 0xC0, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7,
    0xC9, 0xCA, 0xCB, 0xCC, 0xCD, 0xCF, 0xD0, 0xD1, 0xD2, 0xD3, 0xD5, 0xD6,
];

/// Sensor code of every DPI value from 200 to 8500 in steps of 100 (84 values, TrueMove Core).
const TRUEMOVE_CORE_DPI_CODES: [u8; 84] = [
    0x04, 0x06, 0x08, 0x0B, 0x0D, 0x0F, 0x12, 0x14, 0x16, 0x19, 0x1B, 0x1D,
    0x20, 0x22, 0x24, 0x27, 0x29, 0x2B, 0x2E, 0x30, 0x32, 0x34, 0x37, 0x39,
    0x3B, 0x3E, 0x40, 0x42, 0x45, 0x47, 0x49, 0x4C, 0x4E, 0x50, 0x53, 0x55,
    0x57, 0x5A, 0x5C, 0x5E, 0x61, 0x63, 0x65, 0x68, 0x6A, 0x6C, 0x6F, 0x71,
    0x73, 0x76, 0x78, 0x7A, 0x7D, 0x7F, 0x81, 0x84, 0x86, 0x88, 0x8B, 0x8D,
    0x8F, 0x92, 0x94, 0x96, 0x99, 0x9B, 0x9D, 0xA0, 0xA2, 0xA4, 0xA7, 0xA9,
    0xAB, 0xAD, 0xB0, 0xB2, 0xB4, 0xB7, 0xB9, 0xBC, 0xBE, 0xC0, 0xC3, 0xC5,
];

const WIRELESS_PROFILE: SensorProfile = SensorProfile {
    dpi_min: 100,
    dpi_max: 18000,
    dpi_codes: &DPI_CODES,
    first_preset: 0,
    polling_codes: [0x03, 0x02, 0x01, 0x00],
};

const AEROX3_WIRED_PROFILE: SensorProfile = SensorProfile {
    dpi_min: 200,
    dpi_max: 8500,
    dpi_codes: &TRUEMOVE_CORE_DPI_CODES,
    // Aerox 3 wired (0x1836) takes a 0-based preset index (00..=04, matching USB captures);
    // rivalcfg 4.17.0 aerox3.py had first_preset: 1, which the mouse ignores for 1 preset.
    first_preset: 0,
    polling_codes: [0x04, 0x03, 0x02, 0x01],
};

const MODELS: [Model; 15] = [
    Model { pid: 0x1836, name: "SteelSeries Aerox 3", receiver: false, has_battery: false, profile: &AEROX3_WIRED_PROFILE },
    Model { pid: 0x1838, name: "SteelSeries Aerox 3 Wireless", receiver: true, has_battery: true, profile: &WIRELESS_PROFILE },
    Model { pid: 0x183A, name: "SteelSeries Aerox 3 Wireless", receiver: false, has_battery: true, profile: &WIRELESS_PROFILE },
    Model { pid: 0x1878, name: "SteelSeries Aerox 3 Wireless CS2 Dragon Lore Edition", receiver: true, has_battery: true, profile: &WIRELESS_PROFILE },
    Model { pid: 0x187A, name: "SteelSeries Aerox 3 Wireless CS2 Dragon Lore Edition", receiver: false, has_battery: true, profile: &WIRELESS_PROFILE },
    Model { pid: 0x1852, name: "SteelSeries Aerox 5 Wireless", receiver: true, has_battery: true, profile: &WIRELESS_PROFILE },
    Model { pid: 0x1854, name: "SteelSeries Aerox 5 Wireless", receiver: false, has_battery: true, profile: &WIRELESS_PROFILE },
    Model { pid: 0x185C, name: "SteelSeries Aerox 5 Wireless Destiny 2 Edition", receiver: true, has_battery: true, profile: &WIRELESS_PROFILE },
    Model { pid: 0x185E, name: "SteelSeries Aerox 5 Wireless Destiny 2 Edition", receiver: false, has_battery: true, profile: &WIRELESS_PROFILE },
    Model { pid: 0x1860, name: "SteelSeries Aerox 5 Wireless Diablo IV Edition", receiver: true, has_battery: true, profile: &WIRELESS_PROFILE },
    Model { pid: 0x1862, name: "SteelSeries Aerox 5 Wireless Diablo IV Edition", receiver: false, has_battery: true, profile: &WIRELESS_PROFILE },
    Model { pid: 0x1858, name: "SteelSeries Aerox 9 Wireless", receiver: true, has_battery: true, profile: &WIRELESS_PROFILE },
    Model { pid: 0x185A, name: "SteelSeries Aerox 9 Wireless", receiver: false, has_battery: true, profile: &WIRELESS_PROFILE },
    Model { pid: 0x1874, name: "SteelSeries Aerox 9 Wireless WOW Edition", receiver: true, has_battery: true, profile: &WIRELESS_PROFILE },
    Model { pid: 0x1876, name: "SteelSeries Aerox 9 Wireless WOW Edition", receiver: false, has_battery: true, profile: &WIRELESS_PROFILE },
];

/// What the app has written in this run; the mouse cannot be asked for its DPI.
static LAST_DPI: Mutex<Option<u32>> = Mutex::new(None);
static LAST_POLLING: Mutex<Option<u32>> = Mutex::new(None);

fn remember(slot: &Mutex<Option<u32>>, value: u32) {
    *slot.lock().unwrap_or_else(|error| error.into_inner()) = Some(value);
}

fn recall(slot: &Mutex<Option<u32>>) -> Option<u32> {
    *slot.lock().unwrap_or_else(|error| error.into_inner())
}

/// Nearest valid DPI (multiple of 100 within the sensor profile) and its sensor code.
fn dpi_code_for(profile: &SensorProfile, value: u32) -> (u32, u8) {
    let clamped = value.clamp(profile.dpi_min, profile.dpi_max);
    let max_idx = profile.dpi_codes.len() - 1;
    let index = (((clamped + DPI_STEP / 2 - profile.dpi_min) / DPI_STEP) as usize).min(max_idx);
    (profile.dpi_min + index as u32 * DPI_STEP, profile.dpi_codes[index])
}

fn with_flag(command: u8, receiver: bool) -> u8 {
    if receiver {
        command | RECEIVER_FLAG
    } else {
        command
    }
}

/// The message that sets ONE DPI preset and selects it (without the report id).
fn dpi_message_for(profile: &SensorProfile, receiver: bool, value: u32) -> (u32, Vec<u8>) {
    let (dpi, code) = dpi_code_for(profile, value);
    (dpi, vec![with_flag(CMD_DPI, receiver), 0x01, profile.first_preset, code])
}

/// The message that sets the polling rate (without the report id).
fn polling_message_for(profile: &SensorProfile, receiver: bool, hz: u32) -> Option<Vec<u8>> {
    let index = POLLING_HZ.iter().position(|&known| known == hz)?;
    Some(vec![with_flag(CMD_POLLING, receiver), profile.polling_codes[index]])
}

fn model_for(pid: u16) -> Option<&'static Model> {
    MODELS.iter().find(|model| model.pid == pid)
}

/// The command byte to send: `92`, or `D2` through the receiver.
pub fn command(receiver: bool) -> u8 {
    with_flag(CMD_BATTERY, receiver)
}

/// What the battery answer says.
#[derive(Debug, PartialEq, Eq)]
pub enum Battery {
    /// Mouse not connected (step out of range)
    Missing,
    Level { percent: u32, charging: bool },
}

/// Reads the answer. The report may start with a `00` report id; the byte after the repeated
/// command is the battery. Returns `None` when the answer is not a battery answer.
pub fn parse_battery(answer: &[u8], receiver: bool) -> Option<Battery> {
    let echo = command(receiver);

    let start = if answer.len() >= 3 && answer[0] == 0 && answer[1] == echo {
        1
    } else if answer.len() >= 2 && answer[0] == echo {
        0
    } else {
        return None;
    };

    let raw = *answer.get(start + 1)?;
    let step = u32::from(raw & 0x7F);

    if !(1..=21).contains(&step) {
        return Some(Battery::Missing);
    }

    Some(Battery::Level {
        percent: (step - 1) * 5,
        charging: raw & 0x80 != 0,
    })
}

/// The mouse interface is the vendor one (usage page FFC0, usage 1), or interface 3.
fn is_control_interface(usage_page: u16, usage: u16, interface: i32, path: &str) -> bool {
    (usage_page == 0xFFC0 && usage == 1) || interface == 3 || path.to_lowercase().contains("&mi_03")
}

/// Sends the battery question and returns the matching answer. The size of the output report is
/// not known without parsing the HID descriptor, so the usual sizes are tried; every attempt
/// carries the same zero padded read command.
fn ask(device: &HidDevice, receiver: bool) -> Option<Battery> {
    let echo = command(receiver);

    for length in [65usize, 33, 64, 32] {
        let mut request = vec![0u8; length];
        request[1] = echo; // request[0] = report id 0

        if device.write(&request).is_err() {
            continue;
        }

        for _ in 0..4 {
            let mut buffer = [0u8; 64];
            let read = device.read_timeout(&mut buffer, 250).ok()?;
            if read == 0 {
                break;
            }
            if let Some(battery) = parse_battery(&buffer[..read], receiver) {
                return Some(battery);
            }
        }
    }

    None
}

/// Sends one setting message (zero padded to the usual report sizes until one is accepted) and
/// drops the answer the receiver sends back.
fn send(device: &HidDevice, message: &[u8], receiver: bool) -> Result<(), String> {
    let mut last_error = String::from("the mouse did not accept the message");

    for length in [65usize, 33, 64, 32] {
        let mut request = vec![0u8; length.max(message.len() + 1)];
        request[1..=message.len()].copy_from_slice(message); // request[0] = report id 0

        match device.write(&request) {
            Ok(_) => {
                if receiver {
                    let mut answer = [0u8; 64];
                    let _ = device.read_timeout(&mut answer, 200);
                }
                return Ok(());
            }
            Err(error) => last_error = error.to_string(),
        }
    }

    Err(last_error)
}

/// Opens the control interface of the first known SteelSeries mouse and sends one message.
fn write_setting(build: impl Fn(&Model) -> Result<(u32, Vec<u8>), String>) -> Result<u32, String> {
    let api = HidApi::new().map_err(|error| error.to_string())?;
    let mut failure = String::from("SteelSeries mouse not found");

    for info in api.device_list() {
        if info.vendor_id() != STEELSERIES_VID {
            continue;
        }
        let Some(model) = model_for(info.product_id()) else {
            continue;
        };
        let path = info.path().to_string_lossy().to_string();
        if !is_control_interface(info.usage_page(), info.usage(), info.interface_number(), &path) {
            continue;
        }

        let (applied, message) = build(model)?;

        let device = match info.open_device(&api) {
            Ok(device) => device,
            Err(error) => {
                failure = format!("cannot open the mouse: {}", error);
                continue;
            }
        };

        match send(&device, &message, model.receiver) {
            Ok(()) => return Ok(applied),
            Err(error) => failure = error,
        }
    }

    Err(failure)
}

/// Aerox 3 (cable) and Aerox 3 / 5 / 9 Wireless share the same 2D/2B command family.
fn has_writes(model: &Model) -> bool {
    let _ = model;
    true
}

fn describe(model: &'static Model, battery: Option<u32>, charging: bool) -> MouseInfoDto {
    let mut info = readonly::build(Reading {
        brand: "SteelSeries",
        name: model.name.to_string(),
        connection: if model.receiver { "wireless" } else { "wired" },
        battery: if model.has_battery { battery } else { None },
        charging: model.has_battery && charging,
        dpi: recall(&LAST_DPI),
        polling: recall(&LAST_POLLING),
    });

    if has_writes(model) {
        info.dpi_min = model.profile.dpi_min;
        info.dpi_max = model.profile.dpi_max;
        info.dpi_step = DPI_STEP;
        info.polling_supported = POLLING_HZ.to_vec();
        info.polling_options = POLLING_HZ.to_vec();
        info.capabilities.dpi_write = true;
        info.capabilities.polling_write = true;
        // The mouse cannot be asked for its DPI or polling rate, so without this the screen
        // said "DPI not available" until something had been written. The controls are shown
        // from the start; DPI begins at a usual value (the real one is unknown until the user
        // picks one) and no polling tile is marked until the user chooses.
        info.capabilities.dpi_read = true;
        info.capabilities.polling_read = true;
        if info.dpi == 0 {
            info.dpi = START_DPI.clamp(model.profile.dpi_min, model.profile.dpi_max);
        }
        // the levels are the ones the user keeps in the app, not a single read-only one
        info.dpi_levels = Vec::new();
    }

    info
}

pub struct SteelSeriesDriver;

impl MouseDriver for SteelSeriesDriver {
    fn id(&self) -> &'static str {
        "steelseries-aerox"
    }

    fn brand(&self) -> &'static str {
        "SteelSeries"
    }

    fn present(&self, api: &HidApi) -> bool {
        api.device_list()
            .any(|device| device.vendor_id() == STEELSERIES_VID && model_for(device.product_id()).is_some())
    }

    fn probe(&self, api: &HidApi, log: &mut Vec<String>) -> Option<MouseInfoDto> {
        let mut first_model: Option<&'static Model> = None;

        for info in api.device_list() {
            if info.vendor_id() != STEELSERIES_VID {
                continue;
            }
            let Some(model) = model_for(info.product_id()) else {
                continue;
            };
            first_model.get_or_insert(model);

            let path = info.path().to_string_lossy().to_string();
            if !is_control_interface(info.usage_page(), info.usage(), info.interface_number(), &path) {
                continue;
            }

            if !model.has_battery {
                log.push(format!(
                    "SteelSeries {:04X}:{:04X} interface {} usage_page={:04X}: wired model (no battery)",
                    info.vendor_id(),
                    info.product_id(),
                    info.interface_number(),
                    info.usage_page()
                ));
                return Some(describe(model, None, false));
            }

            log.push(format!(
                "SteelSeries {:04X}:{:04X} interface {} usage_page={:04X}: asking the battery",
                info.vendor_id(),
                info.product_id(),
                info.interface_number(),
                info.usage_page()
            ));

            let device = match info.open_device(api) {
                Ok(device) => device,
                Err(error) => {
                    log.push(format!("  cannot open: {}", error));
                    continue;
                }
            };

            let answer = ask(&device, model.receiver);
            log.push(format!("  battery answer: {:?}", answer));

            let (battery, charging) = match answer {
                Some(Battery::Level { percent, charging }) => (Some(percent), charging),
                _ => (None, false),
            };

            return Some(describe(model, battery, charging));
        }

        // Plugged in but no interface answered: show the model anyway, without values.
        first_model.map(|model| describe(model, None, false))
    }

    fn set_polling(&self, hz: u32) -> Result<u32, String> {
        let applied = write_setting(|model| {
            let msg = polling_message_for(model.profile, model.receiver, hz).ok_or_else(|| {
                format!(
                    "{} Hz is not a valid polling rate for this mouse (125, 250, 500, 1000)",
                    hz
                )
            })?;
            Ok((hz, msg))
        })?;
        remember(&LAST_POLLING, applied);
        Ok(applied)
    }

    fn set_dpi(&self, value: u32) -> Result<u32, String> {
        let applied = write_setting(|model| Ok(dpi_message_for(model.profile, model.receiver, value)))?;
        remember(&LAST_DPI, applied);
        Ok(applied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_bytes() {
        assert_eq!(command(false), 0x92);
        assert_eq!(command(true), 0xD2);
    }

    #[test]
    fn battery_with_and_without_report_id() {
        // step 11 -> 50 %
        assert_eq!(
            parse_battery(&[0x00, 0x92, 0x0B, 0x00], false),
            Some(Battery::Level { percent: 50, charging: false })
        );
        assert_eq!(
            parse_battery(&[0xD2, 0x0B], true),
            Some(Battery::Level { percent: 50, charging: false })
        );
    }

    #[test]
    fn battery_limits_and_charging() {
        // step 1 -> 0 %, step 21 -> 100 %, bit 7 = charging
        assert_eq!(
            parse_battery(&[0x00, 0x92, 0x01], false),
            Some(Battery::Level { percent: 0, charging: false })
        );
        assert_eq!(
            parse_battery(&[0x00, 0x92, 0x95], false),
            Some(Battery::Level { percent: 100, charging: true })
        );
        // out of range = mouse not connected
        assert_eq!(parse_battery(&[0x00, 0x92, 0x00], false), Some(Battery::Missing));
        assert_eq!(parse_battery(&[0x00, 0x92, 0x16], false), Some(Battery::Missing));
    }

    #[test]
    fn other_answers_are_ignored() {
        // wrong command byte, too short
        assert_eq!(parse_battery(&[0x00, 0x91, 0x0B], false), None);
        assert_eq!(parse_battery(&[0x00, 0x92, 0x0B], true), None);
        assert_eq!(parse_battery(&[0x92], false), None);
        assert_eq!(parse_battery(&[], false), None);
    }

    #[test]
    fn interface_choice() {
        assert!(is_control_interface(0xFFC0, 1, -1, ""));
        assert!(is_control_interface(0x0001, 2, -1, "\\\\?\\hid#vid_1038&pid_1838&mi_03#7"));
        assert!(is_control_interface(0x0001, 2, 3, ""));
        assert!(!is_control_interface(0x0001, 2, 0, "\\\\?\\hid#vid_1038&pid_1838&mi_00#7"));
    }

    #[test]
    fn ids_are_known() {
        assert!(model_for(0x1836).is_some());
        assert!(!model_for(0x1836).unwrap().has_battery);
        assert!(model_for(0x1838).is_some());
        assert!(model_for(0x185A).is_some() && !model_for(0x185A).unwrap().receiver);
        assert!(model_for(0x1858).unwrap().receiver);
        assert!(model_for(0x1234).is_none());
    }

    #[test]
    fn dpi_table_matches_rivalcfg_samples() {
        assert_eq!(dpi_code_for(&WIRELESS_PROFILE, 100), (100, 0x00));
        assert_eq!(dpi_code_for(&WIRELESS_PROFILE, 200), (200, 0x02));
        assert_eq!(dpi_code_for(&WIRELESS_PROFILE, 400), (400, 0x04));
        assert_eq!(dpi_code_for(&WIRELESS_PROFILE, 800), (800, 0x09));
        assert_eq!(dpi_code_for(&WIRELESS_PROFILE, 1600), (1600, 0x12));
        assert_eq!(dpi_code_for(&WIRELESS_PROFILE, 18000), (18000, 0xD6));
        assert_eq!(DPI_CODES.len(), 180);
        // strictly increasing codes
        assert!(DPI_CODES.windows(2).all(|pair| pair[0] < pair[1]));

        // TrueMove Core (Aerox 3 wired)
        assert_eq!(dpi_code_for(&AEROX3_WIRED_PROFILE, 200), (200, 0x04));
        assert_eq!(dpi_code_for(&AEROX3_WIRED_PROFILE, 800), (800, 0x12));
        assert_eq!(dpi_code_for(&AEROX3_WIRED_PROFILE, 1600), (1600, 0x24));
        assert_eq!(dpi_code_for(&AEROX3_WIRED_PROFILE, 8500), (8500, 0xC5));
        assert_eq!(TRUEMOVE_CORE_DPI_CODES.len(), 84);
        assert!(TRUEMOVE_CORE_DPI_CODES.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn dpi_is_rounded_and_clamped() {
        assert_eq!(dpi_code_for(&WIRELESS_PROFILE, 0).0, 100);
        assert_eq!(dpi_code_for(&WIRELESS_PROFILE, 449).0, 400);
        assert_eq!(dpi_code_for(&WIRELESS_PROFILE, 450).0, 500);
        assert_eq!(dpi_code_for(&WIRELESS_PROFILE, 99999).0, 18000);

        assert_eq!(dpi_code_for(&AEROX3_WIRED_PROFILE, 0).0, 200);
        assert_eq!(dpi_code_for(&AEROX3_WIRED_PROFILE, 449).0, 400);
        assert_eq!(dpi_code_for(&AEROX3_WIRED_PROFILE, 450).0, 500);
        assert_eq!(dpi_code_for(&AEROX3_WIRED_PROFILE, 99999).0, 8500);
    }

    #[test]
    fn dpi_and_polling_messages() {
        assert_eq!(dpi_message_for(&WIRELESS_PROFILE, false, 800), (800, vec![0x2D, 0x01, 0x00, 0x09]));
        assert_eq!(dpi_message_for(&WIRELESS_PROFILE, true, 800), (800, vec![0x6D, 0x01, 0x00, 0x09]));
        assert_eq!(polling_message_for(&WIRELESS_PROFILE, false, 1000), Some(vec![0x2B, 0x00]));
        assert_eq!(polling_message_for(&WIRELESS_PROFILE, true, 125), Some(vec![0x6B, 0x03]));
        assert_eq!(polling_message_for(&WIRELESS_PROFILE, false, 2000), None);

        // Aerox 3 wired: 0-based preset, TrueMove Core table, 1..=4 polling codes
        assert_eq!(
            dpi_message_for(&AEROX3_WIRED_PROFILE, false, 800),
            (800, vec![0x2D, 0x01, 0x00, 0x12])
        );
        assert_eq!(
            polling_message_for(&AEROX3_WIRED_PROFILE, false, 1000),
            Some(vec![0x2B, 0x01])
        );
        assert_eq!(
            polling_message_for(&AEROX3_WIRED_PROFILE, false, 125),
            Some(vec![0x2B, 0x04])
        );
        assert_eq!(polling_message_for(&AEROX3_WIRED_PROFILE, false, 2000), None);
    }

    #[test]
    fn no_message_is_a_dangerous_command() {
        // never the save-to-memory commands (11 / 51) or anything but 2D/2B/92 families
        for profile in [&WIRELESS_PROFILE, &AEROX3_WIRED_PROFILE] {
            for receiver in [false, true] {
                let first = [
                    dpi_message_for(profile, receiver, 1200).1[0],
                    polling_message_for(profile, receiver, 500).unwrap()[0],
                    command(receiver),
                ];
                for byte in first {
                    assert!(matches!(byte & !RECEIVER_FLAG, 0x2D | 0x2B | 0x92));
                }
            }
        }
    }

    #[test]
    fn dpi_and_polling_controls_are_shown_from_the_start() {
        // Aerox 3 Wireless: the mouse cannot report DPI, but the screen must still offer it.
        let model = model_for(0x183A).expect("Aerox 3 Wireless is known");
        let info = describe(model, None, false);
        assert_eq!(info.connection, "wired");
        assert!(info.capabilities.dpi_read && info.capabilities.dpi_write);
        assert!(info.capabilities.polling_read && info.capabilities.polling_write);
        assert_eq!(info.dpi, START_DPI);
        assert_eq!((info.dpi_min, info.dpi_max, info.dpi_step), (100, 18000, 100));
        assert!(info.dpi_levels.is_empty());
        assert_eq!(info.polling_supported, vec![125, 250, 500, 1000]);

        // Aerox 3 Wired: no battery, TrueMove Core range 200..=8500.
        let wired = model_for(0x1836).expect("Aerox 3 wired is known");
        let wired_info = describe(wired, Some(50), true);
        assert_eq!(wired_info.connection, "wired");
        assert!(!wired_info.capabilities.battery);
        assert_eq!(wired_info.battery, None);
        assert!(!wired_info.charging);
        assert!(wired_info.capabilities.dpi_read && wired_info.capabilities.dpi_write);
        assert!(wired_info.capabilities.polling_read && wired_info.capabilities.polling_write);
        assert_eq!(wired_info.dpi, START_DPI);
        assert_eq!((wired_info.dpi_min, wired_info.dpi_max, wired_info.dpi_step), (200, 8500, 100));
        assert_eq!(wired_info.polling_supported, vec![125, 250, 500, 1000]);
    }
}
