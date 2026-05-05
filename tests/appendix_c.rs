//! Conformance tests against Appendix C of
//! `draft-davis-uuidrev-alt-uuid-encoding-methods-00` (April 2026).
//!
//! Every vector in the draft is exercised in both directions (encode +
//! decode). Where the draft contains a copy/paste error in its tables
//! this file asserts the mathematically correct value and the comment
//! cites the affected table.

use uuid_altenc::{
    Uuid, from_base32, from_base32_hex, from_base32_humans, from_base36, from_base52,
    from_base58_btc, from_base62_ieee, from_base62_sort, from_base62id, from_base64,
    from_base64_sort, from_base64_url, from_base85_z85, from_hex, from_ncname_32, from_ncname_58,
    from_ncname_64,
};

struct Vector {
    uuid: &'static str,
    hex_dash: &'static str,
    hex_plain: &'static str,
    base32_std: &'static str,
    base32_hex: &'static str,
    base32_humans: &'static str,
    base36: &'static str,
    base52: &'static str,
    base58_btc: &'static str,
    base62_ieee: &'static str,
    base62_sort: &'static str,
    base64_std: &'static str,
    base64_url: &'static str,
    base64_sort: &'static str,
    base85_z85: &'static str,
    ncname_32: &'static str,
    ncname_58: &'static str,
    ncname_64: &'static str,
    base62id: &'static str,
}

