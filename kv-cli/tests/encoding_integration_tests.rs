use std::io::Cursor;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use anyhow::Result;

use kvcli::server::config::ConfigLoad;
use kvcli::server::session::Session;

/// Integration tests for encoding CLI commands
/// Tests cover ENCODE, DECODE, MENCCODE, MDECODE, DETECT, and SHOW ENCODINGS commands

#[tokio::test]
async fn test_encode_command_basic() -> Result<()> {
    let config = ConfigLoad::default();
    
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
    let config = ConfigLoad::default();
    
    let running = Arc::new(AtomicBool::new(true));
    let mut session = Session::try_new(config, false, false, running).await?;
    
    // Set encoded values
    let set_base64 = "SET encoded_base64 SGVsbG8gV29ybGQ="; // "Hello World" in base64
    session.handle_reader(Cursor::new(set_base64)).await?;
    
    let set_hex = "SET encoded_hex 48656c6c6f20576f726c64"; // "Hello World" in hex
    session.handle_reader(Cursor::new(set_hex)).await?;
    
    let set_json = r#"SET encoded_json "Hello World""#; // "Hello World" in JSON
    session.handle_reader(Cursor::new(set_json)).await?;
    
    // Test DECODE with explicit format
    let decode_base64 = "DECODE encoded_base64 base64";
    session.handle_reader(Cursor::new(decode_base64)).await?;
    
    let decode_hex = "DECODE encoded_hex hex";
    session.handle_reader(Cursor::new(decode_hex)).await?;
    
    let decode_json = "DECODE encoded_json json";
    session.handle_reader(Cursor::new(decode_json)).await?;
    
    // Test DECODE with auto-detection
    let decode_auto = "DECODE encoded_base64";
    session.handle_reader(Cursor::new(decode_auto)).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_encoding_error_handling() -> Result<()> {
    let config = ConfigLoad::default();
    
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