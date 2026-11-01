pub mod commands;
pub mod database;
mod database_data;
pub mod drivers;
pub mod hidpp;

use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MouseCapabilitiesDto {
    pub battery: bool,
    pub dpi_read: bool,
    pub dpi_write: bool,
    pub polling_read: bool,
    pub polling_write: bool,
}

impl MouseCapabilitiesDto {
    pub fn unsupported() -> Self {
        Self {
            battery: false,
            dpi_read: false,
            dpi_write: false,
            polling_read: false,
            polling_write: false,
        }
    }
}

#[derive(Serialize, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SupportLevel {
    Tested,
    Expected,
    /// Brand/model identified from the built-in list, but no driver can configure it yet.
    Recognized,
    Unknown,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MouseInfoDto {
    pub connected: bool,
    pub name: String,
    pub connection: String,
    pub battery: Option<u32>,
    pub charging: bool,
    pub dpi: u32,
    pub dpi_min: u32,
    pub dpi_max: u32,
    pub dpi_step: u32,
    pub dpi_levels: Vec<u32>,
    pub polling_rate: u32,
    pub polling_options: Vec<u32>,
    pub polling_supported: Vec<u32>,
    pub brand: String,
    pub capabilities: MouseCapabilitiesDto,
    pub support: SupportLevel,
}

#[derive(Serialize, Clone, Debug)]
pub struct ProbeResult {
    pub info: MouseInfoDto,
    pub diagnostics: Vec<String>,
}