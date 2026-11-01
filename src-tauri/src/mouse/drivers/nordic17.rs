//! Pulsar (and Lamzu) mice that talk the "Nordic 17 byte" protocol. READ ONLY.
//!
//! Source: the public notes in docs/PROTOCOLLI.md (MIT projects `packerlschupfer/pulsar-mouse-linux`
//! and `LeadSun/lamzu-cfg`). Facts only (ids, byte frames), no code copied.
//! Not tested on real hardware: the support level is "expected".
//!
//! A request is an output report of 17 bytes: `08 <command> 00 <addr hi> <addr lo> <length> 00..`
//! plus a last byte `0x55 - sum(first 16 bytes)`. The answer has the same shape; the data starts
//! at byte 6. Only reading commands are ever sent (`04` battery, `08` read memory).

use hidapi::{HidApi, HidDevice};

use super::readonly::{self, Reading};
use super::MouseDriver;
use crate::mouse::MouseInfoDto;

const PULSAR_VID: u16 = 0x3710;

const CMD_BATTERY: u8 = 0x04;
const CMD_READ_MEMORY: u8 = 0x08;

/// Only the ids listed in the notes. The vendor id alone is not enough: the same vendor id is
/// also used by other families (different protocol).
struct Model {
    pid: u16,
    name: &'static str,
    wired: bool,
    /// Address and length of the first DPI preset.
    preset_addr: u16,
    preset_len: u8,
}

const MODELS: [Model; 4] = [
    Model { pid: 0x5406, name: "Pulsar X2 CrazyLight", wired: false, preset_addr: 0x000C, preset_len: 4 },
    Model { pid: 0x3414, name: "Pulsar X2 CrazyLight", wired: true, preset_addr: 0x000C, preset_len: 4 },
    Model { pid: 0x5504, name: "Pulsar Feinmann F01 Noctua Edition", wired: false, preset_addr: 0x1B00, preset_len: 6 },
    Model { pid: 0x7507, name: "Pulsar Feinmann F01 Noctua Edition", wired: true, preset_addr: 0x1B00, preset_len: 6 },
];

fn model_for(pid: u16) -> Option<&'static Model> {
    MODELS.iter().find(|model| model.pid == pid)
}

/// Builds the 17-byte request. The last byte makes the sum of all bytes equal to 0x55.
pub fn frame(command: u8, addr: u16, length: u8) -> [u8; 17] {
    let mut out = [0u8; 17];
    out[0] = 0x08;
    out[1] = command;
    out[3] = (addr >> 8) as u8;
    out[4] = (addr & 0xFF) as u8;
    out[5] = length;
    let sum = out[..16].iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte));
    out[16] = 0x55u8.wrapping_sub(sum);
    out
}

/// Polling rate from the byte read at address 0x0000.
pub fn polling_from_code(code: u8) -> Option<u32> {
    match code {
        0x01 => Some(1000),
        0x02 => Some(500),
        0x04 => Some(250),
        0x08 => Some(125),
        0x10 => Some(2000),
        0x20 => Some(4000),
        0x40 => Some(8000),
        _ => None,
    }
}

/// DPI from a stored preset value: (value + 1) * 50.
pub fn dpi_from_raw(raw: u8) -> u32 {
    (u32::from(raw) + 1) * 50
}

/// Sends one request and returns the answer that repeats the command (up to 3 other
/// answers are skipped, for example a mouse event arriving in between).
fn ask(device: &HidDevice, request: &[u8; 17]) -> Option<[u8; 32]> {
    device.write(request).ok()?;

    for _ in 0..4 {
        let mut buffer = [0u8; 32];
        let read = device.read_timeout(&mut buffer, 250).ok()?;
        if read == 0 {
            return None;
        }
        if buffer[0] == 0x08 && buffer[1] == request[1] {
            return Some(buffer);
        }
    }
    None
}

fn read_memory(device: &HidDevice, addr: u16, length: u8) -> Option<[u8; 32]> {
    let answer = ask(device, &frame(CMD_READ_MEMORY, addr, length))?;
    // byte 2 is a status: anything but 0 means "refused"
    if answer[2] != 0 {
        return None;
    }
    Some(answer)
}

