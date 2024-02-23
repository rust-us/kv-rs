use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigLoad {
    version: u8,

    api_key: String,

    pub prompt: Option<String>,
    pub progress_color: Option<String>,

    /// Show progress [bar] when executing queries.
    /// Only works with output format `table` and `null`.
    pub show_progress: Option<bool>,

    /// Show stats after executing queries.
    /// Only works with non-interactive mode.
    pub show_stats: Option<bool>,

    /// Multi line mode, default is true.
    pub multi_line: Option<bool>,

    /// whether replace '\n' with '\\n', default true.
    pub replace_newline: Option<bool>,
}

impl Default for ConfigLoad {
    fn default() -> Self {
        ConfigLoad {
            version: 0,
            api_key: "".to_string(),
            prompt: Some("kvcli".to_string()),
            progress_color: None,
            show_progress: Some(false),
            show_stats: Some(false),
            multi_line: Some(true),
            replace_newline: Some(true),
        }
    }
}

impl ConfigLoad {
    pub fn terminal_update(&mut self) {
        self.show_progress = Some(true);
        self.show_stats = Some(true);
    }
}