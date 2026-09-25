use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DeviceModel {
    Remarkable2,
    RemarkablePaperPro,
    RemarkablePaperPure,
    Unknown,
}

impl DeviceModel {
    pub fn from_string(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "rm2" | "remarkable2" | "remarkable-2" => Ok(DeviceModel::Remarkable2),
            "rmpp" | "remarkable-paper-pro" | "remarkablepaperpro" | "paperpro" => Ok(DeviceModel::RemarkablePaperPro),
            "rmpure" | "tatsu" | "paperpure" => Ok(DeviceModel::RemarkablePaperPure),
            _ => Err(anyhow::anyhow!("Invalid device model: {}. Use 'rm2' or 'rmpp'", s)),
        }
    }

    pub fn detect() -> Self {
        if Path::new("/etc/hwrevision").exists() {
            if let Ok(hwrev) = std::fs::read_to_string("/etc/hwrevision") {
                if hwrev.contains("ferrari 1.0") {
                    return DeviceModel::RemarkablePaperPro;
                }
                if hwrev.contains("reMarkable2 1.0") {
                    return DeviceModel::Remarkable2;
                }
                if hwrev.contains("tatsu 1.0") {
                    return DeviceModel::RemarkablePaperPure;
                }
            }
        }

        // Older firmware has no /etc/hwrevision; the device tree still names the model.
        for path in ["/sys/devices/soc0/machine", "/proc/device-tree/model"] {
            if let Ok(model) = std::fs::read_to_string(path) {
                let model = model.to_lowercase();
                if model.contains("remarkable 2") {
                    return DeviceModel::Remarkable2;
                }
                if model.contains("ferrari") {
                    return DeviceModel::RemarkablePaperPro;
                }
                if model.contains("tatsu") {
                    return DeviceModel::RemarkablePaperPure;
                }
            }
        }

        // Nothing matched :shrug:
        DeviceModel::Unknown
    }

    pub fn name(&self) -> &str {
        match self {
            DeviceModel::Remarkable2 => "Remarkable2",
            DeviceModel::RemarkablePaperPro => "RemarkablePaperPro",
            DeviceModel::RemarkablePaperPure => "RemarkablePaperPure",
            DeviceModel::Unknown => "Unknown",
        }
    }
}
