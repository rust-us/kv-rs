use std::path::PathBuf;
use crate::error::{CResult, Error};
use crate::storage::{KeyDir, ScanIteratorT, Status};
use crate::storage::engine::Engine;
use crate::storage::log::Log;

/// LogCask 是一个非常简单的日志结构的键值引擎。
///
/// LogCask将键值对写入一个只追加数据的日志文件中，并保留一个内存索引(hash mapping)， 内存索引维护key在文件中的position。
///
/// 所有活动的key都必须出现在内存索引中。
/// 删除某个key是将逻辑删除值写入日志文件。去除该key的索引。
///
/// - 该实现不写多个固定大小的日志文件，而是使用单个任意大小的日志文件，且只做追加。这样实现的好处是：增加了紧密度，避免小文件产生，但坏处是，不适合大数据量的场景。
///
/// - 打开数据文件时会扫描日志本身以构建 keydir。
///
/// - log entry 不包含timestamps or checksums.
///
/// log entry 的结构为：
/// - Key length as big-endian u32.
/// - Value length as big-endian i32, or -1 for tombstones.
/// - Key as raw bytes (max 2 GB).
/// - Value as raw bytes (max 2 GB).
pub struct LogCask {
    /// The active append-only log file
    log: Log,

    /// use index, Maps keys to a value position and length in the log file.
    keydir: KeyDir,
}

impl LogCask {
    /// 新建一个 LogCask，并调用上面分析过的log.build_keydir来从日志文件当中恢复内存当中的map
    pub fn new(path: PathBuf) -> CResult<Self> {
        Self::new_with_lock(path, true)
    }

    pub fn new_with_lock(path: PathBuf, try_lock: bool) -> CResult<Self> {
        let mut log = Log::new_with_lock(path, try_lock)?;

        let keydir = log.build_keydir()?;

        Ok(Self { log, keydir })
    }

    /// 用于处理小规模数据集的引擎模式。
    ///
    /// 只有在kvdb启动时才会执行 Compact 操作，并且此过程将锁定日志文件。
    /// 在new_compact当中，会计算当前的garbage_ratio，无效数据(垃圾量)超过阈值，就进行compact。
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

    fn delete(&mut self, key: &[u8]) -> CResult<i64> {
        // 写入的内容为tombstone(None)，标志key对应的val已经被删除，同时删除内存索引中的kv
        self.log.write_entry(key, None)?;
        self.keydir.remove(key);
        Ok(1)
    }

    fn flush(&mut self) -> CResult<()> {
        Ok(self.log.file.sync_all()?)
    }

    fn get(&mut self, key: &[u8]) -> CResult<Option<Vec<u8>>> {
        // 首先查询内存当中的map，如果不存在返回不存在，如果能查询到，那么就根据metadata去磁盘当中读取出对应的value
        if let Some((value_pos, value_len)) = self.keydir.get(key) {
            Ok(Some(self.log.read_value(*value_pos, *value_len)?))
        } else {
            Ok(None)
        }
    }

    fn scan(&mut self, range: impl std::ops::RangeBounds<Vec<u8>>) -> Self::ScanIterator<'_>
        where Self: Sized {
        LogScanIterator { inner: self.keydir.range(range), log: &mut self.log }
    }

