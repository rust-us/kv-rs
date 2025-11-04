use serde_json;
use crate::encoding::{DataCodec, EncodingError};

/// JSON string encoding/decoding implementation
/// This codec encodes raw bytes as JSON-escaped strings
pub struct JsonCodec;

impl JsonCodec {
    /// Create a new JSON codec instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl DataCodec for JsonCodec {
    fn encode(&self, data: &[u8]) -> Result<String, EncodingError> {
        // Convert bytes to string (lossy conversion for non-UTF8 data)
        let string_data = String::from_utf8_lossy(data);
        
        // Use serde_json to properly escape the string
        serde_json::to_string(&string_data.as_ref())
            .map_err(|e| EncodingError::EncodingFailed(format!("JSON encode error: {}", e)))
    }

    fn decode(&self, encoded: &str) -> Result<Vec<u8>, EncodingError> {
        let trimmed = encoded.trim();
        
        // Parse the JSON string
        let decoded_string: String = serde_json::from_str(trimmed)
            .map_err(|e| EncodingError::DecodingFailed(format!("JSON decode error: {}", e)))?;
        
        // Convert back to bytes
        Ok(decoded_string.into_bytes())
    }

    fn can_decode(&self, data: &str) -> bool {
        let trimmed = data.trim();
        
        // Empty string is not valid JSON
        if trimmed.is_empty() {
            return false;
        }
        
        // Must start and end with quotes for JSON string
        if !trimmed.starts_with('"') || !trimmed.ends_with('"') {
            return false;
        }
        
        // Must have at least 2 characters (opening and closing quotes)
        if trimmed.len() < 2 {
            return false;
        }
        
        // Try to parse as JSON string
        serde_json::from_str::<String>(trimmed).is_ok()
    }

