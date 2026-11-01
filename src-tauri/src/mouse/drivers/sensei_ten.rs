//! SteelSeries Sensei Ten (cable): DPI and polling rate. No battery (it has a cable).
//!
//! Sources (only facts are used, the code is written from scratch): rivalcfg 4.17.0 by Fabien
//! Loison (WTFPL) and OpenSeries 1.0.0 (MIT). Not tested on real hardware by us: "expected".
//!
//! Output reports with report id 0:
//! * DPI: `55 00 <mask> <selected> <lo> <hi>` with one preset: mask 01, selected 01, value = DPI / 50
//!   little endian (50..18000 DPI);
//! * polling rate: `54 00 <code>` with 125 Hz = 4, 250 Hz = 3, 500 Hz = 2, 1000 Hz = 1.
//! The "save" command (`59 00`), lighting and buttons are never sent.

use std::sync::Mutex;

use hidapi::HidApi;

use super::readonly::{self, Reading};
use super::MouseDriver;
use crate::mouse::MouseInfoDto;

const VID: u16 = 0x1038;
const PIDS: [(u16, &str); 2] = [
    (0x1832, "SteelSeries Sensei Ten"),
    (0x1834, "SteelSeries Sensei Ten CS:GO Neon Rider Edition"),
];
const DPI_MIN: u32 = 50;
const DPI_MAX: u32 = 18000;
const DPI_STEP: u32 = 50;
const POLLING_HZ: [u32; 4] = [125, 250, 500, 1000];

static LAST_DPI: Mutex<Option<u32>> = Mutex::new(None);
static LAST_POLLING: Mutex<Option<u32>> = Mutex::new(None);

fn name_for(pid: u16) -> Option<&'static str> {
    PIDS.iter().find(|(p, _)| *p == pid).map(|(_, n)| *n)
}

/// (DPI that will be set, message without the report id)
pub fn dpi_message(value: u32) -> (u32, Vec<u8>) {
    let dpi = (value.clamp(DPI_MIN, DPI_MAX) + DPI_STEP / 2) / DPI_STEP * DPI_STEP;
    let dpi = dpi.min(DPI_MAX);
    let scaled = dpi / DPI_STEP;
    (dpi, vec![0x55, 0x00, 0x01, 0x01, (scaled & 0xFF) as u8, (scaled >> 8) as u8])
}

pub fn polling_message(hz: u32) -> Option<Vec<u8>> {
    let code = match hz {
        125 => 4,
        250 => 3,
        500 => 2,
        1000 => 1,
        _ => return None,
    };
    Some(vec![0x54, 0x00, code])
}

/// Tries the usual output report sizes on every interface of the mouse until one accepts the message.
fn write_message(message: &[u8]) -> Result<(), String> {
    let api = HidApi::new().map_err(|error| error.to_string())?;
    let mut failure = String::from("Sensei Ten not found");

    // Interface 0 first (that is the one rivalcfg uses)
    let mut interfaces: Vec<_> = api
        .device_list()
        .filter(|d| d.vendor_id() == VID && name_for(d.product_id()).is_some())
        .collect();
    interfaces.sort_by_key(|d| d.interface_number() != 0);

    for info in interfaces {
        let device = match info.open_device(&api) {
            Ok(device) => device,
            Err(error) => {
                failure = format!("cannot open the mouse: {}", error);
                continue;
            }
        };
        for length in [15usize, 16, 33, 64, 65] {
            let mut request = vec![0u8; length.max(message.len() + 1)];
            request[1..=message.len()].copy_from_slice(message);
            match device.write(&request) {
                Ok(_) => return Ok(()),
                Err(error) => failure = error.to_string(),
            }
        }
    }

    Err(failure)
}

fn describe(pid: u16) -> MouseInfoDto {
    let mut info = readonly::build(Reading {
        brand: "SteelSeries",
        name: name_for(pid).unwrap_or("SteelSeries Sensei Ten").to_string(),
        connection: "wired",
        battery: None,
        charging: false,
        dpi: *LAST_DPI.lock().unwrap_or_else(|error| error.into_inner()),
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

pub struct SenseiTenDriver;

impl MouseDriver for SenseiTenDriver {
    fn id(&self) -> &'static str {
        "steelseries-sensei-ten"
    }

    fn brand(&self) -> &'static str {
        "SteelSeries"
    }

    fn present(&self, api: &HidApi) -> bool {
        api.device_list().any(|d| d.vendor_id() == VID && name_for(d.product_id()).is_some())
    }

    fn probe(&self, api: &HidApi, log: &mut Vec<String>) -> Option<MouseInfoDto> {
        let pid = api
            .device_list()
            .find(|d| d.vendor_id() == VID && name_for(d.product_id()).is_some())?
            .product_id();
        log.push(format!("SteelSeries Sensei Ten {:04X}:{:04X} found (settings are write only)", VID, pid));
        Some(describe(pid))
    }

    fn set_polling(&self, hz: u32) -> Result<u32, String> {
        let message = polling_message(hz)
            .ok_or_else(|| format!("{} Hz is not a valid polling rate for this mouse (125, 250, 500, 1000)", hz))?;
        write_message(&message)?;
        *LAST_POLLING.lock().unwrap_or_else(|error| error.into_inner()) = Some(hz);
        Ok(hz)
    }

    fn set_dpi(&self, value: u32) -> Result<u32, String> {
        let (applied, message) = dpi_message(value);
        write_message(&message)?;
        *LAST_DPI.lock().unwrap_or_else(|error| error.into_inner()) = Some(applied);
        Ok(applied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages() {
        assert_eq!(dpi_message(800), (800, vec![0x55, 0x00, 0x01, 0x01, 16, 0]));
        // 18000 / 50 = 360 = 0x0168
        assert_eq!(dpi_message(18000).1, vec![0x55, 0x00, 0x01, 0x01, 0x68, 0x01]);
        assert_eq!(dpi_message(0).0, 50);
        assert_eq!(dpi_message(99999).0, 18000);
        assert_eq!(dpi_message(124).0, 100);
        assert_eq!(polling_message(1000), Some(vec![0x54, 0x00, 1]));
        assert_eq!(polling_message(125), Some(vec![0x54, 0x00, 4]));
        assert_eq!(polling_message(2000), None);
    }

    #[test]
    fn no_save_command() {
        assert_eq!(dpi_message(400).1[0], 0x55);
        assert_eq!(polling_message(500).unwrap()[0], 0x54);
    }

    #[test]
    fn ids() {
        assert!(name_for(0x1832).is_some() && name_for(0x1834).is_some());
        assert!(name_for(0x1836).is_none());
    }
}
