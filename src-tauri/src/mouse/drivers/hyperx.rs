//! HyperX Pulsefire Dart (2.4 GHz dongle and cable): battery, DPI (read and write), polling rate.
//!
//! Sources (only facts are used, the code is written from scratch):
//! * "HyperX Gaming Mouse protocol documentation (NGenuity2 protocol)" by santeri3700, licence
//!   CC0 1.0: ids, interfaces, polling command `D0`, DPI command `D3`, heartbeat `51`;
//! * "hyperx-pulsefire-battery" 2.0.0 by Radoslav Chobanov, licence MIT: the DPI question `53`
//!   and the layout of its answer.
//! Not tested on real hardware by us: the support level is "expected".
//!
//! Every message is 64 bytes, sent as an output report with report id 0 (65 bytes with the id).
//! * battery: `51`; answer byte 4 = percent, byte 5 = 1 when charging;
//! * DPI question: `53`; answer byte 5 = active profile (0..4), 16 bit little endian values
//!   (DPI / 50) at bytes 10, 12, 14, 16, 18;
//! * set the DPI of a profile: `D3 02 <profile> 02 <lo> <hi>` (applied at once);
//! * polling rate: `D0 00 00 01 <code>` with 125 Hz = 0, 250 Hz = 1, 500 Hz = 2, 1000 Hz = 3.
//! The "save to the mouse memory" command (`DE`), lighting, buttons and macros are never sent,
//! so the settings are lost when the mouse is switched off.

use std::sync::Mutex;

use hidapi::{HidApi, HidDevice};

use super::readonly::{self, Reading};
use super::MouseDriver;
use crate::mouse::MouseInfoDto;

const VID: u16 = 0x0951;
const PID_WIRELESS: u16 = 0x16E1;
const PID_WIRED: u16 = 0x16E2;

const CMD_BATTERY: u8 = 0x51;
const CMD_DPI_QUERY: u8 = 0x53;
const CMD_POLLING: u8 = 0xD0;
const CMD_DPI_SET: u8 = 0xD3;

const DPI_MIN: u32 = 50;
const DPI_MAX: u32 = 16000;
const DPI_STEP: u32 = 50;
const POLLING_HZ: [u32; 4] = [125, 250, 500, 1000];

static LAST_POLLING: Mutex<Option<u32>> = Mutex::new(None);

fn is_known(pid: u16) -> bool {
    pid == PID_WIRELESS || pid == PID_WIRED
}

/// The control interface: usage page FF00 / interface 2 (dongle), FF13 / interface 1 (cable).
fn is_control_interface(pid: u16, usage_page: u16, interface: i32) -> bool {
    match pid {
        PID_WIRELESS => usage_page == 0xFF00 || interface == 2,
        PID_WIRED => usage_page == 0xFF13 || interface == 1,
        _ => false,
    }
}

/// Packet to send: report id 0, command, then `args`, padded to 65 bytes.
pub fn packet(command: u8, args: &[u8]) -> Vec<u8> {
    let mut data = vec![0u8; 65];
    data[1] = command;
    data[2..2 + args.len()].copy_from_slice(args);
    data
}

pub fn polling_packet(hz: u32) -> Option<Vec<u8>> {
    let code = POLLING_HZ.iter().position(|&rate| rate == hz)? as u8;
    Some(packet(CMD_POLLING, &[0x00, 0x00, 0x01, code]))
}

/// Returns the DPI the packet will set (rounded to 50, clamped) and the packet.
pub fn dpi_packet(profile: u8, value: u32) -> (u32, Vec<u8>) {
    let dpi = (value.clamp(DPI_MIN, DPI_MAX) + DPI_STEP / 2) / DPI_STEP * DPI_STEP;
    let dpi = dpi.min(DPI_MAX);
    let scaled = dpi / DPI_STEP;
    (
        dpi,
        packet(CMD_DPI_SET, &[0x02, profile, 0x02, (scaled & 0xFF) as u8, (scaled >> 8) as u8]),
    )
}

/// The answer may or may not start with the report id 0.
fn payload<'a>(answer: &'a [u8], command: u8) -> Option<&'a [u8]> {
    if answer.first() == Some(&command) {
        Some(answer)
    } else if answer.len() > 1 && answer[0] == 0 && answer[1] == command {
        Some(&answer[1..])
    } else {
        None
    }
}

