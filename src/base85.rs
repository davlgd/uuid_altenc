//! Z85 (Base85) — 16 bytes ↔ 20 characters. Each 4-byte big-endian chunk
//! is encoded as a 5-character base-85 number, most significant character
//! first. The alphabet is the `ZeroMQ` Z85 alphabet (Table 1 of the draft).
//!
//! The 32-bit chunk model means there is no padding: 16 / 4 = 4 chunks
//! exactly, 4 × 5 = 20 output characters. On decode we explicitly check
//! that each 5-character group fits in `u32` so `99999` (= 6 765 201 875)
//! can't pretend to be a chunk.
#![allow(clippy::cast_possible_truncation)] // bounded by MAX_CHUNK check

use crate::Uuid;
use crate::alphabets::{BASE85_Z85, DEC_BASE85_Z85, NONE};
use crate::error::DecodeError;

const ENC: &str = "Z85";
const Z85_LEN: usize = 20;
const MAX_CHUNK: u64 = 0xffff_ffff;

impl Uuid {
    /// 20-character Z85 encoding of the UUID.
    #[must_use]
    pub fn to_base85_z85(&self) -> String {
        let mut buf = [0u8; Z85_LEN];
        for chunk in 0..4 {
            let base = chunk * 4;
            let mut n = u32::from_be_bytes([
                self.bytes[base],
                self.bytes[base + 1],
                self.bytes[base + 2],
                self.bytes[base + 3],
            ]);
            let out = chunk * 5;
            // Write 5 base-85 digits, LSD-first into the high-index slots.
            for slot in (0..5).rev() {
                buf[out + slot] = BASE85_Z85[(n % 85) as usize];
                n /= 85;
            }
        }
        String::from_utf8(buf.to_vec()).expect("Z85 alphabet is ASCII")
    }
}

/// Parse a 20-character Z85 string into a UUID.
pub fn from_base85_z85(input: &str) -> Result<Uuid, DecodeError> {
    let bytes = input.as_bytes();
    if bytes.len() != Z85_LEN {
        return Err(DecodeError::InvalidLength {
            encoding: ENC,
            expected: Z85_LEN,
            got: bytes.len(),
        });
    }
    let mut out = [0u8; 16];
    for chunk in 0..4 {
        let mut n: u64 = 0;
        for i in 0..5 {
            let pos = chunk * 5 + i;
            let c = bytes[pos];
            let v = DEC_BASE85_Z85[c as usize];
            if v == NONE {
                return Err(DecodeError::InvalidCharacter { encoding: ENC, position: pos, byte: c });
            }
            n = n * 85 + u64::from(v);
        }
        if n > MAX_CHUNK {
            return Err(DecodeError::Overflow { encoding: ENC });
        }
        let base = chunk * 4;
        out[base..base + 4].copy_from_slice(&(n as u32).to_be_bytes());
    }
    Ok(Uuid::from_bytes(out))
}
