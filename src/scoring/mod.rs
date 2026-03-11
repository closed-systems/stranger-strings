//! Modular scoring system for different languages and scripts
//!
//! This module provides a trait-based scoring architecture that allows
//! different scoring algorithms to be used based on the detected language/script.

pub mod trigram;
pub mod chinese;
pub mod arabic;
pub mod cyrillic;

use crate::language::{ScriptType, LanguageDetectionResult, LanguageDetector};
use crate::model::TrigramModel;
use crate::error::StrangerError;
use std::sync::Arc;

/// Trait for scoring string validity based on language-specific criteria
pub trait StringScorer {
    /// Score a string and return (score, threshold, is_valid)
    fn score_string(&self, text: &str) -> ScoringResult;
    
    /// Get the script type this scorer handles
    fn script_type(&self) -> ScriptType;
    
    /// Get a human-readable name for this scorer
    fn name(&self) -> &'static str;
}

/// Result of scoring a string
#[derive(Debug, Clone)]
pub struct ScoringResult {
    /// The calculated score
    pub score: f64,
    /// The threshold for validity
    pub threshold: f64,
    /// Whether the string is considered valid
    pub is_valid: bool,
    /// The script type used for scoring
    pub script_type: ScriptType,
    /// The scorer used
    pub scorer_name: String,
}

impl ScoringResult {
    pub fn new(score: f64, threshold: f64, script_type: ScriptType, scorer_name: String) -> Self {
        Self {
            score,
            threshold,
            is_valid: score > threshold,
            script_type,
            scorer_name,
        }
    }
}

/// Factory for creating appropriate scorers based on language detection
pub struct ScoringFactory {
    language_detector: LanguageDetector,
    trigram_model: Option<Arc<TrigramModel>>,
}

impl ScoringFactory {
    /// Create a new scoring factory
    pub fn new() -> Self {
        Self {
            language_detector: LanguageDetector::new(),
            trigram_model: None,
        }
    }

    /// Create a scoring factory with a trigram model for Latin text
    pub fn with_trigram_model(model: TrigramModel) -> Self {
        Self {
            language_detector: LanguageDetector::new(),
            trigram_model: Some(Arc::new(model)),
        }
    }

    /// Score a string using the appropriate scorer based on detected language
    pub fn score_string(&self, text: &str) -> Result<ScoringResult, StrangerError> {
        // Detect the primary script/language
        let detection = self.language_detector.detect_language(text);
        
        // Select appropriate scorer
        let scorer = self.get_scorer_for_script(detection.primary_script)?;
        
        // Score the string
        Ok(scorer.score_string(text))
    }

    /// Score a string with a specific script type (bypassing auto-detection)
    pub fn score_string_with_script(&self, text: &str, script_type: ScriptType) -> Result<ScoringResult, StrangerError> {
        let scorer = self.get_scorer_for_script(script_type)?;
        Ok(scorer.score_string(text))
    }

    /// Get the appropriate scorer for a script type
    fn get_scorer_for_script(&self, script_type: ScriptType) -> Result<Box<dyn StringScorer + '_>, StrangerError> {
        match script_type {
            ScriptType::Latin => {
                if let Some(model) = &self.trigram_model {
                    Ok(Box::new(trigram::TrigramStringScorer::new(Arc::clone(model))))
                } else {
                    Err(StrangerError::ModelNotLoaded)
                }
            }
            ScriptType::Han => Ok(Box::new(chinese::ChineseStringScorer::new())),
            ScriptType::Arabic => Ok(Box::new(arabic::ArabicStringScorer::new())),
            ScriptType::Cyrillic => Ok(Box::new(cyrillic::CyrillicStringScorer::new())),
            ScriptType::Mixed => {
                // For mixed scripts, use a composite scorer
                Ok(Box::new(MixedScriptScorer::new(self)))
            }
            ScriptType::Unknown => {
                // Unknown script falls back to Latin trigram scoring
                if let Some(model) = &self.trigram_model {
                    Ok(Box::new(trigram::TrigramStringScorer::new(Arc::clone(model))))
                } else {
                    Err(StrangerError::ModelNotLoaded)
                }
            }
        }
    }

    /// Detect language of a string
    pub fn detect_language(&self, text: &str) -> LanguageDetectionResult {
        self.language_detector.detect_language(text)
    }

    /// Check if a trigram model is available
    pub fn has_trigram_model(&self) -> bool {
        self.trigram_model.is_some()
    }
}

impl Default for ScoringFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Scorer for mixed-script text
pub struct MixedScriptScorer<'a> {
    factory: &'a ScoringFactory,
}

impl<'a> MixedScriptScorer<'a> {
    pub fn new(factory: &'a ScoringFactory) -> Self {
        Self { factory }
    }
}

impl<'a> StringScorer for MixedScriptScorer<'a> {
    fn score_string(&self, text: &str) -> ScoringResult {
        // For mixed scripts, try scoring with different scorers and take the best result
        let detection = self.factory.language_detector.detect_language(text);
        
        let mut best_score = ScoringResult::new(-20.0, 10.0, ScriptType::Mixed, "Mixed".to_string());
        
        // Try scoring with different script types based on what we found
        for (&script_type, &count) in &detection.script_distribution {
            if count > 0 && script_type != ScriptType::Mixed && script_type != ScriptType::Unknown {
                if let Ok(scorer) = self.factory.get_scorer_for_script(script_type) {
                    let result = scorer.score_string(text);
                    if result.score > best_score.score {
                        best_score = result;
                    }
                }
            }
        }
        
        best_score
    }

    fn script_type(&self) -> ScriptType {
        ScriptType::Mixed
    }

    fn name(&self) -> &'static str {
        "Mixed Script"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::ScriptType;

    #[test]
    fn test_scoring_factory() {
        let factory = ScoringFactory::new();
        
        // Test language detection
        let detection = factory.detect_language("Hello World");
        assert_eq!(detection.primary_script, ScriptType::Latin);
        
        let detection2 = factory.detect_language("你好");
        assert_eq!(detection2.primary_script, ScriptType::Han);
    }

    #[test]
    fn test_scoring_factory_without_model() {
        let factory = ScoringFactory::new();
        
        // Latin scoring requires a trigram model
        let result = factory.score_string("hello");
        assert!(matches!(result, Err(StrangerError::ModelNotLoaded)));
    }
}
