//! Cyrillic script (Russian) language analysis and validation
//!
//! This module provides Cyrillic-specific text analysis, adapted from the
//! trigram approach but optimized for Cyrillic characters and Russian patterns.

use unicode_script::{Script, UnicodeScript};

/// Cyrillic text analyzer
pub struct CyrillicAnalyzer;

impl CyrillicAnalyzer {
    /// Check if a character is Cyrillic
    pub fn is_cyrillic_character(ch: char) -> bool {
        matches!(ch.script(), Script::Cyrillic)
    }

    /// Check if text appears to be valid Cyrillic/Russian
    pub fn is_likely_cyrillic(text: &str) -> bool {
        if text.is_empty() {
            return false;
        }

        let cyrillic_chars: Vec<char> = text.chars().filter(|c| Self::is_cyrillic_character(*c)).collect();
        
        // Must have at least 70% Cyrillic characters (excluding whitespace/punctuation)
        let total_non_space = text.chars().filter(|c| !c.is_whitespace() && !c.is_ascii_punctuation()).count();
        if total_non_space == 0 {
            return false;
        }

        let cyrillic_ratio = cyrillic_chars.len() as f64 / total_non_space as f64;
        cyrillic_ratio >= 0.7 && cyrillic_chars.len() >= 2
    }

    /// Get Cyrillic text statistics
    pub fn get_cyrillic_stats(text: &str) -> CyrillicTextStats {
        let mut cyrillic_chars = 0;
        let mut total_chars = 0;
        let mut unique_chars = std::collections::HashSet::new();
        let mut common_chars = 0;
        let mut vowels = 0;
        let mut consonants = 0;

        for ch in text.chars() {
            if ch.is_whitespace() || ch.is_ascii_punctuation() {
                continue;
            }

            total_chars += 1;
            unique_chars.insert(ch);

            if Self::is_cyrillic_character(ch) {
                cyrillic_chars += 1;
                
                if Self::is_common_cyrillic_char(ch) {
                    common_chars += 1;
                }

                if Self::is_cyrillic_vowel(ch) {
                    vowels += 1;
                } else if Self::is_cyrillic_consonant(ch) {
                    consonants += 1;
                }
            }
        }

        CyrillicTextStats {
            cyrillic_characters: cyrillic_chars,
            total_characters: total_chars,
            unique_characters: unique_chars.len(),
            common_characters: common_chars,
            vowels,
            consonants,
            cyrillic_ratio: if total_chars > 0 { cyrillic_chars as f64 / total_chars as f64 } else { 0.0 },
        }
    }

    /// Check if a character is a commonly used Cyrillic character
    pub fn is_common_cyrillic_char(ch: char) -> bool {
        // Most frequently used Cyrillic characters in Russian
        matches!(ch,
            // Most common Russian letters
            'а' | 'е' | 'и' | 'о' | 'р' | 'н' | 'т' | 'л' | 'с' | 'в' |
            'к' | 'м' | 'п' | 'у' | 'д' | 'я' | 'ы' | 'з' | 'б' | 'г' |
            'ч' | 'й' | 'х' | 'ж' | 'ш' | 'ю' | 'ц' | 'щ' | 'э' | 'ф' |
            'ё' | 'ь' | 'ъ' |
            // Uppercase variants
            'А' | 'Е' | 'И' | 'О' | 'Р' | 'Н' | 'Т' | 'Л' | 'С' | 'В' |
            'К' | 'М' | 'П' | 'У' | 'Д' | 'Я' | 'Ы' | 'З' | 'Б' | 'Г' |
            'Ч' | 'Й' | 'Х' | 'Ж' | 'Ш' | 'Ю' | 'Ц' | 'Щ' | 'Э' | 'Ф' |
            'Ё' | 'Ь' | 'Ъ'
        )
    }

    /// Check if a character is a Cyrillic vowel
    pub fn is_cyrillic_vowel(ch: char) -> bool {
        matches!(ch, 'а' | 'е' | 'ё' | 'и' | 'о' | 'у' | 'ы' | 'э' | 'ю' | 'я' |
                     'А' | 'Е' | 'Ё' | 'И' | 'О' | 'У' | 'Ы' | 'Э' | 'Ю' | 'Я')
    }

    /// Check if a character is a Cyrillic consonant
    pub fn is_cyrillic_consonant(ch: char) -> bool {
        Self::is_cyrillic_character(ch) && !Self::is_cyrillic_vowel(ch) && ch != 'ь' && ch != 'ъ'
    }

    /// Validate Cyrillic text authenticity
    pub fn validate_cyrillic_text(text: &str) -> bool {
        if text.trim().is_empty() {
            return false;
        }

        let stats = Self::get_cyrillic_stats(text);
        
        // Validation criteria:
        // 1. Must have at least 2 Cyrillic characters
        // 2. Cyrillic characters should make up at least 70% of non-whitespace text
        // 3. Should have both vowels and consonants (like natural language)
        // 4. Should have some common characters
        
        if stats.cyrillic_characters < 2 {
            return false;
        }

        if stats.cyrillic_ratio < 0.7 {
            return false;
        }

        // Should have both vowels and consonants for natural text
        if stats.cyrillic_characters > 3 && (stats.vowels == 0 || stats.consonants == 0) {
            return false;
        }

        // Should have some common characters
        if stats.cyrillic_characters > 4 && stats.common_characters == 0 {
            return false;
        }

        // Check for reasonable character diversity
        let diversity_ratio = stats.unique_characters as f64 / stats.total_characters as f64;
        if stats.total_characters > 10 && diversity_ratio < 0.25 {
            return false; // Too repetitive
        }

        true
    }

