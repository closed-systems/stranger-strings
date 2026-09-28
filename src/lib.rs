//! Stranger Strings - Extract human-readable strings from binary files using trigram-based scoring
//!
//! This crate provides functionality to analyze strings and determine which are most likely
//! to be meaningful to human analysts. It uses a trigram-based scoring system compatible
//! with Ghidra's string analysis algorithm.
//!
//! # Examples
//!
//! ```
//! use stranger_strings::{StrangerStrings, AnalysisOptions};
//!
//! let mut analyzer = StrangerStrings::new();
//! analyzer.load_model(&AnalysisOptions::default()).unwrap();
//!
//! let result = analyzer.analyze_string("hello world").unwrap();
//! println!("Valid: {}, Score: {:.3}", result.is_valid, result.score);
//! ```

pub mod constants;
pub mod encoding;
pub mod error;
pub mod language;
pub mod model;
pub mod processing;
pub mod scoring;

use std::path::Path;

pub use constants::*;
pub use encoding::{SupportedEncoding, MultiEncodingExtractor, EncodedString, MultiEncodingResult};
pub use error::StrangerError;
pub use language::{ScriptType, LanguageDetectionResult, LanguageDetector};
pub use model::{TrigramModel, ModelParser};
pub use processing::{StringProcessor, StringScorer};
pub use scoring::{StringScorer as NewStringScorer, ScoringResult, ScoringFactory};

/// Options for string analysis
#[derive(Debug, Clone, Default)]
pub struct AnalysisOptions {
    /// Path to a custom .sng model file (takes precedence over model_content)
    pub model_path: Option<String>,
    /// Custom model content; uses embedded StringModel.sng when neither source is set
    pub model_content: Option<String>,
    /// Minimum string length for binary extraction
    pub minimum_length: Option<usize>,
}

/// Result of analyzing a single string
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StringAnalysisResult {
    /// The original string that was analyzed
    pub original_string: String,
    /// The score for this string
    pub score: f64,
    /// The threshold this string must exceed to be considered valid
    pub threshold: f64,
    /// Whether this string is considered valid (score > threshold)
    pub is_valid: bool,
    /// The normalized string used for scoring
    pub normalized_string: String,
    /// Optional file offset where this string was found (for binary analysis)
    pub offset: Option<usize>,
    /// Detected script/language type
    pub detected_script: Option<ScriptType>,
    /// Name of the scorer used
    pub scorer_name: Option<String>,
}

/// String extracted from binary data with its location
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BinaryString {
    /// The extracted string
    pub string: String,
    /// Byte offset in the original file
    pub offset: usize,
}

/// Options for binary file analysis
#[derive(Debug, Clone, Default)]
pub struct BinaryAnalysisOptions {
    /// Minimum string length to extract (default: 4)
    pub min_length: Option<usize>,
    /// Encodings to use for string extraction (default: all supported)
    pub encodings: Option<Vec<SupportedEncoding>>,
    /// Languages/scripts to detect and score (default: auto-detect)
    pub target_languages: Option<Vec<ScriptType>>,
    /// Whether to use language-specific scoring (default: false for backwards compatibility)
    pub use_language_scoring: bool,
}

/// Main analyzer for extracting and scoring strings
pub struct StrangerStrings {
    model: Option<TrigramModel>,
    scorer: Option<StringScorer>,
    scoring_factory: Option<ScoringFactory>,
}

impl StrangerStrings {
    /// Create a new analyzer instance
    pub fn new() -> Self {
        Self {
            model: None,
            scorer: None,
            scoring_factory: None,
        }
    }

    /// Load a custom model from file or string content, or the embedded model by default
    pub fn load_model(&mut self, options: &AnalysisOptions) -> Result<(), StrangerError> {
        let model = if let Some(path) = &options.model_path {
            ModelParser::parse_model_file(Path::new(path))?
        } else if let Some(content) = &options.model_content {
            ModelParser::parse_model_string(content)?
        } else {
            ModelParser::parse_model_string(include_str!("../StringModel.sng"))?
        };

        self.scorer = Some(StringScorer::new(&model));
        self.scoring_factory = Some(ScoringFactory::with_trigram_model(model.clone()));
        self.model = Some(model);
        Ok(())
    }

    /// Analyze a single string and return detailed scoring information
    pub fn analyze_string(&self, candidate_string: &str) -> Result<StringAnalysisResult, StrangerError> {
        self.analyze_string_with_offset(candidate_string, None)
    }

    /// Analyze a single string with an optional file offset
    pub fn analyze_string_with_offset(
        &self,
        candidate_string: &str,
        offset: Option<usize>,
    ) -> Result<StringAnalysisResult, StrangerError> {
        self.analyze_string_with_options(candidate_string, offset, false, None)
    }

