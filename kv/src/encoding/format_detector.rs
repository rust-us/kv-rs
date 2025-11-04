use crate::encoding::{EncodingFormat, EncodingError};
use std::collections::HashMap;

/// Detection result with format and confidence score
#[derive(Debug, Clone, PartialEq)]
pub struct DetectionResult {
    pub format: EncodingFormat,
    pub confidence: f32,
}

impl DetectionResult {
    pub fn new(format: EncodingFormat, confidence: f32) -> Self {
        Self { format, confidence }
    }
}

/// Format detector for automatic encoding format detection
pub struct FormatDetector {
    /// Minimum confidence threshold for detection results
    min_confidence: f32,
}

impl FormatDetector {
    /// Create a new format detector with default settings
    pub fn new() -> Self {
        Self {
            min_confidence: 0.1,
        }
    }

    /// Create a new format detector with custom minimum confidence threshold
    pub fn with_min_confidence(min_confidence: f32) -> Self {
        Self { min_confidence }
    }

    /// Detect the encoding format of the given data
    /// Returns a sorted list of possible formats with confidence scores (highest first)
    pub fn detect(&self, data: &str) -> Vec<DetectionResult> {
        let mut results = Vec::new();
        
        // Detect Base64
        if let Some(confidence) = self.detect_base64(data) {
            if confidence >= self.min_confidence {
                results.push(DetectionResult::new(EncodingFormat::Base64, confidence));
            }
        }
        
        // Detect Hex
        if let Some(confidence) = self.detect_hex(data) {
            if confidence >= self.min_confidence {
                results.push(DetectionResult::new(EncodingFormat::Hex, confidence));
            }
        }
        
        // Detect JSON
        if let Some(confidence) = self.detect_json(data) {
            if confidence >= self.min_confidence {
                results.push(DetectionResult::new(EncodingFormat::Json, confidence));
            }
        }
        
        // Sort by confidence (highest first)
        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        
        results
    }

    /// Detect the most likely format with highest confidence
    pub fn detect_best(&self, data: &str) -> Option<DetectionResult> {
        self.detect(data).into_iter().next()
    }

    /// Check if the data matches any supported format with reasonable confidence
    pub fn is_encoded(&self, data: &str) -> bool {
        !self.detect(data).is_empty()
    }

    /// Get detection statistics for debugging
    pub fn get_detection_stats(&self, data: &str) -> HashMap<EncodingFormat, f32> {
        let mut stats = HashMap::new();
        
        if let Some(confidence) = self.detect_base64(data) {
            stats.insert(EncodingFormat::Base64, confidence);
        }
        
        if let Some(confidence) = self.detect_hex(data) {
            stats.insert(EncodingFormat::Hex, confidence);
        }
        
        if let Some(confidence) = self.detect_json(data) {
            stats.insert(EncodingFormat::Json, confidence);
        }
        
        stats
    }

