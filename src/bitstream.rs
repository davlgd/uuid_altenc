//! Generic bit-stream encode/decode used by Base32 and Base64 (and reused
//! by the `NCName` variants for the body section).
//!
//! For a 128-bit UUID the output sizes are:
//!   * Base32 — 26 characters (5 bits/char, 2 trailing pad bits set to 0)
//!   * Base64 — 22 characters (6 bits/char, 2 trailing pad bits set to 0)
//!
//! Padding (`'='` in RFC 4648) is intentionally omitted: every UUID has the
//! same width so the suffix carries no information. Appendix C of the draft
//! also omits it.

use crate::error::DecodeError;

/// Pack `data` (big-endian bit stream) into `bits_per_char` slots from
/// `alphabet`. The output buffer is filled from the start; missing bits at
/// the tail are treated as zero.
///
/// `out` length determines how many characters are produced; callers size
/// it to the encoding's fixed UUID width.
pub(crate) fn encode_bits(data: &[u8], alphabet: &[u8], bits_per_char: u32, out: &mut [u8]) {
    debug_assert!(bits_per_char <= 6, "callers use 5 or 6 bits");
    let mask = (1u32 << bits_per_char) - 1;
    let mut acc: u32 = 0;
    let mut nacc: u32 = 0;
    let mut byte_idx = 0;
    for slot in out.iter_mut() {
        while nacc < bits_per_char {
            let next = if byte_idx < data.len() {
                let b = data[byte_idx];
                byte_idx += 1;
                u32::from(b)
            } else {
                0
            };
            acc = (acc << 8) | next;
            nacc += 8;
        }
        let shift = nacc - bits_per_char;
        let idx = ((acc >> shift) & mask) as usize;
        *slot = alphabet[idx];
        nacc = shift;
        acc &= (1u32 << nacc).wrapping_sub(1);
    }
}

/// Decode a fixed-width string back into `out_bytes` bytes. Errors on any
/// non-alphabet character and on non-zero trailing pad bits.
pub(crate) fn decode_bits(
    encoding: &'static str,
    input: &[u8],
    table: &[u8; 256],
    bits_per_char: u32,
    out: &mut [u8],
) -> Result<(), DecodeError> {
    let out_bytes = out.len();
    let mut acc: u32 = 0;
    let mut nacc: u32 = 0;
    let mut byte_idx = 0;
    for (pos, &c) in input.iter().enumerate() {
        let v = table[c as usize];
        if v == crate::alphabets::NONE {
            return Err(DecodeError::InvalidCharacter { encoding, position: pos, byte: c });
        }
        acc = (acc << bits_per_char) | u32::from(v);
        nacc += bits_per_char;
        while nacc >= 8 && byte_idx < out_bytes {
            let shift = nacc - 8;
            out[byte_idx] = ((acc >> shift) & 0xff) as u8;
            byte_idx += 1;
            nacc = shift;
            acc &= (1u32 << nacc).wrapping_sub(1);
        }
    }
    if byte_idx != out_bytes {
        return Err(DecodeError::InvalidLength {
            encoding,
            expected: (out_bytes * 8).div_ceil(bits_per_char as usize),
            got: input.len(),
        });
    }
    if acc != 0 {
        return Err(DecodeError::NonCanonicalPadding { encoding });
    }
    Ok(())
}
