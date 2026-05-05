//! Base62id (the `[Base62id]` reference in the draft).
//!
//! Algorithm: prepend the bits `0b10` to the 128-bit UUID, producing a
//! 130-bit unsigned integer `V = (1 << 129) | uuid`. Encode `V` with the
//! Base62sort alphabet (`0-9A-Za-z`).
//!
//! The `0b10` prefix forces `V` into `[2^129, 2^130)`. Since
//! `62^21 ≈ 4.36 × 10^37 < 2^129 ≈ 6.81 × 10^38 < 2^130 < 62^22`, every
//! such value encodes to exactly 22 characters and the leading digit lies
//! in `[F..V]` (15..31 in Base62sort), guaranteeing the result starts
//! with an uppercase letter — the property that makes Base62id safe to
//! drop into HTML/CSS/XML id attributes.

use crate::Uuid;
use crate::alphabets::{BASE62_SORT, DEC_BASE62_SORT};
use crate::error::DecodeError;
use crate::integer_base::{decode_integer, encode_integer};

const ENC: &str = "base62id";
const LEN: usize = 22;
const SENTINEL: u8 = 0x02;

impl Uuid {
    /// 22-character Base62id encoding. Always starts with an uppercase
    /// letter, so the result is safe to use as an XML/HTML/CSS id.
    #[must_use]
    pub fn to_base62id(&self) -> String {
        // 17-byte big-endian buffer holding (1 << 129) | uuid_int. The top
        // 6 bits of byte 0 are zero; bits 129..128 = 0b10 → byte 0 = 2.
        let mut prefixed = [0u8; 17];
        prefixed[0] = SENTINEL;
        prefixed[1..].copy_from_slice(&self.bytes);
        let mut buf = [0u8; 32];
        let written = encode_integer(&prefixed, BASE62_SORT, &mut buf).len();
        // Sanity check: the algorithm guarantees 22 characters.
        debug_assert_eq!(written, LEN);
        String::from_utf8(buf[..written].to_vec()).expect("alphabet is ASCII")
    }
}

/// Parse a 22-character Base62id string.
pub fn from_base62id(input: &str) -> Result<Uuid, DecodeError> {
    if input.len() != LEN {
        return Err(DecodeError::InvalidLength {
            encoding: ENC,
            expected: LEN,
            got: input.len(),
        });
    }
    let mut buf = [0u8; 17];
    decode_integer(ENC, input.as_bytes(), &DEC_BASE62_SORT, 62, &mut buf)?;
    if buf[0] != SENTINEL {
        return Err(DecodeError::SchemeViolation {
            encoding: ENC,
            reason: "sentinel byte mismatch (input is corrupt or not Base62id)",
        });
    }
    let mut out = [0u8; 16];
    out.copy_from_slice(&buf[1..]);
    Ok(Uuid::from_bytes(out))
}
