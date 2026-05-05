//! Base16 (hex). Two flavours, both lossless: the canonical
//! `8-4-4-4-12` form (RFC 9562 §4) and the plain 32-character form
//! (RFC 4648 §8). Both are case-insensitive on input and lowercase on
//! output, matching the RFC examples.

use crate::Uuid;
use crate::alphabets::{BASE16_LOWER, DEC_BASE16, NONE};
use crate::error::DecodeError;

const ENC: &str = "base16";

/// Write the canonical `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` lowercase
/// form into `out` (must be exactly 36 bytes).
pub(crate) fn encode_dash(bytes: &[u8; 16], out: &mut [u8; 36]) {
    let mut j = 0;
    for (i, &b) in bytes.iter().enumerate() {
        if matches!(i, 4 | 6 | 8 | 10) {
            out[j] = b'-';
            j += 1;
        }
        out[j] = BASE16_LOWER[(b >> 4) as usize];
        out[j + 1] = BASE16_LOWER[(b & 0x0f) as usize];
        j += 2;
    }
}

/// Plain 32-character lowercase hex.
pub(crate) fn encode_plain(bytes: &[u8; 16], out: &mut [u8; 32]) {
    for (i, &b) in bytes.iter().enumerate() {
        out[i * 2] = BASE16_LOWER[(b >> 4) as usize];
        out[i * 2 + 1] = BASE16_LOWER[(b & 0x0f) as usize];
    }
}

/// Parse strictly the canonical 36-character
/// `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` form (RFC 9562 §4). Returns
/// `InvalidLength` for anything that isn't exactly 36 bytes.
pub(crate) fn decode_dash(input: &str) -> Result<Uuid, DecodeError> {
    let bytes = input.as_bytes();
    if bytes.len() != 36 {
        return Err(DecodeError::InvalidLength { encoding: ENC, expected: 36, got: bytes.len() });
    }
    decode_36(bytes)
}

/// Parse strictly the plain 32-character hex form (no dashes).
pub(crate) fn decode_plain(input: &str) -> Result<Uuid, DecodeError> {
    let bytes = input.as_bytes();
    if bytes.len() != 32 {
        return Err(DecodeError::InvalidLength { encoding: ENC, expected: 32, got: bytes.len() });
    }
    decode_hex_slice(bytes)
}

/// Parse either a 32-character plain hex string OR a 36-character
/// `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` string. Case-insensitive.
///
/// Strict on alphabet: the only punctuation tolerated is dashes in their
/// canonical positions (8, 13, 18, 23) of a 36-character input. Callers
/// that need to forbid one of the two forms should use `decode_dash` or
/// `decode_plain` directly.
pub(crate) fn decode(input: &str) -> Result<Uuid, DecodeError> {
    let bytes = input.as_bytes();
    let hex: &[u8] = match bytes.len() {
        32 => bytes,
        36 => return decode_36(bytes),
        n => {
            // The lenient decoder accepts 32 or 36; we report the closer
            // of the two so the caller's "off by N" hint is at least
            // useful for the form they almost typed.
            let expected = if n >= 34 { 36 } else { 32 };
            return Err(DecodeError::InvalidLength { encoding: ENC, expected, got: n });
        }
    };
    decode_hex_slice(hex)
}

fn decode_36(bytes: &[u8]) -> Result<Uuid, DecodeError> {
    // Validate dash positions strictly. Other punctuation is rejected.
    for (i, &c) in bytes.iter().enumerate() {
        let is_dash_pos = matches!(i, 8 | 13 | 18 | 23);
        let is_dash = c == b'-';
        if is_dash_pos != is_dash {
            return Err(DecodeError::InvalidCharacter {
                encoding: ENC,
                position: i,
                byte: c,
            });
        }
    }
    let mut buf = [0u8; 32];
    let mut j = 0;
    for (i, &c) in bytes.iter().enumerate() {
        if !matches!(i, 8 | 13 | 18 | 23) {
            buf[j] = c;
            j += 1;
        }
    }
    decode_hex_slice(&buf)
}

fn decode_hex_slice(hex: &[u8]) -> Result<Uuid, DecodeError> {
    debug_assert_eq!(hex.len(), 32);
    let mut out = [0u8; 16];
    for i in 0..16 {
        let hi = DEC_BASE16[hex[i * 2] as usize];
        let lo = DEC_BASE16[hex[i * 2 + 1] as usize];
        if hi == NONE {
            return Err(DecodeError::InvalidCharacter {
                encoding: ENC,
                position: i * 2,
                byte: hex[i * 2],
            });
        }
        if lo == NONE {
            return Err(DecodeError::InvalidCharacter {
                encoding: ENC,
                position: i * 2 + 1,
                byte: hex[i * 2 + 1],
            });
        }
        out[i] = (hi << 4) | lo;
    }
    Ok(Uuid::from_bytes(out))
}
