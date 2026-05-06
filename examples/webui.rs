//! Self-contained web UI for `uuid_altenc`.
//!
//! Run with: `cargo run --example webui`
//! Then open: <http://localhost:8080>
//!
//! Bound to `0.0.0.0:8080` by default; override the port with `PORT` and
//! the bind address with `HOST`. The random-UUID endpoint uses a demo-grade
//! LCG that must not be reused for real identifiers.
//!
//! The server uses only the standard library — no `tokio`, `hyper`,
//! `serde_json` — so the example compiles in a couple of seconds and
//! stays in lock-step with the crate's zero-dependency philosophy.
//!
//! Routes:
//!   * `GET /`                            → HTML page
//!   * `GET /encode?uuid=…`               → JSON: every encoding of the UUID
//!   * `GET /decode?value=…&encoding=…`   → JSON: decoded UUID
//!   * `GET /random?v=4|7|nil|max`        → JSON: a freshly-built UUID

use std::env;
use std::fmt::Write as FmtWrite;
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use uuid_altenc::{DecodeError, MAX, NIL, Uuid};

const INDEX_HTML: &str = include_str!("webui_index.html");

/// Per-line cap when draining HTTP request data. 8 KiB is generous for
/// every legitimate header / request line we care about.
const MAX_LINE_BYTES: u64 = 8 * 1024;

const HELP_TEXT: &str = "\
uuid_altenc webui — interactive demo of every encoding in the crate.

USAGE:
    cargo run --example webui

ROUTES:
    GET /                            HTML page
    GET /encode?uuid=…               JSON: every encoding of the UUID
    GET /decode?value=…&encoding=…   JSON: decoded UUID
    GET /random?v=4|7|nil|max        JSON: a freshly-built UUID

ENVIRONMENT:
    HOST   bind address           (default: 0.0.0.0)
    PORT   listening port         (default: 8080)

The random endpoint uses a demo-grade LCG; never reuse those UUIDs as
real identifiers.
";

type EncodeFn = fn(&Uuid) -> String;
type DecodeFn = fn(&str) -> Result<Uuid, DecodeError>;

/// One supported encoding. Adding a new encoding means adding one row
/// here (and one `<option>` to the HTML dropdown). Set `decode_key` to
/// `""` if the encoding is decode-only via auto-detect.
struct Encoding {
    name: &'static str,
    group: &'static str,
    decode_key: &'static str,
    encode: EncodeFn,
    decode: DecodeFn,
    note: &'static str,
}

const ENCODINGS: &[Encoding] = &[
    Encoding { name: "hex-dash",     group: "Hex baseline",                    decode_key: "",            encode: Uuid::to_hex_dash,     decode: uuid_altenc::from_hex,        note: "RFC 9562 canonical, 36 chars" },
    Encoding { name: "hex",          group: "Hex baseline",                    decode_key: "hex",         encode: Uuid::to_hex,          decode: uuid_altenc::from_hex,        note: "Plain lowercase hex, 32 chars" },
    Encoding { name: "base32",       group: "Fixed-width bases (32 / 64 / 85)", decode_key: "base32",       encode: Uuid::to_base32,        decode: uuid_altenc::from_base32,       note: "RFC 4648 §6 standard alphabet" },
    Encoding { name: "base32hex",    group: "Fixed-width bases (32 / 64 / 85)", decode_key: "base32hex",    encode: Uuid::to_base32_hex,    decode: uuid_altenc::from_base32_hex,   note: "Sortable: lex order = binary order" },
    Encoding { name: "base32humans", group: "Fixed-width bases (32 / 64 / 85)", decode_key: "base32humans", encode: Uuid::to_base32_humans, decode: uuid_altenc::from_base32_humans,note: "Crockford-style — no I/L/O/U" },
    Encoding { name: "base64",       group: "Fixed-width bases (32 / 64 / 85)", decode_key: "base64",       encode: Uuid::to_base64,        decode: uuid_altenc::from_base64,       note: "RFC 4648 §4 — uses + and /" },
    Encoding { name: "base64url",    group: "Fixed-width bases (32 / 64 / 85)", decode_key: "base64url",    encode: Uuid::to_base64_url,    decode: uuid_altenc::from_base64_url,   note: "URL-safe — uses - and _" },
    Encoding { name: "base64sort",   group: "Fixed-width bases (32 / 64 / 85)", decode_key: "base64sort",   encode: Uuid::to_base64_sort,   decode: uuid_altenc::from_base64_sort,  note: "Sortable text = sortable binary" },
    Encoding { name: "Z85",          group: "Fixed-width bases (32 / 64 / 85)", decode_key: "base85z85",    encode: Uuid::to_base85_z85,    decode: uuid_altenc::from_base85_z85,   note: "20 chars — most compact form" },
    Encoding { name: "base36",       group: "Variable-width integer bases",     decode_key: "base36",       encode: Uuid::to_base36,        decode: uuid_altenc::from_base36,       note: "Digits + uppercase letters" },
    Encoding { name: "base52",       group: "Variable-width integer bases",     decode_key: "base52",       encode: Uuid::to_base52,        decode: uuid_altenc::from_base52,       note: "Letters only, no digits" },
    Encoding { name: "base58btc",    group: "Variable-width integer bases",     decode_key: "base58btc",    encode: Uuid::to_base58_btc,    decode: uuid_altenc::from_base58_btc,   note: "Bitcoin alphabet — no 0/O/I/l" },
    Encoding { name: "base62 IEEE",  group: "Variable-width integer bases",     decode_key: "base62ieee",   encode: Uuid::to_base62_ieee,   decode: uuid_altenc::from_base62_ieee,  note: "A-Z, a-z, 0-9" },
    Encoding { name: "base62 sort",  group: "Variable-width integer bases",     decode_key: "base62sort",   encode: Uuid::to_base62_sort,   decode: uuid_altenc::from_base62_sort,  note: "Sortable: digits before letters" },
    Encoding { name: "ncname-32",    group: "Safe as HTML/XML id",              decode_key: "ncname32",     encode: Uuid::to_ncname_32,     decode: uuid_altenc::from_ncname_32,    note: "Case-insensitive XML id" },
    Encoding { name: "ncname-58",    group: "Safe as HTML/XML id",              decode_key: "ncname58",     encode: Uuid::to_ncname_58,     decode: uuid_altenc::from_ncname_58,    note: "Mixed case, transcribable" },
    Encoding { name: "ncname-64",    group: "Safe as HTML/XML id",              decode_key: "ncname64",     encode: Uuid::to_ncname_64,     decode: uuid_altenc::from_ncname_64,    note: "22 chars, A-P bookends" },
    Encoding { name: "base62id",     group: "Safe as HTML/XML id",              decode_key: "base62id",     encode: Uuid::to_base62id,      decode: uuid_altenc::from_base62id,     note: "22 chars, sentinel-validated" },
];