    fn scan_dyn<'a>(
        &'a mut self,
        range: (std::ops::Bound<Vec<u8>>, std::ops::Bound<Vec<u8>>),
    ) -> Box<dyn ScanIteratorT + '_> {
        Box::new(self.scan(range))
    }

    fn set(&mut self, key: &[u8], value: Vec<u8>) -> CResult<()> {
        // 首先向磁盘当中写入一条新的Entry，并且更新内存的map，保存新Entry的offset
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
    /// 在写入过程当中，会有key被更新或者删除，但是旧版本的key依旧会存在于日志文件当中，随着时间的增加，日志文件当中的无效数据就会越来越多，占用额外的存储空间。因此就需要compaction将其清除。
    /// LogCask compact 实现是，遍历当前内存当中存在的key，创建一个新文件，调用“write_log”重建日志文件并保存。并用它替换当前文件。
    pub fn compact(&mut self) -> CResult<()> {
        let mut tmp_path = self.log.path.clone();
        // need double disk size
        tmp_path.set_extension("new");

        let (mut new_log, new_keydir) = self.write_log(tmp_path)?;

        if cfg!(target_os = "windows") {
            // println!("on Windows, from can be anything, \
            // but to must not be a directory.{}, {}, {}, {}, {}",
            //          &self.log.path.is_dir(),
            //          &self.log.path.is_absolute(),
            //          &self.log.path.is_relative(),
            //          &self.log.path.is_symlink(),
            //          &self.log.path.is_file());

            match std::fs::rename(&new_log.path, &self.log.path) {
                Ok(_) => {}
                Err(err) => {
                    return Err(Error::Value(
                        format!("db file compact error on Windows, from {:?} to {:?}, cause:{}.",
                                &new_log.path.to_str(),
                                &self.log.path.to_str(), err.to_string())
                    ))
                }
            };
        } else if cfg!(target_os = "linux"){
            match std::fs::rename(&new_log.path, &self.log.path) {
                Ok(_) => {}
                Err(err) => {
                    return Err(Error::Value(
                        format!("db file compact error on Linux, from {:?} to {:?}, cause:{}.",
                                &new_log.path.to_str(),
                                &self.log.path.to_str(), err.to_string())
                    ))
                }
            };
        } else {
            match std::fs::rename(&new_log.path, &self.log.path) {
                Ok(_) => {}
                Err(err) => {
                    return Err(Error::Value(
                        format!("db file compact error on Unknown os, from {:?} to {:?}, cause:{}.",
                                &new_log.path.to_str(),
                                &self.log.path.to_str(), err.to_string())
                    ))
                }
            };
        };

        new_log.path = self.log.path.clone();

        self.log = new_log;
        self.keydir = new_keydir;
        Ok(())
    }

    /// 遍历当前的map，去原本的日志文件当中读取，写入到新的日志文件当中，并且构建新的map
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

/// Attempt to flush the file when the LogCask is closed.
impl Drop for LogCask {
    fn drop(&mut self) {
        if let Err(error) = self.flush() {
            log::error!("failed to flush file: {}", error)
        }
    }
}

/// 用于进行范围读取
pub struct LogScanIterator<'a> {
    inner: std::collections::btree_map::Range<'a, Vec<u8>, (u64, u32)>,
    log: &'a mut Log,
}

impl<'a> LogScanIterator<'a> {
    /// map函数，调用self.log.read_value()去磁盘当中进行读取，用于将BTreeMap当中的key与offset转换为真实的kv。
    /// 由于inner和log都是引用类型，因此标注了生命周期
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
    use crate::codec::{Codec,};
    use crate::codec::bytes_codec::BytesCodec;
    use crate::error::{CResult, Error};
    use crate::storage::engine::Engine;
    use crate::storage::log::Log;
    use crate::storage::log_cask::LogCask;
    use crate::storage::Status;

    super::super::tests::test_engine!({
        let path = tempdir::TempDir::new("demo")?.path().join("whosdb");
        LogCask::new(path)?
    });

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Persion {
        name: String,

        age: i16,

        address: String,
    }

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
    /// Tests that writing and then reading a file yields the same results.
    fn reopen() -> CResult<()> {
        // NB: Don't use setup(), because the tempdir will be removed when
        // the path falls out of scope.
        let path = tempdir::TempDir::new("demo")?.path().join("adb");
        let mut s = LogCask::new(path.clone())?;
        setup_log(&mut s)?;

        let expect = s.scan(..).collect::<CResult<Vec<_>>>()?;
        drop(s);
        let mut s = LogCask::new(path)?;
        assert_eq!(expect, s.scan(..).collect::<CResult<Vec<_>>>()?,);

        Ok(())
    }

    #[test]
    /// Tests that new_compact() will automatically compact the file when appropriate.
    fn new_compact() -> CResult<()> {
        // Create an initial log file with a few entries.
        let dir = tempdir::TempDir::new("demo")?;
        let path = dir.path().join("orig");
        let compactpath = dir.path().join("compact");

        let mut s = LogCask::new_compact(path.clone(), 0.2)?;
        setup_log(&mut s)?;
        let status = s.status()?;
        let garbage_ratio = status.garbage_disk_size as f64 / status.total_disk_size as f64;
        drop(s);

        // Test a few threshold value and assert whether it should trigger compaction.
        let cases = vec![
            (-1.0, true),
            (0.0, true),
            (garbage_ratio - 0.001, true),
            (garbage_ratio, true),
            (garbage_ratio + 0.001, false),
            (1.0, false),
            (2.0, false),
        ];
        for (threshold, expect_compact) in cases.into_iter() {
            std::fs::copy(&path, &compactpath)?;
            let mut s = LogCask::new_compact(compactpath.clone(), threshold)?;
            let new_status = s.status()?;
            assert_eq!(new_status.live_disk_size, status.live_disk_size);
            if expect_compact {
                assert_eq!(new_status.total_disk_size, status.live_disk_size);
                assert_eq!(new_status.garbage_disk_size, 0);
            } else {
                assert_eq!(new_status, status);
            }
        }

        Ok(())
    }

