//! Conformance tests against RFC 9562 directly (the source of truth for
//! the canonical 36-character hex-dash form, the version/variant fields,
//! and the Nil/Max special values).
//!
//! Each test cites the section it exercises so a reviewer can map back
//! to the RFC without leaving the file.

use uuid_altenc::{MAX, NIL, Uuid, from_hex, from_hex_dash, from_hex_plain};

/// RFC 9562 §4: the canonical text form is
/// `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` — 32 lowercase hex digits with
/// dashes at positions 8, 13, 18, 23 (0-indexed), for a total of 36
/// characters.
#[test]
fn canonical_form_shape() {
    let u: Uuid = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6".parse().unwrap();
    let s = u.to_hex_dash();
    assert_eq!(s.len(), 36);
    let bytes = s.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        let is_dash_pos = matches!(i, 8 | 13 | 18 | 23);
        let is_dash = *b == b'-';
        assert_eq!(is_dash_pos, is_dash, "wrong dash at position {i}");
    }
    assert!(
        s.bytes().all(|b| b == b'-' || (b.is_ascii_digit() || (b'a'..=b'f').contains(&b))),
        "canonical form must be lowercase hex digits + dashes only"
    );
}

/// RFC 9562 §4 (Variant + Version Fields). Every worked example from
/// Appendix A — covers UUID versions 1, 2, 3, 4, 5, 6, 7, 8 — pins the
/// version nibble (high 4 bits of byte 6) and the variant top-bits
/// (high bits of byte 8).
#[test]
fn version_and_variant_against_rfc_appendix_a() {
    // (label, uuid, expected version, expected variant nibble)
    // RFC 9562 variant 1 ("10xx") nibble values are 0x8..=0xb.
    let cases: &[(&str, &str, u8, u8)] = &[
        ("§A.1 v1",  "c232ab00-9414-11ec-b3c8-9f6bdeced846", 1, 0xb),
        ("§A.2 v2",  "000003e8-cbb9-21ea-b201-00045a86c8a1", 2, 0xb),
        ("§A.3 v3",  "5df41881-3aed-3515-88a7-2f4a814cf09e", 3, 0x8),
        ("§A.4 v4",  "919108f7-52d1-4320-9bac-f847db4148a8", 4, 0x9),
        ("§A.5 v5",  "2ed6657d-e927-568b-95e1-2665a8aea6a2", 5, 0x9),
        ("§A.6 v6",  "1ec9414c-232a-6b00-b3c8-9f6bdeced846", 6, 0xb),
        ("§A.7 v7",  "017f22e2-79b0-7cc3-98c4-dc0c0c07398f", 7, 0x9),
        ("§A.8 v8",  "2489e9ad-2ee2-8e00-8ec9-32d5f69181c0", 8, 0x8),
    ];
    for (label, s, want_ver, want_var_nibble) in cases {
        let u: Uuid = s.parse().expect(label);
        assert_eq!(u.version(), *want_ver, "{label}: version");
        assert_eq!(u.variant(), *want_var_nibble, "{label}: variant nibble");
        // RFC 9562 variant-1 family: top 2 bits of byte 8 are `10`.
        assert_eq!(u.variant() >> 2, 0b10, "{label}: not variant 1");
    }
}

/// RFC 9562 §4.1 (Variant Field) defines four families by the top bits
/// of byte 8. The crate's `variant()` exposes the full nibble; this test
/// pins the interpretation for non-variant-1 UUIDs so consumers can
/// classify by the top bits.
#[test]
fn variant_field_decoding() {
    // NCS (variant 0): top bit = 0 → nibble in 0x0..=0x7
    let ncs: Uuid = "00000000-0000-0000-7000-000000000000".parse().unwrap();
    assert_eq!(ncs.variant() >> 3, 0b0);

    // RFC 4122/9562 (variant 1): top 2 bits = 10 → nibble 0x8..=0xb
    let rfc: Uuid = "00000000-0000-0000-a000-000000000000".parse().unwrap();
    assert_eq!(rfc.variant() >> 2, 0b10);

    // Microsoft (variant 2): top 3 bits = 110 → nibble 0xc..=0xd
    // Real example from the draft appendix.
    let ms: Uuid = "00000013-0000-0000-c000-000000000000".parse().unwrap();
    assert_eq!(ms.variant() >> 1, 0b110);

    // Future/Reserved: top 3 bits = 111 → nibble 0xe..=0xf
    let fut: Uuid = "00000000-0000-0000-e000-000000000000".parse().unwrap();
    assert_eq!(fut.variant() >> 1, 0b111);
}