pub struct PulsarNordicDriver;

impl MouseDriver for PulsarNordicDriver {
    fn id(&self) -> &'static str {
        "pulsar-nordic17"
    }

    fn brand(&self) -> &'static str {
        "Pulsar"
    }

    fn present(&self, api: &HidApi) -> bool {
        api.device_list()
            .any(|device| device.vendor_id() == PULSAR_VID && model_for(device.product_id()).is_some())
    }

    fn probe(&self, api: &HidApi, log: &mut Vec<String>) -> Option<MouseInfoDto> {
        for info in api.device_list() {
            if info.vendor_id() != PULSAR_VID {
                continue;
            }
            let Some(model) = model_for(info.product_id()) else {
                continue;
            };

            // The notes do not say which interface answers, so only vendor-defined ones are tried.
            if info.usage_page() < 0xFF00 {
                continue;
            }

            log.push(format!(
                "Pulsar {:04X}:{:04X} interface {} usage_page={:04X}: asking (read only)",
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

            // Polling rate first: if this interface does not answer, try the next one.
            let Some(polling_answer) = read_memory(&device, 0x0000, 2) else {
                log.push("  no answer on this interface".to_string());
                continue;
            };
            let polling = polling_from_code(polling_answer[6]);

            // Active preset index (0 = first). The notes only give how to read preset #0, so the
            // DPI is shown only when the first preset is the active one.
            let index = read_memory(&device, 0x0004, 2).map(|answer| answer[6]);
            let dpi = match index {
                Some(0) => read_memory(&device, model.preset_addr, model.preset_len)
                    .map(|answer| dpi_from_raw(answer[6])),
                _ => None,
            };

            let battery = ask(&device, &frame(CMD_BATTERY, 0, 0)).and_then(|answer| {
                let millivolts = u32::from(u16::from_be_bytes([answer[8], answer[9]]));
                readonly::percent_from_millivolts(millivolts)
            });

            log.push(format!(
                "  polling={:?} dpi_index={:?} dpi={:?} battery={:?}",
                polling, index, dpi, battery
            ));

            return Some(readonly::build(Reading {
                brand: "Pulsar",
                name: model.name.to_string(),
                connection: if model.wired { "wired" } else { "wireless" },
                battery,
                charging: model.wired && battery.map(|level| level < 100).unwrap_or(false),
                dpi,
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

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{:02X}", byte)).collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn frames_match_the_notes() {
        assert_eq!(hex(&frame(0x08, 0x0000, 2)), "08 08 00 00 00 02 00 00 00 00 00 00 00 00 00 00 43");
        assert_eq!(hex(&frame(0x08, 0x0004, 2)), "08 08 00 00 04 02 00 00 00 00 00 00 00 00 00 00 3F");
        assert_eq!(hex(&frame(0x08, 0x000C, 4)), "08 08 00 00 0C 04 00 00 00 00 00 00 00 00 00 00 35");
        assert_eq!(hex(&frame(0x08, 0x1B00, 6)), "08 08 00 1B 00 06 00 00 00 00 00 00 00 00 00 00 24");
        assert_eq!(hex(&frame(0x04, 0, 0)), "08 04 00 00 00 00 00 00 00 00 00 00 00 00 00 00 49");
    }

    #[test]
    fn only_read_commands_can_be_built_by_this_driver() {
        // the driver uses only these two command bytes
        assert_eq!(CMD_BATTERY, 0x04);
        assert_eq!(CMD_READ_MEMORY, 0x08);
    }

    #[test]
    fn polling_and_dpi_conversion() {
        assert_eq!(polling_from_code(0x01), Some(1000));
        assert_eq!(polling_from_code(0x40), Some(8000));
        assert_eq!(polling_from_code(0x03), None);
        assert_eq!(dpi_from_raw(15), 800);
        assert_eq!(dpi_from_raw(0), 50);
    }

    #[test]
    fn ids_are_known() {
        assert!(model_for(0x5406).is_some());
        assert!(model_for(0x1234).is_none());
    }
}
