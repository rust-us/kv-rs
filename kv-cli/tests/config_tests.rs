use tempfile::TempDir;
use anyhow::Result;

use kvcli::server::config::{ConfigLoad, EncodingConfig};
use kv_rs::encoding::EncodingFormat;

#[test]
fn test_encoding_config_default() {
    let config = EncodingConfig::default();
    
    assert_eq!(config.default_format, "base64");
    assert!(config.auto_detect);
    assert_eq!(config.batch_size, 100);
}

#[test]
fn test_encoding_config_validation() -> Result<()> {
    let mut config = EncodingConfig::default();
    
    // Valid configuration should pass
    assert!(config.validate().is_ok());
    
    // Invalid batch size (0) should fail
    config.batch_size = 0;
    assert!(config.validate().is_err());
    
    // Invalid batch size (too large) should fail
    config.batch_size = 10001;
    assert!(config.validate().is_err());
    
    // Valid batch size should pass
    config.batch_size = 500;
    assert!(config.validate().is_ok());
    
    // Invalid format should fail
    config.default_format = "invalid_format".to_string();
    assert!(config.validate().is_err());
    
    Ok(())
}

#[test]
fn test_encoding_config_format_conversion() -> Result<()> {
    let mut config = EncodingConfig::default();
    
    // Test getting default format as enum
    assert_eq!(config.get_default_format()?, EncodingFormat::Base64);
    
    // Test setting format from enum
    config.set_default_format(EncodingFormat::Hex);
    assert_eq!(config.default_format, "hex");
    assert_eq!(config.get_default_format()?, EncodingFormat::Hex);
    
    config.set_default_format(EncodingFormat::Json);
    assert_eq!(config.default_format, "json");
    assert_eq!(config.get_default_format()?, EncodingFormat::Json);
    
    Ok(())
}

#[test]
fn test_config_load_default() {
    let config = ConfigLoad::default();
    
    // Test that encoding config is properly initialized
    assert!(config.encoding.is_some());
    
    let encoding_config = config.get_encoding_config();
    assert_eq!(encoding_config.default_format, "base64");
    assert!(encoding_config.auto_detect);
    assert_eq!(encoding_config.batch_size, 100);
}

#[test]
fn test_config_load_encoding_methods() -> Result<()> {
    let mut config = ConfigLoad::default();
    
    // Test getting default encoding format
    assert_eq!(config.get_default_encoding_format()?, EncodingFormat::Base64);
    
    // Test setting default encoding format
    config.set_default_encoding_format(EncodingFormat::Hex);
    assert_eq!(config.get_default_encoding_format()?, EncodingFormat::Hex);
    
    // Test auto-detect settings
    assert!(config.is_auto_detect_enabled());
    config.set_auto_detect(false);
    assert!(!config.is_auto_detect_enabled());
    
    // Test batch size settings
    assert_eq!(config.get_batch_size(), 100);
    config.set_batch_size(200)?;
    assert_eq!(config.get_batch_size(), 200);
    
    Ok(())
}

#[test]
fn test_config_load_batch_size_validation() {
    let mut config = ConfigLoad::default();
    
    // Valid batch sizes should work
    assert!(config.set_batch_size(1).is_ok());
    assert!(config.set_batch_size(100).is_ok());
    assert!(config.set_batch_size(10000).is_ok());
    
    // Invalid batch sizes should fail
    assert!(config.set_batch_size(0).is_err());
    assert!(config.set_batch_size(10001).is_err());
}

#[test]
fn test_config_load_encoding_config_persistence() -> Result<()> {
    let mut config = ConfigLoad::default();
    
    // Create a custom encoding config
    let encoding_config = EncodingConfig {
        default_format: "hex".to_string(),
        auto_detect: false,
        batch_size: 150,
    };
    
    // Set the encoding config
    config.set_encoding_config(encoding_config.clone());
    
    // Verify it was set correctly
    let retrieved_config = config.get_encoding_config();
    assert_eq!(retrieved_config.default_format, encoding_config.default_format);
    assert_eq!(retrieved_config.auto_detect, encoding_config.auto_detect);
    assert_eq!(retrieved_config.batch_size, encoding_config.batch_size);
    
    Ok(())
}

#[test]
fn test_config_load_inject_cmd() -> Result<()> {
    let mut config = ConfigLoad::default();
    
    // Test injecting encoding-related commands
    config.inject_cmd("default_encoding_format", "hex")?;
    assert_eq!(config.get_default_encoding_format()?, EncodingFormat::Hex);
    
    config.inject_cmd("auto_detect", "false")?;
    assert!(!config.is_auto_detect_enabled());
    
    config.inject_cmd("batch_size", "250")?;
    assert_eq!(config.get_batch_size(), 250);
    
    // Test invalid commands
    assert!(config.inject_cmd("invalid_command", "value").is_err());
    
    // Test invalid values
    assert!(config.inject_cmd("default_encoding_format", "invalid_format").is_err());
    assert!(config.inject_cmd("auto_detect", "invalid_bool").is_err());
    assert!(config.inject_cmd("batch_size", "invalid_number").is_err());
    assert!(config.inject_cmd("batch_size", "0").is_err());
    
    Ok(())
}

#[test]
fn test_config_load_validation() -> Result<()> {
    let config = ConfigLoad::default();
    
    // Default config should be valid
    assert!(config.validate_encoding_config().is_ok());
    
    Ok(())
}

#[test]
fn test_encoding_format_string_parsing() -> Result<()> {
    // Test valid format strings
    assert_eq!("base64".parse::<EncodingFormat>()?, EncodingFormat::Base64);
    assert_eq!("BASE64".parse::<EncodingFormat>()?, EncodingFormat::Base64);
    assert_eq!("Base64".parse::<EncodingFormat>()?, EncodingFormat::Base64);
    
    assert_eq!("hex".parse::<EncodingFormat>()?, EncodingFormat::Hex);
    assert_eq!("HEX".parse::<EncodingFormat>()?, EncodingFormat::Hex);
    assert_eq!("Hex".parse::<EncodingFormat>()?, EncodingFormat::Hex);
    
    assert_eq!("json".parse::<EncodingFormat>()?, EncodingFormat::Json);
    assert_eq!("JSON".parse::<EncodingFormat>()?, EncodingFormat::Json);
    assert_eq!("Json".parse::<EncodingFormat>()?, EncodingFormat::Json);
    
    // Test invalid format strings
    assert!("invalid".parse::<EncodingFormat>().is_err());
    assert!("base32".parse::<EncodingFormat>().is_err());
    assert!("".parse::<EncodingFormat>().is_err());
    
    Ok(())
}