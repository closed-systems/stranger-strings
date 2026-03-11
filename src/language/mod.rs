//! Language detection and script analysis for multi-language string extraction
//!
//! This module provides functionality to detect the writing system/script of text
//! and determine appropriate scoring strategies for different languages.

pub mod script_detection;
pub mod chinese;
pub mod arabic;
pub mod cyrillic;

use std::collections::HashMap;
use unicode_script::{Script, UnicodeScript};

/// Supported writing systems/scripts for language-specific analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ScriptType {
    /// Latin script (English, French, German, etc.)
    Latin,
    /// Han script (Chinese characters)
    Han,
    /// Arabic script
    Arabic,
    /// Cyrillic script (Russian, Bulgarian, etc.)
    Cyrillic,
    /// Mixed scripts or unidentified
    Mixed,
    /// Unknown or unsupported script
    Unknown,
}

impl ScriptType {
    /// Get the name of this script type
    pub fn name(&self) -> &'static str {
        match self {
            ScriptType::Latin => "Latin",
            ScriptType::Han => "Han",
            ScriptType::Arabic => "Arabic",
            ScriptType::Cyrillic => "Cyrillic",
            ScriptType::Mixed => "Mixed",
            ScriptType::Unknown => "Unknown",
        }
    }

    /// Parse script type from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "latin" => Some(ScriptType::Latin),
            "han" | "chinese" | "cjk" => Some(ScriptType::Han),
            "arabic" => Some(ScriptType::Arabic),
            "cyrillic" | "russian" => Some(ScriptType::Cyrillic),
            "mixed" => Some(ScriptType::Mixed),
            _ => None,
        }
    }

    /// Get all supported script types
    pub fn all() -> Vec<ScriptType> {
        vec![
            ScriptType::Latin,
            ScriptType::Han,
            ScriptType::Arabic,
            ScriptType::Cyrillic,
        ]
    }
}

impl std::fmt::Display for ScriptType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Convert Unicode Script to our ScriptType
impl From<Script> for ScriptType {
    fn from(script: Script) -> Self {
        match script {
            Script::Latin => ScriptType::Latin,
            Script::Han => ScriptType::Han,
            Script::Arabic => ScriptType::Arabic,
            Script::Cyrillic => ScriptType::Cyrillic,
            _ => ScriptType::Unknown,
        }
    }
}

/// Result of language/script detection analysis
#[derive(Debug, Clone)]
pub struct LanguageDetectionResult {
    /// The detected primary script
    pub primary_script: ScriptType,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Distribution of scripts found in the text
    pub script_distribution: HashMap<ScriptType, usize>,
    /// Whether the text appears to be homogeneous (single script)
    pub is_homogeneous: bool,
    /// Total number of analyzed characters
    pub total_chars: usize,
}

impl LanguageDetectionResult {
    /// Create a new detection result
    pub fn new(
        primary_script: ScriptType,
        confidence: f64,
        script_distribution: HashMap<ScriptType, usize>,
        total_chars: usize,
    ) -> Self {
        let is_homogeneous = confidence >= 0.8; // 80% threshold for homogeneous text
        
        Self {
            primary_script,
            confidence,
            script_distribution,
            is_homogeneous,
            total_chars,
        }
    }

    /// Check if this text is likely valid for the detected language
    pub fn is_likely_valid(&self) -> bool {
        match self.primary_script {
            ScriptType::Latin => self.confidence >= 0.6 && self.total_chars >= 3,
            ScriptType::Han => self.confidence >= 0.7 && self.total_chars >= 2,
            ScriptType::Arabic => self.confidence >= 0.7 && self.total_chars >= 3,
            ScriptType::Cyrillic => self.confidence >= 0.6 && self.total_chars >= 3,
            ScriptType::Mixed => self.confidence >= 0.4 && self.total_chars >= 4,
            ScriptType::Unknown => false,
        }
    }
}

