//! Error types surfaced by every fallible operation.

use core::fmt;

/// Errors returned when parsing a UUID from any text encoding.
///
/// The error carries the encoding name and a small payload so callers can
/// produce helpful diagnostics without parsing error strings.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DecodeError {
    /// The input had the wrong length for its declared encoding.
    InvalidLength {
        /// Encoding name, e.g. `"base32"`, `"Z85"`.
        encoding: &'static str,
        /// Required length in characters.
        expected: usize,
        /// Length actually received.
        got: usize,
    },
    /// The input contained a byte that is not part of the encoding's alphabet.
    InvalidCharacter {
        /// Encoding name.
        encoding: &'static str,
        /// 0-based index of the offending byte.
        position: usize,
        /// The byte itself (may not be valid UTF-8 on its own).
        byte: u8,
    },
    /// The decoded integer would not fit in 128 bits (or in the
    /// 32-bit-per-chunk slot for Z85).
    Overflow {
        /// Encoding name.
        encoding: &'static str,
    },
    /// Trailing pad bits in a fixed-width encoding were non-zero. This means
    /// the input is corrupt or non-canonical: two distinct strings would
    /// otherwise decode to the same UUID.
    NonCanonicalPadding {
        /// Encoding name.
        encoding: &'static str,
    },
    /// The input is too long to be any valid encoding of a UUID. Decoders
    /// return this in O(1) before doing any work to avoid `DoS`.
    InputTooLong {
        /// Encoding name.
        encoding: &'static str,
        /// Maximum length the encoding accepts.
        max: usize,
        /// Length actually received.
        got: usize,
    },
    /// The encoding has a constraint specific to its scheme that was
    /// violated (e.g. `NCName` bookend not in `A`–`P`, Base62id sentinel
    /// mismatch).
    SchemeViolation {
        /// Encoding name.
        encoding: &'static str,
        /// Human-readable description of the violation.
        reason: &'static str,
    },
    /// `parse(...)` could not autodetect an encoding for the input length.
    AmbiguousLength {
        /// Length of the input.
        got: usize,
    },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength { encoding, expected, got } => write!(
                f,
                "{encoding}: expected {expected} characters, got {got}",
            ),
            Self::InvalidCharacter { encoding, position, byte } => {
                let printable = if byte.is_ascii_graphic() {
                    char::from(*byte)
                } else {
                    '\u{fffd}'
                };
                write!(
                    f,
                    "{encoding}: invalid character {printable:?} (0x{byte:02x}) at position {position}",
                )
            }
            Self::Overflow { encoding } => write!(f, "{encoding}: input overflows 128 bits"),
            Self::NonCanonicalPadding { encoding } => {
                write!(f, "{encoding}: non-canonical padding (corrupt input)")
            }
            Self::InputTooLong { encoding, max, got } => {
                write!(f, "{encoding}: input too long ({got} > {max})")
            }
            Self::SchemeViolation { encoding, reason } => write!(f, "{encoding}: {reason}"),
            Self::AmbiguousLength { got } => write!(
                f,
                "parse: cannot autodetect encoding for {got}-character input — call a specific from_* function",
            ),
        }
    }
}

impl core::error::Error for DecodeError {}

#[cfg(test)]
mod assertions {
    use super::DecodeError;

    // Compile-time guarantee that the error type implements the standard
    // bounds — easy to break by accident, very disruptive downstream.
    const fn assert_traits<T: Send + Sync + Clone + core::error::Error + 'static>() {}
    const _: () = assert_traits::<DecodeError>();
}
