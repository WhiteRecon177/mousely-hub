use hidapi::HidApi;

use super::database::{self, Recognised};
use super::{MouseCapabilitiesDto, MouseInfoDto, SupportLevel};

pub mod attackshark;
pub mod glorious;
pub mod hyperx;
pub mod lamzu_maya;
pub mod logitech;
pub mod nordic17;
pub mod pulsar_x3;
mod readonly;
pub mod sensei_ten;
pub mod steelseries;
pub mod vaxee;

const GENERIC_DESKTOP_USAGE_PAGE: u16 = 0x0001;
const MOUSE_USAGE: u16 = 0x0002;

pub trait MouseDriver: Send + Sync {
    #[allow(dead_code)]
    fn id(&self) -> &'static str;
    fn brand(&self) -> &'static str;
    /// Quick check: is a device of this brand plugged in? No reading, no waiting.
    fn present(&self, api: &HidApi) -> bool;
    /// Full reading of the mouse.
    fn probe(&self, api: &HidApi, log: &mut Vec<String>) -> Option<MouseInfoDto>;
    fn set_polling(&self, hz: u32) -> Result<u32, String>;
    fn set_dpi(&self, value: u32) -> Result<u32, String>;
}

pub fn registered() -> Vec<Box<dyn MouseDriver>> {
    vec![
        Box::new(logitech::LogitechDriver),
        // Experimental, read only (they never change a setting on the mouse)
        Box::new(nordic17::PulsarNordicDriver),
        Box::new(vaxee::VaxeeDriver),
        Box::new(lamzu_maya::LamzuMayaDriver),
        Box::new(steelseries::SteelSeriesDriver),
        Box::new(attackshark::AttackSharkDriver),
        Box::new(glorious::GloriousDriver),
        Box::new(hyperx::HyperXDriver),
        Box::new(sensei_ten::SenseiTenDriver),
        Box::new(pulsar_x3::PulsarX3Driver),
    ]
}

pub fn probe(api: &HidApi, log: &mut Vec<String>) -> Option<MouseInfoDto> {
    for driver in registered() {
        if !driver.present(api) {
            continue;
        }
        if let Some(info) = driver.probe(api, log) {
            return Some(info);
        }
    }

    probe_unknown_mouse(api, log)
}

fn probe_unknown_mouse(api: &HidApi, log: &mut Vec<String>) -> Option<MouseInfoDto> {
    let mut first: Option<MouseInfoDto> = None;

    for device in api.device_list() {
        if device.usage_page() != GENERIC_DESKTOP_USAGE_PAGE || device.usage() != MOUSE_USAGE {
            continue;
        }

        let name = device
            .product_string()
            .unwrap_or("Unknown HID mouse")
            .to_string();

        let info = describe_unknown(device.vendor_id(), device.product_id(), name, log);
        let recognised = matches!(info.support, SupportLevel::Recognized);

        if recognised {
            return Some(info);
        }

        if first.is_none() {
            first = Some(info);
        }
    }

    first
}

/// Builds the answer for a mouse that has no driver: names it from the built-in list when it is known.
fn describe_unknown(vid: u16, pid: u16, usb_name: String, log: &mut Vec<String>) -> MouseInfoDto {
    let mut info = MouseInfoDto {
        connected: true,
        name: usb_name.clone(),
        connection: "unknown".to_string(),
        battery: None,
        charging: false,
        dpi: 0,
        dpi_min: 0,
        dpi_max: 0,
        dpi_step: 50,
        dpi_levels: Vec::new(),
        polling_rate: 0,
        polling_options: Vec::new(),
        polling_supported: Vec::new(),
        brand: "Unknown".to_string(),
        capabilities: MouseCapabilitiesDto::unsupported(),
        support: SupportLevel::Unknown,
    };

    match database::recognise(vid, pid) {
        Some(Recognised::Model { entry, kind }) => {
            log.push(format!(
                "Mouse {:04X}:{:04X} recognised as {} {} (no driver for this brand yet).",
                vid, pid, entry.brand, entry.model
            ));

            info.brand = entry.brand.to_string();
            info.name = format!("{} {}", entry.brand, entry.model);
            info.connection = match kind {
                0 => "wired",
                1 => "wireless",
                2 => "bluetooth",
                _ => "unknown",
            }
            .to_string();

            if entry.verified && entry.dpi_max > 0 {
                info.dpi_min = entry.dpi_min;
                info.dpi_max = entry.dpi_max;
            }

            info.support = SupportLevel::Recognized;
        }
        Some(Recognised::Brand(brand)) => {
            log.push(format!(
                "Mouse {:04X}:{:04X} '{}' recognised as brand {} (model not in the list).",
                vid, pid, usb_name, brand
            ));

            info.brand = brand.to_string();
            info.support = SupportLevel::Recognized;
        }
        None => {
            log.push(format!(
                "Mouse model '{}' ({:04X}:{:04X}) is not supported yet.",
                usb_name, vid, pid
            ));
        }
    }

    info
}