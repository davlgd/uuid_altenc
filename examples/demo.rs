//! Run with `cargo run --example demo`.
//!
//! A guided tour of `uuid_altenc`. We take one UUID, encode it in every
//! supported format, and explain — for each — what the format is good for
//! and how its output differs from the canonical hex-dash form.
//!
//! The same 128 bits get printed many ways. Compare the widths and
//! alphabets to see why you might pick one encoding over another.

use std::env;

use uuid_altenc::{MAX, NIL, Uuid};

const HELP: &str = "\
uuid_altenc demo — fixed walkthrough of every encoding.

USAGE:
    cargo run --example demo

This example is a scripted tour: it takes one UUID and prints every
encoding side by side, with explanatory notes. No flags or env vars.

For an interactive playground (random UUIDs, paste-to-decode), run:
    cargo run --example webui
";

fn section(title: &str) {
    println!();
    println!("== {title} ==");
}

fn line(label: &str, value: &str, note: &str) {
    // Two-column layout: encoded value, then a short hint about width
    // and the alphabet/use case. The value column is wide enough to fit
    // the longest output (32-char plain hex) without misaligning notes.
    println!("  {label:<14} {value:<34}  {note}");
}

fn main() {
    if env::args().any(|a| a == "--help" || a == "-h") {
        print!("{HELP}");
        return;
    }

    // A real-world example: RFC 9562 §A.3 v4 (random) UUID.
    let uuid: Uuid = "919108f7-52d1-4320-9bac-f847db4148a8"
        .parse()
        .expect("static input is valid");

    println!("Input UUID (RFC 9562 §A.3 worked example, v4):");
    println!("  {uuid}");
    println!();
    println!("That is 36 characters of UTF-8 carrying only 128 bits of");
    println!("information. The encodings below pack the same 128 bits into");
    println!("smaller, URL-safe, sortable, or id-safe strings.");

    section("Hex (the baseline)");
    line("hex-dash",     &uuid.to_hex_dash(), "36 chars — RFC 9562 canonical");
    line("hex (plain)",  &uuid.to_hex(),      "32 chars — no separators");

    section("Bit-stream encodings (fixed width)");
    println!("  Each output is the same length for every UUID.");
    println!();
    line("base32",        &uuid.to_base32(),        "26 chars — RFC 4648 §6, A-Z + 2-7");
    line("base32hex",     &uuid.to_base32_hex(),    "26 chars — sortable (lex = binary)");
    line("base32humans",  &uuid.to_base32_humans(), "26 chars — Crockford, no I/L/O/U");
    line("base64",        &uuid.to_base64(),        "22 chars — RFC 4648 §4");
    line("base64url",     &uuid.to_base64_url(),    "22 chars — URL-safe, no padding");
    line("base64sort",    &uuid.to_base64_sort(),   "22 chars — sortable as text");
    line("Z85 (base85)",  &uuid.to_base85_z85(),    "20 chars — most compact, all ASCII printable");

    section("Integer-style encodings (variable width)");
    println!("  The UUID is treated as one big integer; leading zeros are");
    println!("  dropped, so the Nil UUID encodes to a single character.");
    println!();
    line("base36",        &uuid.to_base36(),        "≤25 chars — digits + uppercase");
    line("base52",        &uuid.to_base52(),        "≤23 chars — letters only");
    line("base58btc",     &uuid.to_base58_btc(),    "≤22 chars — Bitcoin: no 0/O/I/l");
    line("base62 IEEE",   &uuid.to_base62_ieee(),   "≤22 chars — A-Z a-z 0-9");
    line("base62 sort",   &uuid.to_base62_sort(),   "≤22 chars — 0-9 A-Z a-z, sortable");

    section("HTML/XML/CSS id-safe encodings");
    println!("  All four below produce strings valid as HTML/XML/CSS ids");
    println!("  without escaping; the three NCName variants additionally");
    println!("  start with a letter in A-P.");
    println!();
    line("ncname-32",     &uuid.to_ncname_32(),     "26 chars — case-insensitive host");
    line("ncname-58",     &uuid.to_ncname_58(),     "23 chars — readable mixed case");
    line("ncname-64",     &uuid.to_ncname_64(),     "22 chars — most compact id-safe");
    line("base62id",      &uuid.to_base62id(),      "22 chars — sentinel-validated");

    section("Round-trip: every encoding is reversible");
    let s = uuid.to_base58_btc();
    let back = uuid_altenc::from_base58_btc(&s).expect("round-trip");
    println!("  uuid.to_base58_btc()        -> {s:?}");
    println!("  from_base58_btc({s:?}) -> {back}");
    assert_eq!(back, uuid);

    section("parse(): autodetect by length");
    println!("  When the length pins down a single encoding, parse() works.");
    println!("  Otherwise (notably 22-char inputs) call the specific decoder.");
    println!();
    let samples = [
        ("hex-dash (36)",  uuid.to_hex_dash()),
        ("hex (32)",       uuid.to_hex()),
        ("base32hex (26)", uuid.to_base32_hex()),
        ("base36 (≤25)",   uuid.to_base36()),
        ("ncname-58 (23)", uuid.to_ncname_58()),
        ("Z85 (20)",       uuid.to_base85_z85()),
    ];
    for (label, s) in samples {
        let got = uuid_altenc::parse(&s).expect("autodetect");
        println!("  {label:<16} {s:<34} -> {got}");
    }
    let ambiguous = uuid.to_ncname_64(); // 22 chars
    let err = uuid_altenc::parse(&ambiguous).unwrap_err();
    println!();
    println!("  parse({ambiguous:?})");
    println!("    -> error: {err}");
    println!("    (22-char inputs overlap across six encoders: ncname-64,");
    println!("     base64url, base62id, base58btc, base62 IEEE, base62 sort —");
    println!("     pick the one you want explicitly.)");

    section("Special UUIDs");
    println!("  Nil  ({NIL})");
    println!("       base58btc -> {:?}", NIL.to_base58_btc());
    println!("  Max  ({MAX})");
    println!("       base58btc -> {:?}", MAX.to_base58_btc());
    println!();
    println!("  Notice: Nil and Max collapse to one character each in");
    println!("  variable-width encodings — that's the integer-style property");
    println!("  in action, contrasting with the fixed-width bit-stream forms.");

    section("Inspecting a UUID");
    println!("  version() = {}    -> RFC 9562 §4 version field", uuid.version());
    println!("                       (this one is 4 = random)");
    println!("  variant() = 0x{:x}  -> top nibble of byte 8 (RFC 9562 variant)", uuid.variant());

    println!();
    println!("Tip: pick `to_base64_url` for URLs, `to_base32_hex` for sortable");
    println!("DB keys, `to_ncname_64` or `to_base62id` for HTML ids, `to_hex`");
    println!("when you just want what RFC 9562 already gives you.");
    println!();
    println!("Next step → for an interactive version with random UUIDs and");
    println!("           paste-to-decode, run: cargo run --example webui");
}