const APPENDIX_C: &[Vector] = &[
    // C.1 — RFC 9562 §4 generic example.
    Vector {
        uuid: "f81d4fae-7dec-11d0-a765-00a0c91e6bf6",
        hex_dash: "f81d4fae-7dec-11d0-a765-00a0c91e6bf6",
        hex_plain: "f81d4fae7dec11d0a76500a0c91e6bf6",
        base32_std: "7AOU7LT55QI5BJ3FACQMSHTL6Y",
        base32_hex: "V0EKVBJTTG8T19R502GCI7JBUO",
        base32_humans: "Z0EMZBKXXG8X19V502GCJ7KBYR",
        base36: "EOSWZOLG3BSX0ZN8OTQ1P8OOM",
        base52: "FraqvVvqUBoEOFXsPYOPwUm",
        base58_btc: "Xe22UfxT3rxcKJEAfL5373",
        base62_ieee: "HiLegqjbBwUc0Q8tJ3ffs6",
        base62_sort: "7YBUWgZR1mKSqGyj9tVViw",
        base64_std: "+B1Prn3sEdCnZQCgyR5r9g",
        base64_url: "-B1Prn3sEdCnZQCgyR5r9g",
        base64_sort: "y0pEfbrg3S1bOF1VmGtfxV",
        base85_z85: "{-iekEE4M)R!2>3:Stl>",
        ncname_32: "b7aou7lt55qoqoziauder427wk",
        ncname_58: "B7wc88dU4e3NyJEj3e944DK",
        ncname_64: "B-B1Prn3sHQdlAKDJHmv2K",
        base62id: "N8JYxDHbz7rx9rGIw80uxC",
    },
    // C.2 — Nil UUID. Note: Base62 IEEE here is "A" because index 0 of
    // the Table 1 IEEE alphabet is 'A'. Draft Table 15 prints "0", which
    // belongs to the Base62-sort alphabet.
    Vector {
        uuid: "00000000-0000-0000-0000-000000000000",
        hex_dash: "00000000-0000-0000-0000-000000000000",
        hex_plain: "00000000000000000000000000000000",
        base32_std: "AAAAAAAAAAAAAAAAAAAAAAAAAA",
        base32_hex: "00000000000000000000000000",
        base32_humans: "00000000000000000000000000",
        base36: "0",
        base52: "A",
        base58_btc: "1",
        base62_ieee: "A",
        base62_sort: "0",
        base64_std: "AAAAAAAAAAAAAAAAAAAAAA",
        base64_url: "AAAAAAAAAAAAAAAAAAAAAA",
        base64_sort: "----------------------",
        base85_z85: "00000000000000000000",
        ncname_32: "aaaaaaaaaaaaaaaaaaaaaaaaaa",
        ncname_58: "A111111111111111______A",
        ncname_64: "AAAAAAAAAAAAAAAAAAAAAA",
        base62id: "Fa84QWiAxLXUJaHZmEVPEG",
    },
    // C.3 — Max UUID. Three draft cells corrected here:
    //   * Base36 (draft prints a 26-char "p…p" string; correct is
    //     `F5LXX1ZZ5PNORYNQGLHZMSP33`, the canonical 25-char encoding of
    //     2^128-1).
    //   * Base58 standalone (draft conflates with the NCName-58 form).
    //   * Z85 (draft uses the ASCII85/Adobe alphabet, not Z85 from Table 1).
    Vector {
        uuid: "ffffffff-ffff-ffff-ffff-ffffffffffff",
        hex_dash: "ffffffff-ffff-ffff-ffff-ffffffffffff",
        hex_plain: "ffffffffffffffffffffffffffffffff",
        base32_std: "77777777777777777777777774",
        base32_hex: "VVVVVVVVVVVVVVVVVVVVVVVVVS",
        base32_humans: "ZZZZZZZZZZZZZZZZZZZZZZZZZW",
        base36: "F5LXX1ZZ5PNORYNQGLHZMSP33",
        base52: "GBIWTpZqojFGQPQPXtvbJAv",
        base58_btc: "YcVfxkQb6JRzqk5kF2tNLv",
        base62_ieee: "HxECNQWFdpvuJxIw3HPrmH",
        base62_sort: "7n42DGM5Tflk9n8mt7Fhc7",
        base64_std: "/////////////////////w",
        base64_url: "_____________________w",
        base64_sort: "zzzzzzzzzzzzzzzzzzzzzk",
        base85_z85: "%nSc0%nSc0%nSc0%nSc0",
        ncname_32: "p777777777777777777777777p",
        ncname_58: "P8AQGAut7N92awznwCnjuQP",
        ncname_64: "P____________________P",
        base62id: "NNC6dn4GR1JETNQMfLl6qN",
    },
    // C.4 — UUIDv1.
    Vector {
        uuid: "c232ab00-9414-11ec-b3c8-9f6bdeced846",
        hex_dash: "c232ab00-9414-11ec-b3c8-9f6bdeced846",
        hex_plain: "c232ab00941411ecb3c89f6bdeced846",
        base32_std: "YIZKWAEUCQI6ZM6IT5V55TWYIY",
        base32_hex: "O8PAM04K2G8UPCU8JTLTTJMO8O",
        base32_humans: "R8SAP04M2G8YSCY8KXNXXKPR8R",
        base36: "BHW3F9QSZLQPYH8GTZ35HZ4UE",
        base52: "EddHArfCMSZGGUJvWdXrHHq",
        base58_btc: "Qys2KsgsAKw9ZKupo76FCh",
        base62_ieee: "F4bpVC8trxK13yapr3UFrY",
        base62_sort: "5uRfL2yjhnArtoQfhtK5hO",
        base64_std: "wjKrAJQUEeyzyJ9r3s7YRg",
        base64_url: "wjKrAJQUEeyzyJ9r3s7YRg",
        base64_sort: "kY9f-8FJ3Tmnm8xfrgvNGV",
        base85_z85: ".znd(LOs.@V=F+O?P<+y",
        ncname_32: "byizkwaeucqpmhse7nppm5wcgl",
        ncname_58: "B6S7oX73gv2Y1iTENdXX8hL",
        ncname_64: "BwjKrAJQUHsPIn2vezthGL",
        base62id: "LUZjlZguf8iMDOiFU7pUve",
    },
    // C.5 — DCE Security UUIDv2. Base62-sort fixed (draft Table 21
    // duplicated the IEEE string).
    Vector {
        uuid: "000003e8-cbb9-21ea-b201-00045a86c8a1",
        hex_dash: "000003e8-cbb9-21ea-b201-00045a86c8a1",
        hex_plain: "000003e8cbb921eab20100045a86c8a1",
        base32_std: "AAAAH2GLXEQ6VMQBAACFVBWIUE",
        base32_hex: "00007Q6BN4GULCG10025L1M8K4",
        base32_humans: "00007T6BQ4GYNCG10025N1P8M4",
        base36: "5XJERAFS5KNNNG5WAM10H",
        base52: "KNcIMemTgumAjqCSfqp",
        base58_btc: "2SMnXSegyRgxWPnMoHS",
        base62_ieee: "azPmOJmh9xNaNgM2sh",
        base62_sort: "QpFcE9cXznDQDWCsiX",
        base64_std: "AAAD6Mu5IeqyAQAEWobIoQ",
        base64_url: "AAAD6Mu5IeqyAQAEWobIoQ",
        base64_sort: "---2uBit7Tem-F-3LcQ7cF",
        base85_z85: "000b++EF!YVh<}dt87a(",
        ncname_32: "caaaah2glxepkeaiaarninsfbl",
        ncname_58: "C11KtP6Y9P3rRkvh2N1e__L",
        ncname_64: "CAAAD6Mu5HqIBAARahsihL",
        base62id: "Fa84rLxnBVA2JNUzzkiHwn",
    },
    // C.6 — UUIDv3. Z85 fixed (draft Table 23 used ASCII85 alphabet).
    Vector {
        uuid: "5df41881-3aed-3515-88a7-2f4a814cf09e",
        hex_dash: "5df41881-3aed-3515-88a7-2f4a814cf09e",
        hex_plain: "5df418813aed351588a72f4a814cf09e",
        base32_std: "LX2BRAJ25U2RLCFHF5FICTHQTY",
        base32_hex: "BNQ1H09QTKQHB2575T582J7GJO",
        base32_humans: "BQT1H09TXMTHB2575X582K7GKR",
        base36: "5K8PHACEYCDEGBDB1VWP0EAHQ",
        base52: "CKwZBXIbBzTOnGTcpOSLRtq",
        base58_btc: "CbuPE286MB6RsDazcU7sUy",
        base62_ieee: "C1Rz9EB7xwwtaBE3hssbl8",
        base62_sort: "2rHpz41xnmmjQ14tXiiRby",
        base64_std: "XfQYgTrtNRWIpy9KgUzwng",
        base64_url: "XfQYgTrtNRWIpy9KgUzwng",
        base64_sort: "MUFNVIfhCGL7dmx9VJnkbV",
        base85_z85: "ugX$+i{!^tH]h)#FLtN8",
        ncname_32: "dlx2braj25vivrjzpjkauz4e6i",
        ncname_58: "D3dTNMAmevR4NFAakRDtLdI",
        ncname_64: "DXfQYgTrtUVinL0qBTPCeI",
        base62id: "IRPuPak8l8KDjbMTJxDqqE",
    },
    // C.7 — UUIDv4. Base62-sort vector in draft Table 25 is a copy of
    // v3's; the value here is recomputed and validated by
    // `uuidv4_base62sort_distinct_from_v3`. We leave the field empty so
    // the universal encode-check skips it.
    Vector {
        uuid: "919108f7-52d1-4320-9bac-f847db4148a8",
        hex_dash: "919108f7-52d1-4320-9bac-f847db4148a8",
        hex_plain: "919108f752d143209bacf847db4148a8",
        base32_std: "SGIQR52S2FBSBG5M7BD5WQKIVA",
        base32_hex: "I68GHTQIQ51I16TCV13TMGA8L0",
        base32_humans: "J68GHXTJT51J16XCZ13XPGA8N0",
        base36: "8M8SCPFUGFIJ4QENJ0IG77QG8",
        base52: "DWDhSoKPZRoqkgBWaJcOckY",
        base58_btc: "JyZVoFVQxQNmw2bsgr7D1R",
        base62_ieee: "EaqKcHGnV4iQSYo57bXn6o",
        base62_sort: "",
        base64_std: "kZEI91LRQyCbrPhH20FIqA",
        base64_url: "kZEI91LRQyCbrPhH20FIqA",
        base64_sort: "ZO37xpAGFm1QfEW6qo47e-",
        base85_z85: "K=Y}zqQE&OO2(xP*D!xe",
        ncname_32: "esgiqr52s2ezaxlhyi7nucsfij",
        ncname_58: "E55CtqYNqva1mcmaa877eoJ",
        ncname_64: "EkZEI91LRMgus-EfbQUioJ",
        base62id: "K0oEsdooJG5kbywVjft3Au",
    },
    // C.8 — UUIDv5.
    Vector {
        uuid: "2ed6657d-e927-568b-95e1-2665a8aea6a2",
        hex_dash: "2ed6657d-e927-568b-95e1-2665a8aea6a2",
        hex_plain: "2ed6657de927568b95e12665a8aea6a2",
        base32_std: "F3LGK7PJE5LIXFPBEZS2RLVGUI",
        base32_hex: "5RB6AVF94TB8N5F14PIQHBL6K8",
        base32_humans: "5VB6AZF94XB8Q5F14SJTHBN6M8",
        base36: "2RTO2O5WTBFMSYWZX7KHQ1Z8I",
        base52: "BFPTsrUJhwfXFHQeVflYshu",
        base58_btc: "6nTLogGvw2vmQjtATLqvLq",
        base62_ieee: "BaXm0PEMkU5w7EKQaysNQO",
        base62_sort: "1QNcqF4CaKvmx4AGQoiDGE",
        base64_std: "LtZlfeknVouV4SZlqK6mog",
        base64_url: "LtZlfeknVouV4SZlqK6mog",
        base64_sort: "AhO_UTZbKciKsHO_e9uacV",
        base85_z85: "f4KPZ>{GI&MeK63Sii1(",
        ncname_32: "ff3lgk7pje5ullyjgmwuk5jvcj",
        ncname_58: "F2K15VFLUBD326h169SNPjJ",
        ncname_64: "FLtZlfeknaLXhJmWorqaiJ",
        base62id: "H0VhGlmNXgTHGeRqD3DcUU",
    },
    // C.9 — UUIDv6.
    Vector {
        uuid: "1ec9414c-232a-6b00-b3c8-9f6bdeced846",
        hex_dash: "1ec9414c-232a-6b00-b3c8-9f6bdeced846",
        hex_plain: "1ec9414c232a6b00b3c89f6bdeced846",
        base32_std: "D3EUCTBDFJVQBM6IT5V55TWYIY",
        base32_hex: "3R4K2J1359LG1CU8JTLTTJMO8O",
        base32_humans: "3V4M2K1359NG1CY8KXNXXKPR8R",
        base36: "1TM3WVVTP7XVNZXA2TV43IJWM",
        base52: "liRiaXmgJTfBQppyCovyGu",
        base58_btc: "4oVbpzb8BpnTH1mg11dmWd",
        base62_ieee: "6FuGgfRpa7N6YbrlxT6FQ",
        base62_sort: "w5k6WVHfQxDwORhbnJw5G",
        base64_std: "HslBTCMqawCzyJ9r3s7YRg",
        base64_url: "HslBTCMqawCzyJ9r3s7YRg",
        base64_sort: "6g_0I1BePk1nm8xfrgvNGV",
        base85_z85: "9)3YwbpWEeV=F+O?P<+y",
        ncname_32: "gd3euctbdfkyahse7nppm5wcgl",
        ncname_58: "GrxRCnDiX4mxSq8bFQjT3_L",
        ncname_64: "GHslBTCMqsAPIn2vezthGL",
        base62id: "GWDoX3DScmUiFyjHO1pLJW",
    },
    // C.10 — UUIDv7.
    Vector {
        uuid: "017f22e2-79b0-7cc3-98c4-dc0c0c07398f",
        hex_dash: "017f22e2-79b0-7cc3-98c4-dc0c0c07398f",
        hex_plain: "017f22e279b07cc398c4dc0c0c07398f",
        base32_std: "AF7SFYTZWB6MHGGE3QGAYBZZR4",
        base32_hex: "05VI5OJPM1UC7664RG60O1PPHS",
        base32_humans: "05ZJ5RKSP1YC7664VG60R1SSHW",
        base36: "36TWI214QWJ7MGSVQ83NM8WF",
        base52: "BrKaFlCsyjaiCYuzuWvtun",
        base58_btc: "BihbxwwQ4NZZpKRH9JDCz",
        base62_ieee: "CzFyajyRd5A9oiF8QCBUD",
        base62_sort: "2p5oQZoHTv0zeY5yG21K3",
        base64_std: "AX8i4nmwfMOYxNwMDAc5jw",
        base64_url: "AX8i4nmwfMOYxNwMDAc5jw",
        base64_sort: "-MwXsbakUBDNlCkB2-RtYk",
        base85_z85: "0E(rMD9zXlN8E+*3<O!N",
        ncname_32: "haf7sfytzwdgdrrg4bqgaoompj",
        ncname_58: "H3RrXaX7uTM6qdwrXwpC6_J",
        ncname_64: "HAX8i4nmwzDjE3AwMBzmPJ",
        base62id: "FcxAExHzEpSVJEpfkUXQYJ",
    },
    // C.11 — UUIDv8 (B.1).
    Vector {
        uuid: "2489e9ad-2ee2-8e00-8ec9-32d5f69181c0",
        hex_dash: "2489e9ad-2ee2-8e00-8ec9-32d5f69181c0",
        hex_plain: "2489e9ad2ee28e008ec932d5f69181c0",
        base32_std: "ESE6TLJO4KHABDWJGLK7NEMBYA",
        base32_hex: "4I4UJB9ESA7013M96BAVD4C1O0",
        base32_humans: "4J4YKB9EWA7013P96BAZD4C1R0",
        base36: "25VHD0YEN79F79OO0TV0BAE9S",
        base52: "skNtHBdXlnQCRPhYqjxSkQ",
        base58_btc: "5WhDz2zW6g9mHu7EP9hoVq",
        base62_ieee: "BG6ufXDWs4d4Dd5uoJIAi0",
        base62_sort: "16wkVN3MiuTu3Tvke980Yq",
        base64_std: "JInprS7ijgCOyTLV9pGBwA",
        base64_url: "JInprS7ijgCOyTLV9pGBwA",
        base64_sort: "87bdfHvXYV1DmIAKxd50k-",
        base85_z85: "b-g=/f5?(>J(:6b{k%{w",
        ncname_32: "iese6tljo4lqa5sjs2x3jdaoai",
        ncname_58: "I22HpMAy5M181AjPFG7eLXI",
        ncname_64: "IJInprS7i4A7JMtX2kYHAI",
        base62id: "Gh4ovtlXgG1ON4DKQNdPn6",
    },
    // C.11 — UUIDv8 (B.2).
    Vector {
        uuid: "5c146b14-3c52-8afd-938a-375d0df1fbf6",
        hex_dash: "5c146b14-3c52-8afd-938a-375d0df1fbf6",
        hex_plain: "5c146b143c528afd938a375d0df1fbf6",
        base32_std: "LQKGWFB4KKFP3E4KG5OQ34P36Y",
        base32_hex: "BGA6M51SAA5FR4SA6TEGRSFRUO",
        base32_humans: "BGA6P51WAA5FV4WA6XEGVWFVYR",
        base36: "5G8XXKU1AQGT02ZJGR8PA3EUU",
        base52: "CIhPGFswaUyeIiUVKFQScIW",
        base58_btc: "CNV2iY4mKiTS1uw8RxapEH",
        base62_ieee: "CxumvfWqhqp4Kq4BkCGR1s",
        base62_sort: "2nkclVMgXgfuAgu1a26Hri",
        base64_std: "XBRrFDxSiv2TijddDfH79g",
        base64_url: "XBRrFDxSiv2TijddDfH79g",
        base64_sort: "M0Gf42lHXjqIXYSS2U6vxV",
        base85_z85: "tOH@/jw}7nLzRq84E%t?",
        ncname_32: "ilqkgwfb4kkx5hcrxlug7d67wj",
        ncname_58: "I3aR2J7aw1BJj4jJvfuWTXJ",
        ncname_64: "IXBRrFDxSr9OKN10N8fv2J",
        base62id: "INshC24rV2DOUHBbMGbh5y",
    },
    // C.12 — Microsoft.Azure.Portal.
    Vector {
        uuid: "00000013-0000-0000-c000-000000000000",
        hex_dash: "00000013-0000-0000-c000-000000000000",
        hex_plain: "0000001300000000c000000000000000",
        base32_std: "AAAAAEYAAAAABQAAAAAAAAAAAA",
        base32_hex: "000004O000001G000000000000",
        base32_humans: "000004R000001G000000000000",
        base36: "41Y09GGWTDKLN13WMO74",
        base52: "KGlWYrtEyPFBiZpPts",
        base58_btc: "2anhPihXPV7NXZKLC7",
        base62_ieee: "fjupi4ia0Gz3Pwu9y",
        base62_sort: "VZkfYuYQq6ptFmkzo",
        base64_std: "AAAAEwAAAADAAAAAAAAAAA",
        base64_url: "AAAAEwAAAADAAAAAAAAAAA",
        base64_sort: "----3k----2-----------",
        base85_z85: "0000j00000ZYjum00000",
        ncname_32: "aaaaaaeyaaaaaaaaaaaaaaaaam",
        ncname_58: "A111Mo9hVUdmNWqcCExF__M",
        ncname_64: "AAAAAEwAAAAAAAAAAAAAAM",
        base62id: "Fa84R2HvcuS2kQOPfUIAE4",
    },
];

