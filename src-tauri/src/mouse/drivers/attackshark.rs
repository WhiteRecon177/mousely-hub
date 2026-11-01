//! Attack Shark X11 (2.4 GHz dongle or USB-C cable). READ ONLY and PASSIVE: battery only.
//!
//! Source: docs/protocol/PROTOCOL_EN.md of OpenSharkX11 (MIT licence; the vendored driver is
//! "Copyright (c) 2026 HarukaYamamoto0", the app "(c) Clevs"). Only facts are used (ids, the
//! meaning of the spontaneous battery packet); the code is written from scratch.
//! Not tested on real hardware by us: the support level is "expected".
//!
//! The documented protocol is write-only (settings are SET_REPORT feature reports). The only thing
//! the mouse says by itself is an input packet `03 55 40 01 <percent>` on the vendor interface
//! (interface 2). So this driver never writes anything: it listens for a short time and remembers
//! the last battery value it heard. DPI and polling rate cannot be read (the DPI button packet only
//! gives the stage number 1..6, not the DPI value).
//! The documentation warns that report id 0x0B made the dongle lose its pairing: nothing is sent here.

use std::sync::Mutex;

use hidapi::HidApi;

use super::readonly::{self, Reading};
use super::MouseDriver;
use crate::mouse::MouseInfoDto;

const ATTACK_SHARK_VID: u16 = 0x1D57;
const PID_DONGLE: u16 = 0xFA60;
const PID_WIRED: u16 = 0xFA55;
const CONFIG_INTERFACE: i32 = 2;

/// Last battery value heard from the mouse (it is sent by the mouse when it wants to).
static LAST_BATTERY: Mutex<Option<u32>> = Mutex::new(None);

/// A packet the mouse sent by itself.
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    Battery(u32),
    /// DPI button pressed, stage 1..6
    DpiStage(u8),
}

/// Reads a spontaneous packet: `03 55 40 01 XX` (battery) or `03 55 10 XX` (DPI stage).
/// The leading report id `03` may or may not be present.
pub fn parse_event(packet: &[u8]) -> Option<Event> {
    let start = match packet {
        [0x03, 0x55, ..] => 2,
        [0x55, ..] => 1,
        _ => return None,
    };

    match packet.get(start)? {
        0x40 if *packet.get(start + 1)? == 0x01 => {
            let percent = u32::from(*packet.get(start + 2)?);
            (percent <= 100).then_some(Event::Battery(percent))
        }
        0x10 => {
            let stage = *packet.get(start + 1)?;
            (1..=6).contains(&stage).then_some(Event::DpiStage(stage))
        }
        _ => None,
    }
}

pub struct AttackSharkDriver;

impl MouseDriver for AttackSharkDriver {
    fn id(&self) -> &'static str {
        "attackshark-x11"
    }

    fn brand(&self) -> &'static str {
        "Attack Shark"
    }

    fn present(&self, api: &HidApi) -> bool {
        let found = api.device_list().any(|device| {
            device.vendor_id() == ATTACK_SHARK_VID
                && (device.product_id() == PID_DONGLE || device.product_id() == PID_WIRED)
        });

        if !found {
            if let Ok(mut last) = LAST_BATTERY.lock() {
                *last = None;
            }
        }
        found
    }

    fn probe(&self, api: &HidApi, log: &mut Vec<String>) -> Option<MouseInfoDto> {
        let mut wired = false;
        let mut battery: Option<u32> = None;

        for info in api.device_list() {
            if info.vendor_id() != ATTACK_SHARK_VID {
                continue;
            }
            if info.product_id() != PID_DONGLE && info.product_id() != PID_WIRED {
                continue;
            }
            wired = info.product_id() == PID_WIRED;

            if info.interface_number() != CONFIG_INTERFACE {
                continue;
            }

            log.push(format!(
                "Attack Shark {:04X}:{:04X} interface 2: listening (nothing is sent)",
                info.vendor_id(),
                info.product_id()
            ));

            let Ok(device) = info.open_device(api) else {
                log.push("  cannot open the interface".to_string());
                continue;
            };

            // A short listen: the mouse is not asked, it only reports when it wants to.
            for _ in 0..3 {
                let mut buffer = [0u8; 32];
                match device.read_timeout(&mut buffer, 100) {
                    Ok(read) if read > 0 => match parse_event(&buffer[..read]) {
                        Some(Event::Battery(percent)) => battery = Some(percent),
                        Some(Event::DpiStage(stage)) => log.push(format!("  DPI stage {}", stage)),
                        None => {}
                    },
                    _ => break,
                }
            }
        }

        if let Ok(mut last) = LAST_BATTERY.lock() {
            match battery {
                Some(value) => *last = Some(value),
                None => battery = *last,
            }
        }

        log.push(format!("  battery={:?} (last heard)", battery));

        Some(readonly::build(Reading {
            brand: "Attack Shark",
            name: "Attack Shark X11".to_string(),
            connection: if wired { "wired" } else { "wireless" },
            battery,
            charging: false,
            dpi: None,
            polling: None,
        }))
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
    fn battery_packet() {
        assert_eq!(parse_event(&[0x03, 0x55, 0x40, 0x01, 0x64]), Some(Event::Battery(100)));
        assert_eq!(parse_event(&[0x55, 0x40, 0x01, 0x32]), Some(Event::Battery(50)));
        assert_eq!(parse_event(&[0x03, 0x55, 0x40, 0x01, 0x65]), None); // over 100
    }

    #[test]
    fn dpi_packet() {
        assert_eq!(parse_event(&[0x03, 0x55, 0x10, 0x03, 0x00]), Some(Event::DpiStage(3)));
        assert_eq!(parse_event(&[0x03, 0x55, 0x10, 0x07, 0x00]), None);
    }

    #[test]
    fn other_packets_are_ignored() {
        assert_eq!(parse_event(&[]), None);
        assert_eq!(parse_event(&[0x03]), None);
        assert_eq!(parse_event(&[0x03, 0x55, 0x50, 0x00, 0x04]), None); // ACK
        assert_eq!(parse_event(&[0x01, 0x00, 0x00]), None);
        assert_eq!(parse_event(&[0x03, 0x55, 0x40]), None);
    }
}