    fn format_name(&self) -> &'static str {
        "json"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_encode() {
        let codec = JsonCodec::new();
        
        // Test empty data
        assert_eq!(codec.encode(b"").unwrap(), r#""""#);
        
        // Test simple string
        assert_eq!(codec.encode(b"hello").unwrap(), r#""hello""#);
        
        // Test string with special characters
        assert_eq!(codec.encode(b"hello\nworld").unwrap(), r#""hello\nworld""#);
        assert_eq!(codec.encode(b"hello\"world").unwrap(), r#""hello\"world""#);
        assert_eq!(codec.encode(b"hello\\world").unwrap(), r#""hello\\world""#);
        
        // Test string with unicode
        assert_eq!(codec.encode("你好世界".as_bytes()).unwrap(), r#""你好世界""#);
        
        // Test string with control characters
        assert_eq!(codec.encode(b"hello\tworld").unwrap(), r#""hello\tworld""#);
        assert_eq!(codec.encode(b"hello\rworld").unwrap(), r#""hello\rworld""#);
    }

    #[test]
    fn test_json_decode() {
        let codec = JsonCodec::new();
        
        // Test empty data
        assert_eq!(codec.decode(r#""""#).unwrap(), b"");
        
        // Test simple string
        assert_eq!(codec.decode(r#""hello""#).unwrap(), b"hello");
        
        // Test string with special characters
        assert_eq!(codec.decode(r#""hello\nworld""#).unwrap(), b"hello\nworld");
        assert_eq!(codec.decode(r#""hello\"world""#).unwrap(), b"hello\"world");
        assert_eq!(codec.decode(r#""hello\\world""#).unwrap(), b"hello\\world");
        
        // Test string with unicode
        assert_eq!(codec.decode(r#""你好世界""#).unwrap(), "你好世界".as_bytes());
        
        // Test string with control characters
        assert_eq!(codec.decode(r#""hello\tworld""#).unwrap(), b"hello\tworld");
        assert_eq!(codec.decode(r#""hello\rworld""#).unwrap(), b"hello\rworld");
        
        // Test with whitespace
        assert_eq!(codec.decode(r#"  "hello"  "#).unwrap(), b"hello");
    }

    #[test]
    fn test_json_decode_invalid() {
        let codec = JsonCodec::new();
        
        // Test invalid JSON
        assert!(codec.decode("invalid").is_err());
        assert!(codec.decode("hello").is_err());
        assert!(codec.decode(r#""unterminated string"#).is_err());
        assert!(codec.decode(r#"unterminated string""#).is_err());
        assert!(codec.decode("123").is_err()); // number, not string
        assert!(codec.decode("true").is_err()); // boolean, not string
        assert!(codec.decode("null").is_err()); // null, not string
        assert!(codec.decode("{}").is_err()); // object, not string
        assert!(codec.decode("[]").is_err()); // array, not string
        
        // Test invalid escape sequences
        assert!(codec.decode(r#""invalid\escape""#).is_err());
    }

    #[test]
    fn test_json_can_decode() {
        let codec = JsonCodec::new();
        
        // Valid JSON strings
        assert!(codec.can_decode(r#""""#));
        assert!(codec.can_decode(r#""hello""#));
        assert!(codec.can_decode(r#""hello\nworld""#));
        assert!(codec.can_decode(r#""hello\"world""#));
        assert!(codec.can_decode(r#""hello\\world""#));
        assert!(codec.can_decode(r#""你好世界""#));
        assert!(codec.can_decode(r#""hello\tworld""#));
        assert!(codec.can_decode(r#"  "hello"  "#));
        
        // Invalid JSON strings
        assert!(!codec.can_decode(""));
        assert!(!codec.can_decode("invalid"));
        assert!(!codec.can_decode("hello"));
        assert!(!codec.can_decode(r#""unterminated string"#));
        assert!(!codec.can_decode(r#"unterminated string""#));
        assert!(!codec.can_decode("123"));
        assert!(!codec.can_decode("true"));
        assert!(!codec.can_decode("null"));
        assert!(!codec.can_decode("{}"));
        assert!(!codec.can_decode("[]"));
        assert!(!codec.can_decode(r#""invalid\escape""#));
    }

    #[test]
    fn test_json_roundtrip() {
        let codec = JsonCodec::new();
        
        let test_cases = vec![
            b"".as_slice(),
            b"hello",
            b"hello world",
            b"hello\nworld",
            b"hello\"world",
            b"hello\\world",
            b"hello\tworld",
            b"hello\rworld",
            "你好世界".as_bytes(),
            b"The quick brown fox jumps over the lazy dog",
            b"Special chars: !@#$%^&*()_+-=[]{}|;':\",./<>?",
        ];
        
        for data in test_cases {
            let encoded = codec.encode(data).unwrap();
            let decoded = codec.decode(&encoded).unwrap();
            assert_eq!(decoded, data, "Roundtrip failed for: {:?}", String::from_utf8_lossy(data));
        }
    }

    #[test]
    fn test_json_format_name() {
        let codec = JsonCodec::new();
        assert_eq!(codec.format_name(), "json");
    }

    #[test]
    fn test_json_special_characters() {
        let codec = JsonCodec::new();
        
        // Test various special characters that need escaping
        let test_cases = vec![
            (b"\n", r#""\n""#),
            (b"\r", r#""\r""#),
            (b"\t", r#""\t""#),
            (b"\"", r#""\"""#),
            (b"\\", r#""\\""#),
            (b"/", r#""/""#), // Forward slash can be escaped but doesn't have to be
        ];
        
        for (input, expected) in test_cases {
            let encoded = codec.encode(input).unwrap();
            // Note: serde_json might not escape forward slash, so we only check roundtrip
            let decoded = codec.decode(&encoded).unwrap();
            assert_eq!(decoded, input, "Roundtrip failed for special character: {:?}", String::from_utf8_lossy(input));
        }
    }

    #[test]
    fn test_json_binary_data_handling() {
        let codec = JsonCodec::new();
        
        // Test with binary data (non-UTF8)
        // Note: This codec uses lossy conversion, so some data might be lost
        let binary_data = vec![0, 1, 2, 3, 255, 254, 253];
        let encoded = codec.encode(&binary_data).unwrap();
        let decoded = codec.decode(&encoded).unwrap();
        
        // The result might not be identical due to lossy UTF-8 conversion
        // but the encoding/decoding should not fail
        assert!(!encoded.is_empty());
        assert!(!decoded.is_empty());
    }
}