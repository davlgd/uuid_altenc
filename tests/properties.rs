//! Property-style round-trip tests on a deterministic stream of bytes.
//!
//! Any 16-byte string is a valid input for *every* encoding (the UUID
//! version/variant bits don't affect the encoding except in `NCName`
//! schemes, which preserve them by construction). We rely on a tiny LCG
//! seeded with a constant so failures are reproducible without bringing
//! in a property-testing dependency.
#![allow(clippy::cast_possible_truncation)]

use uuid_altenc::{
    Uuid, from_base32, from_base32_hex, from_base32_humans, from_base36, from_base52,
    from_base58_btc, from_base62_ieee, from_base62_sort, from_base62id, from_base64,
    from_base64_sort, from_base64_url, from_base85_z85, from_ncname_32, from_ncname_58,
    from_ncname_64,
};

struct Lcg(u64);
impl Lcg {
    fn next_u8(&mut self) -> u8 {
        self.0 = self.0.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) as u8
    }
    fn fill(&mut self, dst: &mut [u8]) {
        for b in dst.iter_mut() {
            *b = self.next_u8();
        }
    }
}

#[test]
fn round_trip_random_inputs() {
    let mut rng = Lcg(0xdead_beef_cafe_f00d);
    for _ in 0..256 {
        let mut bytes = [0u8; 16];
        rng.fill(&mut bytes);
        let u = Uuid::from_bytes(bytes);

        macro_rules! roundtrip {
            ($enc:ident, $dec:path) => {{
                let s = u.$enc();
                let back = $dec(&s).expect(concat!(stringify!($enc), " round-trip"));
                assert_eq!(back, u);
            }};
        }

        roundtrip!(to_hex,            uuid_altenc::from_hex);
        roundtrip!(to_hex_dash,       uuid_altenc::from_hex);
        roundtrip!(to_base32,         from_base32);
        roundtrip!(to_base32_hex,     from_base32_hex);
        roundtrip!(to_base32_humans,  from_base32_humans);
        roundtrip!(to_base36,         from_base36);
        roundtrip!(to_base52,         from_base52);
        roundtrip!(to_base58_btc,     from_base58_btc);
        roundtrip!(to_base62_ieee,    from_base62_ieee);
        roundtrip!(to_base62_sort,    from_base62_sort);
        roundtrip!(to_base64,         from_base64);
        roundtrip!(to_base64_url,     from_base64_url);
        roundtrip!(to_base64_sort,    from_base64_sort);
        roundtrip!(to_base85_z85,     from_base85_z85);
        roundtrip!(to_ncname_32,      from_ncname_32);
        roundtrip!(to_ncname_58,      from_ncname_58);
        roundtrip!(to_ncname_64,      from_ncname_64);
        roundtrip!(to_base62id,       from_base62id);
    }
}

#[test]
fn parse_autodetects_unique_lengths() {
    let want: Uuid = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6".parse().unwrap();
    for s in [
        want.to_hex_dash(),     // 36
        want.to_hex(),          // 32
        want.to_base32_hex(),   // 26 uppercase → routed to base32hex
        want.to_ncname_32(),    // 26 lowercase, bookends a-p → routed to ncname-32
        want.to_base36(),       // 25 → routed to base36 (only encoder reaching 24/25)
        want.to_base52(),       // 23 with O/I/l-style chars (lowercase, no digits) → base52
        want.to_base85_z85(),   // 20
        want.to_ncname_58(),    // 23, base58 fingerprint → ncname-58
    ] {
        let got = uuid_altenc::parse(&s).unwrap_or_else(|e| panic!("parse({s}): {e}"));
        assert_eq!(got, want);
    }
    // 22-char family is intentionally ambiguous: Base62id, Base64url and
    // NCName-64 alphabets overlap, so `parse` rejects them rather than
    // silently picking a scheme that decodes to the wrong UUID.
    for s in [want.to_base62id(), want.to_base64_url(), want.to_ncname_64()] {
        assert!(matches!(
            uuid_altenc::parse(&s),
            Err(uuid_altenc::DecodeError::AmbiguousLength { got: 22 }),
        ));
    }
}

#[test]
fn ord_matches_byte_order() {
    let small = Uuid::from_bytes([0x00; 16]);
    let mid = Uuid::from_bytes([0x80; 16]);
    let large = Uuid::from_bytes([0xff; 16]);
    assert!(small < mid);
    assert!(mid < large);
}
