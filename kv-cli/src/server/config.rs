use std::fmt::{Debug, Display};
use std::path::PathBuf;
use anyhow::anyhow;
use serde_derive::{Serialize, Deserialize};
use kv_rs::error::CResult;
use kv_rs::encoding::EncodingFormat;

const DEFAULT_STORAGE_PATH: &str = "storage";
const DEFAULT_DB: &str = "kvdb";
pub const DEFAULT_PROMPT: &str = "kvcli";
pub const DEFAULT_DB_NAME: &str = "kvdb";
pub const AUTO_APPEND_PART_CMD_SYMBOL: char = ';';

/// Encoding configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingConfig {
    /// Default encoding format for new data
    pub default_format: String,
    /// Enable automatic format detection
    pub auto_detect: bool,
    /// Batch processing size for bulk operations
    pub batch_size: usize,
}

impl Default for EncodingConfig {
    fn default() -> Self {
        EncodingConfig {
            default_format: "base64".to_string(),
            auto_detect: true,
            batch_size: 100,
        }
    }
}

impl EncodingConfig {
    /// Get the default encoding format as EncodingFormat enum
    pub fn get_default_format(&self) -> Result<EncodingFormat, anyhow::Error> {
        self.default_format.parse()
            .map_err(|e| anyhow!("Invalid default encoding format '{}': {}", self.default_format, e))
    }

    /// Set the default encoding format from EncodingFormat enum
    pub fn set_default_format(&mut self, format: EncodingFormat) {
        self.default_format = format.to_string();
    }

    /// Validate the encoding configuration
    pub fn validate(&self) -> Result<(), anyhow::Error> {
        // Validate default format
        self.get_default_format()?;
        
        // Validate batch size
        if self.batch_size == 0 {
            return Err(anyhow!("Batch size must be greater than 0"));
        }
        
        if self.batch_size > 10000 {
            return Err(anyhow!("Batch size must not exceed 10000"));
        }
        
        Ok(())
    }
}

/// load configration
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct ConfigLoad {
    version: u8,

    api_key: String,

    /// load config path, default '${pwd}/data'
    data_dir: String,
    /// compact_threshold, default '0.2
    compact_threshold: f64,

    /// prompt, default 'kvcli'
    pub prompt: Option<String>,

    /// Show stats after executing queries.  Only works with non-interactive mode.
    pub show_stats: Option<bool>,

    /// fix part cmd options. default false
    auto_append_part_cmd: Option<bool>,

    /// Multi line mode, default is true.
    pub multi_line: Option<bool>,

    /// whether replace '\n' with '\\n', default true.
    pub replace_newline: Option<bool>,

    /// cli
    /// Show rows affected
    show_affected: Option<bool>,

    /// progress
    pub progress_color: Option<String>,

    /// Show progress [bar] when executing queries.
    pub show_progress: Option<bool>,

    /// Encoding configuration
    pub encoding: Option<EncodingConfig>,

}

impl Default for ConfigLoad {
    fn default() -> Self {
        ConfigLoad {
            version: 1,
            api_key: "".to_string(),
            data_dir: "storage".to_owned(),
            compact_threshold: 0.2,
            prompt: Some(DEFAULT_PROMPT.to_string()),
            show_stats: Some(false),
            auto_append_part_cmd: Some(false),
            multi_line: Some(true),
            replace_newline: Some(true),
            show_affected: Some(false),
            progress_color: None,
            show_progress: Some(false),
            encoding: Some(EncodingConfig::default()),
        }
    }
}

impl ConfigLoad {
    pub fn new(file: &str) -> CResult<Self> {
        let df = ConfigLoad::default();

        Ok(config::Config::builder()
            .set_default("version", df.version)?
            .set_default("api_key", df.api_key)?
            .set_default("data_dir", df.data_dir)?
            .set_default("compact_threshold", 0.2)?
            .set_default("prompt", df.prompt)?
            .set_default("show_stats", df.show_stats)?
            .set_default("auto_append_part_cmd", df.auto_append_part_cmd)?
            .set_default("multi_line", df.multi_line)?
            .set_default("replace_newline", df.replace_newline)?
            .set_default("show_affected", df.show_affected)?
            .set_default("progress_color", df.progress_color)?
            .set_default("show_progress", df.show_progress)?
            .set_default("encoding.default_format", "base64")?
            .set_default("encoding.auto_detect", true)?
            .set_default("encoding.batch_size", 100)?
            .add_source(config::File::with_name(file))
            .add_source(config::Environment::with_prefix("KVDB"))
            .build()?
            .try_deserialize()?)
    }

