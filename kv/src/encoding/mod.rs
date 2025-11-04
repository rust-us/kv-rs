use std::collections::HashMap;
use std::fmt;
use crate::error::Error;

pub mod base64_codec;
pub mod hex_codec;
pub mod json_codec;

pub use base64_codec::Base64Codec;
pub use hex_codec::HexCodec;
pub use json_codec::JsonCodec;

/// Supported encoding formats for data transformation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EncodingFormat {
    Base64,
    Hex,
    Json,
}

impl fmt::Display for EncodingFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodingFormat::Base64 => write!(f, "base64"),
            EncodingFormat::Hex => write!(f, "hex"),
            EncodingFormat::Json => write!(f, "json"),
        }
    }
}

impl std::str::FromStr for EncodingFormat {
    type Err = EncodingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "base64" => Ok(EncodingFormat::Base64),
            "hex" => Ok(EncodingFormat::Hex),
            "json" => Ok(EncodingFormat::Json),
            _ => Err(EncodingError::UnsupportedFormat(s.to_string())),
        }
    }
}

/// Errors that can occur during encoding/decoding operations
#[derive(Debug, Clone, PartialEq)]
pub enum EncodingError {
    UnsupportedFormat(String),
    InvalidData(String),
    KeyNotFound(String),
    EncodingFailed(String),
    DecodingFailed(String),
    DetectionFailed(String),
}

impl fmt::Display for EncodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodingError::UnsupportedFormat(format) => {
                write!(f, "Unsupported encoding format: {}", format)
            }
            EncodingError::InvalidData(msg) => write!(f, "Invalid encoded data: {}", msg),
            EncodingError::KeyNotFound(key) => write!(f, "Key not found: {}", key),
            EncodingError::EncodingFailed(msg) => write!(f, "Encoding operation failed: {}", msg),
            EncodingError::DecodingFailed(msg) => write!(f, "Decoding operation failed: {}", msg),
            EncodingError::DetectionFailed(msg) => write!(f, "Format detection failed: {}", msg),
        }
    }
}

impl std::error::Error for EncodingError {}

impl From<EncodingError> for Error {
    fn from(err: EncodingError) -> Self {
        Error::Internal(err.to_string())
    }
}

/// Trait for data encoding/decoding implementations
pub trait DataCodec: Send + Sync {
    /// Encode raw bytes into a string representation
    fn encode(&self, data: &[u8]) -> Result<String, EncodingError>;
    
    /// Decode string representation back to raw bytes
    fn decode(&self, encoded: &str) -> Result<Vec<u8>, EncodingError>;
    
    /// Check if the given string can be decoded by this codec
    fn can_decode(&self, data: &str) -> bool;
    
    /// Get the name of this encoding format
    fn format_name(&self) -> &'static str;
}

/// Core encoding engine that manages different encoding formats
pub struct EncodingEngine {
    default_format: EncodingFormat,
    codecs: HashMap<EncodingFormat, Box<dyn DataCodec>>,
}

impl EncodingEngine {
    /// Create a new encoding engine with the specified default format
    pub fn new(default_format: EncodingFormat) -> Self {
        Self {
            default_format,
            codecs: HashMap::new(),
        }
    }

    /// Register a codec for a specific encoding format
    pub fn register_codec(&mut self, format: EncodingFormat, codec: Box<dyn DataCodec>) {
        self.codecs.insert(format, codec);
    }

    /// Get the default encoding format
    pub fn default_format(&self) -> EncodingFormat {
        self.default_format
    }

    /// Set the default encoding format
    pub fn set_default_format(&mut self, format: EncodingFormat) {
        self.default_format = format;
    }

    /// Encode data using the specified format
    pub fn encode(&self, data: &[u8], format: EncodingFormat) -> Result<String, EncodingError> {
        match self.codecs.get(&format) {
            Some(codec) => codec.encode(data),
            None => Err(EncodingError::UnsupportedFormat(format.to_string())),
        }
    }

    /// Decode data using the specified format
    pub fn decode(&self, encoded: &str, format: EncodingFormat) -> Result<Vec<u8>, EncodingError> {
        match self.codecs.get(&format) {
            Some(codec) => codec.decode(encoded),
            None => Err(EncodingError::UnsupportedFormat(format.to_string())),
        }
    }

    /// Encode data using the default format
    pub fn encode_default(&self, data: &[u8]) -> Result<String, EncodingError> {
        self.encode(data, self.default_format)
    }