fn main() {
    if env::args().any(|a| a == "--help" || a == "-h") {
        print!("{HELP_TEXT}");
        return;
    }
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let addr = format!("{host}:{port}");
    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("could not bind to {addr}: {e}");
            if e.kind() == ErrorKind::AddrInUse {
                eprintln!("hint: another process is using port {port}.");
                eprintln!(
                    "      retry with `PORT={} cargo run --example webui`.",
                    port.saturating_add(1).max(1024),
                );
            }
            std::process::exit(1);
        }
    };
    println!("uuid_altenc webui listening on http://{addr}");
    println!("  open http://localhost:{port} in a browser");
    println!("  override with PORT=… HOST=… (Ctrl-C to stop)");
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                std::thread::spawn(move || {
                    if let Err(e) = handle_connection(s) {
                        eprintln!("connection error: {e}");
                    }
                });
            }
            Err(e) => eprintln!("accept error: {e}"),
        }
    }
}

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    if read_line_capped(&mut reader, &mut request_line)? == 0 {
        return Ok(());
    }
    // `read_line` stops at the cap without consuming a newline; a fully
    // formed line always ends in '\n'. Missing terminator ⇒ over budget.
    if !request_line.ends_with('\n') {
        return write_response(&mut stream, &http(414, "text/plain", "URI too long"));
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("");

    // Drain headers; we don't need their contents.
    loop {
        let mut h = String::new();
        let n = read_line_capped(&mut reader, &mut h)?;
        if n == 0 {
            // Connection closed mid-headers — caller went away; nothing to do.
            return Ok(());
        }
        if h == "\r\n" || h == "\n" {
            break;
        }
        if !h.ends_with('\n') {
            return write_response(&mut stream, &http(431, "text/plain", "headers too large"));
        }
    }

    let (path, query) = target
        .split_once('?')
        .map_or((target, ""), |(p, q)| (p, q));

    let response = match (method, path) {
        ("GET", "/") => http(200, "text/html; charset=utf-8", INDEX_HTML),
        ("GET", "/encode") => api_encode(query),
        ("GET", "/decode") => api_decode(query),
        ("GET", "/random") => api_random(query),
        _ => http(404, "text/plain; charset=utf-8", "not found"),
    };

    write_response(&mut stream, &response)
}

