//! Multi-encoding string extraction for binary files
//!
//! This module provides functionality to extract strings from binary data
//! using various character encodings beyond just ASCII.

use encoding_rs::{Encoding, UTF_8, UTF_16BE, UTF_16LE, WINDOWS_1252, ISO_8859_15};
use crate::{BinaryString, StrangerError};

/// Supported encodings for string extraction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedEncoding {
    /// UTF-8 encoding
    Utf8,
    /// UTF-16 Little Endian
    Utf16Le,
    /// UTF-16 Big Endian
    Utf16Be,
    /// Latin-1 (ISO-8859-1), using Windows-1252 as close approximation
    Latin1,
    /// Latin-9 (ISO-8859-15)
    Latin9,
    /// ASCII (original behavior)
    Ascii,
}

impl SupportedEncoding {
    /// Get all supported encodings
    pub fn all() -> Vec<SupportedEncoding> {
        vec![
            SupportedEncoding::Utf8,
            SupportedEncoding::Utf16Le,
            SupportedEncoding::Utf16Be,
            SupportedEncoding::Latin1,
            SupportedEncoding::Latin9,
            SupportedEncoding::Ascii,
        ]
    }

    /// Get the encoding_rs Encoding for this encoding
    fn to_encoding_rs(&self) -> Option<&'static Encoding> {
        match self {
            SupportedEncoding::Utf8 => Some(UTF_8),
            SupportedEncoding::Utf16Le => Some(UTF_16LE),
            SupportedEncoding::Utf16Be => Some(UTF_16BE),
            SupportedEncoding::Latin1 => Some(WINDOWS_1252), // Close approximation to ISO-8859-1
            SupportedEncoding::Latin9 => Some(ISO_8859_15),
            SupportedEncoding::Ascii => None, // Handle ASCII separately
        }
    }

    /// Get the name of this encoding
    pub fn name(&self) -> &'static str {
        match self {
            SupportedEncoding::Utf8 => "UTF-8",
            SupportedEncoding::Utf16Le => "UTF-16LE",
            SupportedEncoding::Utf16Be => "UTF-16BE",
            SupportedEncoding::Latin1 => "Latin-1",
            SupportedEncoding::Latin9 => "Latin-9",
            SupportedEncoding::Ascii => "ASCII",
        }
    }

    /// Parse encoding from string
    pub fn from_str(s: &str) -> Result<Self, StrangerError> {
        match s.to_lowercase().as_str() {
            "utf8" | "utf-8" => Ok(SupportedEncoding::Utf8),
            "utf16le" | "utf-16le" | "utf16-le" | "utf-16-le" => Ok(SupportedEncoding::Utf16Le),
            "utf16be" | "utf-16be" | "utf16-be" | "utf-16-be" => Ok(SupportedEncoding::Utf16Be),
            "latin1" | "latin-1" | "iso-8859-1" | "windows-1252" => Ok(SupportedEncoding::Latin1),
            "latin9" | "latin-9" | "iso-8859-15" => Ok(SupportedEncoding::Latin9),
            "ascii" => Ok(SupportedEncoding::Ascii),
            _ => Err(StrangerError::InvalidInput(format!("Unsupported encoding: {}", s))),
        }
    }
}

impl std::fmt::Display for SupportedEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Result of multi-encoding string extraction
#[derive(Debug, Clone)]
pub struct MultiEncodingResult {
    /// Strings found with their encoding information
    pub strings: Vec<EncodedString>,
}

/// A string found in binary data with its encoding
#[derive(Debug, Clone, PartialEq)]
pub struct EncodedString {
    /// The actual string content
    pub string: String,
    /// Byte offset in the original file
    pub offset: usize,
    /// Encoding used to decode this string
    pub encoding: SupportedEncoding,
    /// Length in bytes in the original encoding
    pub byte_length: usize,
}

impl From<EncodedString> for BinaryString {
    fn from(encoded_string: EncodedString) -> Self {
        BinaryString {
            string: encoded_string.string,
            offset: encoded_string.offset,
        }
    }
}

/// Multi-encoding string extractor
pub struct MultiEncodingExtractor {
    encodings: Vec<SupportedEncoding>,
    min_length: usize,
    allow_unicode: bool,
}

impl MultiEncodingExtractor {
    /// Create a new extractor with the specified encodings
    pub fn new(encodings: Vec<SupportedEncoding>, min_length: usize) -> Self {
        Self {
            encodings,
            min_length,
            allow_unicode: false,
        }
    }

    /// Preserve printable Unicode candidates for language-aware scoring.
    pub fn with_language_scoring(mut self, enabled: bool) -> Self {
        self.allow_unicode = enabled;
        self
    }

