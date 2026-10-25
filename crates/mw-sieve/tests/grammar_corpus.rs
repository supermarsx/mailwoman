//! The full Sieve grammar against real scripts: the RFC examples for every
//! extension the server may offer, the generator's own output for every rule
//! shape, and a seeded mutation pass over all of them (the in-tree stand-in for
//! the `sieve_grammar` fuzz target, which needs nightly and a sanitizer).

use mw_sieve::{
    Action, Condition, MatchOp, Rule, StringTest, check, fuzz_sieve_grammar, generate, managed,
};

/// Scripts taken from (or shaped after) the RFC examples. Each must parse and
/// validate.
const CORPUS: &[(&str, &str)] = &[
    (
        "RFC 5228 §9",
        "#\r\n# Example Sieve Filter\r\n# Declare any optional features or extension used by the script\r\n#\r\n\
         require [\"fileinto\"];\r\n\r\n\
         #\r\n# Handle messages from known mailing lists\r\n# Move messages from IETF filter discussion list to filter mailbox\r\n#\r\n\
         if header :is \"Sender\" \"owner-ietf-mta-filters@imc.org\"\r\n        {\r\n        fileinto \"filter\";  # move to \"filter\" mailbox\r\n        }\r\n\
         #\r\n# Keep all messages to or from people in my company\r\n#\r\n\
         elsif address :DOMAIN :is [\"From\", \"To\"] \"example.com\"\r\n        {\r\n        keep;               # keep in \"In\" mailbox\r\n        }\r\n\r\n\
         #\r\n# Try and catch unsolicited email.  If a message is not to me,\r\n# or it contains a subject known to be spam, file it away.\r\n#\r\n\
         elsif anyof (NOT address :all :contains\r\n               [\"To\", \"Cc\", \"Bcc\"] \"me@example.com\",\r\n             header :matches \"subject\"\r\n               [\"*make*money*fast*\", \"*university*dipl*mas*\"])\r\n        {\r\n        fileinto \"spam\";   # move to \"spam\" mailbox\r\n        }\r\n\
         else\r\n        {\r\n        # Move all other (non-company) mail to \"personal\"\r\n        # mailbox.\r\n        fileinto \"personal\";\r\n        }\r\n",
    ),
    (
        "RFC 5228 §2.4.2 multi-line + reject",
        "require \"reject\";\r\nif header :contains \"subject\" \"$$$\" {\r\n  reject text:\r\n  Your message was rejected.\r\n..and this line starts with a dot.\r\n.\r\n;\r\n}\r\n",
    ),
    (
        "RFC 5230 vacation",
        "require \"vacation\";\r\nif header :contains \"subject\" \"cyrus\" {\r\n  vacation :days 7 :subject \"Away\" :from \"me@example.com\"\r\n    :addresses [\"me@example.com\", \"alias@example.com\"] :mime :handle \"h1\"\r\n    \"I'm out -- send mail to cyrus-bugs\";\r\n} else {\r\n  vacation \"I'm out -- call me at +1 304 555 0123\";\r\n}\r\n",
    ),
    (
        "RFC 6131 vacation-seconds",
        "require [\"vacation\", \"vacation-seconds\"];\r\nvacation :seconds 1800 text:\r\nI am in a meeting.\r\n.\r\n;\r\n",
    ),
    (
        "RFC 5232 imap4flags",
        "require [\"imap4flags\", \"fileinto\"];\r\nif header :contains \"from\" \"boss@example.com\" {\r\n  setflag \"\\\\Flagged\";\r\n  addflag \"flagvar\" [\"\\\\Deleted\", \"\\\\Answered\"];\r\n  removeflag \"\\\\Deleted\";\r\n  fileinto :flags \"\\\\Seen\" \"INBOX.boss\";\r\n}\r\nif hasflag :contains \"MyVar\" \"Junk\" { discard; stop; }\r\n",
    ),
    (
        "RFC 5229 variables",
        "require [\"variables\", \"fileinto\"];\r\nif header :matches \"List-ID\" \"*<*@*\" {\r\n  set :lower :upperfirst \"list\" \"${2}\";\r\n  fileinto \"INBOX.lists.${list}\";\r\n  stop;\r\n}\r\nif string :is \"${list}\" \"\" { keep; }\r\n",
    ),
    (
        "RFC 5231 relational + ascii-numeric",
        "require [\"relational\", \"comparator-i;ascii-numeric\", \"fileinto\"];\r\nif header :value \"ge\" :comparator \"i;ascii-numeric\" \"X-Spam-Score\" \"5\" { fileinto \"Junk\"; }\r\nif address :count \"ge\" :comparator \"i;ascii-numeric\" [\"to\", \"cc\"] \"3\" { keep; }\r\n",
    ),
    (
        "RFC 5173 body + RFC 5228 envelope",
        "require [\"body\", \"envelope\", \"fileinto\"];\r\nif body :raw :contains \"MAKE MONEY FAST\" { discard; }\r\nif body :content \"text\" :contains \"project schedule\" { fileinto \"work\"; }\r\nif envelope :all :is \"from\" \"tim@example.com\" { discard; }\r\n",
    ),
    (
        "RFC 5260 date + index",
        "require [\"date\", \"relational\", \"index\", \"fileinto\"];\r\nif allof (currentdate :value \"ge\" \"hour\" \"09\", currentdate :value \"lt\" \"hour\" \"17\") { keep; }\r\nif date :index 1 :last :zone \"+0000\" :is \"received\" \"year\" \"2026\" { fileinto \"this-year\"; }\r\n",
    ),
    (
        "RFC 3894 copy",
        "require [\"copy\", \"fileinto\"];\r\nfileinto :copy \"archive\";\r\nredirect :copy \"other@example.com\";\r\n",
    ),
    (
        "RFC 6609 include",
        "require [\"include\", \"variables\"];\r\nglobal \"test\";\r\ninclude :personal \"spam_tests\";\r\ninclude :global :once \"always_allow\";\r\nif string :is \"${test}\" \"spam\" { discard; return; }\r\n",
    ),
    (
        "RFC 5490 mailbox",
        "require [\"mailbox\", \"fileinto\"];\r\nif mailboxexists \"Partners\" { fileinto :create \"Partners/2026\"; }\r\n",
    ),
    (
        "RFC 5293 editheader",
        "require [\"editheader\"];\r\naddheader :last \"X-Sieve-Filtered\" \"yes\";\r\ndeleteheader :index 1 :is \"X-Spam\" \"bogus\";\r\n",
    ),
    (
        "RFC 5235 spamtest/virustest",
        "require [\"spamtest\", \"virustest\", \"relational\", \"comparator-i;ascii-numeric\", \"fileinto\"];\r\nif spamtest :value \"ge\" :comparator \"i;ascii-numeric\" \"6\" { fileinto \"spam\"; }\r\nif virustest :value \"eq\" :comparator \"i;ascii-numeric\" \"5\" { discard; }\r\n",
    ),
    (
        "RFC 5233 subaddress",
        "require [\"subaddress\", \"envelope\", \"fileinto\"];\r\nif envelope :detail \"to\" \"mta-filters\" { fileinto \"lists.mta-filters\"; }\r\nif address :user :is \"to\" \"ken\" { keep; }\r\n",
    ),
    (
        "RFC 7352 duplicate",
        "require [\"duplicate\", \"fileinto\", \"mailbox\"];\r\nif duplicate :seconds 3600 :header \"message-id\" { fileinto :create \"Trash/Duplicate\"; }\r\n",
    ),
    (
        "RFC 5435 enotify",
        "require [\"enotify\", \"fileinto\", \"variables\"];\r\nif header :contains \"from\" \"boss@example.org\" {\r\n  notify :importance \"1\" :message \"This is probably very important\" \"mailto:alm@example.com\";\r\n  stop;\r\n}\r\nif valid_notify_method \"mailto:x@example.com\" { keep; }\r\n",
    ),
    (
        "RFC 5429 ereject",
        "require [\"ereject\"];\r\nif address :is \"from\" \"spam@example.org\" { ereject \"No thanks.\"; }\r\n",
    ),
    (
        "RFC 5463 ihave",
        "require \"ihave\";\r\nif ihave \"fileinto\" { keep; } else { error \"fileinto is missing\"; }\r\n",
    ),
    (
        "RFC 5228 §5 tests",
        "if allof (exists [\"From\", \"Date\"], not size :over 100K, true, not false) { keep; }\r\nif header :comparator \"i;octet\" :is \"X-Case\" \"Exact\" { stop; }\r\n",
    ),
];