    /// load config path
    pub fn get_data_dir(&self) -> PathBuf {
        std::path::Path::new(&self.data_dir).join(DEFAULT_DB)
    }

    pub fn get_compact_threshold(&self) -> f64 {
        self.compact_threshold
    }

    /// fix part cmd options. default false
    pub fn get_auto_append_part_cmd(&self) -> bool {
        if self.auto_append_part_cmd.is_none() {
            false
        } else {
            self.auto_append_part_cmd.as_ref().unwrap().clone()
        }
    }

    /// Division symbol
    pub fn get_auto_append_part_cmd_symbol(&self) -> char {
        AUTO_APPEND_PART_CMD_SYMBOL
    }

    /// change cmd:
    /// show_progress、show_stats、show_affected、auto_append_part_cmd、auto_append_part_cmd_symbol、multi_line、replace_newline
    /// default_encoding_format、auto_detect、batch_size
    pub fn inject_cmd(&mut self, cmd_name: &str, cmd_value: &str) -> anyhow::Result<()> {
        match cmd_name {
            // cli
            "show_progress" => {
                self.set_show_progress(cmd_value.parse()?);
            },
            "show_affected" => {
                self.set_show_affected(cmd_value.parse()?);
            },
            "show_stats" => self.show_stats = Some(cmd_value.parse()?),
            "auto_append_part_cmd" => self.auto_append_part_cmd = Some(cmd_value.parse()?),
            "multi_line" => self.multi_line = Some(cmd_value.parse()?),
            "replace_newline" => self.replace_newline = Some(cmd_value.parse()?),
            // encoding
            "default_encoding_format" => {
                let format: EncodingFormat = cmd_value.parse()
                    .map_err(|e| anyhow!("Invalid encoding format '{}': {}", cmd_value, e))?;
                self.set_default_encoding_format(format);
            },
            "auto_detect" => {
                self.set_auto_detect(cmd_value.parse()?);
            },
            "batch_size" => {
                let size: usize = cmd_value.parse()
                    .map_err(|e| anyhow!("Invalid batch size '{}': {}", cmd_value, e))?;
                self.set_batch_size(size)?;
            },
            _ => return Err(anyhow!("Unknown command: {}", cmd_name)),
        }
        Ok(())
    }

    pub fn terminal_update(&mut self) {
        self.set_show_progress(true);

        self.show_stats = Some(true);
    }

    fn set_show_progress(&mut self, v: bool) {
        self.show_progress = Some(v)
    }

    pub fn is_show_affected(&self) -> bool {
        match self.show_affected {
            None => {
                false
            }
            Some(r) => {
                r
            }
        }
    }

    fn set_show_affected(&mut self, v: bool) {
        self.show_affected= Some(v)
    }

    /// Get encoding configuration with defaults
    pub fn get_encoding_config(&self) -> EncodingConfig {
        self.encoding.clone().unwrap_or_default()
    }

    /// Set encoding configuration
    pub fn set_encoding_config(&mut self, config: EncodingConfig) {
        self.encoding = Some(config);
    }

    /// Get the default encoding format
    pub fn get_default_encoding_format(&self) -> Result<EncodingFormat, anyhow::Error> {
        self.get_encoding_config().get_default_format()
    }

    /// Set the default encoding format
    pub fn set_default_encoding_format(&mut self, format: EncodingFormat) {
        let mut config = self.get_encoding_config();
        config.set_default_format(format);
        self.set_encoding_config(config);
    }

    /// Check if auto-detection is enabled
    pub fn is_auto_detect_enabled(&self) -> bool {
        self.get_encoding_config().auto_detect
    }

    /// Set auto-detection enabled/disabled
    pub fn set_auto_detect(&mut self, enabled: bool) {
        let mut config = self.get_encoding_config();
        config.auto_detect = enabled;
        self.set_encoding_config(config);
    }

    /// Get batch size for bulk operations
    pub fn get_batch_size(&self) -> usize {
        self.get_encoding_config().batch_size
    }

    /// Set batch size for bulk operations
    pub fn set_batch_size(&mut self, size: usize) -> Result<(), anyhow::Error> {
        let mut config = self.get_encoding_config();
        config.batch_size = size;
        config.validate()?;
        self.set_encoding_config(config);
        Ok(())
    }

    /// Validate encoding configuration
    pub fn validate_encoding_config(&self) -> Result<(), anyhow::Error> {
        self.get_encoding_config().validate()
    }
}
