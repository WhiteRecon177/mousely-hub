//! Pulsar X3 and X3 LHD on their cable: battery, charging, DPI and polling rate (read and write).
//!
//! Source (only facts are used, the code is written from scratch): the project
//! "pulsar-mouse-linux" by Packerlschupfer, licence MIT (files `drivers/x2a.py`, `drivers/x3.py`,
//! `drivers/x3_lhd.py`, `drivers/xlite_v4_wireless.py`). Not tested on real hardware by us: the
//! support level is "expected". Only the cable ids are handled (3710:3409 X3, 3710:3410 X3 LHD);
//! the dongles need extra radio-wait rules and are not handled.
//!
//! Interface 3, 64 byte FEATURE report with report id 0 (65 bytes for hidapi, the first is the id).
//! Request (payload indexes): 0 = 00, 1 = category, 2 = register (or 80 for a read), 3 = sub register,
//! 6 = profile, 7.. = data, 62..63 = sum of bytes 0..61 as little endian 16 bit. After sending, the same
//! report is read until byte 0 becomes 01 (answer ready; 02 means "busy", 00 "not taken yet").
//! * active profile: read `02 86 01` -> byte 6 (1..6)
//! * DPI table of a profile: read `05 84 15 <profile>` -> byte 7 = active stage, byte 8 = count,
//!   stage i: byte 9+5i = number, X little endian at 10+5i, Y at 12+5i (absolute DPI, 100..32000)
//! * write the table: `05 04 21 <profile>` + `active, count, (number, xlo, xhi, ylo, yhi)...`
//! * polling: read `01 89 02` -> byte 7; write `01 09 02` + code (125 = 1, 250 = 2, 500 = 4, 1000 = 8)
//! * battery: read `08 81 01` -> byte 6 = percent (FF = no reading); charging: read `08 83 01` -> byte 6
//! Setting the DPI rewrites the whole table of the ACTIVE profile with only the active stage changed.
//! The source documents no RAM-only mode: the mouse keeps the values in the profile, as with the
//! official software. Never sent: reset, profile change, buttons, macros, lighting, firmware.

use std::thread::sleep;
use std::time::Duration;

use hidapi::{HidApi, HidDevice};

use super::readonly::{self, Reading};
use super::MouseDriver;
use crate::mouse::MouseInfoDto;

const VID: u16 = 0x3710;
const PIDS: [(u16, &str); 2] = [(0x3409, "Pulsar X3"), (0x3410, "Pulsar X3 LHD")];
const INTERFACE: i32 = 3;

const DPI_MIN: u32 = 100;
const DPI_MAX: u32 = 32000;
const DPI_STEP: u32 = 10;
const POLLING: [(u32, u8); 4] = [(125, 1), (250, 2), (500, 4), (1000, 8)];
const MAX_STAGES: usize = 6;

fn name_for(pid: u16) -> Option<&'static str> {
    PIDS.iter().find(|(p, _)| *p == pid).map(|(_, n)| *n)
}

/// 65 bytes for hidapi: report id 0, then the 64 byte payload.
pub fn build(category: u8, register: u8, sub: u8, profile: u8, data: &[u8]) -> Vec<u8> {
    let mut payload = [0u8; 64];
    payload[1] = category;
    payload[2] = register;
    payload[3] = sub;
    payload[6] = profile;
    for (index, byte) in data.iter().enumerate() {
        if 7 + index < 62 {
            payload[7 + index] = *byte;
        }
    }
    let sum: u32 = payload[..62].iter().map(|b| u32::from(*b)).sum();
    payload[62] = (sum & 0xFF) as u8;
    payload[63] = ((sum >> 8) & 0xFF) as u8;

    let mut report = vec![0u8];
    report.extend_from_slice(&payload);
    report
}

pub fn dpi_table_message(profile: u8, active: u8, stages: &[u32]) -> Vec<u8> {
    let mut data = vec![active, stages.len() as u8];
    for (index, dpi) in stages.iter().enumerate() {
        let lo = (dpi & 0xFF) as u8;
        let hi = ((dpi >> 8) & 0xFF) as u8;
        data.extend_from_slice(&[index as u8 + 1, lo, hi, lo, hi]);
    }
    build(0x05, 0x04, 0x21, profile, &data)
}

