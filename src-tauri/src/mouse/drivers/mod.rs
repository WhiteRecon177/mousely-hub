use std::collections::HashSet;

use hidapi::HidApi;

use super::database::{self, Recognised};
use super::{MouseCapabilitiesDto, MouseInfoDto, SupportLevel, ValueSource};

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

pub fn slugify(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_dash = false;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

pub fn canonical_id(prefix: &str, model_name: &str) -> String {
    let slug = slugify(model_name);
    if slug.is_empty() {
        prefix.to_string()
    } else {
        format!("{}:{}", prefix, slug)
    }
}

pub trait MouseDriver: Send + Sync {
    fn id(&self) -> &'static str;
    fn brand(&self) -> &'static str;
    /// Quick check: is a device of this brand plugged in? No reading, no waiting.
    fn present(&self, api: &HidApi) -> bool;
    /// Full reading of the mouse.
    fn probe(&self, api: &HidApi, log: &mut Vec<String>) -> Option<MouseInfoDto>;
    /// Full reading of all connected mice handled by this driver.
    fn probe_all(&self, api: &HidApi, log: &mut Vec<String>) -> Vec<MouseInfoDto> {
        self.probe(api, log).into_iter().collect()
    }
    /// USB `(vid, pid)` pairs claimed by this driver's probed devices so fallback HID
    /// enumeration does not emit duplicate `Recognized` entries for the same hardware.
    fn claimed_usb_ids(&self, _api: &HidApi, _probed: &[MouseInfoDto]) -> Vec<(u16, u16)> {
        Vec::new()
    }
    fn set_polling(&self, hz: u32) -> Result<u32, String>;
    fn set_dpi(&self, value: u32) -> Result<u32, String>;
    /// Writes polling rate to a specific canonical device `id`.
    fn set_polling_for(&self, _device_id: &str, hz: u32) -> Result<u32, String> {
        self.set_polling(hz)
    }
    /// Writes DPI to a specific canonical device `id`.
    fn set_dpi_for(&self, _device_id: &str, value: u32) -> Result<u32, String> {
        self.set_dpi(value)
    }
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

fn device_richness(info: &MouseInfoDto) -> u8 {
    let mut score = 0u8;
    if info.connection == "wired" {
        score += 4;
    }
    if info.battery.is_some() {
        score += 2;
    }
    if info.capabilities.dpi_read {
        score += 1;
    }
    if info.capabilities.polling_read {
        score += 1;
    }
    score
}

pub fn merge_device(list: &mut Vec<MouseInfoDto>, candidate: MouseInfoDto) {
    if let Some(existing) = list.iter_mut().find(|d| d.id == candidate.id) {
        if device_richness(&candidate) > device_richness(existing) {
            *existing = candidate;
        }
    } else {
        list.push(candidate);
    }
}

/// Combines driver-probed mice with HID-enumerated fallback mice:
/// - Deduplicates by canonical `id`.
/// - Keeps all driver and `Recognized` mice.
/// - Excludes generic `Unknown` HID mice whenever >= 1 driver or `Recognized` mouse is present.
pub fn filter_and_merge_devices(
    driver_devices: Vec<MouseInfoDto>,
    fallback_devices: Vec<MouseInfoDto>,
) -> Vec<MouseInfoDto> {
    let mut result = Vec::new();
    for dev in driver_devices {
        merge_device(&mut result, dev);
    }

    let mut recognized = Vec::new();
    let mut unknown = Vec::new();
    for dev in fallback_devices {
        if result.iter().any(|existing| existing.id == dev.id) {
            continue;
        }
        match dev.support {
            SupportLevel::Recognized => merge_device(&mut recognized, dev),
            SupportLevel::Unknown => merge_device(&mut unknown, dev),
            _ => merge_device(&mut result, dev),
        }
    }

    result.extend(recognized);
    if result.is_empty() {
        result.extend(unknown);
    }
    result
}

pub fn probe_all(api: &HidApi, log: &mut Vec<String>) -> Vec<MouseInfoDto> {
    let mut driver_devices = Vec::new();
    let mut claimed: HashSet<(u16, u16)> = HashSet::new();

    for driver in registered() {
        if !driver.present(api) {
            continue;
        }
        let probed = driver.probe_all(api, log);
        for usb_id in driver.claimed_usb_ids(api, &probed) {
            claimed.insert(usb_id);
        }
        for info in probed {
            merge_device(&mut driver_devices, info);
        }
    }

    let fallback_devices = probe_fallback_mice(api, &claimed, log);
    filter_and_merge_devices(driver_devices, fallback_devices)
}

#[allow(dead_code)]
pub fn probe_active(
    api: &HidApi,
    log: &mut Vec<String>,
) -> (Option<&'static str>, Option<MouseInfoDto>) {
    let mut all = probe_all(api, log);
    if all.is_empty() {
        return (None, None);
    }
    let first = all.remove(0);
    let driver_id = first
        .id
        .split(':')
        .next()
        .and_then(|prefix| registered().into_iter().find(|d| d.id() == prefix).map(|d| d.id()));
    (driver_id, Some(first))
}

#[allow(dead_code)]
pub fn probe(api: &HidApi, log: &mut Vec<String>) -> Option<MouseInfoDto> {
    probe_active(api, log).1
}

fn probe_fallback_mice(
    api: &HidApi,
    claimed: &HashSet<(u16, u16)>,
    log: &mut Vec<String>,
) -> Vec<MouseInfoDto> {
    let mut seen_usb: HashSet<(u16, u16)> = HashSet::new();
    let mut out = Vec::new();

    for device in api.device_list() {
        if device.usage_page() != GENERIC_DESKTOP_USAGE_PAGE || device.usage() != MOUSE_USAGE {
            continue;
        }
        let key = (device.vendor_id(), device.product_id());
        if claimed.contains(&key) || !seen_usb.insert(key) {
            continue;
        }

        let name = device
            .product_string()
            .unwrap_or("Unknown HID mouse")
            .to_string();

        out.push(describe_unknown(device.vendor_id(), device.product_id(), name, log));
    }

    out
}

/// Builds the answer for a mouse that has no driver: names it from the built-in list when it is known.
fn describe_unknown(vid: u16, pid: u16, usb_name: String, log: &mut Vec<String>) -> MouseInfoDto {
    let mut info = MouseInfoDto {
        id: format!("unknown:{:04x}:{:04x}", vid, pid),
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
        dpi_source: ValueSource::Unset,
        polling_rate: 0,
        polling_options: Vec::new(),
        polling_supported: Vec::new(),
        polling_source: ValueSource::Unset,
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
            info.id = canonical_id("recognized", &info.name);
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
            info.id = format!("recognized:{}:{:04x}:{:04x}", slugify(brand), vid, pid);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_mouse(id: &str, brand: &str, connection: &str, support: SupportLevel) -> MouseInfoDto {
        MouseInfoDto {
            id: id.to_string(),
            connected: true,
            name: id.to_string(),
            connection: connection.to_string(),
            battery: None,
            charging: false,
            dpi: 0,
            dpi_min: 100,
            dpi_max: 16000,
            dpi_step: 50,
            dpi_levels: Vec::new(),
            dpi_source: ValueSource::Unset,
            polling_rate: 0,
            polling_options: Vec::new(),
            polling_supported: Vec::new(),
            polling_source: ValueSource::Unset,
            brand: brand.to_string(),
            capabilities: MouseCapabilitiesDto::unsupported(),
            support,
        }
    }

    #[test]
    fn canonical_id_unifies_wired_and_wireless_models() {
        assert_eq!(
            canonical_id("steelseries-aerox", "SteelSeries Aerox 3 Wireless"),
            "steelseries-aerox:steelseries-aerox-3-wireless"
        );
        assert_eq!(
            canonical_id("glorious-model-o", "Glorious Model O- Wireless"),
            "glorious-model-o:glorious-model-o-wireless"
        );
        assert_eq!(
            canonical_id("logitech", "PRO X SUPERLIGHT 2"),
            "logitech:pro-x-superlight-2"
        );
    }

    #[test]
    fn filter_excludes_unknown_when_driver_or_recognized_present() {
        let driver = sample_mouse("logitech:pro-x", "Logitech", "wireless", SupportLevel::Tested);
        let recognized = sample_mouse("recognized:razer-viper", "Razer", "wired", SupportLevel::Recognized);
        let unknown = sample_mouse("unknown:1234:5678", "Unknown", "unknown", SupportLevel::Unknown);

        // Driver + Unknown -> Unknown excluded
        let res = filter_and_merge_devices(vec![driver.clone()], vec![unknown.clone()]);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].id, driver.id);

        // Driver + Recognized + Unknown -> Driver + Recognized kept, Unknown excluded
        let res = filter_and_merge_devices(
            vec![driver.clone()],
            vec![recognized.clone(), unknown.clone()],
        );
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].id, driver.id);
        assert_eq!(res[1].id, recognized.id);

        // Recognized only + Unknown -> Recognized kept, Unknown excluded
        let res = filter_and_merge_devices(Vec::new(), vec![recognized.clone(), unknown.clone()]);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].id, recognized.id);

        // Only Unknown -> Unknown kept
        let res = filter_and_merge_devices(Vec::new(), vec![unknown.clone()]);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].id, unknown.id);
    }

    #[test]
    fn filter_deduplicates_same_canonical_id_preferring_richer_connection() {
        let mut wireless = sample_mouse(
            "steelseries-aerox:steelseries-aerox-3-wireless",
            "SteelSeries",
            "wireless",
            SupportLevel::Expected,
        );
        wireless.battery = Some(80);
        let wired = sample_mouse(
            "steelseries-aerox:steelseries-aerox-3-wireless",
            "SteelSeries",
            "wired",
            SupportLevel::Expected,
        );

        let res = filter_and_merge_devices(vec![wireless, wired], Vec::new());
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].connection, "wired");
    }
}