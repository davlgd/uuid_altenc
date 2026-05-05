//! Variable-width integer-style encoders (Base36, 52, 58, 62) plus the
//! padded variant used by UUID-NCName-58.
//!
//! These treat the input as a single big-endian non-negative integer and
//! write it in the chosen base **without leading-zero padding**. The Nil
//! UUID therefore encodes as a single character (the alphabet's zero), and
//! UUIDs whose high bytes happen to be zero produce shorter strings. This
//! matches the draft's Appendix C tables.
//!
//! Implementation: byte-wise long division on a mutable copy of the input
//! buffer. This works for any width without depending on `u128`, and lets
//! us reuse the same routine for the 17-byte buffer used by Base62id.
#![allow(clippy::cast_possible_truncation)] // bases ≤ 62, digits ≤ 255

use crate::error::DecodeError;

/// Maximum input length any integer-base UUID decoder will accept. 26
/// covers every alphabet (Base36 of `2^128 - 1` is 25 characters, the
/// 17-byte Base62id buffer fits in 22 characters); rejecting in O(1) above
/// this prevents pathological input from doing unbounded work.
pub(crate) const MAX_INTEGER_BASE_LEN: usize = 26;

/// Encode `data` (big-endian) as a base-`alphabet.len()` integer.
///
/// `out` is a scratch buffer the caller owns; we write the encoded bytes
/// to it in *forward* (most-significant-first) order and return the slice
/// of the buffer that was actually used. This avoids allocation and lets
/// the caller place the result wherever it wants (e.g. inside a longer
/// `NCName` output buffer).
///
/// `data` must not be empty. The function always writes at least one
/// character.
pub(crate) fn encode_integer<'b>(data: &[u8], alphabet: &[u8], out: &'b mut [u8]) -> &'b [u8] {
    encode_integer_padded(data, alphabet, false, out)
}

/// Like [`encode_integer`] but, when `preserve_leading_zeros` is true,
/// emits one `alphabet[0]` character per leading null byte in the input.
/// This is the standard Base58 behaviour used inside UUID-NCName-58.
pub(crate) fn encode_integer_padded<'b>(
    data: &[u8],
    alphabet: &[u8],
    preserve_leading_zeros: bool,
    out: &'b mut [u8],
) -> &'b [u8] {
    let base = alphabet.len() as u32; // ≤ 62
    // Reverse divmod on a stack copy. For UUID-sized inputs the buffer
    // is tiny (≤17 bytes) so we use a fixed-size array for it.
    let mut buf = [0u8; 32];
    let n = data.len();
    debug_assert!(n <= buf.len());
    buf[..n].copy_from_slice(data);

    let mut start = 0usize;
    while start < n && buf[start] == 0 {
        start += 1;
    }
    if start == n {
        // Whole input is zero. Emit `n` zero digits if padded, else 1.
        let count = if preserve_leading_zeros { n } else { 1 };
        for slot in out.iter_mut().take(count) {
            *slot = alphabet[0];
        }
        return &out[..count];
    }
    let leading_zero_bytes = start;
    // Write digits (least-significant-first) into a scratch reversed-output
    // area and reverse at the end.
    let mut tmp = [0u8; 32];
    let mut written = 0usize;
    while start < n {
        let mut rem: u32 = 0;
        for byte in &mut buf[start..n] {
            let cur = (rem << 8) | u32::from(*byte);
            *byte = (cur / base) as u8;
            rem = cur % base;
        }
        tmp[written] = alphabet[rem as usize];
        written += 1;
        while start < n && buf[start] == 0 {
            start += 1;
        }
    }
    if preserve_leading_zeros {
        for _ in 0..leading_zero_bytes {
            tmp[written] = alphabet[0];
            written += 1;
        }
    }
    // Reverse into the caller's output buffer.
    debug_assert!(out.len() >= written);
    for i in 0..written {
        out[i] = tmp[written - 1 - i];
    }
    &out[..written]
}

/// Decode a base-`base` integer string into `out` (big-endian). Returns an
/// error on any non-alphabet character or if the value would not fit.
pub(crate) fn decode_integer(
    encoding: &'static str,
    input: &[u8],
    table: &[u8; 256],
    base: u32,
    out: &mut [u8],
) -> Result<(), DecodeError> {
    out.fill(0);
    for (pos, &c) in input.iter().enumerate() {
        let v = table[c as usize];
        if v == crate::alphabets::NONE {
            return Err(DecodeError::InvalidCharacter { encoding, position: pos, byte: c });
        }
        let mut carry = u32::from(v);
        for byte in out.iter_mut().rev() {
            let cur = u32::from(*byte) * base + carry;
            *byte = (cur & 0xff) as u8;
            carry = cur >> 8;
        }
        if carry != 0 {
            return Err(DecodeError::Overflow { encoding });
        }
    }
    Ok(())
}

/// Decode a Base58btc-style padded encoding: leading `alphabet_zero`
/// characters represent leading null bytes in the output. Used by
/// UUID-NCName-58.
pub(crate) fn decode_integer_padded(
    encoding: &'static str,
    input: &[u8],
    table: &[u8; 256],
    base: u32,
    out: &mut [u8],
    alpha_zero: u8,
) -> Result<(), DecodeError> {
    let mut leading_zeros = 0usize;
    while leading_zeros < input.len() && input[leading_zeros] == alpha_zero {
        leading_zeros += 1;
    }
    if leading_zeros > out.len() {
        return Err(DecodeError::SchemeViolation {
            encoding,
            reason: "too many leading zero markers for output width",
        });
    }
    out.fill(0);
    let body = &input[leading_zeros..];
    decode_integer(encoding, body, table, base, &mut out[leading_zeros..])
}

/// Reject obviously hostile inputs in O(1) before allocating or looping.
pub(crate) fn check_integer_input_len(
    encoding: &'static str,
    len: usize,
) -> Result<(), DecodeError> {
    if len == 0 {
        return Err(DecodeError::InvalidLength { encoding, expected: 1, got: 0 });
    }
    if len > MAX_INTEGER_BASE_LEN {
        return Err(DecodeError::InputTooLong {
            encoding,
            max: MAX_INTEGER_BASE_LEN,
            got: len,
        });
    }
    Ok(())
}