pub fn polling_message(hz: u32) -> Option<Vec<u8>> {
    let code = POLLING.iter().find(|(rate, _)| *rate == hz)?.1;
    Some(build(0x01, 0x09, 0x02, 0x00, &[code]))
}

/// DPI rounded to the 10 step and clamped.
pub fn round_dpi(value: u32) -> u32 {
    ((value.clamp(DPI_MIN, DPI_MAX) + DPI_STEP / 2) / DPI_STEP * DPI_STEP).min(DPI_MAX)
}

#[derive(Debug, PartialEq, Eq)]
pub struct Table {
    pub active: usize, // 1-based
    pub stages: Vec<u32>,
}

/// `payload` = the 64 bytes of the answer (without the report id).
pub fn parse_table(payload: &[u8]) -> Option<Table> {
    if payload.len() < 64 || payload[0] != 0x01 {
        return None;
    }
    let active = usize::from(payload[7]);
    let count = usize::from(payload[8]);
    if count == 0 || count > MAX_STAGES || active == 0 || active > count {
        return None;
    }
    let mut stages = Vec::new();
    for index in 0..count {
        let base = 9 + 5 * index;
        let x = u32::from(payload[base + 1]) | (u32::from(payload[base + 2]) << 8);
        if !(DPI_MIN..=DPI_MAX).contains(&x) {
            return None;
        }
        stages.push(x);
    }
    Some(Table { active, stages })
}

pub fn parse_polling(payload: &[u8]) -> Option<u32> {
    if payload.len() < 8 || payload[0] != 0x01 {
        return None;
    }
    POLLING.iter().find(|(_, code)| *code == payload[7]).map(|(hz, _)| *hz)
}

/// (percent, charging) from the two answers; 0xFF = no reading.
pub fn parse_battery(percent: &[u8], charging: &[u8]) -> Option<(u32, bool)> {
    if percent.len() < 7 || percent[0] != 0x01 || percent[6] > 100 {
        return None;
    }
    let charging = charging.len() >= 7 && charging[0] == 0x01 && charging[6] != 0;
    Some((u32::from(percent[6]), charging))
}

/// Sends a request and waits for the answer. A read is ready when byte 0 is 01; a write is accepted
/// when it is 01 or 02. A read still on 00 is resent a few times (the source documents this).
fn transact(device: &HidDevice, request: &[u8], is_read: bool) -> Result<Vec<u8>, String> {
    let mut resends = if is_read { 3 } else { 0 };

    loop {
        device.send_feature_report(request).map_err(|error| error.to_string())?;

        let mut last = 0xFFu8;
        for _ in 0..8 {
            let mut buffer = [0u8; 65];
            let read = device.get_feature_report(&mut buffer).map_err(|error| error.to_string())?;
            if read < 2 {
                return Err("empty answer".to_string());
            }
            last = buffer[1];
            let ready = last == 0x01 || (!is_read && last == 0x02);
            if ready {
                return Ok(buffer[1..65].to_vec());
            }
            if last == 0x02 && is_read {
                // busy: keep waiting, do not resend
            }
            sleep(Duration::from_millis(20));
        }

        if last == 0x00 && resends > 0 {
            resends -= 1;
            sleep(Duration::from_millis(100));
            continue;
        }
        return Err(format!("the mouse did not answer (last state {:02X})", last));
    }
}

fn open_control(api: &HidApi) -> Option<(HidDevice, u16)> {
    for info in api.device_list() {
        if info.vendor_id() == VID && name_for(info.product_id()).is_some() && info.interface_number() == INTERFACE {
            if let Ok(device) = info.open_device(api) {
                return Some((device, info.product_id()));
            }
        }
    }
    None
}

fn active_profile(device: &HidDevice) -> Result<u8, String> {
    let answer = transact(device, &build(0x02, 0x86, 0x01, 0, &[]), true)?;
    let profile = answer[6];
    if (1..=6).contains(&profile) {
        Ok(profile)
    } else {
        Err(format!("unexpected active profile {}", profile))
    }
}

fn read_table(device: &HidDevice, profile: u8) -> Result<Table, String> {
    let answer = transact(device, &build(0x05, 0x84, 0x15, profile, &[]), true)?;
    parse_table(&answer).ok_or_else(|| "unreadable DPI table".to_string())
}

