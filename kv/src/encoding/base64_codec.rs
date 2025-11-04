use base64::{Engine as _, engine::general_purpose};
use crate::encoding::{DataCodec, EncodingError};

/// Base64 encoding/decoding implementation
pub struct Base64Codec;

impl Base64Codec {
    /// Create a new Base64 codec instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for Base64Codec {
    fn default() -> Self {
        Self::new()
    }
}

impl DataCodec for Base64Codec {
    fn encode(&self, data: &[u8]) -> Result<String, EncodingError> {
        Ok(general_purpose::STANDARD.encode(data))
    }

    fn decode(&self, encoded: &str) -> Result<Vec<u8>, EncodingError> {
        general_purpose::STANDARD
            .decode(encoded.trim())
            .map_err(|e| EncodingError::DecodingFailed(format!("Base64 decode error: {}", e)))
    }

    fn can_decode(&self, data: &str) -> bool {
        // Check if the string contains only valid Base64 characters
        let trimmed = data.trim();
        
        // Empty string is valid Base64
        if trimmed.is_empty() {
            return true;
        }
        
        // Check length (must be multiple of 4)
        if trimmed.len() % 4 != 0 {
            return false;
        }
        
        // Check for valid Base64 characters
        let valid_chars = trimmed.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='
        });
        
        if !valid_chars {
            return false;
        }
        
        // Check padding rules
        let padding_count = trimmed.chars().rev().take_while(|&c| c == '=').count();
        if padding_count > 2 {
            return false;
        }
        
        // If there's padding, it should only be at the end
        if padding_count > 0 {
            let non_padding_part = &trimmed[..trimmed.len() - padding_count];
            if non_padding_part.contains('=') {
                return false;
            }
        }
        
        // Try to decode to verify it's valid Base64
        general_purpose::STANDARD.decode(trimmed).is_ok()
    }

    fn format_name(&self) -> &'static str {
        "base64"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encode() {
        let codec = Base64Codec::new();
        
        // Test empty data
        assert_eq!(codec.encode(b"").unwrap(), "");
        
        // Test simple string
        assert_eq!(codec.encode(b"hello").unwrap(), "aGVsbG8=");
        
        // Test binary data
        assert_eq!(codec.encode(&[0, 1, 2, 3, 255]).unwrap(), "AAECA/8=");
        
        // Test longer string
        assert_eq!(codec.encode(b"hello world").unwrap(), "aGVsbG8gd29ybGQ=");
    }

    #[test]
    fn test_base64_decode() {
        let codec = Base64Codec::new();
        
        // Test empty data
        assert_eq!(codec.decode("").unwrap(), b"");
        
        // Test simple string
        assert_eq!(codec.decode("aGVsbG8=").unwrap(), b"hello");
        
        // Test binary data
        assert_eq!(codec.decode("AAECA/8=").unwrap(), vec![0, 1, 2, 3, 255]);
        
        // Test longer string
        assert_eq!(codec.decode("aGVsbG8gd29ybGQ=").unwrap(), b"hello world");
        
        // Test with whitespace
        assert_eq!(codec.decode("  aGVsbG8=  ").unwrap(), b"hello");
    }

    #[test]
    fn test_base64_decode_invalid() {
        let codec = Base64Codec::new();
        
        // Test invalid characters
        assert!(codec.decode("invalid!@#").is_err());
        
        // Test invalid length
        assert!(codec.decode("abc").is_err());
        
        // Test invalid padding
        assert!(codec.decode("abc===").is_err());
        
        // Test padding in wrong position
        assert!(codec.decode("a=bc").is_err());
    }

    #[test]
    fn test_base64_can_decode() {
        let codec = Base64Codec::new();
        
        // Valid Base64 strings
        assert!(codec.can_decode(""));
        assert!(codec.can_decode("aGVsbG8="));
        assert!(codec.can_decode("aGVsbG8gd29ybGQ="));
        assert!(codec.can_decode("AAECAv8="));
        assert!(codec.can_decode("  aGVsbG8=  "));
        
        // Invalid Base64 strings
        assert!(!codec.can_decode("invalid!@#"));
        assert!(!codec.can_decode("abc"));
        assert!(!codec.can_decode("abc==="));
        assert!(!codec.can_decode("a=bc"));
        assert!(!codec.can_decode("hello world"));
    }

    #[test]
    fn test_base64_roundtrip() {
        let codec = Base64Codec::new();
        
        let test_cases = vec![
            b"".as_slice(),
            b"a",
            b"hello",
            b"hello world",
            b"The quick brown fox jumps over the lazy dog",
            &[0, 1, 2, 3, 4, 5, 255, 254, 253],
        ];
        
        for data in test_cases {
            let encoded = codec.encode(data).unwrap();
            let decoded = codec.decode(&encoded).unwrap();
            assert_eq!(decoded, data, "Roundtrip failed for: {:?}", data);
        }
    }

    #[test]
    fn test_base64_format_name() {
        let codec = Base64Codec::new();
        assert_eq!(codec.format_name(), "base64");
    }
}