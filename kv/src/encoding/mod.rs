use std::collections::HashMap;
use std::fmt;
use crate::error::Error;

pub mod base64_codec;
pub mod hex_codec;
pub mod json_codec;
pub mod format_detector;

pub use base64_codec::Base64Codec;
pub use hex_codec::HexCodec;
pub use json_codec::JsonCodec;
pub use format_detector::{FormatDetector, DetectionResult};

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

/// Detection cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    results: Vec<DetectionResult>,
    timestamp: std::time::Instant,
}

/// Core encoding engine that manages different encoding formats
pub struct EncodingEngine {
    default_format: EncodingFormat,
    codecs: HashMap<EncodingFormat, Box<dyn DataCodec>>,
    detector: FormatDetector,
    detection_cache: HashMap<String, CacheEntry>,
    cache_ttl: std::time::Duration,
    max_cache_size: usize,
}

impl EncodingEngine {
    /// Create a new encoding engine with the specified default format
    pub fn new(default_format: EncodingFormat) -> Self {
        Self {
            default_format,
            codecs: HashMap::new(),
            detector: FormatDetector::new(),
            detection_cache: HashMap::new(),
            cache_ttl: std::time::Duration::from_secs(300), // 5 minutes
            max_cache_size: 1000,
        }
    }

    /// Create a new encoding engine with custom detector settings
    pub fn with_detector(default_format: EncodingFormat, detector: FormatDetector) -> Self {
        Self {
            default_format,
            codecs: HashMap::new(),
            detector,
            detection_cache: HashMap::new(),
            cache_ttl: std::time::Duration::from_secs(300),
            max_cache_size: 1000,
        }
    }

