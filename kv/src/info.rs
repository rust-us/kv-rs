use crate::storage::engine::Engine;
use crate::storage::log_cask::LogCask;

pub fn get_info(engine: &mut LogCask) -> Vec<String> {
    let mut infos = Vec::<String>::new();
    infos.push("KV Storage:".to_ascii_lowercase());

    let status = engine.status();
    let size = if status.is_ok() {
        status.unwrap().keys as i64
    } else {
        0
    };

    infos
}