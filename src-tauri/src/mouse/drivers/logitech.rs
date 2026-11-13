use hidapi::HidApi;

use super::MouseDriver;
use crate::mouse::hidpp::{Hidpp, HidppError};
use crate::mouse::{MouseCapabilitiesDto, MouseInfoDto, SupportLevel};

const LOGITECH_VID: u16 = 0x046D;
const HIDPP_USAGE_PAGE: u16 = 0xFF00;
const HIDPP_LONG_USAGE: u16 = 0x0002;
const DEVICE_INDEXES: [u8; 7] = [0x01, 0xFF, 0x02, 0x03, 0x04, 0x05, 0x06];

// Index in this list = extended report rate code (0 = 8 ms ... 6 = 125 us).
const EXT_RATES_HZ: [u32; 7] = [125, 250, 500, 1000, 2000, 4000, 8000];
const POLLING_OPTIONS: [u32; 7] = [125, 250, 500, 1000, 2000, 4000, 8000];

pub struct LogitechDriver;

impl MouseDriver for LogitechDriver {
    fn id(&self) -> &'static str {
        "logitech"
    }

    fn brand(&self) -> &'static str {
        "Logitech"
    }

    fn present(&self, api: &HidApi) -> bool {
        api.device_list().any(|device| {
            device.vendor_id() == LOGITECH_VID
                && device.usage_page() == HIDPP_USAGE_PAGE
                && device.usage() == HIDPP_LONG_USAGE
        })
    }

    fn probe(&self, api: &HidApi, log: &mut Vec<String>) -> Option<MouseInfoDto> {
        let mut candidates = Vec::new();

        for device in api.device_list() {
            if device.vendor_id() != LOGITECH_VID {
                continue;
            }

            log.push(format!(
                "Interface: pid={:04X} usage_page={:04X} usage={:04X} product={}",
                device.product_id(),
                device.usage_page(),
                device.usage(),
                device.product_string().unwrap_or("?")
            ));

            if device.usage_page() == HIDPP_USAGE_PAGE && device.usage() == HIDPP_LONG_USAGE {
                candidates.push(device);
            }
        }

        if candidates.is_empty() {
            log.push("No Logitech HID++ interface found.".to_string());
            return None;
        }

        for info in candidates {
            let product = info.product_string().unwrap_or("").to_string();
            let via_receiver = product.to_lowercase().contains("receiver");

            let device = match info.open_device(api) {
                Ok(device) => device,
                Err(error) => {
                    log.push(format!(
                        "Cannot open pid={:04X}: {}",
                        info.product_id(),
                        error
                    ));
                    continue;
                }
            };

            let hidpp = Hidpp::new(device);

            for index in DEVICE_INDEXES {
                match hidpp.request(index, 0x00, 1, &[0x00, 0x00, 0x5A], 200) {
                    Ok(payload) => {
                        log.push(format!(
                            "pid={:04X} device {:02X}: HID++ {}.{} answered",
                            info.product_id(),
                            index,
                            payload.first().copied().unwrap_or(0),
                            payload.get(1).copied().unwrap_or(0)
                        ));
                    }
                    Err(error) => {
                        log.push(format!(
                            "pid={:04X} device {:02X}: {}",
                            info.product_id(),
                            index,
                            error
                        ));
                        continue;
                    }
                }

                let connection = if via_receiver || index != 0xFF {
                    "wireless"
                } else {
                    "wired"
                };

                if let Some(mouse) = read_mouse(&hidpp, index, connection, self.brand(), log) {
                    return Some(mouse);
                }
            }
        }

        log.push("No mouse with adjustable DPI answered.".to_string());
        None
    }

    fn set_polling(&self, hz: u32) -> Result<u32, String> {
        set_polling(hz)
    }

    fn set_dpi(&self, value: u32) -> Result<u32, String> {
        set_dpi(value)
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{:02X}", byte))
        .collect::<Vec<_>>()
        .join(" ")
}

fn req(
    hp: &Hidpp,
    dev: u8,
    feature: u8,
    function: u8,
    params: &[u8],
    label: &str,
    log: &mut Vec<String>,
) -> Option<Vec<u8>> {
    match hp.request(dev, feature, function, params, 400) {
        Ok(payload) => {
            log.push(format!("  {} -> {}", label, hex(&payload)));
            Some(payload)
        }
        Err(error) => {
            log.push(format!("  {} failed: {}", label, error));
            None
        }
    }
}

fn find_feature(hp: &Hidpp, dev: u8, id: u16, log: &mut Vec<String>) -> Option<u8> {
    match hp.feature_index(dev, id) {
        Ok(Some(index)) => {
            log.push(format!("  feature {:04X} at index {}", id, index));
            Some(index)
        }
        Ok(None) => {
            log.push(format!("  feature {:04X} not present", id));
            None
        }
        Err(HidppError::Timeout) => {
            log.push(format!("  feature {:04X}: timeout", id));
            None
        }
        Err(error) => {
            log.push(format!("  feature {:04X}: {}", id, error));
            None
        }
    }
}

