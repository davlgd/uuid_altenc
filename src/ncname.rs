//! UUID-NCName encodings (`draft-taylor-uuid-ncname-04`, referenced from
//! Appendix C of `draft-davis`). Three variants — Base32, Base58,
//! Base64url — each prefixed and (for Base32/Base64) suffixed by a
//! "bookend" character that encodes the UUID version and variant nibble.
//!
//! Each variant has a single canonical form per UUID, exactly matching
//! the Appendix C test vectors:
//!
//!   * UUID-NCName-32  → 26 characters, fully **lowercase** (bookends `a`–`p`).
//!   * UUID-NCName-58  → 23 characters, **uppercase** bookends, mixed-case body.
//!   * UUID-NCName-64  → 22 characters, **uppercase** bookends, mixed-case body.
//!
//! Decoders reject any input that doesn't already match its canonical
//! form so the invariant "two distinct strings never decode to the same
//! UUID" holds. The output is always safe to embed in NCName-restricted
//! grammars: XML, HTML/CSS `id` attributes, etc.
//!
//! The encoder/decoder go through a "shifted" 16-byte representation
//! where the version and variant nibbles are removed from the body and
//! the remaining 120 bits are packed contiguously, leaving the variant
//! byte at `bin[15]`.

use crate::Uuid;
use crate::alphabets::{
    BASE32_STD, BASE58_BTC, BASE64_URL, DEC_BASE32_STD, DEC_BASE58_BTC, DEC_BASE64_URL, NONE,
};
use crate::bitstream::{decode_bits, encode_bits};
use crate::error::DecodeError;
use crate::integer_base::{decode_integer_padded, encode_integer_padded};

const NC32_LEN: usize = 26;
const NC58_LEN: usize = 23;
const NC64_LEN: usize = 22;
const NC58_BODY_LEN: usize = 21;

/// Holder for the (version, variant, shifted-body) triple.
struct Shifted {
    version: u8,
    variant: u8,
    bin: [u8; 16],
}

fn ncname_shifted(uuid: &Uuid) -> Shifted {
    let b = &uuid.bytes;
    let ints = [
        u32::from_be_bytes([b[0], b[1], b[2], b[3]]),
        u32::from_be_bytes([b[4], b[5], b[6], b[7]]),
        u32::from_be_bytes([b[8], b[9], b[10], b[11]]),
        u32::from_be_bytes([b[12], b[13], b[14], b[15]]),
    ];
    let version = ((ints[1] & 0x0000_f000) >> 12) as u8;
    let variant = ((ints[2] & 0xf000_0000) >> 24) as u8;

    let new1 =
        (ints[1] & 0xffff_0000) | ((ints[1] & 0x0000_0fff) << 4) | ((ints[2] & 0x0fff_ffff) >> 24);
    let new2 = ((ints[2] & 0x00ff_ffff) << 8) | (ints[3] >> 24);
    let new3 = (ints[3] << 8) | u32::from(variant);

    let mut bin = [0u8; 16];
    bin[0..4].copy_from_slice(&ints[0].to_be_bytes());
    bin[4..8].copy_from_slice(&new1.to_be_bytes());
    bin[8..12].copy_from_slice(&new2.to_be_bytes());
    bin[12..16].copy_from_slice(&new3.to_be_bytes());
    Shifted { version, variant, bin }
}

fn ncname_unshift(version: u8, variant: u8, bin: &[u8; 16]) -> Uuid {
    let ints = [
        u32::from_be_bytes([bin[0], bin[1], bin[2], bin[3]]),
        u32::from_be_bytes([bin[4], bin[5], bin[6], bin[7]]),
        u32::from_be_bytes([bin[8], bin[9], bin[10], bin[11]]),
        u32::from_be_bytes([bin[12], bin[13], bin[14], bin[15]]),
    ];
    let v_byte = u32::from(variant) << 24;
    let new3 = (ints[3] >> 8) | ((ints[2] & 0xff) << 24);
    let new2 = (ints[2] >> 8) | ((ints[1] & 0xf) << 24) | v_byte;
    let new1 =
        (ints[1] & 0xffff_0000) | (u32::from(version) << 12) | ((ints[1] >> 4) & 0xfff);

    let mut out = [0u8; 16];
    out[0..4].copy_from_slice(&ints[0].to_be_bytes());
    out[4..8].copy_from_slice(&new1.to_be_bytes());
    out[8..12].copy_from_slice(&new2.to_be_bytes());
    out[12..16].copy_from_slice(&new3.to_be_bytes());
    Uuid::from_bytes(out)
}

