#![allow(non_camel_case_types)]
#![feature(is_terminal)]
#![feature(const_trait_impl)]
#![feature(exact_size_is_empty)]
// just for cursor.is_empty()
#![feature(cursor_remaining)]


//! `kv-rs` is a key-value pairs to an append-only log file,
//! and keeps a mapping of keys to file positions in memory.
//! All live keys must fit in memory.
//! Deletes write a tombstone value to the log file.
//! To remove old garbage, logs can be compacted by writing new logs containing only live
//! data, skipping replaced values and tombstones. [Author fengyang]
//!
//! ## Getting started
//!
//! ```rust
//! use std::path::PathBuf;
//! use kv_rs::error::Error;
//! use kv_rs::storage::engine::Engine;
//! use kv_rs::storage::log_cask::LogCask;
//!
//! fn main() {
//!     println!("Hello, kv CLI!");
//!
//!     run().unwrap();
//!
//!     println!("Bye~");
//! }
//!
//! fn run() -> Result<(), Error> {
//!     let storage_path = PathBuf::new().join("D:/workspace/kv/storage/kvdb");
//!     // let storage_path = tempdir::TempDir::new("demo")?.path().join("kvdb");
//!
//!     let mut engine = LogCask::new(storage_path)?;
//!     engine.set(b"b", vec![0x01])?;
//!     engine.set(b"b", vec![0x02])?;
//!
//!     engine.set(b"e", vec![0x05])?;
//!     engine.delete(b"e")?;
//!
//!     engine.set(b"c", vec![0x00])?;
//!     engine.delete(b"c")?;
//!     engine.set(b"c", vec![0x03])?;
//!
//!     engine.set(b"", vec![])?;
//!
//!     engine.set(b"a", vec![0x01])?;
//!
//!     engine.delete(b"f")?;
//!
//!     engine.delete(b"d")?;
//!     engine.set(b"d", vec![0x04])?;
//!
//!     // Make sure the scan yields the expected results.
//!     assert_eq!(
//!             vec![
//!                 (b"".to_vec(), vec![]),
//!                 (b"a".to_vec(), vec![0x01]),
//!                 (b"b".to_vec(), vec![0x02]),
//!                 (b"c".to_vec(), vec![0x03]),
//!                 (b"d".to_vec(), vec![0x04]),
//!             ],
//!             engine.scan(..).collect::<Result<Vec<_>,Error>> ()?,
//!         );
//!
//!     let rs = engine.flush();
//!
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod storage;
pub mod codec;
pub mod mvcc;
pub mod row;
pub mod snapshot;
pub mod info;

#[cfg(test)]
mod test {
    use crate::error::Error;
    use crate::storage::engine::Engine;
    use crate::storage::log_cask::LogCask;

    #[test]
    fn run() -> Result<(), Error> {
        let storage_path = "";
        let path = tempdir::TempDir::new("demo")?.path().join("whosdb");

        let mut engine = LogCask::new(path)?;
        engine.set(b"b", vec![0x01])?;
        engine.set(b"b", vec![0x02])?;

        engine.set(b"e", vec![0x05])?;
        engine.delete(b"e")?;

        engine.set(b"c", vec![0x00])?;
        engine.delete(b"c")?;
        engine.set(b"c", vec![0x03])?;

        engine.set(b"", vec![])?;

        engine.set(b"a", vec![0x01])?;

        engine.delete(b"f")?;

        engine.delete(b"d")?;
        engine.set(b"d", vec![0x04])?;

        // Make sure the scan yields the expected results.
        assert_eq!(
            vec![
                (b"".to_vec(), vec![]),
                (b"a".to_vec(), vec![0x01]),
                (b"b".to_vec(), vec![0x02]),
                (b"c".to_vec(), vec![0x03]),
                (b"d".to_vec(), vec![0x04]),
            ],
            engine.scan(..).collect::<Result<Vec<_>,Error>> ()?,
        );

        let rs = engine.flush();

        Ok(())
    }
}