fn list_features(hp: &Hidpp, dev: u8, log: &mut Vec<String>) {
    let Some(set) = find_feature(hp, dev, 0x0001, log) else {
        return;
    };

    let Ok(count) = hp.request(dev, set, 0, &[], 400) else {
        return;
    };

    let total = count.first().copied().unwrap_or(0).min(60);
    let mut ids = Vec::new();

    for index in 1..=total {
        if let Ok(payload) = hp.request(dev, set, 1, &[index], 400) {
            if payload.len() >= 2 {
                ids.push(format!(
                    "{}:{:02X}{:02X}",
                    index, payload[0], payload[1]
                ));
            }
        }
    }

    log.push(format!("  features (index:id) {}", ids.join(" ")));
}

struct DpiData {
    current: u32,
    min: u32,
    max: u32,
    step: u32,
    levels: Vec<u32>,
    writable: bool,
}

const CLASSIC_DPI_FEATURE: u16 = 0x2201;
const EXT_DPI_FEATURE: u16 = 0x2202;
// Feature 0x2202 functions (sensor 0). Layout from the open HID++ documentation (0BSD crate
// "openlogi-hidpp"): 2 = DPI ranges (paged), 3 = DPI list, 5 = get DPI, 6 = set DPI.
const EXT_FN_RANGES: u8 = 2;
const EXT_FN_LIST: u8 = 3;
const EXT_FN_GET: u8 = 5;
const EXT_FN_SET: u8 = 6;

fn parse_classic_dpi_range(list_payload: &[u8]) -> Option<(u32, u32, u32)> {
    parse_dpi_list(list_payload.get(1..)?)
}

fn decode_dpi_current(payload: &[u8]) -> Option<u32> {
    let high = *payload.get(1)?;
    let low = *payload.get(2)?;
    Some(u32::from(u16::from_be_bytes([high, low])))
}

fn encode_classic_dpi_set(value: u32) -> [u8; 3] {
    let clamped = value.min(u32::from(u16::MAX)) as u16;
    let bytes = clamped.to_be_bytes();
    [0, bytes[0], bytes[1]]
}

// Reads DPI. Tries the classic feature (0x2201), then the extended one (0x2202).
fn read_dpi(hp: &Hidpp, dev: u8, log: &mut Vec<String>) -> Option<DpiData> {
    let sensor = [0u8];

    if let Some(feature) = find_feature(hp, dev, CLASSIC_DPI_FEATURE, log) {
        let list = req(hp, dev, feature, 1, &sensor, "DPI list", log)?;
        let (min, max, step) = parse_classic_dpi_range(&list)?;
        let current = req(hp, dev, feature, 2, &sensor, "current DPI", log)?;
        let dpi = decode_dpi_current(&current)?;

        return Some(DpiData {
            current: dpi,
            min,
            max,
            step,
            levels: Vec::new(),
            writable: dpi > 0,
        });
    }

    if let Some(feature) = find_feature(hp, dev, EXT_DPI_FEATURE, log) {
        let params = [0u8, 0, 0];

        // Read-only calls. Everything is logged raw so the layout can be checked.
        let _ = req(hp, dev, feature, 0, &params, "ext DPI fn 0 (sensor count)", log);
        let _ = req(hp, dev, feature, 1, &params, "ext DPI fn 1 (capabilities)", log);
        let ranges = read_ext_ranges(hp, dev, feature, log);
        let levels = req(hp, dev, feature, EXT_FN_LIST, &params, "ext DPI fn 3 (levels)", log);
        let _ = req(hp, dev, feature, 4, &params, "ext DPI fn 4 (lod)", log);
        let current = req(
            hp,
            dev,
            feature,
            EXT_FN_GET,
            &params,
            "ext DPI fn 5 (current)",
            log,
        );

        if let Some(data) = parse_ext_dpi(ranges.as_deref(), levels.as_deref(), current.as_deref())
        {
            log.push(format!(
                "  extended DPI decoded: current {}, range {}-{}, levels {:?}",
                data.current, data.min, data.max, data.levels
            ));
            return Some(data);
        }

        log.push("  extended DPI could not be decoded".to_string());
    }

    None
}

