use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;
use serde_derive::{Serialize, Deserialize};

const DEFAULT_STORAGE_PATH: &str = "storage/kvdb";
pub const DEFAULT_PROMPT: &str = "kvcli";
pub const DEFAULT_DB_NAME: &str = "kvdb";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigLoad {
    version: u8,

    api_key: String,

    /// load config path, default '${pwd}/config'
    storage_path: Option<PathBuf>,

    /// prompt
    pub prompt: Option<String>,

    /// progress
    pub progress_color: Option<String>,
    /// Show progress [bar] when executing queries.
    /// Only works with output format `table` and `null`.
    pub show_progress: Option<bool>,

    /// Show stats after executing queries.
    /// Only works with non-interactive mode.
    pub show_stats: Option<bool>,
    show_affected: Option<bool>,

    /// fix part cmd options. default false
    auto_append_part_cmd: Option<bool>,
    /// Division symbol
    auto_append_part_cmd_symbol: Option<char>,

    /// Multi line mode, default is true.
    pub multi_line: Option<bool>,

    /// whether replace '\n' with '\\n', default true.
    pub replace_newline: Option<bool>,

    // 输出格式化

}

impl Default for ConfigLoad {
    fn default() -> Self {
        ConfigLoad {
            version: 0,
            api_key: "".to_string(),
            storage_path: None,
            prompt: Some(DEFAULT_PROMPT.to_string()),
            progress_color: None,
            show_progress: Some(false),
            show_stats: Some(false),
            show_affected: Some(false),
            auto_append_part_cmd: Some(false),
            auto_append_part_cmd_symbol: Some(';'),
            multi_line: Some(true),
            replace_newline: Some(true),
        }
    }
}

// impl Debug for ConfigLoad {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         let mut builder = f.debug_struct("ConfigLoad");
//
//         builder.field("version", &self.version);
//         builder.field("api_key", &self.api_key);
//
//         if self.storage_path.is_some() {
//             builder.field("storage_path", &self.storage_path.as_ref().unwrap());
//         } else {
//             builder.field("storage_path", &"config");
//         }
//
//         if self.prompt.is_some() {
//             builder.field("prompt", &self.prompt.as_ref().unwrap());
//         }
//
//         if self.progress_color.is_some() {
//             builder.field("progress_color", &self.progress_color.as_ref().unwrap());
//         }
//         if self.show_progress.is_some() {
//             builder.field("show_progress", &self.show_progress.as_ref().unwrap());
//         } else {
//             builder.field("show_progress", &"false");
//         }
//
//         if self.show_stats.is_some() {
//             builder.field("show_stats", &self.show_stats.as_ref().unwrap());
//         } else {
//             builder.field("show_stats", &"false");
//         }
//
//         if self.show_affected.is_some() {
//             builder.field("show_affected", &self.show_affected.as_ref().unwrap());
//         } else {
//             builder.field("show_affected", &"false");
//         }
//
//         if self.multi_line.is_some() {
//             builder.field("multi_line", &self.multi_line.as_ref().unwrap());
//         } else {
//             builder.field("multi_line", &"false");
//         }
//
//         if self.replace_newline.is_some() {
//             builder.field("replace_newline", &self.replace_newline.as_ref().unwrap());
//         } else {
//             builder.field("replace_newline", &"false");
//         }
//
//         builder.finish()
//     }
// }

impl ConfigLoad {
    pub fn is_show_affected(&self) -> bool {
        if self.show_affected.is_none() {
            false
        } else {
            self.show_affected.clone().unwrap()
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

    pub fn terminal_update(&mut self) {
        self.show_progress = Some(true);
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
}