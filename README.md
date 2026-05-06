# uuid_altenc

A small Rust crate that encodes and decodes UUIDs in 18 alternate text
formats — Base16/32/36/52/58/62/64/85, the three UUID-NCName variants,
and Base62id — as defined in IETF
[`draft-davis-uuidrev-alt-uuid-encoding-methods-00`][draft] (April 2026).

- Pure Rust, no runtime dependencies (`serde` is opt-in).
- `forbid(unsafe_code)`, `clippy::pedantic` clean.
- Every Appendix C test vector from the draft + RFC 9562 conformance suite.
- Strict decoders: hostile-length inputs rejected in O(1), no aliasing.
- MSRV: Rust 1.85.

## What the RFC brings (and why this exists)

A UUID is just **128 bits**. The canonical RFC 9562 form
`f81d4fae-7dec-11d0-a765-00a0c91e6bf6` is **36 UTF-8 characters** — fine
for logs, wasteful in URLs, and invalid as an XML/HTML `id` (it starts
with a digit). The IETF draft collects the 14 most useful alternative
text forms in one document and tells you which to pick:

| You want…                          | Use                       | Width  |
|-----------------------------------|---------------------------|-------:|
| URL slug, public identifier        | `to_base64_url()`         | 22     |
| Sortable database key              | `to_base32_hex()`         | 26     |
| Sortable + no special chars        | `to_base62_sort()`        | ≤22    |
| Compact, transcribable             | `to_base58_btc()`         | ≤22    |
| Most compact, symbols OK           | `to_base85_z85()`         | 20     |
| HTML/CSS/XML `id="…"`              | `to_ncname_64()` / `to_base62id()` | 22 |
| Human-friendly, sortable           | `to_base32_humans()`      | 26     |

Every `to_*` has a matching `from_*`.

## Install

```toml
[dependencies]
uuid_altenc = "0.1"
# Optional serde support:
# uuid_altenc = { version = "0.1", features = ["serde"] }
```

## Use it

```rust
use uuid_altenc::{Uuid, from_base58_btc};

let uuid: Uuid = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6".parse().unwrap();

// Encode
assert_eq!(uuid.to_base64_url(), "-B1Prn3sEdCnZQCgyR5r9g");
assert_eq!(uuid.to_base58_btc(), "Xe22UfxT3rxcKJEAfL5373");
assert_eq!(uuid.to_ncname_64(),  "B-B1Prn3sHQdlAKDJHmv2K"); // safe as <div id="…">

// Decode back
let back = from_base58_btc("Xe22UfxT3rxcKJEAfL5373").unwrap();
assert_eq!(back, uuid);
```

`parse()` autodetects by length when it is unambiguous (36, 32, 26, 23,
20 chars). 22-character inputs are intentionally rejected as ambiguous —
Base62id, Base64url and NCName-64 alphabets overlap, so call the
specific decoder when you know the format.

## Run the examples

The crate ships two examples, with deliberately different audiences:

- **`demo`** — a CLI walkthrough that prints every encoding for a
  fixed UUID with a short note next to each output. Start here if you
  want to see what the library can do in one screenful.

  ```sh
  cargo run --example demo
  ```

- **`webui`** — an interactive web page backed by a stdlib-only HTTP
  server (no `tokio`/`hyper`/`serde_json`). Run it when you want to
  share a URL with a teammate, decode pasted values, generate random
  UUIDs, or compare widths visually.

  ```sh
  cargo run --example webui            # open http://localhost:8080
  cargo run --example webui -- --help  # routes + env vars
  ```

  Defaults bind to `0.0.0.0:8080`. Override with `PORT=…` and `HOST=…`.

## Public API in one screen

```rust
pub struct Uuid { /* 16 bytes */ }
pub const NIL: Uuid;
pub const MAX: Uuid;

// Build / inspect
Uuid::from_bytes([u8; 16])
Uuid::from_u128(u128) / to_u128()
uuid.version() / uuid.variant()
impl FromStr / Display / Debug / From / TryFrom / AsRef<[u8]>

// Encoders (every one returns String)
to_hex / to_hex_dash
to_base32 / to_base32_hex / to_base32_humans
to_base64 / to_base64_url / to_base64_sort
to_base85_z85
to_base36 / to_base52 / to_base58_btc / to_base62_ieee / to_base62_sort
to_ncname_32 / to_ncname_58 / to_ncname_64
to_base62id

// Decoders — &str → Result<Uuid, DecodeError>
from_hex / from_base32 / … / from_ncname_64 / from_base62id
parse(&str)  // autodetect by length
```

All decoders are strict: oversized inputs are rejected in O(1), bad
characters return `DecodeError::InvalidCharacter` with the byte
position, and the NCName decoders reject *non-canonical* aliases (so a
UUID always has exactly one valid NCName encoding — important when
the encoded form is a primary key).

## Optional `serde`

```toml
uuid_altenc = { version = "0.1", features = ["serde"] }
```

Human-readable formats (JSON, YAML, TOML…) emit the canonical hex-dash
form; binary formats (bincode, MessagePack…) emit the raw 16 bytes.

## Conformance

`cargo test` runs every Appendix C vector from the draft in both
directions, plus property round-trips on 256 random UUIDs, an RFC 9562
conformance suite (`tests/rfc9562_conformance.rs`), and a security
regression suite (`tests/security.rs`).

A small Dart project that runs the reference [`uuid-format-tester`][uft]
encoders on the same UUIDs and prints the outputs side by side, as an
independent cross-check, lives in the
[`comparison/`](https://github.com/davlgd/uuid_altenc/tree/main/comparison)
directory of the repository (not shipped on crates.io).

[uft]: https://github.com/daegalus/uuid-format-tester

## License

Apache-2.0. © 2026 davlgd. See [`LICENSE`](LICENSE).

## References

- IETF draft: <https://datatracker.ietf.org/doc/draft-davis-uuidrev-alt-uuid-encoding-methods/>
- Draft source & issue tracker: <https://github.com/uuid6/new-uuid-encoding-techniques-ietf-draft>
- Reference encoder/cross-check tool: <https://github.com/daegalus/uuid-format-tester>
- RFC 9562 (UUIDs, May 2024): <https://datatracker.ietf.org/doc/html/rfc9562>
- UUID-NCName: <https://datatracker.ietf.org/doc/draft-taylor-uuid-ncname/>

[draft]: https://datatracker.ietf.org/doc/draft-davis-uuidrev-alt-uuid-encoding-methods/
