//! Public API for the variable-width integer-style encodings:
//! Base36, Base52, Base58btc, Base62 IEEE, Base62 sort.

use crate::Uuid;
use crate::alphabets::{
    BASE36, BASE52, BASE58_BTC, BASE62_IEEE, BASE62_SORT, DEC_BASE36, DEC_BASE52, DEC_BASE58_BTC,
    DEC_BASE62_IEEE, DEC_BASE62_SORT,
};
use crate::error::DecodeError;
use crate::integer_base::{check_integer_input_len, decode_integer, encode_integer};

#[inline]
fn encode(uuid: &Uuid, alphabet: &[u8]) -> String {
    let mut buf = [0u8; 32];
    let written = encode_integer(&uuid.bytes, alphabet, &mut buf).len();
    String::from_utf8(buf[..written].to_vec()).expect("alphabet is ASCII")
}

impl Uuid {
    /// Variable-length Base36 encoding (digits + uppercase). Up to 25
    /// characters; the Nil UUID encodes as `"0"`.
    #[must_use]
    pub fn to_base36(&self) -> String {
        encode(self, BASE36)
    }
    /// Variable-length Base52 encoding (uppercase then lowercase, no
    /// digits). Up to 23 characters; guaranteed to start with a letter.
    #[must_use]
    pub fn to_base52(&self) -> String {
        encode(self, BASE52)
    }
    /// Variable-length Base58 (Bitcoin) encoding. Up to 22 characters.
    /// Drops `0`/`O`/`I`/`l` to reduce transcription errors.
    #[must_use]
    pub fn to_base58_btc(&self) -> String {
        encode(self, BASE58_BTC)
    }
    /// Variable-length Base62 IEEE encoding (uppercase, lowercase,
    /// digits). Up to 22 characters. Follows Table 1's IEEE alphabet
    /// (`A`–`Z`, `a`–`z`, `0`–`9`), so the Nil UUID encodes as `"A"`.
    #[must_use]
    pub fn to_base62_ieee(&self) -> String {
        encode(self, BASE62_IEEE)
    }
    /// Variable-length Base62 sortable encoding (digits, uppercase,
    /// lowercase). Up to 22 characters. Lex order matches numeric order —
    /// useful for time-ordered UUIDs (v6, v7) stored as text.
    #[must_use]
    pub fn to_base62_sort(&self) -> String {
        encode(self, BASE62_SORT)
    }
}

fn decode(
    encoding: &'static str,
    input: &str,
    table: &[u8; 256],
    base: u32,
) -> Result<Uuid, DecodeError> {
    check_integer_input_len(encoding, input.len())?;
    let mut out = [0u8; 16];
    decode_integer(encoding, input.as_bytes(), table, base, &mut out)?;
    Ok(Uuid::from_bytes(out))
}

/// Parse a Base36 string. Accepts either case.
pub fn from_base36(input: &str) -> Result<Uuid, DecodeError> {
    decode("base36", input, &DEC_BASE36, 36)
}
/// Parse a Base52 string. Case-sensitive.
pub fn from_base52(input: &str) -> Result<Uuid, DecodeError> {
    decode("base52", input, &DEC_BASE52, 52)
}
/// Parse a Base58 (Bitcoin) string. Case-sensitive.
pub fn from_base58_btc(input: &str) -> Result<Uuid, DecodeError> {
    decode("base58btc", input, &DEC_BASE58_BTC, 58)
}
/// Parse a Base62 (IEEE alphabet) string. Case-sensitive.
pub fn from_base62_ieee(input: &str) -> Result<Uuid, DecodeError> {
    decode("base62ieee", input, &DEC_BASE62_IEEE, 62)
}
/// Parse a Base62 (sortable alphabet) string. Case-sensitive.
pub fn from_base62_sort(input: &str) -> Result<Uuid, DecodeError> {
    decode("base62sort", input, &DEC_BASE62_SORT, 62)
}