/// Reads the DPI range list of the extended feature. It comes in pages of 13 bytes and ends
/// with a 0x0000 entry. Returns the bytes of all pages joined.
fn read_ext_ranges(hp: &Hidpp, dev: u8, feature: u8, log: &mut Vec<String>) -> Option<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();

    for page in 0..16u8 {
        let payload = req(
            hp,
            dev,
            feature,
            EXT_FN_RANGES,
            &[0, 0, page],
            &format!("ext DPI fn 2 (ranges, page {})", page),
            log,
        )?;

        bytes.extend_from_slice(payload.get(3..)?);

        // an aligned 0x0000 pair ends the list
        if bytes.chunks_exact(2).any(|pair| pair == [0, 0]) {
            return Some(bytes);
        }
    }

    Some(bytes)
}

/// Decodes the extended DPI feature (0x2202) from what the mouse answered.
/// `ranges` = all pages of function 2, `levels` = function 3, `current` = function 5.
fn parse_ext_dpi(
    ranges: Option<&[u8]>,
    levels: Option<&[u8]>,
    current: Option<&[u8]>,
) -> Option<DpiData> {
    let mut list = Vec::new();

    if let Some(payload) = levels {
        // [sensor, direction, then big-endian DPI values]
        for chunk in payload.get(2..).unwrap_or(&[]).chunks_exact(2) {
            let value = u32::from(u16::from_be_bytes([chunk[0], chunk[1]]));

            if value == 0 || value & 0xE000 == 0xE000 || !(50..=100_000).contains(&value) {
                break;
            }

            list.push(value);

            if list.len() >= 8 {
                break;
            }
        }
    }

    // [sensor, DPI X (2 bytes), default DPI X (2), DPI Y (2), default DPI Y (2), lod]
    let now = current
        .and_then(|payload| payload.get(1..3))
        .map(|bytes| u32::from(u16::from_be_bytes([bytes[0], bytes[1]])))
        .filter(|value| (50..=100_000).contains(value))
        .unwrap_or(0);

    if now == 0 && list.is_empty() {
        return None;
    }

    let (range_min, range_max) = ranges
        .and_then(parse_dpi_list_bytes)
        .map(|(min, max, _)| (min, max))
        .unwrap_or((0, 0));

    let min = if range_min > 0 {
        range_min
    } else {
        list.iter().copied().min().unwrap_or(0)
    };

    Some(DpiData {
        current: now,
        min,
        max: range_max,
        step: 50,
        levels: list,
        writable: now > 0,
    })
}

fn parse_dpi_list_bytes(bytes: &[u8]) -> Option<(u32, u32, u32)> {
    // the first 0xE000 entries are step markers, so reuse the classic list parser
    parse_dpi_list(bytes)
}

fn read_mouse(
    hp: &Hidpp,
    dev: u8,
    connection: &str,
    brand: &str,
    log: &mut Vec<String>,
) -> Option<MouseInfoDto> {
    list_features(hp, dev, log);

    let name = read_name(hp, dev, log);
    let dpi_data = read_dpi(hp, dev, log);
    let (battery, charging) = read_battery(hp, dev, log);
    let (polling_rate, polling_supported) = read_polling(hp, dev, connection, log);

    if name.is_none() && dpi_data.is_none() && battery.is_none() {
        return None;
    }

    let name = name.unwrap_or_else(|| "Logitech mouse".to_string());
    let (dpi, dpi_min, mut dpi_max, dpi_step, dpi_levels) = match &dpi_data {
        Some(data) => (
            data.current,
            data.min,
            data.max,
            data.step,
            data.levels.clone(),
        ),
        None => (0, 0, 0, 50, Vec::new()),
    };

    // If the range list could not be read, fall back to the declared maximum
    // (PRO X SUPERLIGHT 2: 100 - 32000 DPI) or the highest preset.
    if dpi_data.is_some() && dpi_max == 0 {
        dpi_max = if name.to_ascii_uppercase().contains("SUPERLIGHT 2") {
            32000
        } else {
            dpi_levels.iter().copied().max().unwrap_or(0)
        };
    }

    let support = if name.to_ascii_uppercase().contains("SUPERLIGHT 2") {
        SupportLevel::Tested
    } else {
        SupportLevel::Expected
    };

    // What the mouse really answered, not what we hope it can do
    let capabilities = MouseCapabilitiesDto {
        battery: battery.is_some(),
        dpi_read: dpi_data.is_some(),
        dpi_write: dpi_data.as_ref().map_or(false, |data| data.writable),
        polling_read: !polling_supported.is_empty(),
        polling_write: !polling_supported.is_empty(),
    };

    Some(MouseInfoDto {
        connected: true,
        name,
        connection: connection.to_string(),
        battery,
        charging,
        dpi,
        dpi_min,
        dpi_max,
        dpi_step,
        dpi_levels,
        polling_rate,
        polling_options: POLLING_OPTIONS.to_vec(),
        polling_supported,
        brand: brand.to_string(),
        capabilities,
        support,
    })
}

