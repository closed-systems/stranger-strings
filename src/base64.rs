//! Detection of Base64 blocks embedded in binary data.
use base64::alphabet::{STANDARD, URL_SAFE};
use base64::engine::{general_purpose::GeneralPurpose, DecodePaddingMode, GeneralPurposeConfig};
use base64::Engine;

/// Decode candidate blocks once; decoded data is never scanned recursively.
pub(crate) fn decode_blocks(buffer: &[u8]) -> Vec<(usize, Vec<u8>)> {
    let config =
        GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent);
    let standard = GeneralPurpose::new(&STANDARD, config);
    let url_safe = GeneralPurpose::new(&URL_SAFE, config);
    let is_symbol =
        |b: u8| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'/' | b'-' | b'_' | b'=');
    let mut blocks = Vec::new();
    let mut i = 0;
    while i < buffer.len() {
        if !is_symbol(buffer[i]) || buffer[i] == b'=' {
            i += 1;
            continue;
        }
        let start = i;
        let mut candidate = Vec::new();
        loop {
            let line_start = i;
            while i < buffer.len() && is_symbol(buffer[i]) {
                candidate.push(buffer[i]);
                i += 1;
            }
            let line_len = i - line_start;
            // Join common MIME/PEM-style wrapped lines, not arbitrary whitespace.
            if line_len >= 16 && line_len % 4 == 0 && candidate.last() != Some(&b'=') {
                let mut next = i;
                if buffer.get(next) == Some(&b'\r') {
                    next += 1;
                }
                if buffer.get(next) == Some(&b'\n') {
                    next += 1;
                    if buffer
                        .get(next)
                        .is_some_and(|b| is_symbol(*b) && *b != b'=')
                    {
                        i = next;
                        continue;
                    }
                }
            }
            break;
        }
        // Six unpadded characters can hold the default four-character minimum.
        if candidate.len() >= 6 {
            if let Ok(decoded) = standard
                .decode(&candidate)
                .or_else(|_| url_safe.decode(&candidate))
            {
                blocks.push((start, decoded));
            }
        }
    }
    blocks
}
