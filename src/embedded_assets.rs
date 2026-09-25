use anyhow::{anyhow, Result};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "prompts/"]
pub struct AssetPrompts;

#[derive(Embed)]
#[folder = "utils/"]
#[include = "rmpp/uinput-*"]
pub struct AssetUtils;

/// Directory on the tablet where an edited prompt or tool file can be dropped.
/// A file here wins over the copy bundled into the binary, so the prompt can be
/// tuned with scp and no rebuild.
pub const PROMPT_DIR: &str = "/home/root/ghostwriter/prompts";

// Function to provide access to the uinput module data
pub fn get_uinput_module_data(version: &str) -> Option<Vec<u8>> {
    let target_module_filename = format!("rmpp/uinput-{}.ko", version);
    AssetUtils::get(target_module_filename.as_str()).map(|asset| asset.data.to_vec())
}

/// Load a prompt or tool definition by name or path.
///
/// Lookup order: an existing file at exactly the given path, then
/// `PROMPT_DIR/<name>`, then the copy embedded in the binary at build time.
/// Returns an error instead of panicking when nothing matches, so a typo in the
/// config cannot take the whole service down.
pub fn load_config(filename: &str) -> Result<String> {
    log::debug!("Loading config from {}", filename);

    let direct = std::path::Path::new(filename);
    if direct.is_file() {
        return Ok(std::fs::read_to_string(direct)?);
    }

    let on_device = std::path::Path::new(PROMPT_DIR).join(filename);
    if on_device.is_file() {
        log::debug!("Using on-device copy {}", on_device.display());
        return Ok(std::fs::read_to_string(&on_device)?);
    }

    AssetPrompts::get(filename)
        .map(|asset| String::from_utf8_lossy(asset.data.as_ref()).into_owned())
        .ok_or_else(|| anyhow!("'{}' was not found on disk, in {}, or bundled in the binary", filename, PROMPT_DIR))
}

#[cfg(test)]
mod tests {
    use super::load_config;

    #[test]
    fn bundled_files_load_and_unknown_names_error() {
        let coach = load_config("coach.json").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&coach).unwrap();
        assert!(parsed["prompt"].as_str().unwrap().contains("-- COACH --"));
        assert!(load_config("tool_draw_text.json").is_ok());
        assert!(load_config("does-not-exist.json").is_err());
    }
}