impl Uuid {
    /// 26-character UUID-NCName-32 (RFC 4648 Base32, lowercase). Best
    /// when the host grammar is case-insensitive.
    #[must_use]
    pub fn to_ncname_32(&self) -> String {
        let s = ncname_shifted(self);
        let mut shifted = s.bin;
        shifted[15] >>= 1;
        let mut body = [0u8; NC32_LEN];
        encode_bits(&shifted, BASE32_STD, 5, &mut body);
        // Replace first char with version bookend; trim last char to drop
        // the duplicated low bit (the variant bookend goes at the end of
        // the lowercase string, taken from BASE32_STD too).
        let mut out = [0u8; NC32_LEN];
        out[0] = BASE32_STD[s.version as usize];
        out[1..NC32_LEN].copy_from_slice(&body[..NC32_LEN - 1]);
        // Lowercase the whole string per draft-taylor-04.
        for slot in &mut out {
            slot.make_ascii_lowercase();
        }
        String::from_utf8(out.to_vec()).expect("Base32 alphabet is ASCII")
    }

    /// 23-character UUID-NCName-58. Mixed case; readable when transcribed.
    #[must_use]
    pub fn to_ncname_58(&self) -> String {
        let s = ncname_shifted(self);
        // Standalone Base58btc-padded encoding of the leading 15 bytes.
        let mut buf = [0u8; 32];
        let body = encode_integer_padded(&s.bin[..15], BASE58_BTC, true, &mut buf);
        let mut out = [0u8; NC58_LEN];
        out[0] = BASE32_STD[s.version as usize];
        out[1..=body.len()].copy_from_slice(body);
        for slot in out.iter_mut().take(1 + NC58_BODY_LEN).skip(1 + body.len()) {
            *slot = b'_';
        }
        out[NC58_LEN - 1] = BASE32_STD[(s.variant >> 4) as usize];
        String::from_utf8(out.to_vec()).expect("alphabet is ASCII")
    }

    /// 22-character UUID-NCName-64 (Base64url body, A-P bookends). Most
    /// compact form usable as an XML/HTML id.
    #[must_use]
    pub fn to_ncname_64(&self) -> String {
        let s = ncname_shifted(self);
        let mut shifted = s.bin;
        shifted[15] >>= 2;
        let mut body = [0u8; NC64_LEN];
        encode_bits(&shifted, BASE64_URL, 6, &mut body);
        let mut out = [0u8; NC64_LEN];
        out[0] = BASE32_STD[s.version as usize];
        out[1..NC64_LEN].copy_from_slice(&body[..NC64_LEN - 1]);
        String::from_utf8(out.to_vec()).expect("Base64url alphabet is ASCII")
    }
}

/// Case-sensitive bookend lookup. The encoder always emits uppercase
/// `A`–`P` for NCName-58 and NCName-64, lowercase `a`–`p` for NCName-32;
/// the decoder must reject the other case to avoid aliasing.
///
/// `encoding` is threaded through so the error retains the variant-specific
/// label (`ncname-32` / `ncname-58` / `ncname-64`).
#[inline]
const fn lookup_bookend_strict(
    table: &[u8; 256],
    c: u8,
    allowed: Range,
    encoding: &'static str,
) -> Result<u8, DecodeError> {
    let in_range = match allowed {
        Range::Upper => c >= b'A' && c <= b'P',
        Range::Lower => c >= b'a' && c <= b'p',
    };
    if !in_range {
        return Err(DecodeError::SchemeViolation {
            encoding,
            reason: "version/variant bookend must be in A–P (case must match canonical form)",
        });
    }
    // The range check above guarantees `table[c]` < 16 for the only table
    // ever passed (`DEC_BASE32_STD`), so the lookup cannot fail; we still
    // surface the value to keep the function honest.
    debug_assert!(table[c as usize] < 16);
    Ok(table[c as usize])
}

#[derive(Copy, Clone)]
enum Range {
    Upper,
    Lower,
}

/// Parse a 26-character UUID-NCName-32 string.
///
/// The canonical form is fully lowercase (`a`–`p` bookends, `a`–`z`/`2`–`7`
/// body); any uppercase character is rejected so that two distinct strings
/// can never decode to the same UUID.
pub fn from_ncname_32(input: &str) -> Result<Uuid, DecodeError> {
    let bytes = input.as_bytes();
    if bytes.len() != NC32_LEN {
        return Err(DecodeError::InvalidLength {
            encoding: "ncname-32",
            expected: NC32_LEN,
            got: bytes.len(),
        });
    }
    // Reject any uppercase ASCII in the input — the canonical form is
    // fully lowercase and accepting mixed case would alias 2^N strings to
    // the same UUID.
    if let Some((position, &byte)) = bytes
        .iter()
        .enumerate()
        .find(|&(_, b)| b.is_ascii_uppercase())
    {
        return Err(DecodeError::InvalidCharacter {
            encoding: "ncname-32",
            position,
            byte,
        });
    }
    let version = lookup_bookend_strict(&DEC_BASE32_STD, bytes[0], Range::Lower, "ncname-32")?;
    // Validate the last body character before constructing the body buf.
    let last_idx = NC32_LEN - 1;
    let last_val = DEC_BASE32_STD[bytes[last_idx] as usize];
    if last_val == NONE {
        return Err(DecodeError::InvalidCharacter {
            encoding: "ncname-32",
            position: last_idx,
            byte: bytes[last_idx],
        });
    }
    if last_val >= 16 {
        return Err(DecodeError::SchemeViolation {
            encoding: "ncname-32",
            reason: "last character must be A–P (canonical form)",
        });
    }
    // Body = chars 1..26 + 'A' (a 0 nibble in Base32) → 26 chars total.
    // DEC_BASE32_STD is case-insensitive, and the no-uppercase gate above
    // already guarantees the input is lowercase, so we copy bytes directly.
    let mut body = [b'A'; NC32_LEN];
    body[..NC32_LEN - 1].copy_from_slice(&bytes[1..NC32_LEN]);
    let mut bin = [0u8; 16];
    decode_bits("ncname-32", &body, &DEC_BASE32_STD, 5, &mut bin)?;
    bin[15] <<= 1;
    let variant = bin[15] & 0xf0;
    Ok(ncname_unshift(version, variant, &bin))
}

