//! # `uuid_altenc` — alternate UUID text encodings
//!
//! Pure-Rust, dependency-free implementation of every encoding listed in
//! [`draft-davis-uuidrev-alt-uuid-encoding-methods-00`][draft] (April 2026).
//! A UUID is just 128 bits — the canonical
//! `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` form is fine for logs but
//! wasteful in URLs, slow as a database key, and breaks XML/HTML
//! identifier grammars (it starts with a digit). This crate exposes one
//! [`Uuid`] type and a family of `to_*` / `from_*` functions covering
//! the formats most often seen in the wild:
//!
//! * Base16 (hex / hex-dash)
//! * Base32 (RFC 4648 standard, base32hex, "humans")
//! * Base64 (RFC 4648 standard, URL-safe, sortable)
//! * Base85 / Z85 (most compact)
//! * Base36, Base52, Base58 (Bitcoin), Base62 (IEEE & sortable)
//! * UUID-NCName-32 / 58 / 64 (XML/HTML id-safe)
//! * Base62id (XML/HTML id-safe, fixed 22-character width)
//!
//! ## Quick start
//!
//! ```
//! use uuid_altenc::{Uuid, from_base58_btc};
//!
//! let uuid: Uuid = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6".parse().unwrap();
//! assert_eq!(uuid.to_base58_btc(), "Xe22UfxT3rxcKJEAfL5373");
//! assert_eq!(uuid.to_base64_url(),  "-B1Prn3sEdCnZQCgyR5r9g");
//! assert_eq!(uuid.to_ncname_64(),   "B-B1Prn3sHQdlAKDJHmv2K");
//!
//! let back = from_base58_btc("Xe22UfxT3rxcKJEAfL5373").unwrap();
//! assert_eq!(back, uuid);
//! ```
//!
//! ## Picking an encoding
//!
//! | Use case                          | Encoding             | Width  | Notes                              |
//! |-----------------------------------|----------------------|-------:|------------------------------------|
//! | URL slug, public identifier       | [`Uuid::to_base64_url`]    | 22     | URL-safe alphabet, no padding      |
//! | Sortable database key             | [`Uuid::to_base32_hex`] / [`Uuid::to_base64_sort`] | 26 / 22 | Lexicographic = binary order |
//! | Compact, no special chars         | [`Uuid::to_base58_btc`]    | ≤22    | No 0/O/I/l, transcription-safe     |
//! | Sortable + no special chars       | [`Uuid::to_base62_sort`]   | ≤22    | Letters and digits only            |
//! | Most compact, symbols OK          | [`Uuid::to_base85_z85`]    | 20     | All 85 printable ASCII             |
//! | XML / HTML / CSS `id="…"`         | [`Uuid::to_ncname_64`]     | 22     | Always starts with A-P             |
//! | Human-friendly, sortable          | [`Uuid::to_base32_humans`] | 26     | Crockford-style, no I/L/O/U        |
//! | Logging, smallest hex             | [`Uuid::to_hex`]           | 32     | RFC 4648 plain hex                 |
//!
//! Every `to_*` has a matching `from_*`. The asymmetry is deliberate: the
//! encoders hang off [`Uuid`] as `to_*` methods (you always have the type
//! you're encoding), while the decoders are free `from_*` functions (the
//! input is a `&str` you don't own and can't extend). The [`parse`] helper
//! auto-detects the encoding by length when that is unambiguous
//! (36/32/26/23/20-char inputs). 22-character inputs are intentionally
//! rejected as ambiguous — Base62id, Base64url and UUID-NCName-64 alphabets
//! overlap, so call the specific decoder when you know the format.
//!
//! ## Conformance & strictness
//!
//! Every test vector in Appendix C of the draft is validated.
//!
//! All decoders are strict: hostile-length inputs are rejected in O(1),
//! malformed characters return [`DecodeError::InvalidCharacter`] with
//! position information, and the `NCName` decoders reject *non-canonical*
//! inputs (two distinct `NCName` strings never decode to the same UUID).
//!
//! [draft]: https://datatracker.ietf.org/doc/draft-davis-uuidrev-alt-uuid-encoding-methods/

#![cfg_attr(docsrs, feature(doc_cfg))]
// Internal panic sites are guarded by class invariants (alphabets are
// ASCII; output buffers are sized at compile time). They cannot fire for
// any user input — the test suite verifies every code path. Documenting
// `# Panics: never` on every public method would just be noise.
#![allow(clippy::missing_panics_doc)]

