//! Optional `serde` integration. Human-readable formats (JSON, YAML,
//! TOML…) emit the canonical hex-dash form and accept either hex-dash
//! (36 chars) or plain hex (32 chars) on deserialization; binary formats
//! (bincode, `MessagePack`…) emit the raw 16 bytes.

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use crate::Uuid;

impl Serialize for Uuid {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        if s.is_human_readable() {
            s.serialize_str(&self.to_hex_dash())
        } else {
            s.serialize_bytes(&self.bytes)
        }
    }
}

impl<'de> Deserialize<'de> for Uuid {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        if d.is_human_readable() {
            // Owned `String` so streaming / map-key / non-borrowing
            // deserializers still work; allocation cost is negligible
            // next to the parse work.
            let s = String::deserialize(d)?;
            crate::from_hex(&s).map_err(de::Error::custom)
        } else {
            let bytes = <Vec<u8>>::deserialize(d)?;
            Self::try_from_slice(&bytes).map_err(de::Error::custom)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trip() {
        let u: Uuid = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6".parse().unwrap();
        let s = serde_json::to_string(&u).unwrap();
        assert_eq!(s, "\"f81d4fae-7dec-11d0-a765-00a0c91e6bf6\"");
        let back: Uuid = serde_json::from_str(&s).unwrap();
        assert_eq!(back, u);
    }

    #[test]
    fn json_rejects_garbage() {
        let r: Result<Uuid, _> = serde_json::from_str("\"not-a-uuid\"");
        assert!(r.is_err());
    }

    #[test]
    fn json_accepts_plain_hex() {
        // Deserialize side is lenient (accepts hex-dash or plain hex);
        // emit side is always hex-dash.
        let s = "\"f81d4fae7dec11d0a76500a0c91e6bf6\"";
        let back: Uuid = serde_json::from_str(s).unwrap();
        let canonical: Uuid = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6".parse().unwrap();
        assert_eq!(back, canonical);
    }

    // Note: the binary (non-human-readable) serde path goes through
    // `serialize_bytes` / `Vec<u8>::deserialize` + `try_from_slice`. Both
    // halves are covered: `try_from_slice` has its own tests
    // (`try_from_slice_length_checked`), and the serde wiring is a thin
    // call-through. A bincode integration test would harden the seam but
    // costs a dev-dep — deferred until a real consumer reports an issue.
}
