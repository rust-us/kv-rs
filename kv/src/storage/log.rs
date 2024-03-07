use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use fs4::FileExt;
use crate::error::{CResult, Error};
use crate::storage::KeyDir;

/// 一个仅追加的日志文件，包含如下要素；
///
/// - Key length as big-endian u32.
/// - Value length as big-endian i32, or -1 for tombstones.
/// - Key as raw bytes (max 2 GB).
/// - Value as raw bytes (max 2 GB).
pub struct Log {
    /// Path to the log file.
    pub(crate) path: PathBuf,
    /// The opened file containing the log.
    pub(crate) file: std::fs::File,
}

impl Log {
    /// 打开日志文件，如果不存在，则创建一个日志文件。
    /// 持有文件的独占锁，直到文件关闭为止；如果锁已被持有，则会出错。
    pub fn new(path: PathBuf) -> CResult<Self> {
        Self::new_with_lock(path, true)
    }

    pub fn new_with_lock(path: PathBuf, try_lock: bool) -> CResult<Self> {
        if let Some(dir) = path.parent() {
            match std::fs::create_dir_all(dir) {
                Ok(_) => {}
                Err(err) => {
                    return Err(Error::Internal(format!("{}:{:?}", err.to_string(), dir.to_str())))
                }
            }
        }

        let file = std::fs::OpenOptions::new()
                            .read(true)
                            .write(true)
                            .create(true)
                            // .create_new(true)
                            .open(&path)?;

        if try_lock {
            // 锁文件。 不允许其他进程篡改。 如果其他进程尝试篡改，则报错： "另一个程序已锁定文件的一部分，进程无法访问。 (os error 33)"
            file.try_lock_exclusive()?;
        }

        Ok(Self { path, file })
    }

    /// 用于在数据库启动时，根据日志重建LogCask，恢复出内存当中的BTreeMap
    ///
    /// 逻辑:
    ///    1. 从日志文件的开头开始遍历
    ///    2. 先读取出key_len和value_len，其中，如果value_len为-1则证明当前为tombstone
    ///    3. 如果是-1就封装一个none，否则计算出value_offset
    ///    4. 读取出key，之后根据是否为tombstone来决定对map是插入还是删除
    ///    5. 错误处理
    ///    6. 循环直至日志文件末尾
    pub fn build_keydir(&mut self) -> CResult<KeyDir> {
        let mut len_buf = [0u8; 4];
        let mut keydir = KeyDir::new();
        let file_len = self.file.metadata()?.len();
        let mut r = BufReader::new(&mut self.file);

        // step 1
        let mut pos = r.seek(SeekFrom::Start(0))?;

        while pos < file_len {
            // Read the next entry from the file, returning the key, value
            // position, and value length or None for tombstones.
            let mut result = || -> Result<(Vec<u8>, u64, Option<u32>), std::io::Error> {
                // step 2
                r.read_exact(&mut len_buf)?;
                let key_len = u32::from_be_bytes(len_buf);
                r.read_exact(&mut len_buf)?;
                let value_len_or_tombstone = match i32::from_be_bytes(len_buf) {
                    l if l >= 0 => Some(l as u32),
                    _ => None, // -1 for tombstones
                };
                // step 3
                let value_pos = pos + 4 + 4 + key_len as u64;

                let mut key = vec![0; key_len as usize];
                r.read_exact(&mut key)?;

                if let Some(value_len) = value_len_or_tombstone {
                    if value_pos + value_len as u64 > file_len {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            "value extends beyond end of file",
                        ));
                    }
                    r.seek_relative(value_len as i64)?; // avoids discarding buffer
                }

                Ok((key, value_pos, value_len_or_tombstone))
            };

            // step 4
            match result() {
                // Populate the keydir with the entry, or remove it on tombstones.
                Ok((key, value_pos, Some(value_len))) => {
                    keydir.insert(key, (value_pos, value_len));
                    pos = value_pos + value_len as u64;
                }
                Ok((key, value_pos, None)) => {
                    keydir.remove(&key);
                    pos = value_pos;
                }

                // step 5
                // If an incomplete entry was found at the end of the file, assume an
                // incomplete write and truncate the file.
                Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                    log::error!("Found incomplete entry at offset {}, truncating file", pos);
                    self.file.set_len(pos)?;
                    break;
                }
                Err(err) => return Err(err.into()),
            }
        }

        Ok(keydir)
    }

    /// 根据传入的偏移量和长度读取相应的值。
    pub fn read_value(&mut self, value_pos: u64, value_len: u32) -> CResult<Vec<u8>> {
        let mut value = vec![0; value_len as usize];
        self.file.seek(SeekFrom::Start(value_pos))?;
        self.file.read_exact(&mut value)?;
        Ok(value)
    }

    /// 分别写入key_len，value_len(or tombstone)，key_bytes，value_bytes(如果是删除那么使用None值)，最后调用flush持久化到磁盘，
    /// 最后返回一个offset和len，用于保存到BTreeMap当中
    pub fn write_entry(&mut self, key: &[u8], value: Option<&[u8]>) -> CResult<(u64, u32)> {
        let key_len = key.len() as u32;
        let value_len = value.map_or(0, |v| v.len() as u32);
        let value_len_or_tombstone = value.map_or(-1, |v| v.len() as i32);
        let len = 4 + 4 + key_len + value_len;

        let pos = self.file.seek(SeekFrom::End(0))?;
        let mut w = BufWriter::with_capacity(len as usize, &mut self.file);
        w.write_all(&key_len.to_be_bytes())?;
        w.write_all(&value_len_or_tombstone.to_be_bytes())?;
        w.write_all(key)?;
        if let Some(value) = value {
            w.write_all(value)?;
        }
        w.flush()?;

        Ok((pos, len))
    }
}

#[cfg(test)]
mod test {
    use crate::storage::log::Log;

    #[test]
    fn test() {
        // path: ~/Temp/demoxxx/mydb
        let path = tempdir::TempDir::new(/* 路径 */"demo").unwrap().path()
            // 文件名
            .join("mydb");
        println!("path:{:?}", &path);

        let mut log = Log::new(path.clone()).unwrap();
        let keydir = log.build_keydir().unwrap();

        let file_rs = std::fs::OpenOptions::new()
            .read(true).write(false).create(false).open(&path);
        assert!(file_rs.is_ok());
        let file = file_rs.unwrap();
        let meta = file.metadata();

        assert_eq!(1, 1);
    }
}