mod alphabets;
mod base16;
mod base32_64;
mod base62id;
mod base85;
mod bitstream;
mod error;
mod integer_base;
mod integer_encodings;
mod ncname;
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
mod serde_impl;

pub use base32_64::{
    from_base32, from_base32_hex, from_base32_humans, from_base64, from_base64_sort,
    from_base64_url,
};
pub use base62id::from_base62id;
pub use base85::from_base85_z85;
pub use error::DecodeError;
pub use integer_encodings::{
    from_base36, from_base52, from_base58_btc, from_base62_ieee, from_base62_sort,
};
pub use ncname::{from_ncname_32, from_ncname_58, from_ncname_64};

use core::fmt;
use core::str::FromStr;

/// A 128-bit UUID stored in network byte order (byte 0 most significant).
///
/// Cheap to copy; `Hash`, `Ord` and `Eq` derive the bit-wise behaviour
/// (matching `to_u128` numeric order, so v6/v7 UUIDs sort by timestamp).
/// The `Display` implementation emits the canonical
/// `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` form (lowercase).
///
/// # Examples
///
/// ```
/// use uuid_altenc::Uuid;
///
/// // From raw bytes
/// let a = Uuid::from_bytes([0xf8, 0x1d, 0x4f, 0xae, 0x7d, 0xec, 0x11, 0xd0,
///                           0xa7, 0x65, 0x00, 0xa0, 0xc9, 0x1e, 0x6b, 0xf6]);
///
/// // From a u128 (big-endian numeric value)
/// let b = Uuid::from_u128(0xf81d4fae_7dec_11d0_a765_00a0c91e6bf6);
///
/// // From a canonical hex-dash string
/// let c: Uuid = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6".parse().unwrap();
///
/// assert_eq!(a, b);
/// assert_eq!(b, c);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Uuid {
    bytes: [u8; 16],
}

/// Nil UUID — all bits zero (RFC 9562 §5.9).
pub const NIL: Uuid = Uuid { bytes: [0u8; 16] };

/// Max UUID — all bits one (RFC 9562 §5.10).
pub const MAX: Uuid = Uuid { bytes: [0xffu8; 16] };

impl Uuid {
    /// Build a UUID from its 16 raw network-order bytes.
    #[inline]
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self { bytes }
    }

    /// Try to build a UUID from a slice. Errors if the slice length is
    /// not exactly 16.
    pub fn try_from_slice(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 16 {
            return Err(DecodeError::InvalidLength {
                encoding: "raw-bytes",
                expected: 16,
                got: bytes.len(),
            });
        }
        let mut out = [0u8; 16];
        out.copy_from_slice(bytes);
        Ok(Self { bytes: out })
    }

    /// Borrow the raw bytes.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.bytes
    }

    /// Take the raw bytes by value.
    #[inline]
    #[must_use]
    pub const fn into_bytes(self) -> [u8; 16] {
        self.bytes
    }

    /// Build a UUID from a 128-bit unsigned integer. The integer is
    /// interpreted big-endian — the most significant byte goes to byte 0
    /// of the UUID, matching RFC 9562's network byte order.
    #[inline]
    #[must_use]
    pub const fn from_u128(n: u128) -> Self {
        Self { bytes: n.to_be_bytes() }
    }

    /// Convert the UUID to a 128-bit unsigned integer (big-endian
    /// interpretation). Inverse of [`Uuid::from_u128`].
    #[inline]
    #[must_use]
    pub const fn to_u128(self) -> u128 {
        u128::from_be_bytes(self.bytes)
    }

    /// UUID version (high nibble of byte 6 — see RFC 9562 §4). Typical
    /// values: 1 (time/MAC), 4 (random), 6, 7 (time-ordered), 8.
    #[inline]
    #[must_use]
    pub const fn version(&self) -> u8 {
        self.bytes[6] >> 4
    }

    /// UUID variant nibble (high four bits of byte 8). RFC 9562 values
    /// only use the top 1–3 bits; the full nibble is surfaced because
    /// the UUID-NCName algorithm needs it.
    #[inline]
    #[must_use]
    pub const fn variant(&self) -> u8 {
        self.bytes[8] >> 4
    }

    /// Canonical 36-character lowercase hex-dash form (RFC 9562 §4).
    #[must_use]
    pub fn to_hex_dash(&self) -> String {
        let mut buf = [0u8; 36];
        base16::encode_dash(&self.bytes, &mut buf);
        core::str::from_utf8(&buf).expect("hex digits are ASCII").to_owned()
    }

    /// Plain 32-character lowercase hex (no dashes). The strict decoder
    /// counterpart is [`from_hex_plain`]; the lenient [`from_hex`] accepts
    /// both this form and the 36-character dashed form.
    #[must_use]
    pub fn to_hex(&self) -> String {
        let mut buf = [0u8; 32];
        base16::encode_plain(&self.bytes, &mut buf);
        core::str::from_utf8(&buf).expect("hex digits are ASCII").to_owned()
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = [0u8; 36];
        base16::encode_dash(&self.bytes, &mut buf);
        // SAFETY argument: hex digits + ASCII '-' are valid UTF-8.
        f.write_str(core::str::from_utf8(&buf).expect("hex digits are ASCII"))
    }
}