    /// Detect the encoding format of the given data
    pub fn detect(&self, data: &str) -> Vec<(EncodingFormat, f32)> {
        let mut results = Vec::new();
        
        for (format, codec) in &self.codecs {
            if codec.can_decode(data) {
                // Simple confidence scoring based on format characteristics
                let confidence = self.calculate_confidence(*format, data);
                results.push((*format, confidence));
            }
        }
        
        // Sort by confidence (highest first)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Get a list of all supported encoding formats
    pub fn supported_formats(&self) -> Vec<EncodingFormat> {
        self.codecs.keys().copied().collect()
    }

    /// Check if a format is supported
    pub fn is_format_supported(&self, format: EncodingFormat) -> bool {
        self.codecs.contains_key(&format)
    }

    /// Calculate confidence score for format detection
    fn calculate_confidence(&self, format: EncodingFormat, data: &str) -> f32 {
        match format {
            EncodingFormat::Base64 => {
                // Base64 should have valid characters and proper padding
                let valid_chars = data.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=');
                let proper_length = data.len() % 4 == 0;
                let padding_count = data.chars().rev().take_while(|&c| c == '=').count();
                
                if valid_chars && proper_length && padding_count <= 2 {
                    0.9
                } else if valid_chars {
                    0.6
                } else {
                    0.1
                }
            }
            EncodingFormat::Hex => {
                // Hex should have only hex characters and even length
                let valid_chars = data.chars().all(|c| c.is_ascii_hexdigit());
                let even_length = data.len() % 2 == 0;
                
                if valid_chars && even_length {
                    0.8
                } else if valid_chars {
                    0.5
                } else {
                    0.1
                }
            }
            EncodingFormat::Json => {
                // JSON should start and end with quotes for string encoding
                if data.starts_with('"') && data.ends_with('"') && data.len() >= 2 {
                    0.7
                } else {
                    0.2
                }
            }
        }
    }
}

impl Default for EncodingEngine {
    fn default() -> Self {
        Self::new(EncodingFormat::Base64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCodec {
        name: &'static str,
    }

    impl DataCodec for MockCodec {
        fn encode(&self, data: &[u8]) -> Result<String, EncodingError> {
            Ok(format!("{}:{}", self.name, String::from_utf8_lossy(data)))
        }

        fn decode(&self, encoded: &str) -> Result<Vec<u8>, EncodingError> {
            if let Some(data) = encoded.strip_prefix(&format!("{}:", self.name)) {
                Ok(data.as_bytes().to_vec())
            } else {
                Err(EncodingError::DecodingFailed("Invalid format".to_string()))
            }
        }

        fn can_decode(&self, data: &str) -> bool {
            data.starts_with(&format!("{}:", self.name))
        }

        fn format_name(&self) -> &'static str {
            self.name
        }
    }

    #[test]
    fn test_encoding_format_display() {
        assert_eq!(EncodingFormat::Base64.to_string(), "base64");
        assert_eq!(EncodingFormat::Hex.to_string(), "hex");
        assert_eq!(EncodingFormat::Json.to_string(), "json");
    }

    #[test]
    fn test_encoding_format_from_str() {
        assert_eq!("base64".parse::<EncodingFormat>().unwrap(), EncodingFormat::Base64);
        assert_eq!("hex".parse::<EncodingFormat>().unwrap(), EncodingFormat::Hex);
        assert_eq!("json".parse::<EncodingFormat>().unwrap(), EncodingFormat::Json);
        assert_eq!("BASE64".parse::<EncodingFormat>().unwrap(), EncodingFormat::Base64);
        
        assert!("invalid".parse::<EncodingFormat>().is_err());
    }

    #[test]
    fn test_encoding_engine_basic() {
        let mut engine = EncodingEngine::new(EncodingFormat::Base64);
        
        assert_eq!(engine.default_format(), EncodingFormat::Base64);
        assert_eq!(engine.supported_formats().len(), 0);
        
        engine.set_default_format(EncodingFormat::Hex);
        assert_eq!(engine.default_format(), EncodingFormat::Hex);
    }

    #[test]
    fn test_encoding_engine_with_codec() {
        let mut engine = EncodingEngine::new(EncodingFormat::Base64);
        let codec = Box::new(MockCodec { name: "test" });
        
        engine.register_codec(EncodingFormat::Base64, codec);
        
        assert!(engine.is_format_supported(EncodingFormat::Base64));
        assert!(!engine.is_format_supported(EncodingFormat::Hex));
        
        let data = b"hello world";
        let encoded = engine.encode(data, EncodingFormat::Base64).unwrap();
        assert_eq!(encoded, "test:hello world");
        
        let decoded = engine.decode(&encoded, EncodingFormat::Base64).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_encoding_engine_unsupported_format() {
        let engine = EncodingEngine::new(EncodingFormat::Base64);
        
        let result = engine.encode(b"test", EncodingFormat::Hex);
        assert!(matches!(result, Err(EncodingError::UnsupportedFormat(_))));
    }

    #[test]
    fn test_detection() {
        let mut engine = EncodingEngine::new(EncodingFormat::Base64);
        let codec = Box::new(MockCodec { name: "test" });
        
        engine.register_codec(EncodingFormat::Base64, codec);
        
        let results = engine.detect("test:hello");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, EncodingFormat::Base64);
        
        let results = engine.detect("invalid");
        assert_eq!(results.len(), 0);
    }
}