    /// Detect Base64 format with confidence scoring
    fn detect_base64(&self, data: &str) -> Option<f32> {
        let trimmed = data.trim();
        
        // Empty string is valid Base64 but low confidence
        if trimmed.is_empty() {
            return Some(0.1);
        }
        
        let mut confidence: f32 = 0.0;
        
        // Check length (must be multiple of 4)
        if trimmed.len() % 4 != 0 {
            return None;
        }
        confidence += 0.2;
        
        // Check for valid Base64 characters
        let valid_chars = trimmed.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='
        });
        
        if !valid_chars {
            return None;
        }
        confidence += 0.3;
        
        // Check padding rules
        let padding_count = trimmed.chars().rev().take_while(|&c| c == '=').count();
        if padding_count > 2 {
            return None;
        }
        
        // Proper padding increases confidence
        if padding_count <= 2 {
            confidence += 0.2;
        }
        
        // If there's padding, it should only be at the end
        if padding_count > 0 {
            let non_padding_part = &trimmed[..trimmed.len() - padding_count];
            if non_padding_part.contains('=') {
                return None;
            }
            confidence += 0.1;
        }
        
        // Character distribution analysis
        let char_distribution = self.analyze_base64_char_distribution(trimmed);
        confidence += char_distribution * 0.2;
        
        // Try to decode to verify it's valid Base64
        if base64::Engine::decode(&base64::engine::general_purpose::STANDARD, trimmed).is_ok() {
            confidence += 0.3;
        } else {
            return None;
        }
        
        // Cap confidence at 1.0
        Some(confidence.min(1.0))
    }

    /// Detect Hex format with confidence scoring
    fn detect_hex(&self, data: &str) -> Option<f32> {
        let trimmed = data.trim();
        
        // Empty string is valid hex but very low confidence
        if trimmed.is_empty() {
            return Some(0.05);
        }
        
        let mut confidence: f32 = 0.0;
        
        // Check length (must be even)
        if trimmed.len() % 2 != 0 {
            return None;
        }
        confidence += 0.3;
        
        // Check for valid hex characters (case insensitive)
        let valid_chars = trimmed.chars().all(|c| c.is_ascii_hexdigit());
        
        if !valid_chars {
            return None;
        }
        confidence += 0.4;
        
        // Character distribution analysis for hex
        let char_distribution = self.analyze_hex_char_distribution(trimmed);
        confidence += char_distribution * 0.2;
        
        // Length-based confidence adjustment
        let length_factor = match trimmed.len() {
            2..=8 => 0.1,      // Very short, could be coincidental
            10..=32 => 0.2,    // Reasonable length
            34..=128 => 0.3,   // Good length for encoded data
            _ => 0.1,          // Very long or very short
        };
        confidence += length_factor;
        
        // Try to decode to verify it's valid hex
        if hex::decode(trimmed).is_ok() {
            confidence += 0.2;
        } else {
            return None;
        }
        
        // Cap confidence at 1.0
        Some(confidence.min(1.0))
    }

    /// Detect JSON string format with confidence scoring
    fn detect_json(&self, data: &str) -> Option<f32> {
        let trimmed = data.trim();
        
        // Empty string is not valid JSON
        if trimmed.is_empty() {
            return None;
        }
        
        let mut confidence: f32 = 0.0;
        
        // Must start and end with quotes for JSON string
        if !trimmed.starts_with('"') || !trimmed.ends_with('"') {
            return None;
        }
        confidence += 0.4;
        
        // Must have at least 2 characters (opening and closing quotes)
        if trimmed.len() < 2 {
            return None;
        }
        
        // Check for JSON escape sequences
        let escape_sequences = [r#"\""#, r#"\\"#, r#"\/"#, r#"\b"#, r#"\f"#, r#"\n"#, r#"\r"#, r#"\t"#];
        let has_escapes = escape_sequences.iter().any(|seq| trimmed.contains(seq));
        if has_escapes {
            confidence += 0.2;
        }
        
        // Check for unicode escape sequences
        if trimmed.contains(r#"\u"#) {
            confidence += 0.1;
        }
        
        // Length-based confidence
        let length_factor = match trimmed.len() {
            2 => 0.1,          // Just empty quotes
            3..=20 => 0.2,     // Short string
            21..=100 => 0.3,   // Medium string
            _ => 0.2,          // Long string
        };
        confidence += length_factor;
        
        // Try to parse as JSON string
        if serde_json::from_str::<String>(trimmed).is_ok() {
            confidence += 0.3;
        } else {
            return None;
        }
        
        // Cap confidence at 1.0
        Some(confidence.min(1.0))
    }

    /// Analyze character distribution for Base64 detection
    fn analyze_base64_char_distribution(&self, data: &str) -> f32 {
        if data.is_empty() {
            return 0.0;
        }
        
        let mut char_counts = [0u32; 4]; // [alphanumeric, +, /, =]
        
        for c in data.chars() {
            match c {
                c if c.is_ascii_alphanumeric() => char_counts[0] += 1,
                '+' => char_counts[1] += 1,
                '/' => char_counts[2] += 1,
                '=' => char_counts[3] += 1,
                _ => return 0.0, // Invalid character
            }
        }
        
        let total = char_counts.iter().sum::<u32>() as f32;
        if total == 0.0 {
            return 0.0;
        }
        
        // Good Base64 should have mostly alphanumeric characters
        let alphanumeric_ratio = char_counts[0] as f32 / total;
        let special_ratio = (char_counts[1] + char_counts[2]) as f32 / total;
        let padding_ratio = char_counts[3] as f32 / total;
        
        // Ideal ratios for Base64: mostly alphanumeric, some special chars, minimal padding
        let score = if alphanumeric_ratio > 0.7 && special_ratio < 0.3 && padding_ratio < 0.3 {
            1.0
        } else if alphanumeric_ratio > 0.5 {
            0.7
        } else {
            0.3
        };
        
        score
    }

    /// Analyze character distribution for Hex detection
    fn analyze_hex_char_distribution(&self, data: &str) -> f32 {
        if data.is_empty() {
            return 0.0;
        }
        
        let mut digit_count = 0u32;
        let mut letter_count = 0u32;
        
        for c in data.chars() {
            match c {
                '0'..='9' => digit_count += 1,
                'a'..='f' | 'A'..='F' => letter_count += 1,
                _ => return 0.0, // Invalid character
            }
        }
        
        let total = (digit_count + letter_count) as f32;
        if total == 0.0 {
            return 0.0;
        }
        
        // Good hex should have a reasonable mix of digits and letters
        let digit_ratio = digit_count as f32 / total;
        let letter_ratio = letter_count as f32 / total;
        
        // Score based on distribution - pure digits or pure letters are less likely to be encoded data
        if digit_ratio == 1.0 || letter_ratio == 1.0 {
            0.3 // Could be hex, but suspicious
        } else if digit_ratio > 0.3 && digit_ratio < 0.7 {
            1.0 // Good mix
        } else {
            0.6 // Acceptable mix
        }
    }
}

