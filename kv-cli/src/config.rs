use std::fmt::{Debug, Display, Formatter};
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
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

impl Debug for ConfigLoad {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("ConfigLoad");

        builder.field("version", &self.version);
        builder.field("api_key", &self.api_key);

        if self.prompt.is_some() {
            builder.field("prompt", &self.prompt.as_ref().unwrap());
        }

        if self.progress_color.is_some() {
            builder.field("progress_color", &self.progress_color.as_ref().unwrap());
        }

        if self.show_progress.is_some() {
            builder.field("show_progress", &self.show_progress.as_ref().unwrap());
        } else {
            builder.field("show_progress", &"false");
        }

        if self.show_stats.is_some() {
            builder.field("show_stats", &self.show_stats.as_ref().unwrap());
        } else {
            builder.field("show_stats", &"false");
        }

        if self.multi_line.is_some() {
            builder.field("multi_line", &self.multi_line.as_ref().unwrap());
        } else {
            builder.field("multi_line", &"false");
        }

        if self.replace_newline.is_some() {
            builder.field("replace_newline", &self.replace_newline.as_ref().unwrap());
        } else {
            builder.field("replace_newline", &"false");
        }

        builder.finish()
    }
}

impl ConfigLoad {
    pub fn terminal_update(&mut self) {
        self.show_progress = Some(true);
        self.show_stats = Some(true);
    }
}