    /// Create an extractor that tries all supported encodings
    pub fn new_all_encodings(min_length: usize) -> Self {
        Self::new(SupportedEncoding::all(), min_length)
    }

    /// Extract strings from binary data using all configured encodings
    pub fn extract_strings(&self, buffer: &[u8]) -> MultiEncodingResult {
        let mut all_strings = Vec::new();

        for &encoding in &self.encodings {
            let strings = self.extract_strings_with_encoding(buffer, encoding);
            all_strings.extend(strings);
        }

        // Remove duplicates based on string content and offset
        all_strings.sort_by(|a, b| {
            a.offset.cmp(&b.offset).then_with(|| a.string.cmp(&b.string))
        });
        all_strings.dedup_by(|a, b| a.offset == b.offset && a.string == b.string);

        MultiEncodingResult {
            strings: all_strings,
        }
    }

    /// Extract strings using a specific encoding
    fn extract_strings_with_encoding(&self, buffer: &[u8], encoding: SupportedEncoding) -> Vec<EncodedString> {
        match encoding {
            SupportedEncoding::Ascii => self.extract_ascii_strings(buffer),
            _ => self.extract_encoded_strings(buffer, encoding),
        }
    }

    /// Extract ASCII strings (original behavior)
    fn extract_ascii_strings(&self, buffer: &[u8]) -> Vec<EncodedString> {
        // Pre-allocate with reasonable capacity to reduce reallocations
        let estimated_strings = buffer.len() / 20; // Estimate based on average string length
        let mut strings = Vec::with_capacity(estimated_strings);
        
        // Pre-allocate string buffer with typical capacity
        let mut current_string = String::with_capacity(64);
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
                if current_string.len() >= self.min_length {
                    let byte_length = current_string.len();
                    let string_to_check = std::mem::take(&mut current_string);
                    if !self.is_garbage_string(&string_to_check) {
                        strings.push(EncodedString {
                            string: string_to_check,
                            offset: string_start_offset,
                            encoding: SupportedEncoding::Ascii,
                            byte_length,
                        });
                    }
                    current_string.reserve(64); // Reserve capacity for next string
                } else {
                    current_string.clear();
                }
            }
        }

        // Don't forget the last string if we hit EOF
        if current_string.len() >= self.min_length && !self.is_garbage_string(&current_string) {
            let byte_length = current_string.len();
            strings.push(EncodedString {
                string: current_string, // Move instead of clone
                offset: string_start_offset,
                encoding: SupportedEncoding::Ascii,
                byte_length,
            });
        }

        strings
    }

    /// Extract strings using encoding_rs for non-ASCII encodings
    fn extract_encoded_strings(&self, buffer: &[u8], encoding: SupportedEncoding) -> Vec<EncodedString> {
        let encoding_rs = match encoding.to_encoding_rs() {
            Some(enc) => enc,
            None => return Vec::new(),
        };

        let mut strings = Vec::new();
        let window_size = match encoding {
            SupportedEncoding::Utf16Le | SupportedEncoding::Utf16Be => 2,
            _ => 1,
        };

        // Try to decode the entire buffer first
        let (decoded, _encoding_used, had_errors) = encoding_rs.decode(buffer);
        
        if !had_errors {
            // If decoding was successful, extract strings from the decoded text
            strings.extend(self.extract_strings_from_text(&decoded, 0, encoding, window_size));
        } else {
            // If there were errors, try sliding window approach for partial extraction
            strings.extend(self.extract_with_sliding_window(buffer, encoding_rs, encoding, window_size));
        }

        strings
    }

    /// Extract strings from already-decoded text
    fn extract_strings_from_text(
        &self,
        text: &str,
        base_offset: usize,
        encoding: SupportedEncoding,
        _bytes_per_char: usize,
    ) -> Vec<EncodedString> {
        let estimated_strings = text.len() / 20;
        let mut strings = Vec::with_capacity(estimated_strings);
        let mut current_string = String::with_capacity(64);
        let mut string_start_offset = 0;
        let mut char_offset = 0;

        for ch in text.chars() {
            if self.is_printable_char(ch) {
                if current_string.is_empty() {
                    string_start_offset = base_offset + char_offset;
                }
                current_string.push(ch);
            } else {
                if current_string.len() >= self.min_length {
                    let byte_length = base_offset + char_offset - string_start_offset;
                    let string_to_check = std::mem::take(&mut current_string);
                    if !self.is_garbage_string(&string_to_check) {
                        strings.push(EncodedString {
                            string: string_to_check,
                            offset: string_start_offset,
                            encoding,
                            byte_length,
                        });
                    }
                    current_string.reserve(64);
                } else {
                    current_string.clear();
                }
            }
            char_offset += match encoding {
                SupportedEncoding::Utf8 => ch.len_utf8(),
                SupportedEncoding::Utf16Le | SupportedEncoding::Utf16Be => ch.len_utf16() * 2,
                _ => 1,
            };
        }

        // Handle the last string
        if current_string.len() >= self.min_length && !self.is_garbage_string(&current_string) {
            let byte_len = base_offset + char_offset - string_start_offset;
            strings.push(EncodedString {
                string: current_string,
                offset: string_start_offset,
                encoding,
                byte_length: byte_len,
            });
        }

        strings
    }

    /// Extract strings using sliding window approach for encodings with errors
    fn extract_with_sliding_window(
        &self,
        buffer: &[u8],
        encoding_rs: &'static Encoding,
        encoding: SupportedEncoding,
        window_size: usize,
    ) -> Vec<EncodedString> {
        let mut strings = Vec::new();
        let chunk_size = 1024; // Process in 1KB chunks

        for chunk_start in (0..buffer.len()).step_by(chunk_size / 2) {
            let chunk_end = (chunk_start + chunk_size).min(buffer.len());
            let chunk = &buffer[chunk_start..chunk_end];

            let (decoded, _encoding_used, _had_errors) = encoding_rs.decode(chunk);
            strings.extend(self.extract_strings_from_text(
                &decoded,
                chunk_start,
                encoding,
                window_size,
            ));
        }

        // Remove duplicates
        strings.sort_by(|a, b| a.offset.cmp(&b.offset).then_with(|| a.string.cmp(&b.string)));
        strings.dedup_by(|a, b| a.offset == b.offset && a.string == b.string);

        strings
    }

    /// Check if a character is printable and should be included in strings
    fn is_printable_char(&self, ch: char) -> bool {
        // Include printable ASCII, whitespace, and most Unicode characters
        // Exclude control characters and private use areas
        if ch.is_ascii() {
            ch.is_ascii_graphic() || ch == ' ' || ch == '\t'
        } else {
            // For non-ASCII, be more restrictive to avoid garbage
            // Exclude replacement characters and other problematic characters
            !ch.is_control() && 
            ch != '\u{FFFD}' && // Unicode replacement character
            !matches!(ch as u32,
                0xFEFF | // BOM
                0xFFFE | 0xFFFF | // Invalid
                0xE000..=0xF8FF | // Private Use Area
                0xF0000..=0xFFFFD | 0x100000..=0x10FFFD // Private Use Planes
            )
        }
    }
    
    /// Check if a string is likely garbage from binary data
    fn is_garbage_string(&self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len();
        if len == 0 { return true; }
        
        let mut replacement_count = 0;
        let mut control_count = 0;
        let mut ascii_printable_count = 0;
        let mut non_ascii_count = 0;
        let mut suspicious_chars = 0;
        
        for &ch in &chars {
            if ch == '\u{FFFD}' { // Unicode replacement character
                replacement_count += 1;
            } else if ch.is_control() && ch != '\t' && ch != '\n' && ch != '\r' {
                control_count += 1;
            } else if ch.is_ascii_graphic() || ch == ' ' {
                ascii_printable_count += 1;
            } else if !ch.is_ascii() {
                non_ascii_count += 1;
                // Flag suspicious character patterns common in garbage
                if matches!(ch as u32, 0x80..=0xFF) { // Extended ASCII range - often garbage
                    suspicious_chars += 1;
                }
            }
        }
        
        let replacement_ratio = replacement_count as f64 / len as f64;
        let control_ratio = control_count as f64 / len as f64; 
        let ascii_ratio = ascii_printable_count as f64 / len as f64;
        let non_ascii_ratio = non_ascii_count as f64 / len as f64;
        let suspicious_ratio = suspicious_chars as f64 / len as f64;
        
        // Consider it garbage if:
        // 1. More than 5% replacement characters
        // 2. More than 15% control characters  
        // 3. Less than 60% ASCII printable characters
        // 4. More than 40% non-ASCII characters (likely binary garbage)
        // 5. More than 20% suspicious extended ASCII characters
        replacement_ratio > 0.05 ||
        control_ratio > 0.15 ||
        (!self.allow_unicode && (ascii_ratio < 0.6 ||
        non_ascii_ratio > 0.4 ||
        suspicious_ratio > 0.2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_encoding_from_str() {
        assert_eq!(SupportedEncoding::from_str("utf-8").unwrap(), SupportedEncoding::Utf8);
        assert_eq!(SupportedEncoding::from_str("UTF8").unwrap(), SupportedEncoding::Utf8);
        assert_eq!(SupportedEncoding::from_str("latin-1").unwrap(), SupportedEncoding::Latin1);
        assert!(SupportedEncoding::from_str("invalid").is_err());
    }

    #[test]
    fn test_ascii_extraction() {
        let extractor = MultiEncodingExtractor::new(vec![SupportedEncoding::Ascii], 4);
        let data = b"Hello\x00World\x01Test";
        let result = extractor.extract_strings(data);
        
        assert_eq!(result.strings.len(), 3);
        assert_eq!(result.strings[0].string, "Hello");
        assert_eq!(result.strings[1].string, "World");
        assert_eq!(result.strings[2].string, "Test");
        assert_eq!(result.strings[0].encoding, SupportedEncoding::Ascii);
    }

    #[test]
    fn test_garbage_string_detection() {
        let extractor = MultiEncodingExtractor::new(vec![SupportedEncoding::Ascii], 4);
        
        // Test cases that should be detected as garbage
        assert!(extractor.is_garbage_string("��'��u�=�˂��0��SP�o�o!f�")); // Unicode replacement chars
        assert!(extractor.is_garbage_string("UNCeñÉ¹ð")); // Mixed ASCII/non-ASCII from Latin-1 garbage
        assert!(extractor.is_garbage_string("\u{80}\u{81}\u{82}\u{83}test")); // Extended ASCII garbage
        assert!(extractor.is_garbage_string("\u{FFFD}\u{FFFD}hello")); // Replacement characters
        assert!(extractor.is_garbage_string("\u{01}\u{02}\u{03}\u{04}abcd")); // Control characters
        assert!(extractor.is_garbage_string("àáâãäåtest")); // Too many non-ASCII chars
        assert!(extractor.is_garbage_string("")); // Empty string
        
        // Test cases that should NOT be detected as garbage
        assert!(!extractor.is_garbage_string("Hello World")); // Pure ASCII
        assert!(!extractor.is_garbage_string("test123")); // ASCII alphanumeric
        assert!(!extractor.is_garbage_string("file.exe")); // ASCII with punctuation
        assert!(!extractor.is_garbage_string("Hello café")); // Mostly ASCII with minimal non-ASCII
        assert!(!extractor.is_garbage_string("test\tdata")); // ASCII with tab
        assert!(!extractor.is_garbage_string("C:\\Program Files")); // Windows path
        assert!(!extractor.is_garbage_string("/usr/bin/test")); // Unix path
    }

    #[test]
    fn test_garbage_filtering_in_extraction() {
        let extractor = MultiEncodingExtractor::new(vec![SupportedEncoding::Ascii], 4);
        
        // Create test data with mix of valid and garbage strings
        let mut test_data = Vec::new();
        test_data.extend_from_slice(b"Hello"); // Valid
        test_data.push(0); // Separator
        test_data.extend_from_slice("UNCeñÉ¹ð".as_bytes()); // Would be garbage from Latin-1
        test_data.push(0); // Separator  
        test_data.extend_from_slice(b"World"); // Valid
        test_data.push(0); // Separator
        test_data.extend_from_slice(&[0x80, 0x81, 0x82, 0x83]); // Garbage
        
        let result = extractor.extract_strings(&test_data);
        
        // Should only get valid strings, garbage filtered out
        let valid_strings: Vec<&str> = result.strings.iter().map(|s| s.string.as_str()).collect();
        assert!(valid_strings.contains(&"Hello"));
        assert!(valid_strings.contains(&"World"));
        
        // Should not contain garbage (this tests ASCII extraction, so Latin-1 chars would be invalid anyway)
        // But this test structure shows how the filtering would work
    }

    #[test]
    fn test_garbage_boundary_cases() {
        let extractor = MultiEncodingExtractor::new(vec![SupportedEncoding::Ascii], 4);
        
        // Test boundary conditions for garbage detection
        assert!(!extractor.is_garbage_string("test")); // Exactly minimum length
        assert!(extractor.is_garbage_string("tés")); // Just under ASCII ratio threshold
        assert!(!extractor.is_garbage_string("testéd")); // Just over ASCII ratio threshold
        
        // Test strings right at the thresholds
        let mostly_ascii = "aaaaaaaaab"; // 90% ASCII
        assert!(!extractor.is_garbage_string(mostly_ascii));
        
        let half_ascii = "aaaaabbbbb"; // 50% ASCII (still valid)
        assert!(!extractor.is_garbage_string(half_ascii));
    }

    #[test]
    fn test_ascii_normalization_issue() {
        // This test demonstrates the issue you identified:
        // A string with non-ASCII chars that becomes valid after normalization
        let extractor = MultiEncodingExtractor::new(vec![SupportedEncoding::Latin1], 4);
        
        // Simulate what happens when binary data is decoded as Latin-1
        // This would create strings like "UNCeñÉ¹ð" which after ASCII normalization becomes "UNCe"
        let garbage_latin1 = "UNCeñÉ¹ð"; // This should be filtered as garbage
        assert!(extractor.is_garbage_string(garbage_latin1));
        
        // But legitimate Latin-1 text should pass (mostly ASCII)
        let valid_latin1 = "café menu"; // Legitimate text with accent
        assert!(!extractor.is_garbage_string(valid_latin1));
    }

    #[test]
    fn test_utf8_extraction() {
        let extractor = MultiEncodingExtractor::new(vec![SupportedEncoding::Utf8], 4);
        let data = "Hello 世界 Test".as_bytes();
        let result = extractor.extract_strings(data);
        
        // Should find strings containing UTF-8 characters
        assert!(result.strings.len() >= 1);
        // Check that we found the entire UTF-8 string or at least a part with Unicode
        let has_unicode = result.strings.iter().any(|s| s.string.contains("世界") || s.string.contains("Hello 世界 Test"));
        if !has_unicode {
            // Debug output to see what we actually found
            println!("Found strings: {:?}", result.strings.iter().map(|s| &s.string).collect::<Vec<_>>());
        }
        assert!(has_unicode, "Should find UTF-8 strings with Unicode characters");
    }

    #[test]
    fn test_end_to_end_garbage_prevention() {
        // This test simulates the exact issue you found:
        // Binary data that when decoded as Latin-1 creates strings that would
        // pass trigram scoring after ASCII normalization, but should be filtered as garbage
        
        let extractor = MultiEncodingExtractor::new(vec![SupportedEncoding::Latin1], 4);
        
        // Create binary data that would produce garbage when decoded as Latin-1
        // This simulates random bytes that happen to decode to a mix of ASCII and Latin-1 chars
        let mut test_data = Vec::new();
        test_data.extend_from_slice(b"Good"); // Valid ASCII
        test_data.push(0); // Separator
        
        // This byte sequence would decode as Latin-1 to something like "UNCeñÉ¹ð"
        // which after ASCII normalization becomes "UNCe" - a valid-looking trigram!
        test_data.extend_from_slice(&[0x55, 0x4E, 0x43, 0x65, 0xF1, 0xC9, 0xB9, 0xF0]); // UNCe + garbage bytes
        test_data.push(0); // Separator
        test_data.extend_from_slice(b"Valid"); // Another valid string
        
        let result = extractor.extract_strings(&test_data);
        
        // Filter out the actual strings we got
        let found_strings: Vec<&str> = result.strings.iter().map(|s| s.string.as_str()).collect();
        
        // Should find the valid strings
        assert!(found_strings.iter().any(|s| s.contains("Good")));
        assert!(found_strings.iter().any(|s| s.contains("Valid")));
        
        // Should NOT find the garbage string that would normalize to "UNCe"
        // The garbage detection should catch strings with high ratios of non-ASCII chars
        let has_mixed_garbage = found_strings.iter().any(|s| {
            s.len() > 4 && // Longer than minimum
            s.chars().any(|c| !c.is_ascii()) && // Contains non-ASCII
            s.chars().filter(|c| c.is_ascii()).count() < s.len() / 2 // Less than 50% ASCII
        });
        assert!(!has_mixed_garbage, "Should not extract mixed ASCII/non-ASCII garbage that would pass trigram scoring after normalization. Found: {:?}", found_strings);
    }
    
    #[test] 
    fn test_legitimate_latin1_vs_garbage() {
        let extractor = MultiEncodingExtractor::new(vec![SupportedEncoding::Latin1], 4);
        
        // Test legitimate Latin-1 text (mostly ASCII with some accents)
        let legitimate = "café menu résumé naïve"; // Real French words with accents
        assert!(!extractor.is_garbage_string(legitimate), "Legitimate Latin-1 text should not be marked as garbage");
        
        // Test garbage that happens to have some ASCII chars
        let garbage = "UNCe\u{F1}\u{C9}\u{B9}\u{F0}\u{E2}\u{80}\u{99}"; // Mixed with random high bytes
        assert!(extractor.is_garbage_string(garbage), "Binary garbage decoded as Latin-1 should be detected");
    }
}