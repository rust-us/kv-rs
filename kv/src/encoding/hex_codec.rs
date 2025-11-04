use hex;
use crate::encoding::{DataCodec, EncodingError};

/// Hexadecimal encoding/decoding implementation
pub struct HexCodec;

impl HexCodec {
    /// Create a new Hex codec instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for HexCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl DataCodec for HexCodec {
    fn encode(&self, data: &[u8]) -> Result<String, EncodingError> {
        Ok(hex::encode(data))
    }

    fn decode(&self, encoded: &str) -> Result<Vec<u8>, EncodingError> {
        let trimmed = encoded.trim();
        hex::decode(trimmed)
            .map_err(|e| EncodingError::DecodingFailed(format!("Hex decode error: {}", e)))
    }

    fn can_decode(&self, data: &str) -> bool {
        let trimmed = data.trim();
        
        // Empty string is valid hex
        if trimmed.is_empty() {
            return true;
        }
        
        // Check length (must be even)
        if trimmed.len() % 2 != 0 {
            return false;
        }
        
        // Check for valid hex characters (case insensitive)
        let valid_chars = trimmed.chars().all(|c| c.is_ascii_hexdigit());
        
        if !valid_chars {
            return false;
        }
        
        // Try to decode to verify it's valid hex
        hex::decode(trimmed).is_ok()
    }

    fn format_name(&self) -> &'static str {
        "hex"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_encode() {
        let codec = HexCodec::new();
        
        // Test empty data
        assert_eq!(codec.encode(b"").unwrap(), "");
        
        // Test simple string
        assert_eq!(codec.encode(b"hello").unwrap(), "68656c6c6f");
        
        // Test binary data
        assert_eq!(codec.encode(&[0, 1, 2, 3, 255]).unwrap(), "00010203ff");
        
        // Test longer string
        assert_eq!(codec.encode(b"hello world").unwrap(), "68656c6c6f20776f726c64");
        
        // Test all byte values
        let all_bytes: Vec<u8> = (0..=255).collect();
        let encoded = codec.encode(&all_bytes).unwrap();
        assert_eq!(encoded.len(), 512); // 256 bytes * 2 hex chars per byte
    }

    #[test]
    fn test_hex_decode() {
        let codec = HexCodec::new();
        
        // Test empty data
        assert_eq!(codec.decode("").unwrap(), b"");
        
        // Test simple string
        assert_eq!(codec.decode("68656c6c6f").unwrap(), b"hello");
        
        // Test binary data
        assert_eq!(codec.decode("00010203ff").unwrap(), vec![0, 1, 2, 3, 255]);
        
        // Test longer string
        assert_eq!(codec.decode("68656c6c6f20776f726c64").unwrap(), b"hello world");
        
        // Test with whitespace
        assert_eq!(codec.decode("  68656c6c6f  ").unwrap(), b"hello");
        
        // Test uppercase
        assert_eq!(codec.decode("68656C6C6F").unwrap(), b"hello");
        
        // Test mixed case
        assert_eq!(codec.decode("68656c6C6F").unwrap(), b"hello");
    }

    #[test]
    fn test_hex_decode_invalid() {
        let codec = HexCodec::new();
        
        // Test invalid characters
        assert!(codec.decode("invalid").is_err());
        assert!(codec.decode("68656c6c6g").is_err()); // 'g' is not hex
        assert!(codec.decode("68656c6c6!").is_err()); // '!' is not hex
        
        // Test odd length
        assert!(codec.decode("68656c6c6").is_err());
        assert!(codec.decode("a").is_err());
        assert!(codec.decode("abc").is_err());
    }

    #[test]
    fn test_hex_can_decode() {
        let codec = HexCodec::new();
        
        // Valid hex strings
        assert!(codec.can_decode(""));
        assert!(codec.can_decode("68656c6c6f"));
        assert!(codec.can_decode("00010203ff"));
        assert!(codec.can_decode("68656c6c6f20776f726c64"));
        assert!(codec.can_decode("  68656c6c6f  "));
        assert!(codec.can_decode("68656C6C6F")); // uppercase
        assert!(codec.can_decode("68656c6C6F")); // mixed case
        assert!(codec.can_decode("0123456789abcdef"));
        assert!(codec.can_decode("0123456789ABCDEF"));
        
        // Invalid hex strings
        assert!(!codec.can_decode("invalid"));
        assert!(!codec.can_decode("68656c6c6g"));
        assert!(!codec.can_decode("68656c6c6!"));
        assert!(!codec.can_decode("68656c6c6")); // odd length
        assert!(!codec.can_decode("a"));
        assert!(!codec.can_decode("abc"));
        assert!(!codec.can_decode("hello world"));
    }

    #[test]
    fn test_hex_roundtrip() {
        let codec = HexCodec::new();
        
        let all_bytes = (0..=255).collect::<Vec<u8>>();
        let test_cases = vec![
            b"".as_slice(),
            b"a",
            b"hello",
            b"hello world",
            b"The quick brown fox jumps over the lazy dog",
            &[0, 1, 2, 3, 4, 5, 255, 254, 253],
            &all_bytes,
        ];
        
        for data in test_cases {
            let encoded = codec.encode(data).unwrap();
            let decoded = codec.decode(&encoded).unwrap();
            assert_eq!(decoded, data, "Roundtrip failed for: {:?}", data);
        }
    }

    #[test]
    fn test_hex_format_name() {
        let codec = HexCodec::new();
        assert_eq!(codec.format_name(), "hex");
    }

    #[test]
    fn test_hex_case_insensitive() {
        let codec = HexCodec::new();
        
        let data = b"Hello World!";
        let encoded_lower = codec.encode(data).unwrap();
        let encoded_upper = encoded_lower.to_uppercase();
        
        // Both should decode to the same result
        let decoded_lower = codec.decode(&encoded_lower).unwrap();
        let decoded_upper = codec.decode(&encoded_upper).unwrap();
        
        assert_eq!(decoded_lower, data);
        assert_eq!(decoded_upper, data);
        assert_eq!(decoded_lower, decoded_upper);
    }
}