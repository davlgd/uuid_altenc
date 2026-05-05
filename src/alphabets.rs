//! Alphabets defined by Table 1 of `draft-davis-uuidrev-alt-uuid-encoding-methods-00`.
//!
//! Reverse lookup tables are generated at compile time via `const fn` so
//! decoders pay no runtime initialisation cost.
#![allow(clippy::cast_possible_truncation)] // alphabets ≤ 85 entries

/// Base16 (RFC 9562 §4 / RFC 4648 §8). Lowercase per the RFC examples;
/// the decoder is built case-insensitively so it accepts either case.
pub(crate) const BASE16_LOWER: &[u8; 16] = b"0123456789abcdef";

/// Base32 standard (RFC 4648 §6).
pub(crate) const BASE32_STD: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
/// Base32hex (RFC 4648 §7) — sortable hex extension.
pub(crate) const BASE32_HEX: &[u8; 32] = b"0123456789ABCDEFGHIJKLMNOPQRSTUV";
/// Base32 for Humans (Crockford-style, no I/L/O/U).
pub(crate) const BASE32_HUMANS: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Base36 — digits then uppercase letters.
pub(crate) const BASE36: &[u8; 36] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
/// Base52 — uppercase then lowercase letters (no digits).
pub(crate) const BASE52: &[u8; 52] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
/// Base58 (Bitcoin) — drops 0/O/I/l for human readability.
pub(crate) const BASE58_BTC: &[u8; 58] =
    b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// Base62 IEEE — uppercase, lowercase, digits.
pub(crate) const BASE62_IEEE: &[u8; 62] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
/// Base62 sort — digits first so lex order matches numeric order.
pub(crate) const BASE62_SORT: &[u8; 62] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Base64 standard (RFC 4648 §4).
pub(crate) const BASE64_STD: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
/// Base64url (RFC 4648 §5) — URL-safe variant.
pub(crate) const BASE64_URL: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
/// Base64 sort — `'-' < digits < uppercase < '_' < lowercase`.
pub(crate) const BASE64_SORT: &[u8; 64] =
    b"-0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz";

/// Z85 — `ZeroMQ` Base85 alphabet, 85 printable ASCII characters.
pub(crate) const BASE85_Z85: &[u8; 85] =
    b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";

/// Sentinel value for `reverse_table` slots whose byte is not a member.
pub(crate) const NONE: u8 = 0xff;

/// Builds a 256-entry lookup table mapping each ASCII byte to its position
/// in `alphabet`, or [`NONE`] for non-members. Case-sensitive.
///
/// Every alphabet defined in this module has at most 85 entries, so the
/// `i as u8` cast can never truncate.
pub(crate) const fn reverse_table(alphabet: &[u8]) -> [u8; 256] {
    let mut t = [NONE; 256];
    let mut i = 0;
    while i < alphabet.len() {
        t[alphabet[i] as usize] = i as u8;
        i += 1;
    }
    t
}

/// Like [`reverse_table`] but folds ASCII case so the table accepts either
/// case. Useful for alphabets that only define one case (Base16, Base32
/// std, Base32hex, Base32 humans).
pub(crate) const fn reverse_table_ci(alphabet: &[u8]) -> [u8; 256] {
    let mut t = [NONE; 256];
    let mut i = 0;
    while i < alphabet.len() {
        let c = alphabet[i];
        t[c as usize] = i as u8;
        if c.is_ascii_uppercase() {
            t[(c | 0x20) as usize] = i as u8;
        } else if c.is_ascii_lowercase() {
            t[(c & !0x20) as usize] = i as u8;
        }
        i += 1;
    }
    t
}

pub(crate) const DEC_BASE16: [u8; 256] = reverse_table_ci(BASE16_LOWER);
pub(crate) const DEC_BASE32_STD: [u8; 256] = reverse_table_ci(BASE32_STD);
pub(crate) const DEC_BASE32_HEX: [u8; 256] = reverse_table_ci(BASE32_HEX);
pub(crate) const DEC_BASE32_HUMANS: [u8; 256] = reverse_table_ci(BASE32_HUMANS);
pub(crate) const DEC_BASE36: [u8; 256] = reverse_table_ci(BASE36);
pub(crate) const DEC_BASE52: [u8; 256] = reverse_table(BASE52);
pub(crate) const DEC_BASE58_BTC: [u8; 256] = reverse_table(BASE58_BTC);
pub(crate) const DEC_BASE62_IEEE: [u8; 256] = reverse_table(BASE62_IEEE);
pub(crate) const DEC_BASE62_SORT: [u8; 256] = reverse_table(BASE62_SORT);
pub(crate) const DEC_BASE64_STD: [u8; 256] = reverse_table(BASE64_STD);
pub(crate) const DEC_BASE64_URL: [u8; 256] = reverse_table(BASE64_URL);
pub(crate) const DEC_BASE64_SORT: [u8; 256] = reverse_table(BASE64_SORT);
pub(crate) const DEC_BASE85_Z85: [u8; 256] = reverse_table(BASE85_Z85);