fn write_response(stream: &mut TcpStream, response: &str) -> std::io::Result<()> {
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

/// Read one line from `reader` but never buffer more than `MAX_LINE_BYTES`
/// — caps slow-loris / huge-URI `DoS` attempts before they OOM the process.
///
/// If the line was truncated at the cap, `out` will be exactly
/// `MAX_LINE_BYTES` long and won't end in `\n`; callers check
/// `!out.ends_with('\n')` to detect that case.
fn read_line_capped(reader: &mut BufReader<TcpStream>, out: &mut String) -> std::io::Result<usize> {
    let mut limited = reader.take(MAX_LINE_BYTES);
    limited.read_line(out)
}

fn http(status: u16, content_type: &str, body: &str) -> String {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        414 => "URI Too Long",
        431 => "Request Header Fields Too Large",
        _ => "Error",
    };
    format!(
        "HTTP/1.1 {status} {reason}\r\n\
         Content-Type: {content_type}\r\n\
         Content-Length: {len}\r\n\
         Cache-Control: no-store\r\n\
         X-Content-Type-Options: nosniff\r\n\
         Referrer-Policy: no-referrer\r\n\
         Connection: close\r\n\
         \r\n\
         {body}",
        len = body.len(),
    )
}

fn json_response(status: u16, body: &str) -> String {
    http(status, "application/json; charset=utf-8", body)
}

// ---- Query parsing -------------------------------------------------------

/// Look up a single key in a `&`-separated, URL-encoded query string.
/// Returns the first match (no allocation if the key is absent).
fn query_get(q: &str, key: &str) -> Option<String> {
    q.split('&').filter(|s| !s.is_empty()).find_map(|pair| {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        (url_decode(k) == key).then(|| url_decode(v))
    })
}

fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                if let (Some(h), Some(l)) = (hex_digit(bytes[i + 1]), hex_digit(bytes[i + 2])) {
                    out.push(h * 16 + l);
                    i += 3;
                } else {
                    out.push(b'%');
                    i += 1;
                }
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

const fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

// ---- JSON helpers (hand-rolled — no serde_json dep) ---------------------

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

fn json_str(s: &str) -> String {
    format!("\"{}\"", json_escape(s))
}

fn json_error(msg: &str) -> String {
    format!("{{\"ok\":false,\"error\":{}}}", json_str(msg))
}

/// Translate `DecodeError` into a sentence a non-Rust visitor can act on.
/// Keeps the encoding name so users know which scheme they were trying.
fn friendly_decode_error(e: &DecodeError) -> String {
    match e {
        DecodeError::InvalidLength { encoding, expected, got } => {
            format!("This doesn't look like {encoding}: expected {expected} characters, got {got}.")
        }
        DecodeError::InvalidCharacter { encoding, position, byte } => {
            let printable = if byte.is_ascii_graphic() { char::from(*byte) } else { '\u{fffd}' };
            format!("Character {printable:?} at position {position} is not part of the {encoding} alphabet.")
        }
        DecodeError::Overflow { encoding } => {
            format!("That {encoding} value is too large to fit in a UUID's 128 bits.")
        }
        DecodeError::NonCanonicalPadding { encoding } => {
            format!("Looks like a corrupted {encoding} value — its padding bits aren't all zero.")
        }
        DecodeError::InputTooLong { encoding, max, got } => {
            format!("That's {got} characters of {encoding}, but valid inputs are at most {max}.")
        }
        DecodeError::SchemeViolation { encoding, reason } => {
            format!("{encoding}: {reason}.")
        }
        DecodeError::AmbiguousLength { got } => {
            format!("Can't tell which encoding this is — {got}-char inputs match multiple schemes. Pick one explicitly.")
        }
        _ => format!("{e}"),
    }
}

// ---- /encode -------------------------------------------------------------

fn api_encode(query: &str) -> String {
    let raw = query_get(query, "uuid").unwrap_or_default();
    if raw.is_empty() {
        return json_response(400, &json_error("missing parameter: uuid"));
    }
    match uuid_altenc::parse(&raw) {
        Ok(uuid) => json_response(200, &encode_payload(uuid)),
        Err(e) => json_response(400, &json_error(&friendly_decode_error(&e))),
    }
}

fn encode_payload(uuid: Uuid) -> String {
    let mut bytes_pretty = String::with_capacity(48);
    for (i, b) in uuid.as_bytes().iter().enumerate() {
        if i > 0 {
            bytes_pretty.push(' ');
        }
        let _ = write!(bytes_pretty, "{b:02x}");
    }

    let mut entries = String::new();
    for (i, enc) in ENCODINGS.iter().enumerate() {
        if i > 0 {
            entries.push(',');
        }
        let value = (enc.encode)(&uuid);
        let _ = write!(
            entries,
            "{{\"name\":{n},\"group\":{g},\"value\":{v},\"width\":{w},\"note\":{note}}}",
            n = json_str(enc.name),
            g = json_str(enc.group),
            v = json_str(&value),
            w = value.chars().count(),
            note = json_str(enc.note),
        );
    }

    format!(
        "{{\"ok\":true,\
           \"uuid\":{u},\
           \"version\":{v},\
           \"variant\":{vr},\
           \"bytes\":{bytes},\
           \"encodings\":[{entries}]}}",
        u = json_str(&uuid.to_hex_dash()),
        v = uuid.version(),
        vr = json_str(&format!("0x{:x}", uuid.variant())),
        bytes = json_str(&bytes_pretty),
    )
}