    /// Create a new encoding engine with custom cache settings
    pub fn with_cache_settings(
        default_format: EncodingFormat,
        cache_ttl: std::time::Duration,
        max_cache_size: usize,
    ) -> Self {
        Self {
            default_format,
            codecs: HashMap::new(),
            detector: FormatDetector::new(),
            detection_cache: HashMap::new(),
            cache_ttl,
            max_cache_size,
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

    /// Detect the encoding format of the given data with caching
    pub fn detect(&mut self, data: &str) -> Result<Vec<DetectionResult>, EncodingError> {
        // Check cache first
        if let Some(cached) = self.get_cached_detection(data) {
            return Ok(cached);
        }

        // Perform detection
        let results = self.detector.detect(data);
        
        // Validate results against available codecs
        let validated_results: Vec<DetectionResult> = results
            .into_iter()
            .filter(|result| self.codecs.contains_key(&result.format))
            .collect();

        // Cache the results
        self.cache_detection_results(data, &validated_results);

        Ok(validated_results)
    }

    /// Detect the best matching format for the given data
    pub fn detect_best(&mut self, data: &str) -> Result<Option<DetectionResult>, EncodingError> {
        let results = self.detect(data)?;
        Ok(results.into_iter().next())
    }

    /// Check if the data appears to be encoded in any supported format
    pub fn is_encoded(&mut self, data: &str) -> Result<bool, EncodingError> {
        let results = self.detect(data)?;
        Ok(!results.is_empty())
    }

    /// Get detection statistics for debugging purposes
    pub fn get_detection_stats(&self, data: &str) -> HashMap<EncodingFormat, f32> {
        self.detector.get_detection_stats(data)
    }

    /// Get a list of all supported encoding formats
    pub fn supported_formats(&self) -> Vec<EncodingFormat> {
        self.codecs.keys().copied().collect()
    }

    /// Check if a format is supported
    pub fn is_format_supported(&self, format: EncodingFormat) -> bool {
        self.codecs.contains_key(&format)
    }

    /// Get cached detection results if available and not expired
    fn get_cached_detection(&self, data: &str) -> Option<Vec<DetectionResult>> {
        if let Some(entry) = self.detection_cache.get(data) {
            if entry.timestamp.elapsed() < self.cache_ttl {
                return Some(entry.results.clone());
            }
        }
        None
    }

    /// Cache detection results for future use
    fn cache_detection_results(&mut self, data: &str, results: &[DetectionResult]) {
        // Add new entry first
        let entry = CacheEntry {
            results: results.to_vec(),
            timestamp: std::time::Instant::now(),
        };
        self.detection_cache.insert(data.to_string(), entry);

        // Clean up expired entries and enforce size limit after adding
        self.cleanup_cache();
    }

    /// Clean up expired cache entries and enforce size limits
    fn cleanup_cache(&mut self) {
        let now = std::time::Instant::now();
        
        // Remove expired entries
        self.detection_cache.retain(|_, entry| {
            now.duration_since(entry.timestamp) < self.cache_ttl
        });

        // Enforce size limit by removing oldest entries
        if self.detection_cache.len() > self.max_cache_size {
            let mut entries: Vec<_> = self.detection_cache.iter()
                .map(|(k, v)| (k.clone(), v.timestamp))
                .collect();
            entries.sort_by_key(|(_, timestamp)| *timestamp);
            
            let to_remove = self.detection_cache.len() - self.max_cache_size;
            let keys_to_remove: Vec<_> = entries.iter()
                .take(to_remove)
                .map(|(key, _)| key.clone())
                .collect();
            
            for key in keys_to_remove {
                self.detection_cache.remove(&key);
            }
        }
    }

    /// Clear the detection cache
    pub fn clear_cache(&mut self) {
        self.detection_cache.clear();
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        (self.detection_cache.len(), self.max_cache_size)
    }

    /// Set cache TTL
    pub fn set_cache_ttl(&mut self, ttl: std::time::Duration) {
        self.cache_ttl = ttl;
    }

    /// Set maximum cache size
    pub fn set_max_cache_size(&mut self, size: usize) {
        self.max_cache_size = size;
        self.cleanup_cache();
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
        let codec = Box::new(Base64Codec::new());
        
        engine.register_codec(EncodingFormat::Base64, codec);
        
        // Test with valid Base64
        let results = engine.detect("aGVsbG8=").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].format, EncodingFormat::Base64);
        
        let results = engine.detect("invalid").unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_detection_caching() {
        let mut engine = EncodingEngine::new(EncodingFormat::Base64);
        let codec = Box::new(Base64Codec::new());
        
        engine.register_codec(EncodingFormat::Base64, codec);
        
        // First detection should populate cache
        let results1 = engine.detect("aGVsbG8=").unwrap();
        assert_eq!(results1.len(), 1);
        
        // Second detection should use cache
        let results2 = engine.detect("aGVsbG8=").unwrap();
        assert_eq!(results2.len(), 1);
        assert_eq!(results1[0].format, results2[0].format);
        
        // Cache stats should show one entry
        let (cache_size, _) = engine.get_cache_stats();
        assert_eq!(cache_size, 1);
    }

    #[test]
    fn test_detect_best() {
        let mut engine = EncodingEngine::new(EncodingFormat::Base64);
        let codec = Box::new(Base64Codec::new());
        
        engine.register_codec(EncodingFormat::Base64, codec);
        
        let result = engine.detect_best("aGVsbG8=").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().format, EncodingFormat::Base64);
        
        let result = engine.detect_best("invalid").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_is_encoded() {
        let mut engine = EncodingEngine::new(EncodingFormat::Base64);
        let codec = Box::new(Base64Codec::new());
        
        engine.register_codec(EncodingFormat::Base64, codec);
        
        assert!(engine.is_encoded("aGVsbG8=").unwrap());
        assert!(!engine.is_encoded("invalid").unwrap());
    }

    #[test]
    fn test_cache_management() {
        let mut engine = EncodingEngine::with_cache_settings(
            EncodingFormat::Base64,
            std::time::Duration::from_millis(100),
            2,
        );
        let codec = Box::new(Base64Codec::new());
        
        engine.register_codec(EncodingFormat::Base64, codec);
        
        // Add entries to cache
        engine.detect("aGVsbG8x").unwrap(); // "hello1" in base64
        let (cache_size, _) = engine.get_cache_stats();
        assert_eq!(cache_size, 1);
        
        engine.detect("aGVsbG8y").unwrap(); // "hello2" in base64
        let (cache_size, _) = engine.get_cache_stats();
        assert_eq!(cache_size, 2);
        
        engine.detect("aGVsbG8z").unwrap(); // "hello3" in base64 - Should evict oldest
        let (cache_size, max_size) = engine.get_cache_stats();
        assert_eq!(max_size, 2);
        assert_eq!(cache_size, 2); // Should maintain max size
        
        // Clear cache
        engine.clear_cache();
        let (cache_size, _) = engine.get_cache_stats();
        assert_eq!(cache_size, 0);
    }

    #[test]
    fn test_get_detection_stats() {
        let engine = EncodingEngine::new(EncodingFormat::Base64);
        
        let stats = engine.get_detection_stats("aGVsbG8=");
        assert!(stats.contains_key(&EncodingFormat::Base64));
    }
}