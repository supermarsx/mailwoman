//! ESEARCH (RFC 4731, RFC 9051 §7.3.4) — the `* ESEARCH` reply and the
//! `RETURN (...)` search options that ask for it.
//!
//! An IMAP4rev2 server answers every SEARCH with `* ESEARCH`, and an
//! ESEARCH-capable IMAP4rev1 server does so whenever the client sends
//! `RETURN`. `imap-proto` 0.16.7 cannot parse the reply (t32 D-2), so this
//! module parses it by hand. The parser also reads the PARTIAL (RFC 9394) and
//! MODSEQ (RFC 7162) return data and skips return data it does not know.

use crate::lex::{Cursor, LexResult, Parsed, bad};

/// Most numbers [`SequenceSet::expand`] will produce for a caller that does
/// not pass its own limit.
pub const DEFAULT_EXPAND_LIMIT: usize = 1_000_000;

/// A parsed `sequence-set`: inclusive ranges with `lo <= hi`, in server order.
/// `*` is stored as `u32::MAX`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SequenceSet(pub Vec<(u32, u32)>);

impl SequenceSet {
    /// How many numbers the set covers (overlaps counted twice).
    pub fn len(&self) -> u64 {
        self.0.iter().map(|&(lo, hi)| u64::from(hi - lo) + 1).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, n: u32) -> bool {
        self.0.iter().any(|&(lo, hi)| lo <= n && n <= hi)
    }

    /// Every number in the set, sorted and de-duplicated, or `None` when the
    /// set covers more than `limit` numbers (a hostile or `*`-bearing reply).
    pub fn expand(&self, limit: usize) -> Option<Vec<u32>> {
        if self.len() > limit as u64 {
            return None;
        }
        let mut out: Vec<u32> = self.0.iter().flat_map(|&(lo, hi)| lo..=hi).collect();
        out.sort_unstable();
        out.dedup();
        Some(out)
    }
}

/// PARTIAL (RFC 9394) return data: the requested window and the matching
/// numbers inside it (`None` for `NIL`, no matches in the window).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialResult {
    /// The window as sent back by the server; negative bounds count from the
    /// last match (`-1:-100`).
    pub range: (i64, i64),
    pub results: Option<SequenceSet>,
}

/// One `* ESEARCH` reply.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Esearch {
    /// The search correlator (`(TAG "A0005")`), when the server sent one.
    pub tag: Option<String>,
    /// True when the numbers are UIDs (the reply carries `UID`).
    pub uid: bool,
    pub all: Option<SequenceSet>,
    pub count: Option<u32>,
    pub min: Option<u32>,
    pub max: Option<u32>,
    /// Highest mod-sequence of the matches (RFC 7162 §3.1.5).
    pub modseq: Option<u64>,
    pub partial: Option<PartialResult>,
}

impl Esearch {
    /// The matching numbers from `ALL` (or from `PARTIAL` when that is all the
    /// server sent), sorted. `None` means the reply covers more than `limit`
    /// numbers.
    pub fn numbers(&self, limit: usize) -> Option<Vec<u32>> {
        match (&self.all, &self.partial) {
            (Some(all), _) => all.expand(limit),
            (
                None,
                Some(PartialResult {
                    results: Some(set), ..
                }),
            ) => set.expand(limit),
            _ => Some(Vec::new()),
        }
    }
}

/// A `RETURN` option for an extended SEARCH.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnOpt {
    Min,
    Max,
    All,
    Count,
    /// RFC 5182: keep the result in `$` for a later command.
    Save,
    /// RFC 9394: return one window of matches; negative bounds count from the
    /// last match. Neither bound may be zero and both share a sign.
    Partial(i32, i32),
}

impl ReturnOpt {
    fn write(self, out: &mut String) {
        match self {
            ReturnOpt::Min => out.push_str("MIN"),
            ReturnOpt::Max => out.push_str("MAX"),
            ReturnOpt::All => out.push_str("ALL"),
            ReturnOpt::Count => out.push_str("COUNT"),
            ReturnOpt::Save => out.push_str("SAVE"),
            ReturnOpt::Partial(a, b) => out.push_str(&format!("PARTIAL {a}:{b}")),
        }
    }

    fn valid(self) -> bool {
        match self {
            ReturnOpt::Partial(a, b) => a != 0 && b != 0 && (a > 0) == (b > 0),
            _ => true,
        }
    }
}

/// The return options the sync path asks for when ESEARCH is advertised.
pub const SYNC_RETURN: &[ReturnOpt] = &[
    ReturnOpt::All,
    ReturnOpt::Count,
    ReturnOpt::Min,
    ReturnOpt::Max,
];

