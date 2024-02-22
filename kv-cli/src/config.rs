use serde_derive::{Serialize, Deserialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ConfigLoad {
    version: u8,

    api_key: String,
}