impl Default for FormatDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detector_new() {
        let detector = FormatDetector::new();
        assert_eq!(detector.min_confidence, 0.1);
    }

    #[test]
    fn test_format_detector_with_min_confidence() {
        let detector = FormatDetector::with_min_confidence(0.5);
        assert_eq!(detector.min_confidence, 0.5);
    }

    #[test]
    fn test_detect_base64() {
        let detector = FormatDetector::new();
        
        // Valid Base64 strings
        let results = detector.detect("aGVsbG8=");
        assert!(!results.is_empty());
        assert_eq!(results[0].format, EncodingFormat::Base64);
        assert!(results[0].confidence > 0.5);
        
        let results = detector.detect("aGVsbG8gd29ybGQ=");
        assert!(!results.is_empty());
        assert_eq!(results[0].format, EncodingFormat::Base64);
        
        // Invalid Base64
        let results = detector.detect("invalid!@#");
        assert!(results.iter().all(|r| r.format != EncodingFormat::Base64));
        
        let results = detector.detect("abc"); // Wrong length
        assert!(results.iter().all(|r| r.format != EncodingFormat::Base64));
    }

    #[test]
    fn test_detect_hex() {
        let detector = FormatDetector::new();
        
        // Valid hex strings
        let results = detector.detect("68656c6c6f");
        assert!(!results.is_empty());
        assert_eq!(results[0].format, EncodingFormat::Hex);
        assert!(results[0].confidence > 0.5);
        
        let results = detector.detect("00010203ff");
        assert!(!results.is_empty());
        assert_eq!(results[0].format, EncodingFormat::Hex);
        
        // Invalid hex
        let results = detector.detect("68656c6c6g"); // Invalid character
        assert!(results.iter().all(|r| r.format != EncodingFormat::Hex));
        
        let results = detector.detect("68656c6c6"); // Odd length
        assert!(results.iter().all(|r| r.format != EncodingFormat::Hex));
    }

    #[test]
    fn test_detect_json() {
        let detector = FormatDetector::new();
        
        // Valid JSON strings
        let results = detector.detect(r#""hello""#);
        assert!(!results.is_empty());
        assert_eq!(results[0].format, EncodingFormat::Json);
        assert!(results[0].confidence > 0.5);
        
        let results = detector.detect(r#""hello\nworld""#);
        assert!(!results.is_empty());
        assert_eq!(results[0].format, EncodingFormat::Json);
        
        // Invalid JSON
        let results = detector.detect("hello"); // No quotes
        assert!(results.iter().all(|r| r.format != EncodingFormat::Json));
        
        let results = detector.detect("123"); // Number, not string
        assert!(results.iter().all(|r| r.format != EncodingFormat::Json));
    }

    #[test]
    fn test_detect_best() {
        let detector = FormatDetector::new();
        
        // Clear Base64
        let result = detector.detect_best("aGVsbG8=");
        assert!(result.is_some());
        assert_eq!(result.unwrap().format, EncodingFormat::Base64);
        
        // Clear hex
        let result = detector.detect_best("68656c6c6f");
        assert!(result.is_some());
        assert_eq!(result.unwrap().format, EncodingFormat::Hex);
        
        // Clear JSON
        let result = detector.detect_best(r#""hello""#);
        assert!(result.is_some());
        assert_eq!(result.unwrap().format, EncodingFormat::Json);
        
        // No clear format
        let result = detector.detect_best("plain text");
        assert!(result.is_none());
    }

    #[test]
    fn test_is_encoded() {
        let detector = FormatDetector::new();
        
        assert!(detector.is_encoded("aGVsbG8="));
        assert!(detector.is_encoded("68656c6c6f"));
        assert!(detector.is_encoded(r#""hello""#));
        assert!(!detector.is_encoded("plain text"));
        
        // Empty string might be detected as base64 with very low confidence
        // but should be filtered out by the default min_confidence threshold
        let results = detector.detect("");
        assert!(results.is_empty() || results.iter().all(|r| r.confidence < 0.2));
    }

    #[test]
    fn test_get_detection_stats() {
        let detector = FormatDetector::new();
        
        let stats = detector.get_detection_stats("aGVsbG8=");
        assert!(stats.contains_key(&EncodingFormat::Base64));
        assert!(stats[&EncodingFormat::Base64] > 0.5);
        
        let stats = detector.get_detection_stats("68656c6c6f");
        assert!(stats.contains_key(&EncodingFormat::Hex));
        assert!(stats[&EncodingFormat::Hex] > 0.5);
        
        let stats = detector.get_detection_stats(r#""hello""#);
        assert!(stats.contains_key(&EncodingFormat::Json));
        assert!(stats[&EncodingFormat::Json] > 0.5);
    }

    #[test]
    fn test_confidence_ordering() {
        let detector = FormatDetector::new();
        
        // Test data that could be multiple formats
        let results = detector.detect("41414141"); // Could be hex or base64
        
        // Results should be sorted by confidence
        for i in 1..results.len() {
            assert!(results[i-1].confidence >= results[i].confidence);
        }
    }

    #[test]
    fn test_min_confidence_threshold() {
        let detector = FormatDetector::with_min_confidence(0.8);
        
        // Low confidence detection should be filtered out
        let results = detector.detect("aa"); // Very short, low confidence
        assert!(results.is_empty() || results.iter().all(|r| r.confidence >= 0.8));
    }

    #[test]
    fn test_empty_string_handling() {
        let detector = FormatDetector::new();
        
        let results = detector.detect("");
        // Empty string might be detected as base64 or hex with very low confidence
        assert!(results.iter().all(|r| r.confidence < 0.2));
    }

    #[test]
    fn test_whitespace_handling() {
        let detector = FormatDetector::new();
        
        // Test with leading/trailing whitespace
        let results1 = detector.detect("  aGVsbG8=  ");
        let results2 = detector.detect("aGVsbG8=");
        
        // Should detect the same format
        if !results1.is_empty() && !results2.is_empty() {
            assert_eq!(results1[0].format, results2[0].format);
        }
    }

    #[test]
    fn test_char_distribution_analysis() {
        let detector = FormatDetector::new();
        
        // Test Base64 character distribution
        let base64_score = detector.analyze_base64_char_distribution("aGVsbG8gd29ybGQ=");
        assert!(base64_score > 0.5);
        
        let invalid_score = detector.analyze_base64_char_distribution("!@#$%^&*");
        assert_eq!(invalid_score, 0.0);
        
        // Test Hex character distribution
        let hex_score = detector.analyze_hex_char_distribution("68656c6c6f20776f726c64");
        assert!(hex_score > 0.3);
        
        let pure_digits = detector.analyze_hex_char_distribution("123456789");
        assert!(pure_digits < 0.5); // Pure digits are less likely to be encoded data
    }
}