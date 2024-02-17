use std::path::PathBuf;
use crate::error::CResult;
use crate::storage::{KeyDir, ScanIteratorT, Status};
use crate::storage::engine::Engine;
use crate::storage::log::Log;

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
        Self::new_with_lock(path, true)
    }

    pub fn new_with_lock(path: PathBuf, try_lock: bool) -> CResult<Self> {
        let mut log = Log::new_with_lock(path, try_lock).unwrap();
        let keydir = log.build_keydir().unwrap();
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

    pub fn get_path(&self) -> Option<&str> {
        self.log.path.to_str()
    }
}

impl std::fmt::Display for LogCask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "log cask")
    }
}

impl Engine for LogCask {
    type ScanIterator<'a> = LogScanIterator<'a>;

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
        LogScanIterator { inner: self.keydir.range(range), log: &mut self.log }
    }

    fn scan_dyn<'a>(
        &'a mut self,
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

pub struct LogScanIterator<'a> {
    inner: std::collections::btree_map::Range<'a, Vec<u8>, (u64, u32)>,
    log: &'a mut Log,
}

impl<'a> LogScanIterator<'a> {
    fn map(&mut self, item: (&Vec<u8>, &(u64, u32))) -> <Self as Iterator>::Item {
        let (key, (value_pos, value_len)) = item;
        Ok((key.clone(), self.log.read_value(*value_pos, *value_len)?))
    }
}

impl<'a> Iterator for LogScanIterator<'a> {
    type Item = CResult<(Vec<u8>, Vec<u8>)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|item| self.map(item))
    }
}

impl<'a> DoubleEndedIterator for LogScanIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|item| self.map(item))
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};
    use std::path::PathBuf;
    use byteorder::ReadBytesExt;
    use bytes::{BufMut, BytesMut};
    use serde_derive::{Deserialize, Serialize};
    use crate::codec::json_codec::JsonCodec;
    use crate::codec::Serialize;
    use crate::error::{CResult, Error};
    use crate::storage::engine::Engine;
    use crate::storage::log_cask::LogCask;

    #[derive(Debug, Serialize, Deserialize)]
    struct Persion {
        name: String,

        age: i16,

        address: String,
    }

    /// Creates a new LogCask engine for testing.
    fn setup() -> CResult<LogCask> {
        let path = tempdir::TempDir::new("demo")?.path().join("mydb");
        println!("path:{:?}", &path);

        LogCask::new_with_lock(path, false)
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
    fn test_log() -> CResult<()> {
        let mut s = setup().unwrap();
        setup_log(&mut s).unwrap();

        let stat = s.status().unwrap();
        println!("stat:{:?}", stat);

        // write 4k
        let mut _4k = Vec::<u8>::with_capacity(1024 * 4);
        for i in 0..1024*4 {
            _4k.push(0);
        }
        s.set("4k".as_bytes(), _4k)?;
        s.flush().unwrap();

        let stat = s.status().unwrap();
        println!("stat:{:?}", stat);

        // test_load_from_log_file
        let mut cask = LogCask::new_with_lock(PathBuf::from(s.get_path().unwrap()), false).unwrap();
        let get = cask.get(b"b");
        assert!(get.is_ok());
        let get_val = get.unwrap().unwrap();
        assert_eq!(get_val, vec![0x02]);

        let get_4k = cask.get("4k".as_bytes());
        assert!(get_4k.is_ok());
        let get_4k_val = get_4k.unwrap().unwrap();
        assert_eq!(get_4k_val.len(), 1024*4);

        Ok(())
    }

    #[test]
    fn test_log_with_persion() {
        let mut log_cask = setup().unwrap();

        let codec = JsonCodec::new();

        let persion_key = "persion_cache_key";

        let mut buf = BytesMut::with_capacity(1024);
        for i in 0..1 {
            let p = Persion {
                name: format!("name{}", i),
                age: i % 85,
                address: format!("address{}", i),
            };

            let b = codec.serde(&p).unwrap();
            buf.put(b.as_bytes());
        }

        let persion_list_len = buf.len();
        log_cask.set(persion_key.as_bytes(), buf.to_vec()).unwrap();
        log_cask.flush().unwrap();

        let stat = log_cask.status().unwrap();
        println!("stat:{:?}", stat);

        // test_load_from_log_file
        let save_path = log_cask.get_path().unwrap();

        let mut two_cask = LogCask::new_with_lock(PathBuf::from(save_path), false).unwrap();
        let persion_list = two_cask.get(persion_key.as_bytes());
        assert!(persion_list.is_ok());
        let persion_list_val = persion_list.unwrap().unwrap();
        assert_eq!(persion_list_val.len(), persion_list_len);

        // let mut cursor = Cursor::new(persion_list_val.as_slice());
        // loop {
        //     let len = cursor.read_u64::<byteorder::LittleEndian>().unwrap() as usize;
        //     let mut by = vec![0; len];
        //     cursor.read_exact(&mut by).unwrap();
        //
        //     let val = String::from_utf8(by).unwrap();
        //     let rs: Result<Persion, serde_json::Error> = serde_json::from_str(val.as_str());
        //     let a = rs.unwrap();
        //     println!("{:?}", a);
        // }
    }
}