/// Build an extended `SEARCH` command (without the tag):
/// `[UID ]SEARCH RETURN (<opts>) <criteria>`. An empty `opts` sends
/// `RETURN ()`, which RFC 4731 defines as `RETURN (ALL)`.
///
/// Returns `None` for an invalid PARTIAL window.
pub fn search_command(uid: bool, opts: &[ReturnOpt], criteria: &str) -> Option<String> {
    if !opts.iter().all(|o| o.valid()) {
        return None;
    }
    let mut out = String::with_capacity(32 + criteria.len());
    if uid {
        out.push_str("UID ");
    }
    out.push_str("SEARCH RETURN (");
    for (i, opt) in opts.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        opt.write(&mut out);
    }
    out.push_str(") ");
    out.push_str(criteria);
    Some(out)
}

/// Parse one `* ESEARCH ...` CRLF response from the start of `buf`.
pub fn parse(buf: &[u8]) -> Parsed<Esearch> {
    parse_inner(buf).into()
}

fn parse_inner(buf: &[u8]) -> LexResult<(usize, Esearch)> {
    let mut c = Cursor::new(buf);
    c.keyword("* ESEARCH")?;
    let mut out = Esearch::default();

    // search-correlator = SP "(" "TAG" SP tag-string ")", with the RFC 7377
    // MAILBOX / UIDVALIDITY pairs tolerated after the tag.
    if !c.at_crlf()? {
        c.sp()?;
        if c.peek()? == b'(' {
            c.byte(b'(')?;
            loop {
                let key = c.atom()?.to_ascii_uppercase();
                c.sp()?;
                if key == "TAG" {
                    out.tag = Some(String::from_utf8_lossy(&c.astring()?).into_owned());
                } else if c.peek()?.is_ascii_digit() {
                    c.number64()?;
                } else {
                    c.astring()?;
                }
                if c.eat(b')')? {
                    break;
                }
                c.sp()?;
            }
            if !c.at_crlf()? {
                c.sp()?;
            }
        }
    }

    let mut first = true;
    while !c.at_crlf()? {
        if !first {
            c.sp()?;
        }
        first = false;
        let name = c.atom()?.to_ascii_uppercase();
        if name == "UID" && !out.uid && out.all.is_none() && out.count.is_none() {
            out.uid = true;
            continue;
        }
        c.sp()?;
        match name.as_str() {
            "MIN" => out.min = Some(c.nz_number()?),
            "MAX" => out.max = Some(c.nz_number()?),
            "COUNT" => out.count = Some(c.number()?),
            "ALL" => out.all = Some(SequenceSet(c.sequence_set()?)),
            "MODSEQ" => out.modseq = Some(c.number64()?),
            "PARTIAL" => out.partial = Some(partial(&mut c)?),
            _ => c.skip_ext_value()?,
        }
    }
    c.crlf()?;
    Ok((c.pos(), out))
}

/// `"(" partial-range SP partial-results ")"`.
fn partial(c: &mut Cursor<'_>) -> LexResult<PartialResult> {
    c.byte(b'(')?;
    let a = signed(c)?;
    c.byte(b':')?;
    let b = signed(c)?;
    if (a > 0) != (b > 0) {
        return bad("PARTIAL bounds differ in sign");
    }
    c.sp()?;
    let results = if c.keyword("NIL").is_ok() {
        None
    } else {
        Some(SequenceSet(c.sequence_set()?))
    };
    c.byte(b')')?;
    Ok(PartialResult {
        range: (a, b),
        results,
    })
}

fn signed(c: &mut Cursor<'_>) -> LexResult<i64> {
    let neg = c.eat(b'-')?;
    let n = i64::from(c.nz_number()?);
    Ok(if neg { -n } else { n })
}

