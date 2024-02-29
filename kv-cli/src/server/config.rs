use std::fmt::{Debug, Display};
use std::path::PathBuf;
use anyhow::anyhow;
use serde_derive::{Serialize, Deserialize};

const DEFAULT_STORAGE_PATH: &str = "storage/kvdb";
pub const DEFAULT_PROMPT: &str = "kvcli";
pub const DEFAULT_DB_NAME: &str = "kvdb";

//! load configration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigLoad {
    version: u8,

    api_key: String,

    /// load config path, default '${pwd}/config'
    storage_path: Option<PathBuf>,

    /// prompt
    pub prompt: Option<String>,

    /// Show stats after executing queries.  Only works with non-interactive mode.
    pub show_stats: Option<bool>,

    /// fix part cmd options. default false
    auto_append_part_cmd: Option<bool>,
    /// Division symbol
    auto_append_part_cmd_symbol: Option<char>,

    /// Multi line mode, default is true.
    pub multi_line: Option<bool>,

    /// whether replace '\n' with '\\n', default true.
    pub replace_newline: Option<bool>,

    cli: Option<CliConfig>,

}

/// load configration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CliConfig {
    /// Show rows affected
    show_affected: Option<bool>,

    /// progress
    pub progress_color: Option<String>,

    /// Show progress [bar] when executing queries.
    pub show_progress: Option<bool>,

    // 输出格式化

}

impl Default for ConfigLoad {
    fn default() -> Self {
        ConfigLoad {
            version: 0,
            api_key: "".to_string(),
            storage_path: None,
            prompt: Some(DEFAULT_PROMPT.to_string()),
            show_stats: Some(false),
            auto_append_part_cmd: Some(false),
            auto_append_part_cmd_symbol: Some(';'),
            multi_line: Some(true),
            replace_newline: Some(true),
            cli: Some(CliConfig::default()),
        }
    }
}

impl ConfigLoad {
    pub fn is_show_affected(&self) -> bool {
        match self.cli.as_ref() {
            None => {
                false
            }
            Some(c) => {
                if c.is_show_affected().is_none() {
                    false
                } else {
                    c.is_show_affected().unwrap().clone()
                }
            }
        }
    }

    /// load config path
    pub fn get_storage_path(&self) -> PathBuf {
        if self.storage_path.is_none() {
            PathBuf::from(DEFAULT_STORAGE_PATH)
        } else {
            self.storage_path.as_ref().unwrap().clone()
        }
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
        if self.auto_append_part_cmd_symbol.is_none() {
            // SemiColon ==>  ;
            ';'
        } else {
            self.auto_append_part_cmd_symbol.as_ref().unwrap().clone()
        }
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
            "auto_append_part_cmd_symbol" => self.auto_append_part_cmd_symbol = Some(cmd_value.parse()?),
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

    pub fn fix_settings(&mut self) {
        if self.storage_path.is_none() {
            self.storage_path = Some(PathBuf::from(DEFAULT_STORAGE_PATH));
        } else {
            let config_path = self.storage_path.as_ref().unwrap().join(DEFAULT_DB_NAME);
            self.storage_path = Some(config_path);
        }
    }

    fn set_show_progress(&mut self, v: bool) {
        match self.cli.as_mut() {
            None => {
                let mut cli = CliConfig::default();
                cli.set_show_progress(v);
                self.cli = Some(cli);
            }
            Some(c) => {
                c.set_show_progress(v);
            }
        }
    }

    fn set_show_affected(&mut self, v: bool) {
        match self.cli.as_mut() {
            None => {
                let mut cli = CliConfig::default();
                cli.set_show_affected(v);
                self.cli = Some(cli);
            }
            Some(c) => {
                c.set_show_affected(v);
            }
        }
    }
}

impl Default for CliConfig {
    fn default() -> Self {
        CliConfig {
            show_affected: Some(false),
            progress_color: None,
            show_progress: Some(false),
        }
    }
}

impl CliConfig {
    pub fn is_show_affected(&self) -> Option<&bool> {
        self.show_affected.as_ref()
    }

    pub fn set_show_affected(&mut self, show_affected: bool) {
        self.show_affected = Some(show_affected);
    }

    pub fn set_show_progress(&mut self, show_progress: bool) {
        self.show_progress = Some(show_progress);
    }
}