pub struct PulsarX3Driver;

impl MouseDriver for PulsarX3Driver {
    fn id(&self) -> &'static str {
        "pulsar-x3"
    }

    fn brand(&self) -> &'static str {
        "Pulsar"
    }

    fn present(&self, api: &HidApi) -> bool {
        api.device_list().any(|d| d.vendor_id() == VID && name_for(d.product_id()).is_some())
    }

    fn probe(&self, api: &HidApi, log: &mut Vec<String>) -> Option<MouseInfoDto> {
        let first_pid = api
            .device_list()
            .find(|d| d.vendor_id() == VID && name_for(d.product_id()).is_some())?
            .product_id();
        let name = name_for(first_pid).unwrap_or("Pulsar X3");

        let Some((device, pid)) = open_control(api) else {
            log.push("Pulsar X3: control interface not available".to_string());
            return Some(build_info(name, None, false, None, None));
        };
        log.push(format!("Pulsar X3 {:04X}:{:04X}: asking battery, DPI and polling", VID, pid));

        let battery = transact(&device, &build(0x08, 0x81, 0x01, 0, &[]), true).and_then(|percent| {
            let charging = transact(&device, &build(0x08, 0x83, 0x01, 0, &[]), true).unwrap_or_default();
            parse_battery(&percent, &charging).ok_or_else(|| "no battery reading".to_string())
        });
        log.push(format!("  battery: {:?}", battery));

        let dpi = active_profile(&device)
            .and_then(|profile| read_table(&device, profile))
            .map(|table| table.stages[table.active - 1]);
        log.push(format!("  dpi: {:?}", dpi));

        let polling = transact(&device, &build(0x01, 0x89, 0x02, 0, &[]), true)
            .ok()
            .and_then(|answer| parse_polling(&answer));
        log.push(format!("  polling: {:?}", polling));

        let (percent, charging) = match battery {
            Ok((percent, charging)) => (Some(percent), charging),
            Err(_) => (None, false),
        };
        Some(build_info(name_for(pid).unwrap_or(name), percent, charging, dpi.ok(), polling))
    }

    fn set_polling(&self, hz: u32) -> Result<u32, String> {
        let message = polling_message(hz)
            .ok_or_else(|| format!("{} Hz is not available on this mouse (125, 250, 500, 1000)", hz))?;
        let api = HidApi::new().map_err(|error| error.to_string())?;
        let (device, _) = open_control(&api).ok_or("Pulsar X3 not found")?;
        transact(&device, &message, false)?;
        Ok(hz)
    }

    fn set_dpi(&self, value: u32) -> Result<u32, String> {
        let dpi = round_dpi(value);
        let api = HidApi::new().map_err(|error| error.to_string())?;
        let (device, _) = open_control(&api).ok_or("Pulsar X3 not found")?;

        // Read first: only the active stage of the active profile is changed.
        let profile = active_profile(&device)?;
        let mut table = read_table(&device, profile)?;
        table.stages[table.active - 1] = dpi;

        transact(&device, &dpi_table_message(profile, table.active as u8, &table.stages), false)?;
        Ok(dpi)
    }
}