    /// Score Cyrillic text using adapted trigram-like approach
    pub fn score_cyrillic_text(text: &str) -> f64 {
        if text.trim().is_empty() {
            return -20.0;
        }

        let stats = Self::get_cyrillic_stats(text);
        
        if stats.cyrillic_characters == 0 {
            return -20.0;
        }

        let mut score = 0.0;

        // Base score for having Cyrillic characters
        score += stats.cyrillic_ratio * 6.0;

        // Bonus for common characters (absolute count + ratio)
        if stats.cyrillic_characters > 0 {
            // Give bonus for having common characters at all
            score += (stats.common_characters.min(6) as f64) * 0.4;
            // Additional ratio-based bonus, but capped to avoid penalizing longer text
            let common_ratio = stats.common_characters as f64 / stats.cyrillic_characters as f64;
            score += common_ratio.min(0.8) * 3.0;
        }

        // Vowel-consonant balance bonus (Russian typically has good balance)
        if stats.vowels > 0 && stats.consonants > 0 {
            let total_letters = stats.vowels + stats.consonants;
            let vowel_ratio = stats.vowels as f64 / total_letters as f64;
            // Russian vowel frequency is around 42-45%
            let balance_score = 1.0 - (vowel_ratio - 0.43).abs() * 2.0;
            score += balance_score.max(0.0) * 2.0;
        }

        // Length bonus (reward longer meaningful text)
        if stats.cyrillic_characters >= 4 {
            score += 1.5;
        }
        if stats.cyrillic_characters >= 8 {
            score += 1.0; // Additional bonus for longer text
        }

        // Diversity bonus (but not penalizing longer text too much)
        if stats.total_characters > 0 {
            let diversity = stats.unique_characters as f64 / stats.total_characters as f64;
            // Cap the diversity requirement to avoid penalizing longer text
            score += diversity.min(0.9) * 2.0;
            // Give a small bonus just for having unique characters
            if stats.unique_characters >= 6 {
                score += 0.5;
            }
        }

        // Trigram-like bonus for common patterns
        score += Self::score_cyrillic_patterns(text);

        // Penalty for very short text
        if stats.cyrillic_characters < 3 {
            score -= 1.5;
        }

        // Convert to negative scale (consistent with existing trigram scores)
        -5.0 + score
    }

    /// Score based on common Cyrillic/Russian letter patterns
    fn score_cyrillic_patterns(text: &str) -> f64 {
        let normalized = text.to_lowercase();
        let mut pattern_score: f64 = 0.0;

        // Common Russian bigrams
        let common_bigrams = [
            "ст", "но", "то", "на", "ен", "ко", "ни", "ти", "во", "ов",
            "ер", "ос", "го", "ро", "ль", "ра", "ле", "ри", "ел", "ор"
        ];

        for bigram in &common_bigrams {
            if normalized.contains(bigram) {
                pattern_score += 1.0; // Strong pattern bonus
            }
        }

        // Common Russian trigrams  
        let common_trigrams = [
            "ост", "сто", "про", "при", "ние", "тся", "что", "ово", "его", "тор"
        ];

        for trigram in &common_trigrams {
            if normalized.contains(trigram) {
                pattern_score += 1.5; // Very strong pattern bonus
            }
        }
        
        // Boost for actual Russian words (small vocabulary check)
        let russian_words = ["это", "что", "для", "они", "есть", "его", "ее"];
        for word in &russian_words {
            if normalized == *word {
                pattern_score += 2.0; // Major bonus for real words
                break;
            }
        }

        // Common Russian endings
        let common_endings = [
            "ать", "ить", "еть", "ный", "ная", "ное", "ием", "ого", "его", "ому"
        ];

        for ending in &common_endings {
            if normalized.ends_with(ending) {
                pattern_score += 0.4;
            }
        }

        pattern_score.min(2.0) // Cap the pattern bonus
    }

    /// Normalize Cyrillic text for analysis
    pub fn normalize_cyrillic_text(text: &str) -> String {
        text.to_lowercase().chars().filter(|c| !c.is_whitespace() || *c == ' ').collect()
    }
}

/// Statistics about Cyrillic text content
#[derive(Debug, Clone)]
pub struct CyrillicTextStats {
    /// Number of Cyrillic characters
    pub cyrillic_characters: usize,
    /// Total non-whitespace, non-punctuation characters
    pub total_characters: usize,
    /// Number of unique characters
    pub unique_characters: usize,
    /// Number of common Cyrillic characters
    pub common_characters: usize,
    /// Number of Cyrillic vowels
    pub vowels: usize,
    /// Number of Cyrillic consonants
    pub consonants: usize,
    /// Ratio of Cyrillic characters to total
    pub cyrillic_ratio: f64,
}

