use std::path::PathBuf;
use kv_rs::error::Error;
use kv_rs::storage::engine::Engine;
use kv_rs::storage::log_cask::LogCask;

fn main() {
    println!("Hello, kv CLI!");

    run().unwrap();

    println!("Bye~");
}

fn run() -> Result<(), Error> {
    let storage_path = PathBuf::new().join("D:/workspace/kv/storage/kvdb");
    // let storage_path = tempdir::TempDir::new("demo")?.path().join("kvdb");

    let mut engine = LogCask::new(storage_path)?;
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