#[test]
fn every_corpus_script_validates() {
    for (name, src) in CORPUS {
        if let Err(e) = check(src) {
            panic!("{name}: {e}\n{src}");
        }
    }
}

#[test]
fn the_inventory_matches_the_requires() {
    let (_, a) = check(CORPUS[4].1).unwrap();
    assert!(a.required.contains("imap4flags") && a.used.contains("imap4flags"));
    assert!(a.used.contains("fileinto"));
    let (_, a) = check(CORPUS[0].1).unwrap();
    assert_eq!(a.required.len(), 1);
    assert!(a.redirects.is_empty() && a.vacations.is_empty());
}

fn st(op: MatchOp, v: &str) -> StringTest {
    StringTest {
        op,
        value: v.into(),
    }
}

fn every_rule_shape() -> Vec<Rule> {
    let conditions = [
        Condition::From(st(MatchOp::Contains, "a@example.com")),
        Condition::To(st(MatchOp::Is, "b \"quoted\" @example.com")),
        Condition::Subject(st(MatchOp::Matches, "*\\back*")),
        Condition::Body(st(MatchOp::Contains, "invoice")),
        Condition::HasAttachment,
        Condition::Size {
            over: true,
            bytes: 1_048_576,
        },
        Condition::Size {
            over: false,
            bytes: 10,
        },
        Condition::Keyword(st(MatchOp::Is, "$label1")),
    ];
    let actions = [
        Action::Move {
            mailbox: "INBOX/News".into(),
        },
        Action::Copy {
            mailbox: "Archive".into(),
        },
        Action::Tag {
            keyword: "$work".into(),
        },
        Action::Mark {
            keyword: "\\Seen".into(),
        },
        Action::Forward {
            address: "me@example.com".into(),
        },
        Action::ReplyTemplate {
            template: "Away\nuntil Monday".into(),
        },
        Action::Notify {
            message: "new mail\r\nfrom boss".into(),
        },
        Action::Stop,
    ];
    let mut rules = Vec::new();
    let mut id = 0;
    for (ci, c) in conditions.iter().enumerate() {
        for (ai, a) in actions.iter().enumerate() {
            id += 1;
            rules.push(Rule {
                id: format!("r{id}"),
                name: format!("rule {ci}/{ai}"),
                match_all: (ci + ai) % 2 == 0,
                conditions: if ai % 3 == 0 {
                    vec![c.clone()]
                } else {
                    vec![c.clone(), conditions[(ci + 1) % conditions.len()].clone()]
                },
                actions: vec![a.clone(), actions[(ai + 3) % actions.len()].clone()],
                enabled: true,
            });
        }
    }
    rules.push(Rule {
        id: "always".into(),
        name: "no conditions".into(),
        match_all: true,
        conditions: vec![],
        actions: vec![Action::Stop],
        enabled: true,
    });
    rules
}

