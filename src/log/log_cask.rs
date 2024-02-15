use std::path::PathBuf;
use crate::error::CResult;
use crate::log::{KeyDir, Log};
use crate::storage::{Engine, ScanIteratorT, Status};

/// A very simple variant of LogCask, itself a very simple log-structured key-value engine.
///
/// LogCask writes key-value pairs to an append-only log file, and keeps a
/// mapping of keys to file positions in memory. All live keys must fit in
/// memory. Deletes write a tombstone value to the log file. To remove old
/// garbage, logs can be compacted by writing new logs containing only live
/// data, skipping replaced values and tombstones.
///
/// This implementation makes several significant simplifications over standard LogCask:
///
/// - Instead of writing multiple fixed-size log files, it uses a single
///   append-only log file of arbitrary size. This increases the compaction
///   volume, since the entire log file must be rewritten on every compaction,
///   and can exceed the filesystem's file size limit.
///
/// - Hint files are not used, the log itself is scanned when opened to
///   build the keydir. Hint files only omit values, the hint files would be nearly as large as
///   the compacted log files themselves.
///
/// - Log entries don't contain timestamps or checksums.
///
/// The structure of a log entry is:
///
/// - Key length as big-endian u32.
/// - Value length as big-endian i32, or -1 for tombstones.
/// - Key as raw bytes (max 2 GB).
/// - Value as raw bytes (max 2 GB).
pub struct LogCask {
    /// The active append-only log file.
    log: Log,
    /// Maps keys to a value position and length in the log file.
    keydir: KeyDir,
}

impl LogCask {
    /// Opens or creates a LogCask database in the given file.
    pub fn new(path: PathBuf) -> CResult<Self> {
        let mut log = Log::new(path)?;
        let keydir = log.build_keydir()?;
        Ok(Self { log, keydir })
    }

    /// Opens a LogCask database, and automatically compacts it if the amount
    /// of garbage exceeds the given ratio when opened.
    pub fn new_compact(path: PathBuf, garbage_ratio_threshold: f64) -> CResult<Self> {
        let mut s = Self::new(path)?;

        let status = s.status()?;
        let garbage_ratio = status.garbage_disk_size as f64 / status.total_disk_size as f64;
        if status.garbage_disk_size > 0 && garbage_ratio >= garbage_ratio_threshold {
            log::info!(
                "Compacting {} to remove {:.3}MB garbage ({:.0}% of {:.3}MB)",
                s.log.path.display(),
                status.garbage_disk_size / 1024 / 1024,
                garbage_ratio * 100.0,
                status.total_disk_size / 1024 / 1024
            );
            s.compact()?;
            log::info!(
                "Compacted {} to size {:.3}MB",
                s.log.path.display(),
                (status.total_disk_size - status.garbage_disk_size) / 1024 / 1024
            );
        }

        Ok(s)
    }
}

impl std::fmt::Display for LogCask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "log cask")
    }
}

impl Engine for LogCask {
    type ScanIterator<'a> = ScanIterator<'a>;

    fn delete(&mut self, key: &[u8]) -> CResult<()> {
        self.log.write_entry(key, None)?;
        self.keydir.remove(key);
        Ok(())
    }

    fn flush(&mut self) -> CResult<()> {
        Ok(self.log.file.sync_all()?)
    }

    fn get(&mut self, key: &[u8]) -> CResult<Option<Vec<u8>>> {
        if let Some((value_pos, value_len)) = self.keydir.get(key) {
            Ok(Some(self.log.read_value(*value_pos, *value_len)?))
        } else {
            Ok(None)
        }
    }

    fn scan(&mut self, range: impl std::ops::RangeBounds<Vec<u8>>) -> Self::ScanIterator<'_> {
        ScanIterator { inner: self.keydir.range(range), log: &mut self.log }
    }

    fn scan_dyn(
        &mut self,
        range: (std::ops::Bound<Vec<u8>>, std::ops::Bound<Vec<u8>>),
    ) -> Box<dyn ScanIteratorT + '_> {
        Box::new(self.scan(range))
    }


    fn set(&mut self, key: &[u8], value: Vec<u8>) -> CResult<()> {
        let (pos, len) = self.log.write_entry(key, Some(&*value))?;
        let value_len = value.len() as u32;
        self.keydir.insert(key.to_vec(), (pos + len as u64 - value_len as u64, value_len));
        Ok(())
    }

    fn status(&mut self) -> CResult<Status> {
        let keys = self.keydir.len() as u64;
        let size = self
            .keydir
            .iter()
            .fold(0, |size, (key, (_, value_len))| size + key.len() as u64 + *value_len as u64);
        let total_disk_size = self.log.file.metadata()?.len();
        let live_disk_size = size + 8 * keys; // account for length prefixes
        let garbage_disk_size = total_disk_size - live_disk_size;
        Ok(Status {
            name: self.to_string(),
            keys,
            size,
            total_disk_size,
            live_disk_size,
            garbage_disk_size,
        })
    }
}