/// RFC 9562 §5.9 (Nil UUID): "all bits set to zero."
#[test]
fn nil_uuid_matches_rfc_5_9() {
    assert_eq!(NIL.as_bytes(), &[0u8; 16]);
    assert_eq!(NIL.to_hex_dash(), "00000000-0000-0000-0000-000000000000");
    let parsed: Uuid = "00000000-0000-0000-0000-000000000000".parse().unwrap();
    assert_eq!(parsed, NIL);
    // §5.9: the version field is conventionally 0 for the Nil UUID.
    assert_eq!(NIL.version(), 0);
}

/// RFC 9562 §5.10 (Max UUID): "all bits set to one."
#[test]
fn max_uuid_matches_rfc_5_10() {
    assert_eq!(MAX.as_bytes(), &[0xffu8; 16]);
    assert_eq!(MAX.to_hex_dash(), "ffffffff-ffff-ffff-ffff-ffffffffffff");
    let parsed: Uuid = "ffffffff-ffff-ffff-ffff-ffffffffffff".parse().unwrap();
    assert_eq!(parsed, MAX);
    // §5.10: version nibble is 0xf.
    assert_eq!(MAX.version(), 0xf);
}

/// RFC 9562 §4: parsers MUST accept both upper- and lower-case hex,
/// while emitters MUST output lowercase. Verify both halves.
#[test]
fn parser_is_case_insensitive_emitter_is_lowercase() {
    let canonical = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6";
    let upper = "F81D4FAE-7DEC-11D0-A765-00A0C91E6BF6";
    let mixed = "F81d4FaE-7dEc-11D0-a765-00A0c91E6Bf6";
    let from_lower: Uuid = canonical.parse().unwrap();
    let from_upper: Uuid = upper.parse().unwrap();
    let from_mixed: Uuid = mixed.parse().unwrap();
    assert_eq!(from_lower, from_upper);
    assert_eq!(from_lower, from_mixed);
    // Emitter always lowercase per §4 ("Each hexadecimal value is
    // displayed in lowercase").
    assert_eq!(from_upper.to_hex_dash(), canonical);
}

/// RFC 9562 §4 illustrates the "8-4-4-4-12" character grouping. A
/// 36-character input with the right total length but misplaced dashes
/// is invalid and must be rejected.
#[test]
fn misplaced_dashes_rejected() {
    // dash at position 9 instead of 8
    assert!(from_hex("f81d4fae7-dec-11d0-a765-00a0c91e6bf6").is_err());
    // dash at position 7 (one early)
    assert!(from_hex("f81d4fa-e7dec-11d0-a765-00a0c91e6bf6").is_err());
}

/// `from_hex_dash` is the strict variant: it MUST reject the plain
/// 32-char form (which `from_hex_plain` and the lenient `from_hex`
/// accept). This pair lets callers opt into the §4 canonical form
/// only.
#[test]
fn strict_decoders_reject_other_form() {
    let dashed = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6";
    let plain = "f81d4fae7dec11d0a76500a0c91e6bf6";

    assert!(from_hex_dash(dashed).is_ok());
    assert!(from_hex_dash(plain).is_err());

    assert!(from_hex_plain(plain).is_ok());
    assert!(from_hex_plain(dashed).is_err());

    // Lenient decoder accepts both.
    assert!(from_hex(dashed).is_ok());
    assert!(from_hex(plain).is_ok());
}

/// Network-order storage: byte 0 is the most-significant byte (RFC 9562
/// §6.7 + the wire-format diagram in §4). Verify by parsing a known
/// UUID and inspecting `as_bytes()`.
#[test]
fn network_byte_order_is_big_endian() {
    let u: Uuid = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6".parse().unwrap();
    let bytes = u.as_bytes();
    assert_eq!(bytes[0], 0xf8);
    assert_eq!(bytes[1], 0x1d);
    assert_eq!(bytes[6], 0x11); // contains version nibble in high half
    assert_eq!(bytes[8], 0xa7); // contains variant nibble in high half
    assert_eq!(bytes[15], 0xf6);

    // `from_u128` also follows §6.7: big-endian interpretation.
    let n: u128 = 0xf81d4fae_7dec_11d0_a765_00a0c91e6bf6;
    let from_int = Uuid::from_u128(n);
    assert_eq!(from_int, u);
    assert_eq!(u.to_u128(), n);
}
