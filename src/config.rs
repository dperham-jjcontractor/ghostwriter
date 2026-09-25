use crate::device::DeviceModel;
use crate::touch::TriggerCorner;
use anyhow::Result;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

/// The one place the default model is spelled out.
pub const DEFAULT_MODEL: &str = "gpt-6-sol";
/// The one place the default prompt is spelled out.
pub const DEFAULT_PROMPT: &str = "coach.json";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct Config {
    // Direct mapping to CLI args - no arbitrary grouping
    pub engine: Option<String>,
    pub engine_base_url: Option<String>,
    pub engine_api_key: Option<String>,
    pub model: String,
    pub prompt: String,
    pub no_submit: bool,
    pub no_draw: bool,
    pub no_svg: bool,
    pub no_keyboard: bool,
    pub no_draw_progress: bool,
    pub input_png: Option<String>,
    pub output_file: Option<String>,
    pub model_output_file: Option<String>,
    pub save_screenshot: Option<String>,
    pub save_bitmap: Option<String>,
    pub no_loop: bool,
    pub no_trigger: bool,
    pub apply_segmentation: bool,
    pub web_search: bool,
    pub thinking: bool,
    pub thinking_tokens: u32,
    pub log_level: String,
    pub trigger_corner: String,
    pub web_server: bool,
    pub web_port: u16,
    /// OpenAI reasoning_effort ("none", "low", "medium", "high"; empty = omit).
    /// gpt-6 models only allow tool calls on chat completions with "none",
    /// which also gives the fastest replies.
    pub openai_reasoning_effort: String,
    /// Turn to the next page (a new one at the end of the notebook) before
    /// typing the reply, so it never lands on top of the handwriting.
    pub reply_on_new_page: bool,
    /// Before drawing, tap the pen palette to select the fineliner (positions
    /// verified on the Paper Pro only; off by default so nothing random is
    /// tapped on other models; drawings then use whichever pen is selected).
    pub select_pen_before_drawing: bool,
    // Simulation/test mode options
    pub test_mode: Option<String>,
    pub test_device_model: Option<DeviceModel>,
    pub test_touch_events_file: Option<String>,
    pub test_screenshot_dir: Option<String>,
    pub test_auto_trigger_delay: Option<u32>, // seconds
    pub test_interaction_log: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            engine: None,
            engine_base_url: None,
            engine_api_key: None,
            model: DEFAULT_MODEL.to_string(),
            prompt: DEFAULT_PROMPT.to_string(),
            no_submit: false,
            no_draw: false,
            // Text only: the assistant types, it never draws on the page.
            no_svg: true,
            no_keyboard: false,
            no_draw_progress: false,
            input_png: None,
            output_file: None,
            model_output_file: None,
            save_screenshot: None,
            save_bitmap: None,
            no_loop: false,
            no_trigger: false,
            apply_segmentation: false,
            web_search: false,
            thinking: false,
            thinking_tokens: 5000,
            log_level: "info".to_string(),
            trigger_corner: "UR".to_string(),
            web_server: false,
            web_port: 8080,
            openai_reasoning_effort: "none".to_string(),
            reply_on_new_page: true,
            select_pen_before_drawing: false,
            // Simulation/test mode defaults
            test_mode: None,
            test_device_model: None,
            test_touch_events_file: None,
            test_screenshot_dir: None,
            test_auto_trigger_delay: None,
            test_interaction_log: None,
        }
    }
}

impl Config {
    /// Load configuration using figment (defaults -> TOML file -> env -> CLI precedence).
    ///
    /// The CLI layer must only contain values the user actually passed: figment's
    /// merge lets a serialized `None` or a clap default overwrite a value from the
    /// TOML file, so `Args` skips unset fields when it serializes.
    pub fn load<T: Serialize>(args: &T) -> Result<Self> {
        let config: Self = Figment::new()
            // Start with built-in defaults
            .merge(Serialized::defaults(Config::default()))
            // Then layer in TOML config file (if it exists)
            .merge(Toml::file(Self::config_path()?))
            // Then environment variables (GHOSTWRITER_MODEL, etc.)
            .merge(Env::prefixed("GHOSTWRITER_"))
            // Finally CLI arguments (highest precedence)
            .merge(Serialized::globals(args))
            .extract()
            .map_err(|e| anyhow::anyhow!("Configuration error: {}", e))?;

        // Validate the final configuration
        config.validate()?;
        Ok(config)
    }

    /// Save current configuration to the TOML file, readable by the owner only
    /// because it may hold the API key.
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        log::info!("Saving config to {:?}", config_path);
        let content = toml::to_string_pretty(self).map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))?;

        std::fs::write(&config_path, content).map_err(|e| anyhow::anyhow!("Failed to write config file {:?}: {}", config_path, e))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600))
                .map_err(|e| anyhow::anyhow!("Failed to set permissions on {:?}: {}", config_path, e))?;
        }

        Ok(())
    }

    /// Get the config file path: ~/.ghostwriter.toml
    pub fn config_path() -> Result<std::path::PathBuf> {
        let home = std::env::var("HOME").map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
        Ok(std::path::Path::new(&home).join(".ghostwriter.toml"))
    }

    /// Validate the configuration and return any errors
    pub fn validate(&self) -> Result<()> {
        // Validate trigger corner
        TriggerCorner::from_string(&self.trigger_corner)?;

        // Validate thinking tokens
        if self.thinking_tokens == 0 {
            return Err(anyhow::anyhow!("thinking_tokens must be greater than 0"));
        }

        Ok(())
    }

    /// Check if test mode is enabled
    pub fn is_test_mode(&self) -> bool {
        self.test_mode.is_some()
    }

    /// Get the test device model, or None if not in test mode
    pub fn get_test_device_model(&self) -> Option<DeviceModel> {
        self.test_device_model
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, DEFAULT_MODEL};

    /// Values from ~/.ghostwriter.toml must survive when the CLI passes nothing.
    /// This is the regression test for settings that silently reverted on restart.
    #[test]
    fn toml_values_survive_an_empty_cli_layer() {
        let dir = std::env::temp_dir().join(format!("ghostwriter-config-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(".ghostwriter.toml"),
            "model = \"model-from-toml\"\nengine_api_key = \"key-from-toml\"\nno_svg = false\ntrigger_corner = \"LL\"\n",
        )
        .unwrap();
        std::env::set_var("HOME", &dir);

        let config = Config::load(&serde_json::json!({})).unwrap();

        assert_eq!(config.model, "model-from-toml");
        assert_eq!(config.engine_api_key.as_deref(), Some("key-from-toml"));
        assert!(!config.no_svg);
        assert_eq!(config.trigger_corner, "LL");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn defaults_are_text_only_and_use_the_default_model() {
        let config = Config::default();
        assert!(config.no_svg);
        assert_eq!(config.model, DEFAULT_MODEL);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn partial_json_from_the_web_ui_still_deserializes() {
        let config: Config = serde_json::from_str("{\"model\": \"x\"}").unwrap();
        assert_eq!(config.model, "x");
        assert_eq!(config.web_port, 8080);
    }
}