pub struct ScanIterator<'a> {
    inner: std::collections::btree_map::Range<'a, Vec<u8>, (u64, u32)>,
    log: &'a mut Log,
}

impl<'a> ScanIterator<'a> {
    fn map(&mut self, item: (&Vec<u8>, &(u64, u32))) -> <Self as Iterator>::Item {
        let (key, (value_pos, value_len)) = item;
        Ok((key.clone(), self.log.read_value(*value_pos, *value_len)?))
    }
}

impl<'a> Iterator for ScanIterator<'a> {
    type Item = CResult<(Vec<u8>, Vec<u8>)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|item| self.map(item))
    }
}

impl<'a> DoubleEndedIterator for ScanIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|item| self.map(item))
    }
}

impl LogCask {
    /// Compacts the current log file by writing out a new log file containing
    /// only live keys and replacing the current file with it.
    pub fn compact(&mut self) -> CResult<()> {
        let mut tmp_path = self.log.path.clone();
        tmp_path.set_extension("new");
        let (mut new_log, new_keydir) = self.write_log(tmp_path)?;

        std::fs::rename(&new_log.path, &self.log.path)?;
        new_log.path = self.log.path.clone();

        self.log = new_log;
        self.keydir = new_keydir;
        Ok(())
    }

    /// Writes out a new log file with the live entries of the current log file
    /// and returns it along with its keydir. Entries are written in key order.
    fn write_log(&mut self, path: PathBuf) -> CResult<(Log, KeyDir)> {
        let mut new_keydir = KeyDir::new();
        let mut new_log = Log::new(path)?;
        new_log.file.set_len(0)?; // truncate file if it exists
        for (key, (value_pos, value_len)) in self.keydir.iter() {
            let value = self.log.read_value(*value_pos, *value_len)?;
            let (pos, len) = new_log.write_entry(key, Some(&value))?;
            new_keydir.insert(key.clone(), (pos + len as u64 - *value_len as u64, *value_len));
        }
        Ok((new_log, new_keydir))
    }
}

/// Attempt to flush the file when the database is closed.
impl Drop for LogCask {
    fn drop(&mut self) {
        if let Err(error) = self.flush() {
            log::error!("failed to flush file: {}", error)
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::error::{CResult, Error};
    use crate::log::log_cask::LogCask;
    use crate::storage::Engine;

    /// Creates a new LogCask engine for testing.
    fn setup() -> CResult<LogCask> {
        LogCask::new(tempdir::TempDir::new("demo")?.path().join("mydb"))
    }

    /// Writes various values primarily for testing log file handling.
    ///
    /// - '': empty key and value
    /// - a: write
    /// - b: write, write
    /// - c: write, delete, write
    /// - d: delete, write
    /// - e: write, delete
    /// - f: delete
    fn setup_log(s: &mut LogCask) -> CResult<()> {
        s.set(b"b", vec![0x01])?;
        s.set(b"b", vec![0x02])?;

        s.set(b"e", vec![0x05])?;
        s.delete(b"e")?;

        s.set(b"c", vec![0x00])?;
        s.delete(b"c")?;
        s.set(b"c", vec![0x03])?;

        s.set(b"", vec![])?;

        s.set(b"a", vec![0x01])?;

        s.delete(b"f")?;

        s.delete(b"d")?;
        s.set(b"d", vec![0x04])?;

        // Make sure the scan yields the expected results.
        assert_eq!(
            vec![
                (b"".to_vec(), vec![]),
                (b"a".to_vec(), vec![0x01]),
                (b"b".to_vec(), vec![0x02]),
                (b"c".to_vec(), vec![0x03]),
                (b"d".to_vec(), vec![0x04]),
            ],
            s.scan(..).collect::<Result<Vec<_>,Error>> ()?,
        );

        let rs = s.flush();

        Ok(())
    }

    #[test]
    /// Tests that logs are written correctly using a golden file.
    fn log() -> CResult<()> {
        let mut s = setup()?;
        setup_log(&mut s)?;

        Ok(())
    }
}