pub mod log;
pub mod engine;

use serde_derive::{Deserialize, Serialize};
use crate::error::CResult;

/// Maps keys to a value position and length in the log file.
pub type KeyDir = std::collections::BTreeMap<Vec<u8>, (u64, u32)>;

/// Engine status.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Status {
    /// The name of the storage engine.
    pub name: String,
    /// The number of live keys in the engine.
    pub keys: u64,
    /// The logical size of live key/value pairs.
    pub size: u64,
    /// The on-disk size of all data, live and garbage.
    pub total_disk_size: u64,
    /// The on-disk size of live data.
    pub live_disk_size: u64,
    /// The on-disk size of garbage data.
    pub garbage_disk_size: u64,
}

/// A scan iterator, with a blanket implementation (in lieu of trait aliases).
pub trait ScanIteratorT: DoubleEndedIterator<Item = CResult<(Vec<u8>, Vec<u8>)>> {}

impl<I: DoubleEndedIterator<Item = CResult<(Vec<u8>, Vec<u8>)>>> ScanIteratorT for I {}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert_eq!(1, 1);
    }
}

