//! Helpers shared by the experimental read-only drivers (Pulsar/Lamzu, VAXEE, Lamzu Maya X).
//!
//! These drivers only ASK the mouse for its state. They never write a setting, so a wrong byte
//! cannot change anything on the mouse. They are marked "expected" (not tested on this model).

use crate::mouse::{MouseCapabilitiesDto, MouseInfoDto, SupportLevel};

pub const POLLING_OPTIONS: [u32; 7] = [125, 250, 500, 1000, 2000, 4000, 8000];

pub const READ_ONLY: &str = "This mouse is read-only for now: Musubi Hub shows its state but does not change it";

/// What a read-only driver managed to read. Anything it could not read stays `None`.
pub struct Reading {
    pub brand: &'static str,
    pub name: String,
    /// "wired", "wireless" or "unknown"
    pub connection: &'static str,
    pub battery: Option<u32>,
    pub charging: bool,
    pub dpi: Option<u32>,
    pub polling: Option<u32>,
}

pub fn build(reading: Reading) -> MouseInfoDto {
    let dpi = reading.dpi.unwrap_or(0);
    let polling = reading.polling.unwrap_or(0);

    MouseInfoDto {
        connected: true,
        name: reading.name,
        connection: reading.connection.to_string(),
        battery: reading.battery,
        charging: reading.charging,
        dpi,
        dpi_min: dpi,
        dpi_max: dpi,
        dpi_step: 50,
        dpi_levels: if dpi > 0 { vec![dpi] } else { Vec::new() },
        polling_rate: polling,
        polling_options: POLLING_OPTIONS.to_vec(),
        // Only the current value: the screen shows it and nothing else can be picked.
        polling_supported: if polling > 0 { vec![polling] } else { Vec::new() },
        brand: reading.brand.to_string(),
        capabilities: MouseCapabilitiesDto {
            battery: reading.battery.is_some(),
            dpi_read: dpi > 0,
            dpi_write: false,
            polling_read: polling > 0,
            polling_write: false,
        },
        support: SupportLevel::Expected,
    }
}

/// Approximate battery percentage of a single lithium cell from its voltage in millivolts.
/// It is an estimate (the real curve depends on the cell and the load), so the app shows it as such.
pub fn percent_from_millivolts(mv: u32) -> Option<u32> {
    // (millivolts, percent), highest first
    const CURVE: [(u32, u32); 12] = [
        (4200, 100),
        (4110, 90),
        (4020, 80),
        (3950, 70),
        (3870, 60),
        (3840, 50),
        (3800, 40),
        (3770, 30),
        (3730, 20),
        (3690, 10),
        (3610, 5),
        (3270, 0),
    ];

    if !(3000..=4500).contains(&mv) {
        return None;
    }
    if mv >= CURVE[0].0 {
        return Some(100);
    }

    for pair in CURVE.windows(2) {
        let (high_mv, high_pct) = pair[0];
        let (low_mv, low_pct) = pair[1];
        if mv >= low_mv {
            let span = high_mv - low_mv;
            let into = mv - low_mv;
            return Some(low_pct + (high_pct - low_pct) * into / span);
        }
    }

    Some(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battery_curve_is_monotonic_and_bounded() {
        assert_eq!(percent_from_millivolts(4300), Some(100));
        assert_eq!(percent_from_millivolts(3270), Some(0));
        assert_eq!(percent_from_millivolts(3100), Some(0));
        assert_eq!(percent_from_millivolts(2000), None);
        assert_eq!(percent_from_millivolts(5000), None);

        let mut last = 0;
        for mv in 3000..=4500 {
            let now = percent_from_millivolts(mv).unwrap();
            assert!(now >= last, "not monotonic at {} mV", mv);
            assert!(now <= 100);
            last = now;
        }
    }

    #[test]
    fn nothing_read_gives_no_capabilities() {
        let info = build(Reading {
            brand: "Test",
            name: "T".into(),
            connection: "unknown",
            battery: None,
            charging: false,
            dpi: None,
            polling: None,
        });
        assert!(!info.capabilities.battery);
        assert!(!info.capabilities.dpi_read);
        assert!(!info.capabilities.polling_read);
        assert!(!info.capabilities.dpi_write && !info.capabilities.polling_write);
        assert!(info.polling_supported.is_empty());
    }
}