/// (percent, charging)
pub fn parse_battery(answer: &[u8]) -> Option<(u32, bool)> {
    let data = payload(answer, CMD_BATTERY)?;
    if data.len() < 6 || data[4] > 100 {
        return None;
    }
    Some((u32::from(data[4]), data[5] == 1))
}

/// (active profile, the five DPI values)
pub fn parse_dpi(answer: &[u8]) -> Option<(usize, [u32; 5])> {
    let data = payload(answer, CMD_DPI_QUERY)?;
    if data.len() < 20 {
        return None;
    }
    let active = usize::from(data[5]);
    if active > 4 {
        return None;
    }
    let mut values = [0u32; 5];
    for (slot, offset) in [10usize, 12, 14, 16, 18].iter().enumerate() {
        values[slot] = (u32::from(data[*offset]) | (u32::from(data[*offset + 1]) << 8)) * DPI_STEP;
    }
    Some((active, values))
}

/// Sends a question and returns the first answer that starts with the same command.
fn ask(device: &HidDevice, command: u8) -> Option<Vec<u8>> {
    device.write(&packet(command, &[])).ok()?;
    for _ in 0..6 {
        let mut buffer = [0u8; 64];
        let read = device.read_timeout(&mut buffer, 500).ok()?;
        if read == 0 {
            return None;
        }
        if payload(&buffer[..read], command).is_some() {
            return Some(buffer[..read].to_vec());
        }
    }
    None
}

fn current_dpi(device: &HidDevice) -> Option<(usize, u32)> {
    let answer = ask(device, CMD_DPI_QUERY)?;
    let (active, values) = parse_dpi(&answer)?;
    Some((active, values[active]))
}

fn open_control(api: &HidApi, log: &mut Vec<String>) -> Option<(HidDevice, u16)> {
    for info in api.device_list() {
        if info.vendor_id() != VID || !is_known(info.product_id()) {
            continue;
        }
        if !is_control_interface(info.product_id(), info.usage_page(), info.interface_number()) {
            continue;
        }
        match info.open_device(api) {
            Ok(device) => return Some((device, info.product_id())),
            Err(error) => log.push(format!("  HyperX: cannot open: {}", error)),
        }
    }
    None
}

fn describe(pid: u16, battery: Option<u32>, charging: bool, dpi: Option<u32>) -> MouseInfoDto {
    let wireless = pid == PID_WIRELESS;
    let mut info = readonly::build(Reading {
        brand: "HyperX",
        name: "HyperX Pulsefire Dart".to_string(),
        connection: if wireless { "wireless" } else { "wired" },
        battery,
        charging,
        dpi,
        polling: *LAST_POLLING.lock().unwrap_or_else(|error| error.into_inner()),
    });
    info.dpi_min = DPI_MIN;
    info.dpi_max = DPI_MAX;
    info.dpi_step = DPI_STEP;
    info.polling_options = POLLING_HZ.to_vec();
    info.polling_supported = POLLING_HZ.to_vec();
    info.capabilities.dpi_write = true;
    info.capabilities.polling_write = true;
    info
}

pub struct HyperXDriver;