/// What the builder generates must be valid Sieve by the full grammar, alone
/// and inside managed blocks.
#[test]
fn generated_scripts_validate() {
    let rules = every_rule_shape();
    let script = generate(&rules).unwrap();
    if let Err(e) = check(&script) {
        panic!("generate(): {e}\n{script}");
    }
    for r in &rules {
        let one = generate(std::slice::from_ref(r)).unwrap();
        check(&one).unwrap_or_else(|e| panic!("{}: {e}\n{one}", r.id));
    }
    let merged = managed::merge(CORPUS[0].1, &rules).unwrap();
    check(&merged).unwrap_or_else(|e| panic!("merged: {e}"));
    assert_eq!(managed::rules(&merged).unwrap().len(), rules.len());
    assert_eq!(managed::hand_written(&merged).unwrap(), CORPUS[0].1);
}

/// A small xorshift so the mutation pass is the same on every run.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next() % n as u64) as usize
        }
    }
}

const SPICE: &[&[u8]] = &[
    b"{",
    b"}",
    b"(",
    b")",
    b"[",
    b"]",
    b";",
    b",",
    b"\"",
    b"\\",
    b":",
    b"text:\r\n",
    b"\r\n.\r\n",
    b"/*",
    b"*/",
    b"#",
    b"999999999999G",
    b"if",
    b"require",
    b"elsif",
    b"else",
    b"\xc3\xa9",
    b"\xe4\xb8\x80",
    b"\xff",
    b"# mailwoman:rule x begin\n",
    b"# mailwoman:rule x end\n",
    b"# mailwoman:require begin\n",
    b"${",
    b"redirect \"a@b\";",
    b"not ",
];

/// Seeded mutations of every corpus script and generated script: nothing may
/// panic or recurse without bound.
#[test]
fn seeded_mutations_never_panic() {
    let mut seeds: Vec<Vec<u8>> = CORPUS.iter().map(|(_, s)| s.as_bytes().to_vec()).collect();
    seeds.push(generate(&every_rule_shape()).unwrap().into_bytes());
    seeds.push(
        managed::merge("keep;\n", &every_rule_shape()[..5])
            .unwrap()
            .into_bytes(),
    );
    let mut rng = Rng(0x5eed_f180_0001);
    for round in 0..6000 {
        let mut data = seeds[round % seeds.len()].clone();
        for _ in 0..1 + rng.below(6) {
            let len = data.len();
            match rng.below(5) {
                0 if len > 0 => {
                    let i = rng.below(len);
                    data[i] ^= 1 << rng.below(8);
                }
                1 => {
                    let i = rng.below(len + 1);
                    let spice = SPICE[rng.below(SPICE.len())];
                    data.splice(i..i, spice.iter().copied());
                }
                2 if len > 0 => {
                    let i = rng.below(len);
                    let j = (i + 1 + rng.below(64)).min(len);
                    data.drain(i..j);
                }
                3 if len > 0 => {
                    let i = rng.below(len);
                    let j = (i + 1 + rng.below(64)).min(len);
                    let chunk: Vec<u8> = data[i..j].to_vec();
                    data.splice(j..j, chunk);
                }
                _ => data.truncate(rng.below(len + 1)),
            }
        }
        fuzz_sieve_grammar(&data);
        mw_sieve::managesieve::fuzz_response(&data);
    }
    // Pathological nesting in both directions.
    fuzz_sieve_grammar("if not ".repeat(100_000).as_bytes());
    fuzz_sieve_grammar("if true {".repeat(100_000).as_bytes());
    fuzz_sieve_grammar("[".repeat(100_000).as_bytes());
    fuzz_sieve_grammar(&vec![b'('; 100_000]);
}