impl CyrillicTextStats {
    /// Check if this represents likely valid Cyrillic text
    pub fn is_likely_valid(&self) -> bool {
        self.cyrillic_characters >= 2 && 
        self.cyrillic_ratio >= 0.7 &&
        (self.cyrillic_characters < 4 || (self.vowels > 0 && self.consonants > 0))
    }

    /// Get vowel-to-consonant ratio
    pub fn vowel_consonant_ratio(&self) -> f64 {
        if self.consonants == 0 {
            0.0
        } else {
            self.vowels as f64 / self.consonants as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cyrillic_character_detection() {
        assert!(CyrillicAnalyzer::is_cyrillic_character('а'));
        assert!(CyrillicAnalyzer::is_cyrillic_character('б'));
        assert!(CyrillicAnalyzer::is_cyrillic_character('А'));
        assert!(CyrillicAnalyzer::is_cyrillic_character('Я'));
        assert!(!CyrillicAnalyzer::is_cyrillic_character('a'));
        assert!(!CyrillicAnalyzer::is_cyrillic_character('1'));
    }

    #[test]
    fn test_common_cyrillic_chars() {
        assert!(CyrillicAnalyzer::is_common_cyrillic_char('а'));
        assert!(CyrillicAnalyzer::is_common_cyrillic_char('е'));
        assert!(CyrillicAnalyzer::is_common_cyrillic_char('р'));
        assert!(CyrillicAnalyzer::is_common_cyrillic_char('А'));
    }

    #[test]
    fn test_vowel_consonant_detection() {
        assert!(CyrillicAnalyzer::is_cyrillic_vowel('а'));
        assert!(CyrillicAnalyzer::is_cyrillic_vowel('е'));
        assert!(CyrillicAnalyzer::is_cyrillic_vowel('А'));
        assert!(!CyrillicAnalyzer::is_cyrillic_vowel('б'));
        
        assert!(CyrillicAnalyzer::is_cyrillic_consonant('б'));
        assert!(CyrillicAnalyzer::is_cyrillic_consonant('в'));
        assert!(!CyrillicAnalyzer::is_cyrillic_consonant('а'));
        assert!(!CyrillicAnalyzer::is_cyrillic_consonant('ь')); // Soft sign, not consonant
    }

    #[test]
    fn test_cyrillic_detection() {
        assert!(CyrillicAnalyzer::is_likely_cyrillic("привет"));
        assert!(CyrillicAnalyzer::is_likely_cyrillic("Москва"));
        assert!(CyrillicAnalyzer::is_likely_cyrillic("русский"));
        assert!(!CyrillicAnalyzer::is_likely_cyrillic("hello"));
        assert!(!CyrillicAnalyzer::is_likely_cyrillic("hello привет")); // Too much English
    }

    #[test]
    fn test_cyrillic_stats() {
        let stats = CyrillicAnalyzer::get_cyrillic_stats("привет");
        assert_eq!(stats.cyrillic_characters, 6);
        assert!(stats.cyrillic_ratio > 0.9);
        assert!(stats.vowels > 0);
        assert!(stats.consonants > 0);
        assert!(stats.common_characters > 0);
    }

    #[test]
    fn test_cyrillic_validation() {
        assert!(CyrillicAnalyzer::validate_cyrillic_text("привет"));
        assert!(CyrillicAnalyzer::validate_cyrillic_text("Москва"));
        assert!(CyrillicAnalyzer::validate_cyrillic_text("русский язык"));
        assert!(!CyrillicAnalyzer::validate_cyrillic_text("hello"));
        assert!(!CyrillicAnalyzer::validate_cyrillic_text(""));
        assert!(!CyrillicAnalyzer::validate_cyrillic_text("а")); // Too short
    }

    #[test]
    fn test_cyrillic_scoring() {
        let score1 = CyrillicAnalyzer::score_cyrillic_text("привет");
        let score2 = CyrillicAnalyzer::score_cyrillic_text("hello");
        let score3 = CyrillicAnalyzer::score_cyrillic_text("русский язык");
        
        assert!(score1 > score2); // Cyrillic should score better than English
        assert!(score3 > score2); // Longer Cyrillic text should still score above non-Cyrillic
        assert!(score3 > -3.0);   // Should still be a strong Cyrillic score
        assert!(score2 < -10.0);  // Non-Cyrillic should get very low score
    }

    #[test]
    fn test_pattern_scoring() {
        let score1 = CyrillicAnalyzer::score_cyrillic_text("это");
        let score2 = CyrillicAnalyzer::score_cyrillic_text("бвгд"); // No common patterns
        
        assert!(score1 > score2); // Text with common patterns should score better
    }

    #[test]
    fn test_normalization() {
        let normalized = CyrillicAnalyzer::normalize_cyrillic_text("Привет Мир!");
        assert_eq!(normalized, "привет мир!");
    }
}