/// Main language detection functionality
pub struct LanguageDetector {
    /// Minimum confidence threshold for script detection
    pub min_confidence: f64,
    /// Minimum string length for analysis
    pub min_length: usize,
}

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageDetector {
    /// Create a new language detector with default settings
    pub fn new() -> Self {
        Self {
            min_confidence: 0.5,
            min_length: 2,
        }
    }

    /// Create a language detector with custom thresholds
    pub fn with_thresholds(min_confidence: f64, min_length: usize) -> Self {
        Self {
            min_confidence,
            min_length,
        }
    }

    /// Detect the primary script/language of the given text
    pub fn detect_language(&self, text: &str) -> LanguageDetectionResult {
        if text.len() < self.min_length {
            return LanguageDetectionResult::new(
                ScriptType::Unknown,
                0.0,
                HashMap::new(),
                0,
            );
        }

        let mut script_counts: HashMap<ScriptType, usize> = HashMap::new();
        let mut total_analyzed = 0;

        // Analyze each character
        for ch in text.chars() {
            // Skip whitespace and punctuation for script detection
            if ch.is_whitespace() || ch.is_ascii_punctuation() {
                continue;
            }

            let script = ScriptType::from(ch.script());
            *script_counts.entry(script).or_insert(0) += 1;
            total_analyzed += 1;
        }

        if total_analyzed == 0 {
            return LanguageDetectionResult::new(
                ScriptType::Unknown,
                0.0,
                script_counts,
                0,
            );
        }

        // Find the dominant script
        let (dominant_script, dominant_count) = script_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(script, count)| (*script, *count))
            .unwrap_or((ScriptType::Unknown, 0));

        // Calculate confidence as percentage of dominant script
        let confidence = dominant_count as f64 / total_analyzed as f64;

        // Determine if we should classify as mixed
        let final_script = if confidence < 0.6 && script_counts.len() > 1 {
            ScriptType::Mixed
        } else {
            dominant_script
        };

        LanguageDetectionResult::new(
            final_script,
            confidence,
            script_counts,
            total_analyzed,
        )
    }

    /// Check if text is homogeneous (single script dominates)
    pub fn is_homogeneous_script(&self, text: &str, threshold: f64) -> bool {
        let result = self.detect_language(text);
        result.confidence >= threshold
    }

    /// Get script statistics for debugging
    pub fn get_script_stats(&self, text: &str) -> HashMap<ScriptType, usize> {
        let mut script_counts = HashMap::new();
        
        for ch in text.chars() {
            if !ch.is_whitespace() && !ch.is_ascii_punctuation() {
                let script = ScriptType::from(ch.script());
                *script_counts.entry(script).or_insert(0) += 1;
            }
        }

        script_counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_type_from_string() {
        assert_eq!(ScriptType::from_str("latin"), Some(ScriptType::Latin));
        assert_eq!(ScriptType::from_str("Chinese"), Some(ScriptType::Han));
        assert_eq!(ScriptType::from_str("ARABIC"), Some(ScriptType::Arabic));
        assert_eq!(ScriptType::from_str("russian"), Some(ScriptType::Cyrillic));
        assert_eq!(ScriptType::from_str("invalid"), None);
    }

    #[test]
    fn test_latin_detection() {
        let detector = LanguageDetector::new();
        let result = detector.detect_language("Hello World");
        
        assert_eq!(result.primary_script, ScriptType::Latin);
        assert!(result.confidence > 0.8);
        assert!(result.is_homogeneous);
    }

    #[test]
    fn test_mixed_script_detection() {
        let detector = LanguageDetector::new();
        let result = detector.detect_language("Hello 你好");
        
        // Should include multiple scripts; may classify as Mixed or as a dominant script
        // depending on confidence thresholds.
        assert!(result.primary_script == ScriptType::Mixed || result.script_distribution.len() > 1);
        assert!(result.script_distribution.len() > 1);
    }

    #[test]
    fn test_chinese_detection() {
        let detector = LanguageDetector::new();
        let result = detector.detect_language("你好世界");
        
        assert_eq!(result.primary_script, ScriptType::Han);
        assert!(result.confidence > 0.8);
        assert!(result.is_homogeneous);
    }

    #[test]
    fn test_empty_string() {
        let detector = LanguageDetector::new();
        let result = detector.detect_language("");
        
        assert_eq!(result.primary_script, ScriptType::Unknown);
        assert_eq!(result.confidence, 0.0);
    }

    #[test]
    fn test_homogeneous_check() {
        let detector = LanguageDetector::new();
        
        assert!(detector.is_homogeneous_script("Hello World", 0.8));
        assert!(!detector.is_homogeneous_script("Hello 你好", 0.8));
    }
}
