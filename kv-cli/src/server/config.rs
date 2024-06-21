use std::fmt::{Debug, Display};
use std::path::PathBuf;
use anyhow::anyhow;
use serde_derive::{Serialize, Deserialize};
use kv_rs::error::CResult;

const DEFAULT_STORAGE_PATH: &str = "storage";
const DEFAULT_DB: &str = "kvdb";
pub const DEFAULT_PROMPT: &str = "kvcli";
pub const DEFAULT_DB_NAME: &str = "kvdb";
pub const AUTO_APPEND_PART_CMD_SYMBOL: char = ';';

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

}

impl Default for ConfigLoad {
    fn default() -> Self {
        ConfigLoad {
            version: 1,
            api_key: "".to_string(),
            data_dir: "data".to_owned(),
            compact_threshold: 0.2,
            prompt: Some(DEFAULT_PROMPT.to_string()),
            show_stats: Some(false),
            auto_append_part_cmd: Some(false),
            multi_line: Some(true),
            replace_newline: Some(true),
            show_affected: Some(false),
            progress_color: None,
            show_progress: Some(false),
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
}