fn build_info(name: &str, battery: Option<u32>, charging: bool, dpi: Option<u32>, polling: Option<u32>) -> MouseInfoDto {
    let mut info = readonly::build(Reading {
        brand: "Pulsar",
        name: name.to_string(),
        connection: "wired",
        battery,
        charging,
        dpi,
        polling,
    });
    info.dpi_min = DPI_MIN;
    info.dpi_max = DPI_MAX;
    info.dpi_step = DPI_STEP;
    info.polling_options = POLLING.iter().map(|(hz, _)| *hz).collect();
    info.polling_supported = info.polling_options.clone();
    info.capabilities.dpi_write = true;
    info.capabilities.polling_write = true;
    info
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checksum_ok(report: &[u8]) -> bool {
        let sum: u32 = report[1..63].iter().map(|b| u32::from(*b)).sum();
        report[63] == (sum & 0xFF) as u8 && report[64] == ((sum >> 8) & 0xFF) as u8
    }

    #[test]
    fn requests_have_the_documented_layout() {
        let r = build(0x05, 0x84, 0x15, 1, &[]);
        assert_eq!(r.len(), 65);
        assert_eq!(&r[..7], &[0, 0, 0x05, 0x84, 0x15, 0, 0]);
        assert_eq!(r[7], 1); // profile at payload index 6
        assert!(checksum_ok(&r));
        // checksum of 00 05 84 15 00 00 01 = 0x9F
        assert_eq!((r[63], r[64]), (0x9F, 0x00));
    }

    #[test]
    fn polling_codes() {
        let p = polling_message(1000).unwrap();
        assert_eq!(&p[2..5], &[0x01, 0x09, 0x02]);
        assert_eq!(p[8], 8);
        assert!(checksum_ok(&p));
        assert_eq!(polling_message(125).unwrap()[8], 1);
        assert_eq!(polling_message(500).unwrap()[8], 4);
        assert!(polling_message(2000).is_none());
    }

    #[test]
    fn table_write_matches_the_documented_example() {
        // 800 DPI = 20 03, 1600 = 40 06 (CC dossier / source)
        let m = dpi_table_message(2, 1, &[800, 1600]);
        assert_eq!(&m[2..5], &[0x05, 0x04, 0x21]);
        assert_eq!(m[7], 2); // profile
        assert_eq!(&m[8..10], &[1, 2]); // active, count
        assert_eq!(&m[10..15], &[1, 0x20, 0x03, 0x20, 0x03]);
        assert_eq!(&m[15..20], &[2, 0x40, 0x06, 0x40, 0x06]);
        assert!(checksum_ok(&m));
        // 32000 = 00 7D
        let m = dpi_table_message(1, 1, &[32000]);
        assert_eq!(&m[11..13], &[0x00, 0x7D]);
    }

    #[test]
    fn rounding() {
        assert_eq!(round_dpi(0), 100);
        assert_eq!(round_dpi(804), 800);
        assert_eq!(round_dpi(805), 810);
        assert_eq!(round_dpi(99999), 32000);
    }

    #[test]
    fn parse_a_table() {
        let mut a = vec![0u8; 64];
        a[0] = 1;
        a[7] = 2;
        a[8] = 3;
        for (i, dpi) in [400u32, 800, 1600].iter().enumerate() {
            let base = 9 + 5 * i;
            a[base] = i as u8 + 1;
            a[base + 1] = (dpi & 0xFF) as u8;
            a[base + 2] = (dpi >> 8) as u8;
            a[base + 3] = a[base + 1];
            a[base + 4] = a[base + 2];
        }
        assert_eq!(parse_table(&a), Some(Table { active: 2, stages: vec![400, 800, 1600] }));
        a[7] = 4; // active > count
        assert_eq!(parse_table(&a), None);
        a[7] = 1;
        a[8] = 9; // too many
        assert_eq!(parse_table(&a), None);
        a[0] = 2; // not ready
        assert_eq!(parse_table(&a), None);
    }

    #[test]
    fn polling_and_battery_answers() {
        let mut a = vec![0u8; 64];
        a[0] = 1;
        a[7] = 4;
        assert_eq!(parse_polling(&a), Some(500));
        a[7] = 3;
        assert_eq!(parse_polling(&a), None);

        let mut p = vec![0u8; 64];
        p[0] = 1;
        p[6] = 87;
        let mut c = vec![0u8; 64];
        c[0] = 1;
        c[6] = 1;
        assert_eq!(parse_battery(&p, &c), Some((87, true)));
        c[6] = 0;
        assert_eq!(parse_battery(&p, &c), Some((87, false)));
        p[6] = 0xFF;
        assert_eq!(parse_battery(&p, &c), None);
    }

    #[test]
    fn only_known_categories_are_built() {
        // reads/writes used: 05/04/21, 05/84/15, 01/09/02, 01/89/02, 02/86/01, 08/81/01, 08/83/01
        for r in [
            dpi_table_message(1, 1, &[800]),
            polling_message(500).unwrap(),
            build(0x05, 0x84, 0x15, 1, &[]),
        ] {
            assert!(matches!((r[2], r[3], r[4]), (0x05, 0x04, 0x21) | (0x01, 0x09, 0x02) | (0x05, 0x84, 0x15)));
        }
    }
}