impl fmt::Debug for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Friendlier than `Uuid { bytes: [...] }` — emits `Uuid("xxxx-...")`.
        write!(f, "Uuid({self})")
    }
}

impl From<[u8; 16]> for Uuid {
    #[inline]
    fn from(bytes: [u8; 16]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<Uuid> for [u8; 16] {
    #[inline]
    fn from(uuid: Uuid) -> Self {
        uuid.bytes
    }
}

impl From<u128> for Uuid {
    #[inline]
    fn from(n: u128) -> Self {
        Self::from_u128(n)
    }
}

impl From<Uuid> for u128 {
    #[inline]
    fn from(uuid: Uuid) -> Self {
        uuid.to_u128()
    }
}

impl AsRef<[u8]> for Uuid {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl TryFrom<&[u8]> for Uuid {
    type Error = DecodeError;
    #[inline]
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from_slice(bytes)
    }
}

impl FromStr for Uuid {
    type Err = DecodeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse(s)
    }
}

/// Lenient hex decoder — accepts **either** the 32-character plain hex
/// form **or** the 36-character `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`
/// form. Case-insensitive. Use [`from_hex_dash`] or [`from_hex_plain`]
/// when you need to reject one form.
pub fn from_hex(input: &str) -> Result<Uuid, DecodeError> {
    base16::decode(input)
}

/// Strict canonical hex-dash decoder (RFC 9562 §4): requires exactly the
/// 36-character `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` form. Reject a 32-
/// character plain hex with `InvalidLength`.
pub fn from_hex_dash(input: &str) -> Result<Uuid, DecodeError> {
    base16::decode_dash(input)
}

/// Strict plain-hex decoder: requires exactly 32 hex characters without
/// dashes. Rejects the 36-character dashed form with `InvalidLength`.
pub fn from_hex_plain(input: &str) -> Result<Uuid, DecodeError> {
    base16::decode_plain(input)
}

/// Best-effort autodetect: pick an encoding from the input length and a
/// quick alphabet/case sniff. Conservative — returns
/// [`DecodeError::AmbiguousLength`] when the length doesn't pin down a
/// unique encoding.
///
/// Disambiguation by length:
///
/// * 36 → hex-dash
/// * 32 → plain hex
/// * 26 → UUID-NCName-32 (canonically lowercase, bookends in `a`–`p`),
///   Base32hex (uppercase, contains digits), Base32 (uppercase, no digits)
/// * 25, 24 → Base36 (only encoder reaching these lengths)
/// * 23 → UUID-NCName-58, Base52 or Base36 by alphabet fingerprint
/// * 20 → Z85
///
/// 26- and 23-character inputs are routed by canonical form / alphabet
/// fingerprint. UUID-NCName-32 is lowercase per
/// `draft-taylor-uuid-ncname-04`, while every Base32-family encoder emits
/// uppercase. UUID-NCName-58 uses the Bitcoin Base58 alphabet (no
/// `0`/`O`/`I`/`l`) — when those characters appear, the input is routed
/// to Base52 or Base36 instead. A non-canonical (e.g. lowercased) Base32
/// string that happens to satisfy the `NCName` bookend constraint will
/// be mis-routed; pass it to the explicit decoder instead.
///
/// Other lengths are intentionally rejected as ambiguous:
///
/// * 22 → Base62id, Base64url, UUID-NCName-64 and the variable-width
///   Base58btc / Base62 sort / Base62 IEEE all reach 22 characters with
///   overlapping alphabets — the same string decodes to different UUIDs.
/// * 21 and 1..=19 → only variable-width integer encodings (Base36 /
///   Base52 / Base58btc / Base62 sort / Base62 IEEE) produce these, and
///   their alphabets overlap.
///
/// In all ambiguous cases, call the specific `from_*` once you know the
/// scheme.
///
/// Note: variable-width encoders (Base36, Base52, Base58btc, Base62 IEEE,
/// Base62 sort) drop leading zeros, so values like the Nil UUID encode to
/// 1-character outputs that `parse` cannot route back. Round-trip with
/// the matching `from_*` instead.
pub fn parse(input: &str) -> Result<Uuid, DecodeError> {
    match input.len() {
        36 | 32 => from_hex(input),
        26 => {
            let bytes = input.as_bytes();
            // UUID-NCName-32 is canonically lowercase with bookends in a..p
            // (draft-taylor-uuid-ncname-04 §3.5). When that signature
            // matches, prefer it over the (uppercase) Base32 family.
            let no_uppercase = !bytes.iter().any(u8::is_ascii_uppercase);
            if no_uppercase
                && matches!(bytes[0], b'a'..=b'p')
                && matches!(bytes[25], b'a'..=b'p')
            {
                return from_ncname_32(input);
            }
            // Base32hex contains digits 0-9; standard Base32 does not.
            if bytes.iter().any(u8::is_ascii_digit) {
                from_base32_hex(input)
            } else {
                from_base32(input)
            }
        }
        // Base36 is the only encoder whose output reaches 24 or 25 chars
        // (max 25 for 2^128 - 1; Base52/58/62 plateau at 23/22/22).
        24 | 25 => from_base36(input),
        23 => {
            let bytes = input.as_bytes();
            // UUID-NCName-58's body uses the Bitcoin Base58 alphabet
            // which excludes 0 / O / I / l. If any of those appear, the
            // input cannot be NCName-58.
            let outside_b58 = bytes.iter().any(|&b| matches!(b, b'0' | b'O' | b'I' | b'l'));
            if outside_b58 {
                let has_digit = bytes.iter().any(u8::is_ascii_digit);
                let has_lower = bytes.iter().any(u8::is_ascii_lowercase);
                // Base52 has no digits; Base36 is canonically uppercase
                // + digits. Pick by signal; otherwise stay ambiguous.
                if has_lower && !has_digit {
                    return from_base52(input);
                }
                if has_digit && !has_lower {
                    return from_base36(input);
                }
                return Err(DecodeError::AmbiguousLength { got: 23 });
            }
            from_ncname_58(input)
        }
        20 => from_base85_z85(input),
        n => Err(DecodeError::AmbiguousLength { got: n }),
    }
}

#[cfg(test)]
mod inline_tests {
    use super::*;

