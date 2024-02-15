use serde_derive::{Deserialize, Serialize};
use crate::error::CResult;

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

/// A key/value storage engine, where both keys and values are arbitrary byte
/// strings between 0 B and 2 GB, stored in lexicographical key order. Writes
/// are only guaranteed durable after calling flush().
///
/// Only supports single-threaded use since all methods (including reads) take a
/// mutable reference -- serialized access can't be avoided anyway, since both
/// Raft execution and file access is serial.
pub trait Engine: std::fmt::Display + Send + Sync {
    /// The iterator returned by scan().
    type ScanIterator<'a>: ScanIteratorT + 'a
        where
            Self: Sized + 'a; // omit in trait objects, for object safety

    /// Deletes a key, or does nothing if it does not exist.
    fn delete(&mut self, key: &[u8]) -> CResult<()>;

    /// Flushes any buffered data to the underlying storage medium.
    fn flush(&mut self) -> CResult<()>;

    /// Gets a value for a key, if it exists.
    fn get(&mut self, key: &[u8]) -> CResult<Option<Vec<u8>>>;

    /// Iterates over an ordered range of key/value pairs.
    fn scan(&mut self, range: impl std::ops::RangeBounds<Vec<u8>>) -> Self::ScanIterator<'_>
        where
            Self: Sized; // omit in trait objects, for object safety

    /// Like scan, but can be used from trait objects. The iterator will use
    /// dynamic dispatch, which has a minor performance penalty.
    fn scan_dyn(
        &mut self,
        range: (std::ops::Bound<Vec<u8>>, std::ops::Bound<Vec<u8>>),
    ) -> Box<dyn ScanIteratorT + '_>;

    /// Iterates over all key/value pairs starting with prefix.
    fn scan_prefix(&mut self, prefix: &[u8]) -> Self::ScanIterator<'_>
        where
            Self: Sized, // omit in trait objects, for object safety
    {
        let start = std::ops::Bound::Included(prefix.to_vec());
        let end = match prefix.iter().rposition(|b| *b != 0xff) {
            Some(i) => std::ops::Bound::Excluded(
                prefix.iter().take(i).copied().chain(std::iter::once(prefix[i] + 1)).collect(),
            ),
            None => std::ops::Bound::Unbounded,
        };
        self.scan((start, end))
    }

    /// Sets a value for a key, replacing the existing value if any.
    fn set(&mut self, key: &[u8], value: Vec<u8>) -> CResult<()>;

    /// Returns engine status.
    fn status(&mut self) -> CResult<Status>;
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