fn parse_dpi_list(values: &[u8]) -> Option<(u32, u32, u32)> {
    let mut plain = Vec::new();
    let mut step = None;

    for chunk in values.chunks_exact(2) {
        let value = u16::from_be_bytes([chunk[0], chunk[1]]);

        if value == 0 {
            break;
        }

        if value & 0xE000 == 0xE000 {
            step = Some(u32::from(value & 0x1FFF));
        } else {
            plain.push(u32::from(value));
        }
    }

    let min = *plain.iter().min()?;
    let max = *plain.iter().max()?;

    Some((min, max, step.unwrap_or(50).max(1)))
}

fn read_name(hp: &Hidpp, dev: u8, log: &mut Vec<String>) -> Option<String> {
    let feature = find_feature(hp, dev, 0x0005, log)?;
    let count = req(hp, dev, feature, 0, &[], "name length", log)?;
    let len = usize::from(*count.first()?).min(48);
    let mut bytes = Vec::new();

    while bytes.len() < len {
        let chunk = req(
            hp,
            dev,
            feature,
            1,
            &[bytes.len() as u8],
            "name part",
            log,
        )?;
        let take = (len - bytes.len()).min(chunk.len());

        if take == 0 {
            break;
        }

        bytes.extend_from_slice(&chunk[..take]);
    }

    let text = String::from_utf8_lossy(&bytes)
        .trim_matches(char::from(0))
        .trim()
        .to_string();

    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn read_battery(hp: &Hidpp, dev: u8, log: &mut Vec<String>) -> (Option<u32>, bool) {
    // Unified battery
    if let Some(feature) = find_feature(hp, dev, 0x1004, log) {
        if let Some(payload) = req(
            hp,
            dev,
            feature,
            1,
            &[],
            "battery status (1004)",
            log,
        ) {
            let percent = payload.first().copied().map(u32::from);
            let charging = matches!(payload.get(2), Some(1) | Some(2));
            return (percent, charging);
        }
    }

    // Battery status
    if let Some(feature) = find_feature(hp, dev, 0x1000, log) {
        if let Some(payload) = req(
            hp,
            dev,
            feature,
            0,
            &[],
            "battery status (1000)",
            log,
        ) {
            let percent = payload.first().copied().map(u32::from);
            let charging = matches!(payload.get(2), Some(1) | Some(2) | Some(4));
            return (percent, charging);
        }
    }

    // Battery voltage: only logged, there is no reliable percentage
    if let Some(feature) = find_feature(hp, dev, 0x1001, log) {
        let _ = req(
            hp,
            dev,
            feature,
            0,
            &[],
            "battery voltage (1001)",
            log,
        );
    }

    (None, false)
}

fn decode_classic_polling_mask(mask: u8) -> Vec<u32> {
    let mut supported = Vec::new();
    for bit in 0..8u32 {
        if mask & (1 << bit) != 0 {
            let milliseconds = bit + 1;
            if 1000 % milliseconds == 0 {
                supported.push(1000 / milliseconds);
            }
        }
    }
    supported.sort_unstable();
    supported
}

fn classic_polling_ms(hz: u32) -> Option<u8> {
    if hz == 0 || 1000 % hz != 0 {
        return None;
    }
    let ms = 1000 / hz;
    if (1..=8).contains(&ms) {
        Some(ms as u8)
    } else {
        None
    }
}

fn classic_polling_hz(ms: u8) -> u32 {
    if ms > 0 {
        1000 / u32::from(ms)
    } else {
        0
    }
}

fn read_polling(hp: &Hidpp, dev: u8, connection: &str, log: &mut Vec<String>) -> (u32, Vec<u32>) {
    // Onboard profiles (read only): fn 0 = description, fn 2 = mode (1 = onboard, 2 = host),
    // fn 4 = current profile. Logged first so diagnostics show onboard mode on both 0x8060 and 0x8061 mice.
    if let Some(profiles) = find_feature(hp, dev, 0x8100, log) {
        let _ = req(hp, dev, profiles, 0, &[0, 0, 0], "onboard profiles fn 0 (description)", log);
        let _ = req(hp, dev, profiles, 2, &[0, 0, 0], "onboard profiles fn 2 (mode)", log);
        let _ = req(hp, dev, profiles, 4, &[0, 0, 0], "onboard profiles fn 4 (current profile)", log);
    }

    // Classic report rate: values in milliseconds
    if let Some(feature) = find_feature(hp, dev, 0x8060, log) {
        let list = req(
            hp,
            dev,
            feature,
            0,
            &[],
            "report rate list (8060)",
            log,
        );
        let current = req(
            hp,
            dev,
            feature,
            1,
            &[],
            "report rate (8060)",
            log,
        );

        if let (Some(list), Some(current)) = (list, current) {
            let mask = list.first().copied().unwrap_or(0);
            let supported = decode_classic_polling_mask(mask);
            let milliseconds = current.first().copied().unwrap_or(0);
            let hz = classic_polling_hz(milliseconds);

            return (hz, supported);
        }
    }

    // Extended report rate (high polling rates).
    // Functions: 0 = capabilities, 1 = rates available now, 2 = current rate, 3 = SET the rate.
    // Only the read functions are called here: function 3 changes the mouse.
    // Functions 0 and 2 take the connection type: 0 = cable, 1 = wireless receiver.
    if let Some(feature) = find_feature(hp, dev, 0x8061, log) {
        let conn = ext_rate_connection(connection);

        let _ = req(
            hp,
            dev,
            feature,
            0,
            &[conn, 0, 0],
            "extended report rate fn 0 (capabilities)",
            log,
        );
        let list = req(
            hp,
            dev,
            feature,
            1,
            &[0, 0, 0],
            "extended report rate fn 1 (available now)",
            log,
        );
        let current = req(
            hp,
            dev,
            feature,
            2,
            &[conn, 0, 0],
            "extended report rate fn 2 (current)",
            log,
        );

        if let (Some(list), Some(current)) = (list, current) {
            if let (Some(code), Some(high), Some(low)) =
                (current.first().copied(), list.first(), list.get(1))
            {
                let code = usize::from(code);
                let flags = (u16::from(*high) << 8) | u16::from(*low);

                if code < EXT_RATES_HZ.len() {
                    let supported: Vec<u32> = (0..EXT_RATES_HZ.len())
                        .filter(|index| flags & (1 << index) != 0)
                        .map(|index| EXT_RATES_HZ[index])
                        .collect();

                    log.push("  extended report rate decoded".to_string());

                    return (EXT_RATES_HZ[code], supported);
                }
            }
        }

        log.push("  extended report rate could not be decoded".to_string());
    }

    (0, Vec::new())
}

/// Connection type parameter of the extended report rate feature.
fn ext_rate_connection(connection: &str) -> u8 {
    if connection == "wired" {
        0
    } else {
        1
    }
}

/// Puts the mouse in host mode (onboard profiles feature 0x8100: mode 1 = onboard, 2 = host).
/// Does nothing when the feature is missing or the mouse is already in host mode.
/// Returns true when it had to switch the mode.
fn ensure_host_mode(hp: &Hidpp, index: u8) -> Result<bool, String> {
    let feature = match hp.feature_index(index, 0x8100) {
        Ok(Some(feature)) => feature,
        Ok(None) | Err(_) => return Ok(false),
    };

    let mode = hp
        .request(index, feature, 2, &[0, 0, 0], 400)
        .map_err(|error| format!("Host mode: reading the mode failed: {}", error))?;

    if mode.first().copied() == Some(2) {
        return Ok(false);
    }

    hp.request(index, feature, 1, &[2, 0, 0], 400)
        .map_err(|error| format!("Host mode: the mouse refused to switch: {}", error))?;

    for _ in 0..5 {
        std::thread::sleep(std::time::Duration::from_millis(60));

        if let Ok(now) = hp.request(index, feature, 2, &[0, 0, 0], 400) {
            if now.first().copied() == Some(2) {
                return Ok(true);
            }
        }
    }

    Err("Host mode: the mouse did not switch to host mode".to_string())
}

enum PollingOutcome {
    Applied(u32),
    RetryNext(String),
    Fatal(String),
}

fn set_polling_ext(hp: &Hidpp, index: u8, feature: u8, conn: u8, hz: u32) -> PollingOutcome {
    let Some(code) = EXT_RATES_HZ.iter().position(|&rate| rate == hz) else {
        return PollingOutcome::Fatal(format!("{} Hz is not a valid polling rate", hz));
    };

    // In onboard mode the polling rate belongs to the profile stored in the mouse
    // and the mouse refuses changes (error 0x02). Switch to host mode first.
    let switched = match ensure_host_mode(hp, index) {
        Ok(switched) => switched,
        Err(error) => return PollingOutcome::Fatal(error),
    };

    // Right after the switch the mouse can ignore changes for a moment.
    if switched {
        std::thread::sleep(std::time::Duration::from_millis(400));
    }

    let list = match hp.request(index, feature, 1, &[0, 0, 0], 400) {
        Ok(list) => list,
        Err(error) => {
            return PollingOutcome::Fatal(format!(
                "Polling rate: reading the available rates failed: {}",
                error
            ))
        }
    };

    let (Some(high), Some(low)) = (list.first(), list.get(1)) else {
        return PollingOutcome::Fatal(
            "The mouse returned an invalid polling rate list".to_string(),
        );
    };

    let flags = (u16::from(*high) << 8) | u16::from(*low);

    if flags & (1 << code) == 0 {
        return PollingOutcome::Fatal(format!("{} Hz is not available with this connection", hz));
    }

    let mut last: Option<u8> = None;
    let mut last_error: Option<String> = None;

    // The mouse can need a moment to apply the new rate (a wireless link may even
    // re-synchronise). If it still reports the old one, send the value once more.
    for _send in 0..2 {
        if let Err(error) = hp.request(index, feature, 3, &[code as u8, 0, 0], 400) {
            return PollingOutcome::Fatal(format!(
                "Polling rate: the mouse refused the new value (sent {:02X} 00 00, device index {:02X}, feature index {:02X}): {}",
                code, index, feature, error
            ));
        }

        for attempt in 0..5 {
            std::thread::sleep(std::time::Duration::from_millis(if attempt == 0 {
                60
            } else {
                120
            }));

            let _other = hp
                .request(index, feature, 2, &[1 - conn, 0, 0], 400)
                .ok()
                .and_then(|value| value.first().copied());

            match hp.request(index, feature, 2, &[conn, 0, 0], 400) {
                Ok(now) => {
                    last = now.first().copied();
                    last_error = None;

                    if matches!(last, Some(current) if usize::from(current) == code) {
                        return PollingOutcome::Applied(hz);
                    }
                }
                Err(error) => last_error = Some(error.to_string()),
            }
        }
    }

    if let Some(error) = last_error {
        return PollingOutcome::Fatal(format!(
            "Polling rate: the new value was sent but could not be checked: {}",
            error
        ));
    }

    PollingOutcome::RetryNext(match last {
        Some(current) => format!(
            "The mouse did not keep the new polling rate (asked {} Hz, it reports {} Hz)",
            hz,
            EXT_RATES_HZ
                .get(usize::from(current))
                .copied()
                .unwrap_or(0)
        ),
        None => "The mouse returned an invalid polling rate response".to_string(),
    })
}

fn set_polling_classic(hp: &Hidpp, index: u8, feature: u8, hz: u32) -> PollingOutcome {
    let Some(ms) = classic_polling_ms(hz) else {
        return PollingOutcome::Fatal(format!("{} Hz is not supported by this mouse", hz));
    };

    let switched = match ensure_host_mode(hp, index) {
        Ok(switched) => switched,
        Err(error) => return PollingOutcome::Fatal(error),
    };

    if switched {
        std::thread::sleep(std::time::Duration::from_millis(400));
    }

    let list = match hp.request(index, feature, 0, &[], 400) {
        Ok(list) => list,
        Err(error) => {
            return PollingOutcome::Fatal(format!(
                "Polling rate: reading the available rates failed: {}",
                error
            ))
        }
    };

    let Some(&mask) = list.first() else {
        return PollingOutcome::Fatal(
            "The mouse returned an invalid polling rate list".to_string(),
        );
    };

    if mask & (1 << (ms - 1)) == 0 {
        return PollingOutcome::Fatal(format!("{} Hz is not available with this connection", hz));
    }

    let mut last: Option<u8> = None;
    let mut last_error: Option<String> = None;

    for _send in 0..2 {
        if let Err(error) = hp.request(index, feature, 2, &[ms], 400) {
            return PollingOutcome::Fatal(format!(
                "Polling rate: the mouse refused the new value (sent {:02X}, device index {:02X}, feature index {:02X}): {}",
                ms, index, feature, error
            ));
        }

        for attempt in 0..5 {
            std::thread::sleep(std::time::Duration::from_millis(if attempt == 0 {
                60
            } else {
                120
            }));

            match hp.request(index, feature, 1, &[], 400) {
                Ok(now) => {
                    last = now.first().copied();
                    last_error = None;

                    if last == Some(ms) {
                        return PollingOutcome::Applied(hz);
                    }
                }
                Err(error) => last_error = Some(error.to_string()),
            }
        }
    }

    if let Some(error) = last_error {
        return PollingOutcome::Fatal(format!(
            "Polling rate: the new value was sent but could not be checked: {}",
            error
        ));
    }

    PollingOutcome::RetryNext(match last {
        Some(current_ms) if current_ms > 0 => format!(
            "The mouse did not keep the new polling rate (asked {} Hz, it reports {} Hz)",
            hz,
            classic_polling_hz(current_ms)
        ),
        _ => "The mouse returned an invalid polling rate response".to_string(),
    })
}

/// Sets the polling rate through the extended (0x8061) or classic (0x8060) report rate feature
/// and checks it by reading it back.
fn set_polling(hz: u32) -> Result<u32, String> {
    if !EXT_RATES_HZ.contains(&hz) && classic_polling_ms(hz).is_none() {
        return Err(format!("{} Hz is not a valid polling rate", hz));
    }

    let api = HidApi::new().map_err(|error| error.to_string())?;
    let mut failure: Option<String> = None;

    for info in api.device_list() {
        if info.vendor_id() != LOGITECH_VID
            || info.usage_page() != HIDPP_USAGE_PAGE
            || info.usage() != HIDPP_LONG_USAGE
        {
            continue;
        }

        let device = match info.open_device(&api) {
            Ok(device) => device,
            Err(_) => continue,
        };
        let hp = Hidpp::new(device);
        let via_receiver = info
            .product_string()
            .unwrap_or("")
            .to_lowercase()
            .contains("receiver");

        for index in DEVICE_INDEXES {
            if hp
                .request(index, 0x00, 1, &[0x00, 0x00, 0x5A], 200)
                .is_err()
            {
                continue;
            }

            let outcome = if let Ok(Some(feature)) = hp.feature_index(index, 0x8061) {
                let conn = if via_receiver || index != 0xFF { 1 } else { 0 };
                set_polling_ext(&hp, index, feature, conn, hz)
            } else if let Ok(Some(feature)) = hp.feature_index(index, 0x8060) {
                set_polling_classic(&hp, index, feature, hz)
            } else {
                continue;
            };

            match outcome {
                PollingOutcome::Applied(rate) => return Ok(rate),
                PollingOutcome::Fatal(err) => return Err(err),
                PollingOutcome::RetryNext(err) => {
                    failure = Some(err);
                    continue;
                }
            }
        }
    }

    Err(failure.unwrap_or_else(|| "No mouse with a configurable polling rate found".to_string()))
}

fn verify_dpi_readback(after: &[u8], expected: u32) -> Result<u32, String> {
    let got = decode_dpi_current(after)
        .ok_or_else(|| "The mouse returned an invalid DPI response".to_string())?;

    if got == expected {
        Ok(expected)
    } else {
        Err(format!(
            "The mouse did not accept {} DPI (it reports {})",
            expected, got
        ))
    }
}

fn set_dpi_classic(hp: &Hidpp, index: u8, feature: u8, value: u32) -> Result<u32, String> {
    if let Ok(list) = hp.request(index, feature, 1, &[0], 400) {
        if let Some((min, max, _)) = parse_classic_dpi_range(&list) {
            if value < min || value > max {
                return Err(format!(
                    "{} DPI is outside the range of this mouse ({}-{})",
                    value, min, max
                ));
            }
        }
    }

    let value = value.min(u32::from(u16::MAX));
    let params = encode_classic_dpi_set(value);

    hp.request(index, feature, 3, &params, 400)
        .map_err(|error| error.to_string())?;

    let after = hp
        .request(index, feature, 2, &[0], 400)
        .map_err(|error| error.to_string())?;

    verify_dpi_readback(&after, value)
}

fn set_dpi_ext(hp: &Hidpp, index: u8, feature: u8, value: u32) -> Result<u32, String> {
    // keep the lift-off distance the mouse already uses
    let now = hp
        .request(index, feature, EXT_FN_GET, &[0, 0, 0], 400)
        .map_err(|error| error.to_string())?;
    let lod = now.get(9).copied().unwrap_or(0);

    let mut ignored = Vec::new();
    if let Some((min, max, _)) =
        read_ext_ranges(hp, index, feature, &mut ignored).and_then(|bytes| parse_dpi_list(&bytes))
    {
        if value < min || value > max {
            return Err(format!(
                "{} DPI is outside the range of this mouse ({}-{})",
                value, min, max
            ));
        }
    }
    // The value is written as two bytes, so it cannot pass 65535 (also when no range is known).
    let value = value.min(u32::from(u16::MAX));

    let high = (value >> 8) as u8;
    let low = (value & 0xFF) as u8;

    hp.request(
        index,
        feature,
        EXT_FN_SET,
        &[0, high, low, high, low, lod],
        400,
    )
    .map_err(|error| error.to_string())?;

    let after = hp
        .request(index, feature, EXT_FN_GET, &[0, 0, 0], 400)
        .map_err(|error| error.to_string())?;

    verify_dpi_readback(&after, value)
}

/// Sets the DPI through the classic (0x2201) or extended (0x2202) DPI feature and checks it by reading it back.
fn set_dpi(value: u32) -> Result<u32, String> {
    if !(50..=100_000).contains(&value) {
        return Err(format!("{} DPI is not a valid value", value));
    }

    let api = HidApi::new().map_err(|error| error.to_string())?;

    for info in api.device_list() {
        if info.vendor_id() != LOGITECH_VID
            || info.usage_page() != HIDPP_USAGE_PAGE
            || info.usage() != HIDPP_LONG_USAGE
        {
            continue;
        }

        let device = match info.open_device(&api) {
            Ok(device) => device,
            Err(_) => continue,
        };
        let hp = Hidpp::new(device);

        for index in DEVICE_INDEXES {
            if hp
                .request(index, 0x00, 1, &[0x00, 0x00, 0x5A], 200)
                .is_err()
            {
                continue;
            }

            if let Ok(Some(feature)) = hp.feature_index(index, CLASSIC_DPI_FEATURE) {
                let switched = ensure_host_mode(&hp, index)?;
                if switched {
                    std::thread::sleep(std::time::Duration::from_millis(400));
                }
                return set_dpi_classic(&hp, index, feature, value);
            }

            if let Ok(Some(feature)) = hp.feature_index(index, EXT_DPI_FEATURE) {
                let switched = ensure_host_mode(&hp, index)?;
                if switched {
                    std::thread::sleep(std::time::Duration::from_millis(400));
                }
                return set_dpi_ext(&hp, index, feature, value);
            }
        }
    }

    Err("No mouse with adjustable DPI found".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real answers of a PRO X SUPERLIGHT 2 SE (payload bytes after the 4 header bytes)
    const LEVELS: [u8; 16] = [
        0x00, 0x00, 0x03, 0x20, 0x04, 0xB0, 0x06, 0x40, 0x09, 0x60, 0x0C, 0x80, 0x00, 0x00, 0x00,
        0x00,
    ];

    // page 0 (3 header bytes + 13 data bytes) and a made-up page 1 that ends the list
    fn ranges() -> Vec<u8> {
        let mut bytes = vec![
            0x00, 0x64, 0xE0, 0x01, 0x00, 0xC8, 0xE0, 0x02, 0x01, 0xF4, 0xE0, 0x05, 0x03,
        ];
        bytes.extend_from_slice(&[0xE8, 0x7D, 0x00, 0x00, 0x00]);
        bytes
    }

    fn current(dpi: u16) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[1] = (dpi >> 8) as u8;
        bytes[2] = dpi as u8;
        bytes[9] = 0x02;
        bytes
    }

    #[test]
    fn decodes_levels_and_current() {
        let data = parse_ext_dpi(Some(&ranges()), Some(&LEVELS), Some(&current(1600))).unwrap();
        assert_eq!(data.levels, vec![800, 1200, 1600, 2400, 3200]);
        assert_eq!(data.current, 1600);
        assert_eq!(data.min, 100);
        assert_eq!(data.max, 32000);
        assert!(data.writable);
    }

    #[test]
    fn current_unknown_is_not_writable() {
        let data = parse_ext_dpi(None, Some(&LEVELS), Some(&[0u8; 16])).unwrap();
        assert_eq!(data.current, 0);
        assert!(!data.writable);
    }

    #[test]
    fn rejects_empty_answers() {
        assert!(parse_ext_dpi(None, Some(&[0u8; 16]), Some(&[0u8; 16])).is_none());
    }

    #[test]
    fn decodes_classic_0x2201_g502x_dpi_range_and_encoding() {
        // Real G502 X 0x2201 fn 1 response prefix: sensor 0, min 100 (00 64), step 50 (E0 32), max 25600 (64 00)
        let g502x_fn1 = [0x00, 0x00, 0x64, 0xE0, 0x32, 0x64, 0x00, 0x00, 0x00];
        assert_eq!(parse_classic_dpi_range(&g502x_fn1), Some((100, 25600, 50)));

        assert_eq!(encode_classic_dpi_set(800), [0x00, 0x03, 0x20]);
        assert_eq!(encode_classic_dpi_set(1600), [0x00, 0x06, 0x40]);
        assert_eq!(encode_classic_dpi_set(25600), [0x00, 0x64, 0x00]);
        assert_eq!(encode_classic_dpi_set(80_000), [0x00, 0xFF, 0xFF]);

        let fn2_resp = [0x00, 0x06, 0x40, 0x03, 0x20];
        assert_eq!(decode_dpi_current(&fn2_resp), Some(1600));
        assert!(verify_dpi_readback(&fn2_resp, 1600).is_ok());
        assert!(verify_dpi_readback(&fn2_resp, 800).is_err());
    }

    #[test]
    fn decodes_classic_0x8060_mask_and_hz_ms_mapping() {
        assert_eq!(decode_classic_polling_mask(0x8B), vec![125, 250, 500, 1000]);

        assert_eq!(classic_polling_ms(1000), Some(1));
        assert_eq!(classic_polling_ms(500), Some(2));
        assert_eq!(classic_polling_ms(250), Some(4));
        assert_eq!(classic_polling_ms(125), Some(8));
        assert_eq!(classic_polling_ms(0), None);
        assert_eq!(classic_polling_ms(333), None);
        assert_eq!(classic_polling_ms(2000), None);

        assert_eq!(classic_polling_hz(1), 1000);
        assert_eq!(classic_polling_hz(2), 500);
        assert_eq!(classic_polling_hz(4), 250);
        assert_eq!(classic_polling_hz(8), 125);
        assert_eq!(classic_polling_hz(0), 0);
    }
}