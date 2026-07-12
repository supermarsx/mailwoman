//! Panic-freedom smoke test — the in-process analogue of the `cargo-fuzz`
//! target (`fuzz/fuzz_targets/parse.rs`). `parse` must never panic on any input;
//! it may only return `Ok`/`Err`. Runs the corpus plus adversarial byte streams.

use mw_mime::parse;

macro_rules! fixture {
    ($name:literal) => {
        include_bytes!(concat!("../../../fixtures/mime/", $name)).as_slice()
    };
}

const CORPUS: &[&[u8]] = &[
    fixture!("simple.eml"),
    fixture!("alternative.eml"),
    fixture!("nested.eml"),
    fixture!("attachment.eml"),
    fixture!("inline_cid.eml"),
    fixture!("qp.eml"),
    fixture!("iso8859-1.eml"),
    fixture!("shift_jis.eml"),
    fixture!("malformed.eml"),
    fixture!("no_headers.eml"),
];

#[test]
fn corpus_never_panics() {
    for raw in CORPUS {
        let _ = parse(raw);
    }
}

#[test]
fn truncations_of_corpus_never_panic() {
    // Every prefix length of every fixture — exercises mid-token cut-offs.
    for raw in CORPUS {
        for len in 0..raw.len() {
            let _ = parse(&raw[..len]);
        }
    }
}

#[test]
fn adversarial_inputs_never_panic() {
    let cases: Vec<Vec<u8>> = vec![
        vec![],
        vec![b'\r', b'\n'],
        vec![0u8; 4096],
        b"Content-Type: multipart/mixed; boundary=x\r\n\r\n--x\r\n".to_vec(),
        // Unterminated / mismatched boundaries.
        b"Content-Type: multipart/mixed; boundary=b\r\n\r\n--b\r\nContent-Type: text/plain\r\n\r\nhi".to_vec(),
        // A header line with no body and no blank line.
        b"Subject: no body".to_vec(),
        // Encoded-word with a bogus charset and truncated base64.
        b"Subject: =?bogus?B?zzz\r\n\r\nx".to_vec(),
        // Content-Transfer-Encoding base64 with invalid payload.
        b"Content-Transfer-Encoding: base64\r\n\r\n!!!!not base64!!!!".to_vec(),
    ];
    for c in cases {
        let _ = parse(&c);
    }
}

#[test]
fn deeply_nested_multipart_is_bounded() {
    // Many nested multipart wrappers — must terminate without stack overflow.
    let depth = 500usize;
    let mut raw = Vec::new();
    for i in 0..depth {
        raw.extend_from_slice(
            format!("Content-Type: multipart/mixed; boundary=b{i}\r\n\r\n--b{i}\r\n").as_bytes(),
        );
    }
    raw.extend_from_slice(b"Content-Type: text/plain\r\n\r\ndeep\r\n");
    let _ = parse(&raw);
}

#[test]
fn pseudo_random_bytes_never_panic() {
    // Deterministic LCG — reproducible fuzz-lite without a dependency.
    let mut state: u64 = 0x1234_5678_9abc_def0;
    let mut next = || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (state >> 33) as u8
    };
    for _ in 0..256 {
        let len = 1 + (next() as usize) * 4;
        let buf: Vec<u8> = (0..len).map(|_| next()).collect();
        let _ = parse(&buf);
    }
}