    #[test]
    /// Tests that exclusive locks are taken out on log files, released when the
    /// cask is closed, and that an error is returned if a lock is already
    /// held.
    fn log_lock() -> CResult<()> {
        let path = tempdir::TempDir::new("demo")?.path().join("t_app");
        let s = LogCask::new(path.clone())?;

        assert!(LogCask::new(path.clone()).is_err());
        drop(s);
        assert!(LogCask::new(path.clone()).is_ok());

        Ok(())
    }

    #[test]
    /// Tests that an incomplete write at the end of the log file can be
    /// recovered by discarding the last entry.
    fn recovery() -> CResult<()> {
        // Create an initial log file with a few entries.
        let dir = tempdir::TempDir::new("demmo")?;
        let path = dir.path().join("complete");
        let truncpath = dir.path().join("truncated");

        let mut log = Log::new(path.clone())?;
        let mut ends = vec![];

        let (pos, len) = log.write_entry("deleted".as_bytes(), Some(&[1, 2, 3]))?;
        ends.push(pos + len as u64);

        let (pos, len) = log.write_entry("deleted".as_bytes(), None)?;
        ends.push(pos + len as u64);

        let (pos, len) = log.write_entry(&[], Some(&[]))?;
        ends.push(pos + len as u64);

        let (pos, len) = log.write_entry("key".as_bytes(), Some(&[1, 2, 3, 4, 5]))?;
        ends.push(pos + len as u64);

        drop(log);

        // Copy the file, and truncate it at each byte, then try to open it
        // and assert that we always retain a prefix of entries.
        let size = std::fs::metadata(&path)?.len();
        for pos in 0..=size {
            std::fs::copy(&path, &truncpath)?;
            let f = std::fs::OpenOptions::new().write(true).open(&truncpath)?;
            f.set_len(pos)?;
            drop(f);

            let mut expect = vec![];
            if pos >= ends[0] {
                expect.push((b"deleted".to_vec(), vec![1, 2, 3]))
            }
            if pos >= ends[1] {
                expect.pop(); // "deleted" key removed
            }
            if pos >= ends[2] {
                expect.push((b"".to_vec(), vec![]))
            }
            if pos >= ends[3] {
                expect.push((b"key".to_vec(), vec![1, 2, 3, 4, 5]))
            }

            let mut s = LogCask::new(truncpath.clone())?;
            assert_eq!(expect, s.scan(..).collect::<CResult<Vec<_>>>()?);
        }

        Ok(())
    }

    #[test]
    /// Tests status(), both for a log file with known garbage, and
    /// after compacting it when the live size must equal the file size.
    fn test_status_full() -> CResult<()> {
        let mut s = setup()?;
        setup_log(&mut s)?;

        // Before compaction.
        assert_eq!(
            s.status()?,
            Status {
                name: "log cask".to_string(),
                keys: 5,
                size: 8,
                total_disk_size: 114,
                live_disk_size: 48,
                garbage_disk_size: 66
            }
        );

        // After compaction.
        s.compact()?;
        assert_eq!(
            s.status()?,
            Status {
                name: "log cask".to_string(),
                keys: 5,
                size: 8,
                total_disk_size: 48,
                live_disk_size: 48,
                garbage_disk_size: 0,
            }
        );

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
    fn test_log_with_bytes_persion() {
        let codec = BytesCodec::new();

        let mut log_cask = setup().unwrap();

        let persion_key = "persion_cache_key";

        let mut list_for_cache = Vec::<Persion>::new();
        let mut buf = BytesMut::with_capacity(1024);
        for i in 0..1 {
            let p = Persion {
                name: format!("name{}", i),
                age: i % 85,
                address: format!("address{}", i),
            };
            list_for_cache.push(p.clone());

            let b = codec.encode(&p).unwrap();
            buf.put(b.as_slice());
        }

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

        let mut i_for_test = 0;
        let mut cursor = Cursor::new(persion_list_val.as_slice());
        loop {
            if cursor.is_empty() {
                break;
            }

            let len = cursor.read_u64::<byteorder::BigEndian>().unwrap() as usize;
            let mut by = vec![0; len];
            cursor.read_exact(&mut by).unwrap();

            let r: Persion = codec.decode_bytes(&by, false).unwrap();
            println!("{:?}", r);

            let cache_p = list_for_cache.get(i_for_test).unwrap();
            assert_eq!(&r.name, &cache_p.name);
            assert_eq!(&r.address, &cache_p.address);
            assert_eq!(&r.age, &cache_p.age);

            i_for_test += 1;
        }

        assert_eq!(1, 1);
    }
}