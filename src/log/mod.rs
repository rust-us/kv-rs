pub mod log_cask;

use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use fs4::FileExt;
use crate::error::CResult;

/// Maps keys to a value position and length in the log file.
pub type KeyDir = std::collections::BTreeMap<Vec<u8>, (u64, u32)>;

/// A append-only log file, containing a sequence of key/value entries encoded as follows;
///
/// - Key length as big-endian u32.
/// - Value length as big-endian i32, or -1 for tombstones.
/// - Key as raw bytes (max 2 GB).
/// - Value as raw bytes (max 2 GB).
pub struct Log {
    /// Path to the log file.
    path: PathBuf,
    /// The opened file containing the log.
    file: std::fs::File,
}

impl Log {
    /// Opens a log file, or creates one if it does not exist. Takes out an
    /// exclusive lock on the file until it is closed, or errors if the lock is
    /// already held.
    pub fn new(path: PathBuf) -> CResult<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?
        }

        let file = std::fs::OpenOptions::new().read(true).write(true).create(true).open(&path)?;
        file.try_lock_exclusive()?;
        Ok(Self { path, file })
    }

    /// Builds a keydir by scanning the log file. If an incomplete entry is
    /// encountered, it is assumed to be caused by an incomplete write operation
    /// and the remainder of the file is truncated.
    pub fn build_keydir(&mut self) -> CResult<KeyDir> {
        let mut len_buf = [0u8; 4];
        let mut keydir = KeyDir::new();
        let file_len = self.file.metadata()?.len();
        let mut r = BufReader::new(&mut self.file);
        let mut pos = r.seek(SeekFrom::Start(0))?;

        while pos < file_len {
            // Read the next entry from the file, returning the key, value
            // position, and value length or None for tombstones.
            let mut result = || -> Result<(Vec<u8>, u64, Option<u32>), std::io::Error> {
                r.read_exact(&mut len_buf)?;
                let key_len = u32::from_be_bytes(len_buf);
                r.read_exact(&mut len_buf)?;
                let value_len_or_tombstone = match i32::from_be_bytes(len_buf) {
                    l if l >= 0 => Some(l as u32),
                    _ => None, // -1 for tombstones
                };
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

    /// Reads a value from the log file.
    pub fn read_value(&mut self, value_pos: u64, value_len: u32) -> CResult<Vec<u8>> {
        let mut value = vec![0; value_len as usize];
        self.file.seek(SeekFrom::Start(value_pos))?;
        self.file.read_exact(&mut value)?;
        Ok(value)
    }

    /// Appends a key/value entry to the log file, using a None value for
    /// tombstones. It returns the position and length of the entry.
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

    #[cfg(test)]
    /// Prints the entire log file to the given writer in human-readable form.
    pub fn print<W: Write>(&mut self, w: &mut W) -> CResult<()> {
        let mut len_buf = [0u8; 4];
        let file_len = self.file.metadata()?.len();
        let mut r = BufReader::new(&mut self.file);
        let mut pos = r.seek(SeekFrom::Start(0))?;
        let mut idx = 0;

        while pos < file_len {
            writeln!(w, "entry = {}, offset {}", idx, pos)?;

            r.read_exact(&mut len_buf)?;
            let key_len = u32::from_be_bytes(len_buf);
            writeln!(w, "klen  = {} {:x?}", key_len, len_buf)?;

            r.read_exact(&mut len_buf)?;
            let value_len_or_tombstone = i32::from_be_bytes(len_buf); // NB: -1 for tombstones
            let value_len = value_len_or_tombstone.max(0) as u32;
            writeln!(w, "vlen  = {} {:x?}", value_len_or_tombstone, len_buf)?;

            let mut key = vec![0; key_len as usize];
            r.read_exact(&mut key)?;
            write!(w, "key   = ")?;
            if let Ok(str) = std::str::from_utf8(&key) {
                write!(w, r#""{}" "#, str)?;
            }
            writeln!(w, "{:x?}", key)?;

            let mut value = vec![0; value_len as usize];
            r.read_exact(&mut value)?;
            write!(w, "value = ")?;
            if value_len_or_tombstone < 0 {
                write!(w, "tombstone ")?;
            } else if let Ok(str) = std::str::from_utf8(&value) {
                if str.chars().all(|c| !c.is_control()) {
                    write!(w, r#""{}" "#, str)?;
                }
            }
            write!(w, "{:x?}\n\n", value)?;

            pos += 4 + 4 + key_len as u64 + value_len as u64;
            idx += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert_eq!(1, 1);
    }
}
