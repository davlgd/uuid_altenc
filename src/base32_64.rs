//! Public API for the bit-stream encodings (Base32 ×3, Base64 ×3).
//!
//! Every encoding produces a fixed-width string for any UUID:
//!
//!  * Base32 (std/hex/humans) → 26 characters.
//!  * Base64 (std/url/sort)   → 22 characters.

use crate::Uuid;
use crate::alphabets::{
    BASE32_HEX, BASE32_HUMANS, BASE32_STD, BASE64_SORT, BASE64_STD, BASE64_URL, DEC_BASE32_HEX,
    DEC_BASE32_HUMANS, DEC_BASE32_STD, DEC_BASE64_SORT, DEC_BASE64_STD, DEC_BASE64_URL,
};
use crate::bitstream::{decode_bits, encode_bits};
use crate::error::DecodeError;

pub(crate) const BASE32_LEN: usize = 26;
pub(crate) const BASE64_LEN: usize = 22;

#[inline]
fn encode_b32(uuid: &Uuid, alphabet: &[u8]) -> String {
    let mut buf = [0u8; BASE32_LEN];
    encode_bits(&uuid.bytes, alphabet, 5, &mut buf);
    // SAFETY: alphabets are ASCII-only by construction.
    String::from_utf8(buf.to_vec()).expect("alphabet is ASCII")
}

#[inline]
fn encode_b64(uuid: &Uuid, alphabet: &[u8]) -> String {
    let mut buf = [0u8; BASE64_LEN];
    encode_bits(&uuid.bytes, alphabet, 6, &mut buf);
    String::from_utf8(buf.to_vec()).expect("alphabet is ASCII")
}

impl Uuid {
    /// 26-character RFC 4648 §6 standard Base32 encoding.
    #[must_use]
    pub fn to_base32(&self) -> String {
        encode_b32(self, BASE32_STD)
    }
    /// 26-character RFC 4648 §7 base32hex. The alphabet preserves binary
    /// sort order, so two UUIDs sort the same as their encoded text.
    #[must_use]
    pub fn to_base32_hex(&self) -> String {
        encode_b32(self, BASE32_HEX)
    }
    /// 26-character "Base32 for Humans" (Crockford-style, no I/L/O/U).
    #[must_use]
    pub fn to_base32_humans(&self) -> String {
        encode_b32(self, BASE32_HUMANS)
    }
    /// 22-character RFC 4648 §4 standard Base64 encoding (no padding).
    #[must_use]
    pub fn to_base64(&self) -> String {
        encode_b64(self, BASE64_STD)
    }
    /// 22-character RFC 4648 §5 URL-safe Base64 (no padding).
    #[must_use]
    pub fn to_base64_url(&self) -> String {
        encode_b64(self, BASE64_URL)
    }
    /// 22-character lex-sortable Base64 (`-` < digits < uppercase < `_` <
    /// lowercase). Useful as a sortable database key.
    #[must_use]
    pub fn to_base64_sort(&self) -> String {
        encode_b64(self, BASE64_SORT)
    }
}

fn decode_fixed(
    encoding: &'static str,
    input: &str,
    expected: usize,
    table: &[u8; 256],
    bits: u32,
) -> Result<Uuid, DecodeError> {
    if input.len() != expected {
        return Err(DecodeError::InvalidLength { encoding, expected, got: input.len() });
    }
    let mut out = [0u8; 16];
    decode_bits(encoding, input.as_bytes(), table, bits, &mut out)?;
    Ok(Uuid::from_bytes(out))
}

/// Parse a 26-character RFC 4648 §6 Base32 string.
pub fn from_base32(input: &str) -> Result<Uuid, DecodeError> {
    decode_fixed("base32", input, BASE32_LEN, &DEC_BASE32_STD, 5)
}
/// Parse a 26-character base32hex string. Accepts either case.
pub fn from_base32_hex(input: &str) -> Result<Uuid, DecodeError> {
    decode_fixed("base32hex", input, BASE32_LEN, &DEC_BASE32_HEX, 5)
}
/// Parse a 26-character Base32-for-Humans string. Accepts either case.
pub fn from_base32_humans(input: &str) -> Result<Uuid, DecodeError> {
    decode_fixed("base32humans", input, BASE32_LEN, &DEC_BASE32_HUMANS, 5)
}
/// Parse a 22-character standard Base64 string.
pub fn from_base64(input: &str) -> Result<Uuid, DecodeError> {
    decode_fixed("base64", input, BASE64_LEN, &DEC_BASE64_STD, 6)
}
/// Parse a 22-character URL-safe Base64 string.
pub fn from_base64_url(input: &str) -> Result<Uuid, DecodeError> {
    decode_fixed("base64url", input, BASE64_LEN, &DEC_BASE64_URL, 6)
}
/// Parse a 22-character sortable Base64 string.
pub fn from_base64_sort(input: &str) -> Result<Uuid, DecodeError> {
    decode_fixed("base64sort", input, BASE64_LEN, &DEC_BASE64_SORT, 6)
}