impl MouseDriver for HyperXDriver {
    fn id(&self) -> &'static str {
        "hyperx-pulsefire-dart"
    }

    fn brand(&self) -> &'static str {
        "HyperX"
    }

    fn present(&self, api: &HidApi) -> bool {
        api.device_list().any(|d| d.vendor_id() == VID && is_known(d.product_id()))
    }

    fn probe(&self, api: &HidApi, log: &mut Vec<String>) -> Option<MouseInfoDto> {
        let first_pid = api
            .device_list()
            .find(|d| d.vendor_id() == VID && is_known(d.product_id()))
            .map(|d| d.product_id())?;

        let Some((device, pid)) = open_control(api, log) else {
            return Some(describe(first_pid, None, false, None));
        };

        log.push(format!("HyperX {:04X}:{:04X}: asking battery and DPI", VID, pid));
        let battery = ask(&device, CMD_BATTERY).and_then(|a| parse_battery(&a));
        let dpi = current_dpi(&device).map(|(_, dpi)| dpi);
        log.push(format!("  battery: {:?}, dpi: {:?}", battery, dpi));

        let (percent, charging) = match battery {
            Some((percent, charging)) => (Some(percent), charging),
            None => (None, false),
        };
        Some(describe(pid, percent, charging, dpi))
    }

    fn set_polling(&self, hz: u32) -> Result<u32, String> {
        let message = polling_packet(hz)
            .ok_or_else(|| format!("{} Hz is not a valid polling rate for this mouse (125, 250, 500, 1000)", hz))?;
        let api = HidApi::new().map_err(|error| error.to_string())?;
        let (device, _) = open_control(&api, &mut Vec::new()).ok_or("HyperX mouse not found")?;
        device.write(&message).map_err(|error| error.to_string())?;
        *LAST_POLLING.lock().unwrap_or_else(|error| error.into_inner()) = Some(hz);
        Ok(hz)
    }

    fn set_dpi(&self, value: u32) -> Result<u32, String> {
        let api = HidApi::new().map_err(|error| error.to_string())?;
        let (device, _) = open_control(&api, &mut Vec::new()).ok_or("HyperX mouse not found")?;
        // The DPI of the ACTIVE profile is changed, so the active profile must be known first.
        let (active, _) = current_dpi(&device).ok_or("could not read the active DPI profile")?;
        let (applied, message) = dpi_packet(active as u8, value);
        device.write(&message).map_err(|error| error.to_string())?;
        Ok(applied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polling_packets() {
        let p = polling_packet(1000).unwrap();
        assert_eq!(p.len(), 65);
        assert_eq!(&p[..6], &[0x00, 0xD0, 0x00, 0x00, 0x01, 0x03]);
        assert_eq!(polling_packet(125).unwrap()[5], 0x00);
        assert!(polling_packet(2000).is_none());
    }

    #[test]
    fn dpi_packet_matches_the_protocol_example() {
        // 16000 DPI -> 320 -> 0x40 0x01 (example in the CC0 document)
        let (dpi, p) = dpi_packet(0, 16000);
        assert_eq!(dpi, 16000);
        assert_eq!(&p[..7], &[0x00, 0xD3, 0x02, 0x00, 0x02, 0x40, 0x01]);
        let (dpi, p) = dpi_packet(3, 800);
        assert_eq!((dpi, p[3], p[5], p[6]), (800, 3, 16, 0));
    }

    #[test]
    fn dpi_is_rounded_and_clamped() {
        assert_eq!(dpi_packet(0, 0).0, 50);
        assert_eq!(dpi_packet(0, 74).0, 50);
        assert_eq!(dpi_packet(0, 75).0, 100);
        assert_eq!(dpi_packet(0, 99999).0, 16000);
    }

    #[test]
    fn battery_answer() {
        let mut a = vec![0u8; 64];
        a[0] = 0x51;
        a[3] = 3;
        a[4] = 87;
        a[5] = 1;
        assert_eq!(parse_battery(&a), Some((87, true)));
        let mut with_id = vec![0u8];
        with_id.extend_from_slice(&a);
        assert_eq!(parse_battery(&with_id), Some((87, true)));
        a[4] = 200;
        assert_eq!(parse_battery(&a), None);
        assert_eq!(parse_battery(&[0x50, 0, 0, 0, 5, 0]), None);
    }

    #[test]
    fn dpi_answer() {
        let mut a = vec![0u8; 64];
        a[0] = 0x53;
        a[5] = 2;
        for (i, off) in [10usize, 12, 14, 16, 18].iter().enumerate() {
            let scaled = (i as u32 + 1) * 8; // 400, 800, ...
            a[*off] = (scaled & 0xFF) as u8;
            a[*off + 1] = (scaled >> 8) as u8;
        }
        assert_eq!(parse_dpi(&a), Some((2, [400, 800, 1200, 1600, 2000])));
        a[5] = 9;
        assert_eq!(parse_dpi(&a), None);
    }

    #[test]
    fn interfaces() {
        assert!(is_control_interface(PID_WIRELESS, 0xFF00, -1));
        assert!(is_control_interface(PID_WIRELESS, 1, 2));
        assert!(is_control_interface(PID_WIRED, 0xFF13, -1));
        assert!(!is_control_interface(PID_WIRED, 1, 0));
    }

    #[test]
    fn never_sends_save_or_other_commands() {
        for p in [polling_packet(500).unwrap(), dpi_packet(1, 800).1, packet(CMD_BATTERY, &[]), packet(CMD_DPI_QUERY, &[])] {
            assert!(matches!(p[1], 0xD0 | 0xD3 | 0x51 | 0x53));
        }
    }
}