// ---- /decode -------------------------------------------------------------

fn api_decode(query: &str) -> String {
    let value = query_get(query, "value").unwrap_or_default();
    let encoding = query_get(query, "encoding").unwrap_or_default();
    if value.is_empty() {
        return json_response(400, &json_error("missing parameter: value"));
    }

    let (label, result) = if encoding.is_empty() {
        ("auto", uuid_altenc::parse(&value))
    } else {
        match ENCODINGS
            .iter()
            .find(|e| !e.decode_key.is_empty() && e.decode_key == encoding)
        {
            Some(enc) => (enc.name, (enc.decode)(&value)),
            None => {
                return json_response(400, &json_error(&format!("Unknown encoding: {encoding}.")));
            }
        }
    };

    match result {
        Ok(uuid) => json_response(
            200,
            &format!(
                "{{\"ok\":true,\
                   \"uuid\":{u},\
                   \"version\":{v},\
                   \"variant\":{vr},\
                   \"encoding\":{e}}}",
                u = json_str(&uuid.to_hex_dash()),
                v = uuid.version(),
                vr = json_str(&format!("0x{:x}", uuid.variant())),
                e = json_str(label),
            ),
        ),
        Err(e) => json_response(400, &json_error(&friendly_decode_error(&e))),
    }
}

// ---- /random -------------------------------------------------------------

fn api_random(query: &str) -> String {
    let v = query_get(query, "v").unwrap_or_else(|| "4".to_string());
    let uuid = match v.as_str() {
        "4" => random_v4(),
        "7" => random_v7(),
        "nil" => NIL,
        "max" => MAX,
        other => {
            return json_response(
                400,
                &json_error(&format!("Unknown version: {other} (expected 4, 7, nil, max).")),
            );
        }
    };
    json_response(
        200,
        &format!("{{\"ok\":true,\"uuid\":{}}}", json_str(&uuid.to_hex_dash())),
    )
}

/// Build a v4 (random) UUID. Demo-quality randomness only — sufficient
/// to show off the encodings, not for production identifiers.
fn random_v4() -> Uuid {
    let mut b = random_bytes();
    b[6] = (b[6] & 0x0f) | 0x40; // version
    b[8] = (b[8] & 0x3f) | 0x80; // RFC 4122 variant
    Uuid::from_bytes(b)
}

/// Build a v7 (Unix-time-ordered) UUID. The first 48 bits encode the
/// current millisecond since the Unix epoch; the rest is random.
fn random_v7() -> Uuid {
    let mut b = random_bytes();
    let now_ms: u128 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    // Copy the low 48 bits of the timestamp into bytes 0..6 without any
    // narrowing cast: `to_be_bytes` gives us the full big-endian array
    // and we slice off the high bytes that don't fit in a v7 layout.
    let ms_be = now_ms.to_be_bytes();
    b[0..6].copy_from_slice(&ms_be[10..16]);
    b[6] = (b[6] & 0x0f) | 0x70;
    b[8] = (b[8] & 0x3f) | 0x80;
    Uuid::from_bytes(b)
}

/// Cheap, time-seeded LCG. Good enough to fill 16 demo bytes; do **not**
/// reuse for security purposes.
///
/// Seeded once from the process clock; subsequent calls advance the
/// shared state so back-to-back invocations within the same nanosecond
/// still yield distinct outputs.
// The two `as` casts in this function are deliberate truncations:
//   1. `u128 → u64` for the seed: any 64 bits of clock entropy will do.
//   2. `u64 → u8`   for byte extraction: the high half of the LCG state
//      has the longest period, so we take the high 8 bits each step.
// Both are exactly what the algorithm wants — narrowing is the point.
#[allow(clippy::cast_possible_truncation)]
fn random_bytes() -> [u8; 16] {
    static SEED: AtomicU64 = AtomicU64::new(0);
    let mut state = SEED.load(Ordering::Relaxed);
    if state == 0 {
        state = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0xdead_beef_cafe_f00d_u64, |d| d.as_nanos() as u64)
            ^ u64::from(std::process::id());
        if state == 0 {
            state = 0xdead_beef_cafe_f00d;
        }
    }
    let mut out = [0u8; 16];
    for slot in &mut out {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        *slot = (state >> 33) as u8;
    }
    SEED.store(state, Ordering::Relaxed);
    out
}
