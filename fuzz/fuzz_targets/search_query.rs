#![no_main]
use libfuzzer_sys::fuzz_target;

// Search-operator queries are typed by users and arrive as `filter.text` on a
// JMAP `Email/query` — unbounded, unvalidated, straight off the wire
// (`mw-engine/src/search_index.rs` hands it to `parse_query` with no length or
// shape check). The parser is contracted to answer with `SearchError::Parse` on
// malformed input, never a panic.
//
// Unlike the parsers behind the other targets, this one is also *recursive*
// (`atom := '(' or ')'`), so nesting depth is part of what is being fuzzed and
// the input is deliberately **not** truncated here.
//
// Writing this target is what surfaced the reason that matters. Measured in
// 26.20 (`t22-e13`), `parse_expr` had no depth guard and cost ~975 bytes of
// stack per nesting level — it died at 1 077 nested `(` on a 1 MiB stack, 2 187
// on 2 MiB (a tokio worker's default, i.e. what serves a JMAP request) and
// 8 852 on 8 MiB. A stack overflow is not a catchable panic; it takes the
// process down. libFuzzer's default `-max_len` is 4096, so this target reaches
// that depth. `mw_search::query::MAX_DEPTH` is what makes it safe to run, and
// the ordering was deliberate: the guard landed first, so this target goes
// green because the bug is fixed rather than red on a bug already known.
//
// Capping the input length here to keep the pass green would have been a fuzz
// target written so it cannot see the class of bug it exists to find, so the
// length stays uncapped — a future red here is a genuinely new finding.
//
// Non-UTF-8 bytes are lossily converted rather than skipped: `parse_query` takes
// `&str`, so gating on `from_utf8` (as `sieve_parse` does) would throw away most
// of libFuzzer's mutations. Lossy conversion keeps every input in play and still
// exercises the multi-byte boundaries the parser slices on.
fuzz_target!(|data: &[u8]| {
    let text = String::from_utf8_lossy(data);

    let first = mw_search::parse_query(&text);

    // Parsing is a pure function of the text: the same input must produce the
    // same answer. A parser that carries state between calls, or whose output
    // depends on interning/hashing order, fails here rather than silently
    // returning different result sets for one query.
    let second = mw_search::parse_query(&text);
    match (&first, &second) {
        (Ok(a), Ok(b)) => {
            assert_eq!(a, b, "parse_query is not deterministic for {text:?}");
            // `raw` is the source text the engine logs and round-trips; it must
            // be what was handed in, not a normalised rewrite of it.
            assert_eq!(a.raw, text, "SearchQuery::raw diverged from its input");
        }
        (Err(_), Err(_)) => {}
        _ => panic!("parse_query accepted and rejected the same input: {text:?}"),
    }
});
