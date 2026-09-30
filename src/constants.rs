use std::collections::HashMap;

/// Total number of ASCII characters (0-127)
pub const ASCII_CHAR_COUNT: usize = 128;

/// Default log value for strings that are too short to score
pub const DEFAULT_LOG_VALUE: f64 = -20.0;

/// Minimum string length for trigram scoring
pub const MINIMUM_STRING_LENGTH: usize = 3;

/// Length-based scoring thresholds
/// Index represents string length, value is the threshold score
/// Strings shorter than 4 characters use threshold of 10.0 (impossible to pass)
pub const NG_THRESHOLDS: [f64; 101] = [
    10.0, 10.0, 10.0, 10.0, -2.71, -3.26, -3.52, -3.84, -4.23, -4.49,        // 0 - 9
    -4.55, -4.74, -4.88, -5.03, -5.06, -5.2, -5.24, -5.29, -5.29, -5.42,     // 10 - 19
    -5.51, -5.52, -5.53, -5.6, -5.6, -5.62, -5.7, -5.7, -5.78, -5.79,        // 20 - 29
    -5.81, -5.81, -5.84, -5.85, -5.86, -5.88, -5.92, -5.92, -5.93, -5.95,     // 30 - 39
    -5.99, -6.0, -6.0, -6.0, -6.02, -6.02, -6.02, -6.05, -6.06, -6.07,       // 40 - 49
    -6.08, -6.1, -6.12, -6.12, -6.13, -6.13, -6.13, -6.13, -6.13, -6.13,     // 50 - 59
    -6.13, -6.15, -6.15, -6.16, -6.16, -6.16, -6.17, -6.19, -6.19, -6.21,    // 60 - 69
    -6.21, -6.21, -6.21, -6.21, -6.21, -6.25, -6.25, -6.25, -6.25, -6.25,    // 70 - 79 
    -6.25, -6.25, -6.26, -6.26, -6.26, -6.26, -6.26, -6.26, -6.26, -6.26,    // 80 - 89
    -6.26, -6.29, -6.29, -6.3, -6.3, -6.3, -6.3, -6.3, -6.3, -6.3, -6.3      // 90 - 100
];

/// Maximum threshold for strings longer than the threshold array
pub const MAX_NG_THRESHOLD: f64 = -6.3;

/// Lower eligible scoring thresholds by the given adjustment.
/// Positive thresholds are eligibility sentinels and must remain unchanged.
pub fn scoring_threshold(threshold: f64, adjustment: f64) -> f64 {
    if threshold < 0.0 { threshold - adjustment } else { threshold }
}

/// Special character markers used in model files
pub const BEGIN_MARKER: &str = "[^]";
pub const END_MARKER: &str = "[$]";

/// ASCII code to description mapping for special characters
/// Returns (short_description, long_description)
pub fn get_ascii_description(code: u8) -> Option<(&'static str, &'static str)> {
    match code {
        0 => Some(("[NUL]", "null")),
        1 => Some(("[SOH]", "start of header")),
        2 => Some(("[STX]", "start of text")),
        3 => Some(("[ETX]", "end of text")),
        4 => Some(("[EOT]", "end of transmission")),
        5 => Some(("[ENQ]", "enquiry")),
        6 => Some(("[ACK]", "acknowledgement")),
        7 => Some(("[BEL]", "bell")),
        8 => Some(("[BS]", "backspace")),
        9 => Some(("[HT]", "horizontal tab")),
        10 => Some(("[LF]", "line feed")),
        11 => Some(("[VT]", "vertical tab")),
        12 => Some(("[FF]", "form feed")),
        13 => Some(("[CR]", "carriage return")),
        14 => Some(("[SO]", "shift out")),
        15 => Some(("[SI]", "shift in")),
        16 => Some(("[DLE]", "data link escape")),
        17 => Some(("[DC1]", "device control 1")),
        18 => Some(("[DC2]", "device control 2")),
        19 => Some(("[DC3]", "device control 3")),
        20 => Some(("[DC4]", "device control 4")),
        21 => Some(("[NAK]", "negative acknowledge")),
        22 => Some(("[SYN]", "synchronous idle")),
        23 => Some(("[ETB]", "end of transmission block")),
        24 => Some(("[CAN]", "cancel")),
        25 => Some(("[EM]", "end of medium")),
        26 => Some(("[SUB]", "substitute")),
        27 => Some(("[ESC]", "escape")),
        28 => Some(("[FS]", "file separator")),
        29 => Some(("[GS]", "group separator")),
        30 => Some(("[RS]", "record separator")),
        31 => Some(("[US]", "unit separator")),
        32 => Some(("[SP]", "space")),
        127 => Some(("[DEL]", "delete")),
        _ => None,
    }
}

/// Create description to ASCII code mapping for parsing model files
pub fn create_description_to_ascii_map() -> HashMap<String, u8> {
    let mut map = HashMap::new();
    
    for code in 0..=127u8 {
        if let Some((desc, _)) = get_ascii_description(code) {
            map.insert(desc.to_string(), code);
        }
    }
    
    map
}

/// Get the threshold for a string of given length
pub fn get_threshold_for_length(length: usize) -> f64 {
    if length >= NG_THRESHOLDS.len() {
        MAX_NG_THRESHOLD
    } else {
        NG_THRESHOLDS[length]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thresholds() {
        assert_eq!(get_threshold_for_length(3), 10.0);
        assert_eq!(get_threshold_for_length(4), -2.71);
        assert_eq!(get_threshold_for_length(100), -6.3);
        assert_eq!(get_threshold_for_length(200), MAX_NG_THRESHOLD);
    }

    #[test]
    fn test_ascii_descriptions() {
        assert_eq!(get_ascii_description(32), Some(("[SP]", "space")));
        assert_eq!(get_ascii_description(9), Some(("[HT]", "horizontal tab")));
        assert_eq!(get_ascii_description(65), None); // Regular 'A'
    }

    #[test]
    fn test_description_mapping() {
        let map = create_description_to_ascii_map();
        assert_eq!(map.get("[SP]"), Some(&32));
        assert_eq!(map.get("[HT]"), Some(&9));
        assert_eq!(map.get("[NUL]"), Some(&0));
    }
}