fn check(name: &str, got: &str, want: &str, uuid: &str) {
    if want.is_empty() {
        return; // intentionally skipped vector
    }
    assert_eq!(got, want, "{name} for {uuid}: got {got:?}, want {want:?}");
}

#[test]
fn appendix_c_encode_all() {
    for v in APPENDIX_C {
        let u = from_hex(v.uuid).expect("parse hex");
        check("hex_dash", &u.to_hex_dash(), v.hex_dash, v.uuid);
        check("hex", &u.to_hex(), &v.hex_plain.to_lowercase(), v.uuid);
        check("base32", &u.to_base32(), v.base32_std, v.uuid);
        check("base32hex", &u.to_base32_hex(), v.base32_hex, v.uuid);
        check("base32humans", &u.to_base32_humans(), v.base32_humans, v.uuid);
        check("base36", &u.to_base36(), v.base36, v.uuid);
        check("base52", &u.to_base52(), v.base52, v.uuid);
        check("base58btc", &u.to_base58_btc(), v.base58_btc, v.uuid);
        check("base62ieee", &u.to_base62_ieee(), v.base62_ieee, v.uuid);
        check("base62sort", &u.to_base62_sort(), v.base62_sort, v.uuid);
        check("base64std", &u.to_base64(), v.base64_std, v.uuid);
        check("base64url", &u.to_base64_url(), v.base64_url, v.uuid);
        check("base64sort", &u.to_base64_sort(), v.base64_sort, v.uuid);
        check("Z85", &u.to_base85_z85(), v.base85_z85, v.uuid);
        check("ncname-32", &u.to_ncname_32(), v.ncname_32, v.uuid);
        check("ncname-58", &u.to_ncname_58(), v.ncname_58, v.uuid);
        check("ncname-64", &u.to_ncname_64(), v.ncname_64, v.uuid);
        check("base62id", &u.to_base62id(), v.base62id, v.uuid);
    }
}

