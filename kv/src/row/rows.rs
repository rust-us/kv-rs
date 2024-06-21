use serde_derive::Deserialize;

#[derive(Clone, Debug)]
pub enum RowWithStats {
    Row(Row),
    Stats(ServerStats),
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct ServerStats {
    #[serde(default)]
    pub total_rows: usize,
    #[serde(default)]
    pub total_bytes: usize,

    #[serde(default)]
    pub read_rows: usize,
    #[serde(default)]
    pub read_bytes: usize,

    #[serde(default)]
    pub write_rows: usize,
    #[serde(default)]
    pub write_bytes: usize,

    #[serde(default)]
    pub running_time_ms: f64,
}

#[derive(Clone, Debug, Default)]
pub struct Row(Vec<String>);

impl ServerStats {
    pub fn normalize(&mut self) {
        if self.total_rows == 0 {
            self.total_rows = self.read_rows;
        }
        if self.total_bytes == 0 {
            self.total_bytes = self.read_bytes;
        }
    }
}