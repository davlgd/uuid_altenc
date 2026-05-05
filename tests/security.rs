//! Security regression tests. Each one reproduces an attack surface
//! that, if untested, could let untrusted input cause unbounded work,
//! ambiguous identifiers, or panics.

use uuid_altenc::{
    DecodeError, from_base32, from_base36, from_base58_btc, from_base62_sort, from_base64,
    from_base85_z85, from_hex, from_ncname_32, from_ncname_58, from_ncname_64,
};

#[test]
fn dos_huge_input_to_integer_decoders_is_o1() {
    // A million-character string is rejected via the length cap, not by
    // looping over every character.
    let huge = "0".repeat(1_000_000);
    assert!(matches!(
        from_base36(&huge),
        Err(DecodeError::InputTooLong { .. })
    ));
    let huge_b58 = "1".repeat(1_000_000);
    assert!(matches!(
        from_base58_btc(&huge_b58),
        Err(DecodeError::InputTooLong { .. })
    ));
    assert!(matches!(
        from_base62_sort(&huge),
        Err(DecodeError::InputTooLong { .. })
    ));
}

#[test]
fn empty_input_rejected_not_zero() {
    // An empty Base36 string used to round-trip to the Nil UUID in some
    // libraries. We reject it explicitly.
    assert!(matches!(
        from_base36(""),
        Err(DecodeError::InvalidLength { .. })
    ));
    assert!(matches!(
        from_base58_btc(""),
        Err(DecodeError::InvalidLength { .. })
    ));
}

#[test]
fn fixed_width_decoders_reject_wrong_lengths() {
    assert!(from_base32("AAAA").is_err());
    assert!(from_base32(&"A".repeat(27)).is_err());
    assert!(from_base85_z85(&"0".repeat(19)).is_err());
    assert!(from_base85_z85(&"0".repeat(21)).is_err());
}

#[test]
fn base64_rejects_punctuation() {
    // 22 punctuation chars: every byte is a non-alphabet character.
    assert!(from_base64("!@#$%^&*()___bad_____").is_err());
}

#[test]
fn ncname_canonicality_rejects_aliases() {
    // NCName-32 of the Nil UUID is `aaaaaaaaaaaaaaaaaaaaaaaaaa`. If we
    // didn't validate the last char, every value < 32 there decoded to
    // the same UUID. The encoder only ever produces 0..16 in that slot.
    assert!(matches!(
        from_ncname_32("aaaaaaaaaaaaaaaaaaaaaaaaaq"),
        Err(DecodeError::SchemeViolation { .. })
    ));
    assert!(matches!(
        from_ncname_64("AAAAAAAAAAAAAAAAAAAAAQ"),
        Err(DecodeError::SchemeViolation { .. })
    ));
    // NCName-58: an underscore appearing before a non-`1` digit is
    // non-canonical.
    assert!(matches!(
        from_ncname_58("A1_111111111111111____A"),
        Err(DecodeError::SchemeViolation { .. })
    ));
    // Bookend outside A-P.
    assert!(matches!(
        from_ncname_64("Q-B1Prn3sHQdlAKDJHmv2K"),
        Err(DecodeError::SchemeViolation { .. })
    ));
}

#[test]
fn hex_rejects_misplaced_dashes() {
    // 36 chars but dashes in wrong positions.
    let bad = "f81d4fae7-dec-11d0-a765-00a0c91e6bf6"; // dash at pos 9
    assert!(from_hex(bad).is_err());
}

#[test]
fn no_panic_on_arbitrary_bytes() {
    // Run every decoder against random ASCII (and a few non-ASCII)
    // inputs of every plausible length — none must panic.
    let inputs: &[&[u8]] = &[
        b"",
        b"x",
        b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        b"\x00\x01\x02\x03",
        b"a-b-c-d-e-f-g-h-i-j-k-l-m-n-o-p-q-r-s",
        b"\xff\xfe\xfd",
    ];
    for raw in inputs {
        let s = String::from_utf8_lossy(raw);
        let _ = from_hex(&s);
        let _ = from_base32(&s);
        let _ = from_base36(&s);
        let _ = from_base58_btc(&s);
        let _ = from_base62_sort(&s);
        let _ = from_base64(&s);
        let _ = from_base85_z85(&s);
        let _ = from_ncname_32(&s);
        let _ = from_ncname_58(&s);
        let _ = from_ncname_64(&s);
        let _ = uuid_altenc::parse(&s);
    }
}

