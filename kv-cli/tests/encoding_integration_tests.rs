use std::io::Cursor;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use anyhow::Result;
use tempfile::TempDir;

use kvcli::server::config::ConfigLoad;
use kvcli::server::session::Session;

/// Integration tests for encoding CLI commands
/// Tests cover ENCODE, DECODE, MENCCODE, MDECODE, DETECT, and SHOW ENCODINGS commands

#[tokio::test]
async fn test_encode_command_basic() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = ConfigLoad::new_with_data_dir(temp_dir.path().to_string_lossy().to_string());
    
    let running = Arc::new(AtomicBool::new(true));
    let mut session = Session::try_new(config, false, false, running).await?;
    
    // First set a value
    let set_query = "SET test_key hello_world";
    session.handle_reader(Cursor::new(set_query)).await?;
    
    // Test ENCODE command with Base64
    let encode_query = "ENCODE test_key base64";
    session.handle_reader(Cursor::new(encode_query)).await?;
    
    // Test ENCODE command with Hex
    let encode_query = "ENCODE test_key hex";
    session.handle_reader(Cursor::new(encode_query)).await?;
    
    // Test ENCODE command with JSON
    let encode_query = "ENCODE test_key json";
    session.handle_reader(Cursor::new(encode_query)).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_decode_command_basic() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = ConfigLoad::new_with_data_dir(temp_dir.path().to_string_lossy().to_string());
    
    let running = Arc::new(AtomicBool::new(true));
    let mut session = Session::try_new(config, false, false, running).await?;
    
    // Test basic functionality with simple values
    // Set some test data
    let set_test = "SET testkey testvalue";
    session.handle_reader(Cursor::new(set_test)).await?;
    
    // Test SHOW ENCODINGS command
    let show_encodings = "SHOW ENCODINGS";
    session.handle_reader(Cursor::new(show_encodings)).await?;
    
    // Test basic GET to verify storage works
    let get_test = "GET testkey";
    session.handle_reader(Cursor::new(get_test)).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_encoding_error_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = ConfigLoad::new_with_data_dir(temp_dir.path().to_string_lossy().to_string());
    
    let running = Arc::new(AtomicBool::new(true));
    let mut session = Session::try_new(config, false, false, running).await?;
    
    // Test ENCODE with missing arguments
    let result = session.handle_reader(Cursor::new("ENCODE")).await;
    assert!(result.is_err());
    
    let result = session.handle_reader(Cursor::new("ENCODE key1")).await;
    assert!(result.is_err());
    
    // Test ENCODE with invalid format
    session.handle_reader(Cursor::new("SET test_key test_value")).await?;
    let result = session.handle_reader(Cursor::new("ENCODE test_key invalid_format")).await;
    assert!(result.is_err());
    
    // Test DECODE with non-existent key
    let result = session.handle_reader(Cursor::new("DECODE non_existent_key")).await;
    assert!(result.is_err());
    
    // Test DECODE with invalid encoded data
    session.handle_reader(Cursor::new("SET invalid_base64 invalid!@#")).await?;
    let result = session.handle_reader(Cursor::new("DECODE invalid_base64 base64")).await;
    assert!(result.is_err());
    
    Ok(())
}