/// Parse a 23-character UUID-NCName-58 string.
///
/// The canonical form has uppercase bookends (`A`–`P`) and a mixed-case
/// Base58 body (the alphabet itself is case-sensitive). Rejects:
///
/// * lowercase bookends (would alias to the same UUID),
/// * underscore padding that isn't a contiguous trailing run,
/// * any input that, while parseable, doesn't byte-exactly match the
///   canonical encoding of its decoded UUID.
pub fn from_ncname_58(input: &str) -> Result<Uuid, DecodeError> {
    let bytes = input.as_bytes();
    if bytes.len() != NC58_LEN {
        return Err(DecodeError::InvalidLength {
            encoding: "ncname-58",
            expected: NC58_LEN,
            got: bytes.len(),
        });
    }
    let version = lookup_bookend_strict(&DEC_BASE32_STD, bytes[0], Range::Upper, "ncname-58")?;
    let variant_nibble =
        lookup_bookend_strict(&DEC_BASE32_STD, bytes[NC58_LEN - 1], Range::Upper, "ncname-58")?;
    let body = &bytes[1..NC58_LEN - 1];

    // Strip trailing underscores; reject any underscore appearing earlier.
    let trailing = body.iter().rev().take_while(|&&b| b == b'_').count();
    let body_no_pad = &body[..body.len() - trailing];
    if body_no_pad.contains(&b'_') {
        return Err(DecodeError::SchemeViolation {
            encoding: "ncname-58",
            reason: "underscore padding is non-canonical",
        });
    }

    let mut leading = [0u8; 15];
    decode_integer_padded(
        "ncname-58",
        body_no_pad,
        &DEC_BASE58_BTC,
        58,
        &mut leading,
        BASE58_BTC[0],
    )?;
    let mut bin = [0u8; 16];
    bin[..15].copy_from_slice(&leading);
    let variant_byte = variant_nibble << 4;
    bin[15] = variant_byte;
    let uuid = ncname_unshift(version, variant_byte, &bin);

    // Round-trip canonicality check: the underscore-padding rules don't
    // pin the encoding length down by themselves (e.g.
    // `A_____________________A` would otherwise alias the Nil UUID).
    // Byte-exact comparison enforces the single canonical string per
    // UUID — bookend case is already locked to uppercase above.
    if uuid.to_ncname_58().as_bytes() != bytes {
        return Err(DecodeError::SchemeViolation {
            encoding: "ncname-58",
            reason: "input is not the canonical encoding of its UUID",
        });
    }
    Ok(uuid)
}

/// Parse a 22-character UUID-NCName-64 string.
///
/// The canonical form has an uppercase first bookend (`A`–`P`) and a
/// mixed-case Base64url body. Rejects a lowercase first bookend (would
/// alias to the same UUID) and any final body character whose bits the
/// encoder would never emit.
pub fn from_ncname_64(input: &str) -> Result<Uuid, DecodeError> {
    let bytes = input.as_bytes();
    if bytes.len() != NC64_LEN {
        return Err(DecodeError::InvalidLength {
            encoding: "ncname-64",
            expected: NC64_LEN,
            got: bytes.len(),
        });
    }
    let version = lookup_bookend_strict(&DEC_BASE32_STD, bytes[0], Range::Upper, "ncname-64")?;
    let last_idx = NC64_LEN - 1;
    let last_val = DEC_BASE64_URL[bytes[last_idx] as usize];
    if last_val == NONE {
        return Err(DecodeError::InvalidCharacter {
            encoding: "ncname-64",
            position: last_idx,
            byte: bytes[last_idx],
        });
    }
    if last_val >= 16 {
        return Err(DecodeError::SchemeViolation {
            encoding: "ncname-64",
            reason: "last character must be A–P (canonical form)",
        });
    }
    let mut body = [b'A'; NC64_LEN];
    body[..NC64_LEN - 1].copy_from_slice(&bytes[1..]);
    let mut bin = [0u8; 16];
    decode_bits("ncname-64", &body, &DEC_BASE64_URL, 6, &mut bin)?;
    bin[15] <<= 2;
    let variant = bin[15] & 0xf0;
    Ok(ncname_unshift(version, variant, &bin))
}