    /// Analyze a single string with language detection options
    pub fn analyze_string_with_options(
        &self,
        candidate_string: &str,
        offset: Option<usize>,
        use_language_scoring: bool,
        target_script: Option<ScriptType>,
    ) -> Result<StringAnalysisResult, StrangerError> {
        // If language scoring is requested and we have a scoring factory
        if use_language_scoring {
            if let Some(factory) = &self.scoring_factory {
                let scoring_result = if let Some(script) = target_script {
                    factory.score_string_with_script(candidate_string, script)?
                } else {
                    factory.score_string(candidate_string)?
                };

                return Ok(StringAnalysisResult {
                    original_string: candidate_string.to_string(),
                    score: scoring_result.score,
                    threshold: scoring_result.threshold,
                    is_valid: scoring_result.is_valid,
                    normalized_string: candidate_string.to_string(),
                    offset,
                    detected_script: Some(scoring_result.script_type),
                    scorer_name: Some(scoring_result.scorer_name),
                });
            }
        }

        // Fall back to traditional trigram scoring
        let model = self.model.as_ref().ok_or(StrangerError::ModelNotLoaded)?;
        let scorer = self.scorer.as_ref().ok_or(StrangerError::ModelNotLoaded)?;

        let processed = StringProcessor::process_string(candidate_string, model.is_lowercase_model());
        let (score, threshold) = scorer.score_string_with_model(&processed, model);

        Ok(StringAnalysisResult {
            original_string: processed.original_string,
            score,
            threshold,
            is_valid: score > threshold,
            normalized_string: processed.scored_string,
            offset,
            detected_script: None,
            scorer_name: None,
        })
    }

    /// Analyze multiple strings at once
    pub fn analyze_strings(&self, candidate_strings: &[String]) -> Result<Vec<StringAnalysisResult>, StrangerError> {
        candidate_strings
            .iter()
            .map(|s| self.analyze_string(s))
            .collect()
    }

    /// Get only the valid strings from a list of candidates
    pub fn extract_valid_strings(&self, candidate_strings: &[String]) -> Result<Vec<StringAnalysisResult>, StrangerError> {
        let results = self.analyze_strings(candidate_strings)?;
        Ok(results.into_iter().filter(|r| r.is_valid).collect())
    }

    /// Get information about the loaded model
    pub fn get_model_info(&self) -> Result<(String, bool), StrangerError> {
        let model = self.model.as_ref().ok_or(StrangerError::ModelNotLoaded)?;
        Ok((model.get_model_type().to_string(), model.is_lowercase_model()))
    }

    /// Extract strings from binary data without scoring
    pub fn extract_strings_from_binary(&self, buffer: &[u8], min_length: usize) -> Vec<BinaryString> {
        let mut strings = Vec::new();
        let mut current_string = String::new();
        let mut string_start_offset = 0;

        for (i, &byte) in buffer.iter().enumerate() {
            // Check if byte is printable ASCII (excluding control characters except space and tab)
            if (byte >= 32 && byte <= 126) || byte == 9 {
                if current_string.is_empty() {
                    string_start_offset = i;
                }
                current_string.push(byte as char);
            } else {
                // Non-printable character - end current string if it meets minimum length
                if current_string.len() >= min_length {
                    strings.push(BinaryString {
                        string: current_string.clone(),
                        offset: string_start_offset,
                    });
                }
                current_string.clear();
            }
        }

        // Don't forget the last string if we hit EOF
        if current_string.len() >= min_length {
            strings.push(BinaryString {
                string: current_string,
                offset: string_start_offset,
            });
        }

        strings
    }

    /// Analyze strings extracted from binary data
    pub fn analyze_binary_file(
        &self,
        buffer: &[u8],
        options: &BinaryAnalysisOptions,
    ) -> Result<Vec<StringAnalysisResult>, StrangerError> {
        let min_length = options.min_length.unwrap_or(4);
        let encodings = options.encodings.clone().unwrap_or_else(|| vec![SupportedEncoding::Ascii]);
        
        let extractor = MultiEncodingExtractor::new(encodings, min_length);
        let multi_result = extractor.extract_strings(buffer);

        let mut results = Vec::new();
        for encoded_string in multi_result.strings {
            let result = self.analyze_string_with_options(
                &encoded_string.string, 
                Some(encoded_string.offset),
                options.use_language_scoring,
                None // Auto-detect language
            )?;
            results.push(result);
        }

        Ok(results)
    }

    /// Analyze strings extracted from binary data with multi-encoding support
    pub fn analyze_binary_file_multi_encoding(
        &self,
        buffer: &[u8],
        options: &BinaryAnalysisOptions,
    ) -> Result<Vec<(StringAnalysisResult, SupportedEncoding)>, StrangerError> {
        let min_length = options.min_length.unwrap_or(4);
        let encodings = options.encodings.clone().unwrap_or_else(SupportedEncoding::all);
        
        let extractor = MultiEncodingExtractor::new(encodings, min_length);
        let multi_result = extractor.extract_strings(buffer);

        let mut results = Vec::new();
        for encoded_string in multi_result.strings {
            let result = self.analyze_string_with_options(
                &encoded_string.string, 
                Some(encoded_string.offset),
                options.use_language_scoring,
                None // Auto-detect language
            )?;
            results.push((result, encoded_string.encoding));
        }

        Ok(results)
    }

    /// Enable language detection and scoring (without requiring a trigram model)
    pub fn enable_language_detection(&mut self) -> Result<(), StrangerError> {
        if self.scoring_factory.is_none() {
            self.scoring_factory = Some(ScoringFactory::new());
        }
        Ok(())
    }

    /// Detect the language of a string
    pub fn detect_language(&self, text: &str) -> Result<LanguageDetectionResult, StrangerError> {
        if let Some(factory) = &self.scoring_factory {
            Ok(factory.detect_language(text))
        } else {
            // Create temporary factory for detection
            let factory = ScoringFactory::new();
            Ok(factory.detect_language(text))
        }
    }

    /// Check if language detection is available
    pub fn has_language_detection(&self) -> bool {
        self.scoring_factory.is_some()
    }
}

impl Default for StrangerStrings {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        let analyzer = StrangerStrings::new();
        assert!(analyzer.model.is_none());
        assert!(analyzer.scorer.is_none());
    }
}