    #[test]
    fn nil_round_trip() {
        assert_eq!(NIL.to_hex_dash(), "00000000-0000-0000-0000-000000000000");
        assert_eq!(NIL, parse("00000000-0000-0000-0000-000000000000").unwrap());
    }

    #[test]
    fn max_round_trip() {
        assert_eq!(MAX.to_hex_dash(), "ffffffff-ffff-ffff-ffff-ffffffffffff");
        assert_eq!(MAX, parse("ffffffff-ffff-ffff-ffff-ffffffffffff").unwrap());
    }

    #[test]
    fn display_matches_to_hex_dash() {
        let u: Uuid = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6".parse().unwrap();
        assert_eq!(u.to_string(), u.to_hex_dash());
        assert_eq!(format!("{u:?}"), "Uuid(f81d4fae-7dec-11d0-a765-00a0c91e6bf6)");
    }

    #[test]
    fn version_and_variant() {
        let u: Uuid = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6".parse().unwrap();
        assert_eq!(u.version(), 1);
        assert_eq!(u.variant(), 0xa);
    }

    #[test]
    fn into_and_from_array() {
        let bytes: [u8; 16] = [0x10; 16];
        let u: Uuid = bytes.into();
        let back: [u8; 16] = u.into();
        assert_eq!(back, bytes);
    }

    #[test]
    fn try_from_slice_length_checked() {
        // 16-byte slice → ok; anything else → InvalidLength.
        let bytes = [0xa5u8; 16];
        let u = Uuid::try_from_slice(&bytes).unwrap();
        assert_eq!(u.as_bytes(), &bytes);
        assert!(Uuid::try_from_slice(&bytes[..15]).is_err());
        assert!(Uuid::try_from_slice(&[0u8; 17]).is_err());
        assert!(Uuid::try_from_slice(&[]).is_err());
    }
}