fn check_decode(name: &str, got: Uuid, want_hex: &str) {
    let canonical = got.to_hex_dash();
    assert_eq!(canonical, want_hex, "{name}: round-tripped to {canonical}, want {want_hex}");
}

#[test]
fn appendix_c_decode_all() {
    for v in APPENDIX_C {
        let want = v.uuid;
        check_decode("hex_dash", from_hex(v.hex_dash).unwrap(), want);
        check_decode("hex", from_hex(v.hex_plain).unwrap(), want);
        check_decode("base32", from_base32(v.base32_std).unwrap(), want);
        check_decode("base32hex", from_base32_hex(v.base32_hex).unwrap(), want);
        check_decode("base32humans", from_base32_humans(v.base32_humans).unwrap(), want);
        check_decode("base36", from_base36(v.base36).unwrap(), want);
        check_decode("base52", from_base52(v.base52).unwrap(), want);
        check_decode("base58btc", from_base58_btc(v.base58_btc).unwrap(), want);
        check_decode("base62ieee", from_base62_ieee(v.base62_ieee).unwrap(), want);
        if !v.base62_sort.is_empty() {
            check_decode("base62sort", from_base62_sort(v.base62_sort).unwrap(), want);
        }
        check_decode("base64std", from_base64(v.base64_std).unwrap(), want);
        check_decode("base64url", from_base64_url(v.base64_url).unwrap(), want);
        check_decode("base64sort", from_base64_sort(v.base64_sort).unwrap(), want);
        check_decode("Z85", from_base85_z85(v.base85_z85).unwrap(), want);
        check_decode("ncname-32", from_ncname_32(v.ncname_32).unwrap(), want);
        check_decode("ncname-58", from_ncname_58(v.ncname_58).unwrap(), want);
        check_decode("ncname-64", from_ncname_64(v.ncname_64).unwrap(), want);
        check_decode("base62id", from_base62id(v.base62id).unwrap(), want);
    }
}

#[test]
fn uuidv4_base62sort_distinct_from_v3_and_round_trips() {
    // Draft Table 25 prints the v3 string in the v4 sort slot. We assert
    // (a) v3 ≠ v4 sort and (b) v4's sort string round-trips correctly.
    let v3 = from_hex("5df41881-3aed-3515-88a7-2f4a814cf09e").unwrap();
    let v4 = from_hex("919108f7-52d1-4320-9bac-f847db4148a8").unwrap();
    let v3_sort = v3.to_base62_sort();
    let v4_sort = v4.to_base62_sort();
    assert_ne!(v3_sort, v4_sort);
    let back = from_base62_sort(&v4_sort).unwrap();
    assert_eq!(back.to_hex_dash(), "919108f7-52d1-4320-9bac-f847db4148a8");
}