/// Fuzz entry point: parse arbitrary bytes as an ESEARCH reply and check the
/// invariants the read loop relies on. Must never panic.
pub fn fuzz(data: &[u8]) {
    if let Parsed::Done { consumed, value } = parse(data) {
        assert!(consumed <= data.len());
        assert!(data[..consumed].ends_with(b"\r\n"));
        let _ = value.numbers(10_000);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn done(buf: &[u8]) -> (usize, Esearch) {
        match parse(buf) {
            Parsed::Done { consumed, value } => (consumed, value),
            other => panic!(
                "expected Done for {:?}, got {other:?}",
                String::from_utf8_lossy(buf)
            ),
        }
    }

    #[test]
    fn rfc_9051_example_with_tag_uid_and_all() {
        let (n, e) = done(b"* ESEARCH (TAG \"A0005\") UID ALL 10:12\r\nA0005 OK done\r\n");
        assert_eq!(n, 39);
        assert_eq!(e.tag.as_deref(), Some("A0005"));
        assert!(e.uid);
        assert_eq!(e.numbers(100), Some(vec![10, 11, 12]));
    }

    #[test]
    fn min_max_count_and_all_in_any_order() {
        let (_, e) = done(b"* ESEARCH (TAG \"a\") MIN 2 COUNT 3 ALL 2,10:11 MAX 11\r\n");
        assert!(!e.uid);
        assert_eq!((e.min, e.max, e.count), (Some(2), Some(11), Some(3)));
        assert_eq!(e.numbers(100), Some(vec![2, 10, 11]));
    }

    #[test]
    fn empty_replies() {
        let (_, e) = done(b"* ESEARCH\r\n");
        assert_eq!(e, Esearch::default());
        let (_, e) = done(b"* ESEARCH (TAG \"x\") UID\r\n");
        assert!(e.uid);
        assert_eq!(e.numbers(10), Some(vec![]));
        let (_, e) = done(b"* ESEARCH (TAG \"x\") COUNT 0\r\n");
        assert_eq!(e.count, Some(0));
    }

    #[test]
    fn modseq_partial_and_unknown_return_data() {
        let (_, e) = done(
            b"* ESEARCH (TAG \"t\") UID MODSEQ 917162500 PARTIAL (-1:-100 200:250,252) \
              RELEVANCY (4 5 (x \")\")) X-FOO 7\r\n",
        );
        assert_eq!(e.modseq, Some(917_162_500));
        assert_eq!(
            e.partial,
            Some(PartialResult {
                range: (-1, -100),
                results: Some(SequenceSet(vec![(200, 250), (252, 252)])),
            })
        );
        assert_eq!(e.numbers(1000).unwrap().len(), 52);
        let (_, e) = done(b"* ESEARCH (TAG \"t\") UID PARTIAL (1:10 NIL)\r\n");
        assert_eq!(e.partial.unwrap().results, None);
    }

    #[test]
    fn multisearch_correlator_and_literal_tag() {
        let (_, e) = done(b"* ESEARCH (TAG \"A1\" MAILBOX \"Sent\" UIDVALIDITY 7) UID ALL 1\r\n");
        assert_eq!(e.tag.as_deref(), Some("A1"));
        let (_, e) = done(b"* ESEARCH (TAG {2}\r\nZ9) ALL 4\r\n");
        assert_eq!(e.tag.as_deref(), Some("Z9"));
    }

    #[test]
    fn keyword_case_does_not_matter() {
        let (_, e) = done(b"* esearch (tag \"a\") uid all 3:1\r\n");
        assert!(e.uid);
        assert_eq!(e.numbers(10), Some(vec![1, 2, 3]));
    }

    #[test]
    fn truncated_input_is_incomplete() {
        let full = b"* ESEARCH (TAG \"A0005\") UID ALL 10:12\r\n";
        for cut in 1..full.len() {
            assert_eq!(
                parse(&full[..cut]),
                Parsed::Incomplete,
                "cut at {cut}: {:?}",
                String::from_utf8_lossy(&full[..cut])
            );
        }
    }

    #[test]
    fn malformed_input_is_bad() {
        for buf in [
            &b"* SEARCH 1 2\r\n"[..],
            b"* ESEARCH ALL 0\r\n",
            b"* ESEARCH MIN 0\r\n",
            b"* ESEARCH COUNT x\r\n",
            b"* ESEARCH (TAG \"a\" UID ALL 1\r\n",
            b"* ESEARCH ALL 1:\r\n",
            b"* ESEARCH PARTIAL (1:-5 NIL)\r\n",
            b"* ESEARCH COUNT 99999999999\r\n",
            b"* ESEARCH X (unbalanced\r\n",
        ] {
            assert!(
                matches!(parse(buf), Parsed::Bad(_)),
                "{:?} should be Bad",
                String::from_utf8_lossy(buf)
            );
        }
    }

    #[test]
    fn huge_sets_are_not_expanded() {
        let (_, e) = done(b"* ESEARCH UID ALL 1:4294967295\r\n");
        assert_eq!(e.all.as_ref().unwrap().len(), u64::from(u32::MAX));
        assert_eq!(e.numbers(DEFAULT_EXPAND_LIMIT), None);
        let (_, e) = done(b"* ESEARCH UID ALL 5:*\r\n");
        assert_eq!(e.numbers(DEFAULT_EXPAND_LIMIT), None);
        assert!(e.all.unwrap().contains(7));
    }

    #[test]
    fn search_command_shapes() {
        assert_eq!(
            search_command(true, SYNC_RETURN, "1:*").unwrap(),
            "UID SEARCH RETURN (ALL COUNT MIN MAX) 1:*"
        );
        assert_eq!(
            search_command(false, &[], "UNSEEN").unwrap(),
            "SEARCH RETURN () UNSEEN"
        );
        assert_eq!(
            search_command(true, &[ReturnOpt::Partial(-1, -500)], "ALL").unwrap(),
            "UID SEARCH RETURN (PARTIAL -1:-500) ALL"
        );
        assert!(search_command(true, &[ReturnOpt::Partial(0, 5)], "ALL").is_none());
        assert!(search_command(true, &[ReturnOpt::Partial(1, -5)], "ALL").is_none());
    }

    #[test]
    fn fuzz_corpus_never_panics() {
        for buf in [
            &b""[..],
            b"*",
            b"* ESEARCH (",
            b"* ESEARCH (TAG {99999999}\r\n",
            b"* ESEARCH UID ALL *:*\r\n",
            b"* ESEARCH \xff\xfe\r\n",
            b"* ESEARCH PARTIAL (-2147483648:-1 1)\r\n",
        ] {
            fuzz(buf);
        }
    }
}