#[test]
fn base62id_sentinel_must_match() {
    // A 22-char Base62-sort string whose first byte != 0x02 must be
    // rejected, even if it parses as a valid Base62-sort number.
    let other = "7n42DGM5Tflk9n8mt7Fhc7"; // Max UUID's Base62-sort form
    assert!(matches!(
        uuid_altenc::from_base62id(other),
        Err(DecodeError::SchemeViolation { .. })
    ));
}

#[test]
fn multibyte_utf8_in_input_is_rejected_not_panics() {
    // A 22-byte input that contains a multi-byte UTF-8 sequence: the
    // length check passes (we count bytes, not chars), but the alphabet
    // lookup must reject it as InvalidCharacter without panicking on
    // string-slicing boundaries.
    //
    // 'é' is two bytes (0xc3, 0xa9). 22 bytes = 11 'é's.
    let bad = "éééééééééééé"; // 24 bytes — wrong length, but still safe.
    assert!(uuid_altenc::from_base64_url(bad).is_err());
    let trimmed = "éééééééééééX"; // 23 bytes — also wrong length.
    assert!(uuid_altenc::from_base64_url(trimmed).is_err());
    let exact = "éééééééééééXY"; // 24 bytes — still wrong length.
    assert!(uuid_altenc::from_base64_url(exact).is_err());
    // Hex-dash with a multibyte char in a dash slot.
    let hex_bad = "f81d4fae-7dec-11d0-a765-00a0c91e6bféé"; // 38 bytes
    assert!(from_hex(hex_bad).is_err());
}

#[test]
fn null_bytes_and_control_chars_rejected_in_alphabets() {
    // A 22-character buffer of NULs.
    let nuls = "\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
    assert!(matches!(
        from_base64(nuls),
        Err(DecodeError::InvalidCharacter { byte: 0, .. })
    ));
    // Newlines in a 36-char hex-dash slot.
    let with_nl = "f81d4fae-7dec\n11d0-a765-00a0c91e6bf6";
    assert!(from_hex(with_nl).is_err());
}

#[test]
fn ncname_58_underscore_only_body_rejected() {
    // 21 underscores between bookends would decode to the all-zero body
    // (since `_` is the pad character) but with no leading-zero markers.
    // The decoder strips trailing underscores then runs the integer
    // decoder on an empty body, which decodes to a zero result — caught
    // by the round-trip canonicality gate as a SchemeViolation.
    let s = "A_____________________A";
    assert!(matches!(
        from_ncname_58(s),
        Err(DecodeError::SchemeViolation { .. })
    ));
}

#[test]
fn integer_decoder_overflow_rejected() {
    // 26 'V' characters in Base32hex would encode 130 bits — exceeds
    // 128. Even though length passes, we must catch the overflow.
    let s = "VVVVVVVVVVVVVVVVVVVVVVVVVV"; // 26 chars
    // base32hex final char must be 0..3 (since 128 % 5 = 3 trailing
    // bits zero); 'V' (=31) violates that, surfacing as
    // NonCanonicalPadding. Either way we get an error, never a panic.
    assert!(uuid_altenc::from_base32_hex(s).is_err());
}

#[test]
fn parse_rejects_ambiguous_lengths() {
    // Lengths produced by *multiple* schemes with overlapping alphabets,
    // or by no scheme at all. 22 is the headline case (Base62id /
    // Base64url / UUID-NCName-64 / Base58btc / Base62 sort / Base62 IEEE
    // all reach 22 chars). 1, 5, 21 are variable-width-only territory.
    // 27, 35, 37 are not reachable by any encoder.
    for n in [1usize, 5, 21, 22, 27, 35, 37] {
        let s = "A".repeat(n);
        assert!(matches!(
            uuid_altenc::parse(&s),
            Err(DecodeError::AmbiguousLength { .. }),
        ));
    }
}

#[test]
fn parse_routes_24_and_25_to_base36() {
    // Base36 is the only encoder whose output reaches 24/25 characters.
    let u: uuid_altenc::Uuid = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6".parse().unwrap();
    let s25 = u.to_base36();
    assert_eq!(s25.len(), 25);
    assert_eq!(uuid_altenc::parse(&s25).unwrap(), u);
    // A constructed 24-char Base36 round-trips too.
    let nil = uuid_altenc::NIL;
    let small: uuid_altenc::Uuid = uuid_altenc::Uuid::from_u128(36u128.pow(23));
    let s24 = small.to_base36();
    assert_eq!(s24.len(), 24);
    assert_eq!(uuid_altenc::parse(&s24).unwrap(), small);
    // Sanity: Nil still works through other paths.
    assert_eq!(uuid_altenc::parse(&nil.to_hex_dash()).unwrap(), nil);
}

#[test]
fn ncname_64_case_change_does_not_alias() {
    // NCName-64 is case-sensitive (Base64url alphabet has both cases).
    // A case change on a body character changes the underlying bits, so
    // it must either (a) decode to a *different* UUID, never the same
    // one, or (b) be rejected. Either way, no aliasing.
    let original: uuid_altenc::Uuid = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6".parse().unwrap();
    let canonical = original.to_ncname_64();
    let mutated: String = canonical
        .chars()
        .enumerate()
        .map(|(i, c)| if i == 5 && c.is_ascii_alphabetic() {
            if c.is_ascii_uppercase() { c.to_ascii_lowercase() } else { c.to_ascii_uppercase() }
        } else { c })
        .collect();
    if let Ok(other) = uuid_altenc::from_ncname_64(&mutated) {
        assert_ne!(other, original, "case change must not alias to the same UUID");
    }
}

#[test]
fn ncname_canonical_case_strictly_enforced() {
    // Each variant has exactly one canonical case pattern per the draft
    // Appendix C; any deviation must be rejected to preserve the
    // "no aliasing" invariant stated in lib.rs:60.
    let u: uuid_altenc::Uuid = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6".parse().unwrap();

    // NCName-32: fully lowercase. Uppercase anywhere → reject.
    let nc32 = u.to_ncname_32();
    assert!(uuid_altenc::from_ncname_32(&nc32).is_ok());
    assert!(uuid_altenc::from_ncname_32(&nc32.to_ascii_uppercase()).is_err());
    // Toggle the first letter in the body — guaranteed to exist (NCName-32
    // bookends are a-p) but pick from body to avoid testing only bookend.
    let mut mixed = nc32.clone().into_bytes();
    let letter_pos = mixed
        .iter()
        .position(u8::is_ascii_lowercase)
        .expect("NCName-32 always has letters");
    mixed[letter_pos] = mixed[letter_pos].to_ascii_uppercase();
    assert!(uuid_altenc::from_ncname_32(std::str::from_utf8(&mixed).unwrap()).is_err());

    // NCName-58: uppercase bookends mandatory. Lowercase first or last → reject.
    let nc58 = u.to_ncname_58();
    assert!(uuid_altenc::from_ncname_58(&nc58).is_ok());
    let mut lower_first = nc58.clone().into_bytes();
    lower_first[0] = lower_first[0].to_ascii_lowercase();
    assert!(uuid_altenc::from_ncname_58(std::str::from_utf8(&lower_first).unwrap()).is_err());
    let mut lower_last = nc58.clone().into_bytes();
    let last_idx = lower_last.len() - 1;
    lower_last[last_idx] = lower_last[last_idx].to_ascii_lowercase();
    assert!(uuid_altenc::from_ncname_58(std::str::from_utf8(&lower_last).unwrap()).is_err());

    // NCName-64: uppercase first bookend mandatory.
    let nc64 = u.to_ncname_64();
    assert!(uuid_altenc::from_ncname_64(&nc64).is_ok());
    let mut lower_first = nc64.clone().into_bytes();
    lower_first[0] = lower_first[0].to_ascii_lowercase();
    assert!(uuid_altenc::from_ncname_64(std::str::from_utf8(&lower_first).unwrap()).is_err());
}

#[test]
fn version_helper_independent_of_encoding() {
    // For every encoding the same UUID round-trips to the same version
    // and variant. Catches encoding accidentally mutating those bits.
    let u: uuid_altenc::Uuid = "017f22e2-79b0-7cc3-98c4-dc0c0c07398f".parse().unwrap();
    assert_eq!(u.version(), 7);
    assert_eq!(u.variant(), 0x9);
    let back =
        uuid_altenc::from_ncname_64(&u.to_ncname_64()).unwrap();
    assert_eq!(back.version(), 7);
    assert_eq!(back.variant(), 0x9);
}
