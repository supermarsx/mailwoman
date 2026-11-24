//! Planner unit tests (t50hp-e31; design part 7 a and f): the route of every
//! EmailFilter field and Clause variant, the conjunction algebra, and the
//! source guard that keeps classification exhaustive.
use super::*;
use serde_json::json;

fn conjuncts(filter: serde_json::Value) -> Vec<Expr> {
    let f = EmailFilter::parse_saved_json(&filter.to_string()).expect("filter parses");
    filter_conjuncts(&f, None).expect("not empty")
}

fn route(filter: serde_json::Value) -> Route {
    route_sorted(filter, Sort::received_desc(), &[], false)
}

fn route_sorted(filter: serde_json::Value, sort: Sort, ties: &[Sort], semantic: bool) -> Route {
    let scope = match filter.get("inMailbox").and_then(serde_json::Value::as_str) {
        Some(mb) => Scope::Folder(mb.to_owned()),
        None => Scope::Account,
    };
    plan(&conjuncts(filter), &scope, &sort, ties, semantic).expect("plannable")
}

fn conj(f: impl FnOnce(&mut MetaConj)) -> MetaConj {
    let mut c = MetaConj::default();
    f(&mut c);
    c
}

/// a: the route of every EmailFilter field alone, and of the forms that
/// combine them. R1 = SQL only, R2 = index only (today's query), R3 = SQL id
/// sets in the index.
#[test]
fn every_filter_field_has_its_route() {
    let cases: Vec<(serde_json::Value, &str)> = vec![
        (json!({}), "R2"),
        (json!({"inMailbox": "mb"}), "R2"),
        (json!({"inMailboxes": ["mb"]}), "R2"),
        (json!({"inMailboxOtherThan": ["mb"]}), "R2"),
        (json!({"text": "alpha"}), "R2"),
        (json!({"from": "a"}), "R2"),
        (json!({"to": "a"}), "R2"),
        (json!({"cc": "a"}), "R2"),
        (json!({"subject": "a"}), "R2"),
        (json!({"body": "a"}), "R2"),
        (json!({"filename": "a"}), "R2"),
        (json!({"hasKeyword": "$seen"}), "R1"),
        (json!({"hasKeyword": "$flagged"}), "R1"),
        (json!({"hasKeyword": "work"}), "R1"),
        (json!({"hasKeyword": "$mw-followup-active"}), "R1"),
        (json!({"hasKeyword": "$list:x"}), "R3"),
        (json!({"hasKeyword": "$priority-high"}), "R3"),
        (json!({"notKeyword": "$seen"}), "R1"),
        (json!({"notKeyword": "work"}), "R1"),
        (json!({"notKeyword": "$supersedes:a@b"}), "R3"),
        (json!({"hasAttachment": true}), "R2"),
        (json!({"hasCalendar": true}), "R2"),
        (json!({"calendarEndAfter": "2026-01-01T00:00:00Z"}), "R2"),
        (json!({"before": "2026-01-01T00:00:00Z"}), "R2"),
        (json!({"after": "2026-01-01T00:00:00Z"}), "R2"),
        (json!({"minSize": 1}), "R2"),
        (json!({"maxSize": 1}), "R2"),
        (json!({"semantic": true}), "R2"),
        // Metadata with mailbox/date/size conditions only: R1.
        (json!({"inMailbox": "mb", "notKeyword": "$seen"}), "R1"),
        (
            json!({"inMailboxes": ["a", "b"], "hasKeyword": "$flagged"}),
            "R1",
        ),
        (
            json!({"inMailboxOtherThan": ["a"], "hasKeyword": "work"}),
            "R1",
        ),
        (
            json!({"notKeyword": "$seen", "after": "2026-01-01T00:00:00Z", "maxSize": 9}),
            "R1",
        ),
        (
            json!({"hasKeyword": "$seen", "notKeyword": "$seen"}),
            "nothing",
        ),
        (json!({"text": "pinned:true pinned:false alpha"}), "nothing"),
        // With content: R3.
        (json!({"notKeyword": "$seen", "text": "alpha"}), "R3"),
        (json!({"hasKeyword": "work", "hasAttachment": true}), "R3"),
        (json!({"hasKeyword": "work", "hasCalendar": true}), "R3"),
        (json!({"hasKeyword": "work", "from": "a"}), "R3"),
        // Text operators.
        (json!({"text": "is:unread"}), "R1"),
        (json!({"text": "tag:a -tag:b"}), "R1"),
        (json!({"text": "pinned:true"}), "R1"),
        (json!({"text": "pinned:false is:read"}), "R1"),
        (json!({"text": "is:followup"}), "R1"),
        (json!({"text": "-is:followup"}), "R3"),
        (json!({"text": "is:unread OR is:flagged"}), "R3"),
        (json!({"text": "is:unread OR from:x"}), "R3"),
        (json!({"text": "NOT in:mb"}), "R2"),
        (json!({"text": "in:mb is:unread"}), "R1"),
        (json!({"text": "list:x"}), "R3"),
        (json!({"text": "is:high-priority"}), "R3"),
        (json!({"text": "NOT (is:unread OR tag:work)"}), "R3"),
    ];
    for (filter, want) in cases {
        let got = route(filter.clone());
        assert_eq!(got.name(), want, "{filter}: {got:?}");
    }
}

/// a: R1 only for the sorts SQL orders, without tie-breakers or re-rank.
#[test]
fn sql_route_needs_an_sql_order() {
    let f = || json!({"notKeyword": "$seen"});
    for (sort, want) in [
        (Sort::received_desc(), Some(ProjectionSort::ReceivedDesc)),
        (
            Sort {
                ascending: true,
                ..Sort::received_desc()
            },
            Some(ProjectionSort::ReceivedAsc),
        ),
        (
            Sort::for_field(SortField::Size),
            Some(ProjectionSort::SizeAsc),
        ),
        (
            Sort {
                ascending: false,
                ..Sort::for_field(SortField::Size)
            },
            Some(ProjectionSort::SizeDesc),
        ),
    ] {
        assert_eq!(
            route_sorted(f(), sort, &[], false),
            Route::Sql(
                conj(|c| {
                    c.not_keywords = vec!["$seen".into()];
                    c.meta_leaves = 1;
                }),
                want.unwrap()
            )
        );
    }
    for field in [
        SortField::SentAt,
        SortField::Importance,
        SortField::GroupReceived,
        SortField::GroupSent,
        SortField::GroupFrom,
        SortField::GroupSubject,
        SortField::GroupSize,
        SortField::From,
        SortField::Subject,
    ] {
        assert_eq!(
            route_sorted(f(), Sort::for_field(field), &[], false).name(),
            "R3",
            "{field:?}"
        );
    }
    assert_eq!(
        route_sorted(
            f(),
            Sort::received_desc(),
            &[Sort::for_field(SortField::Subject)],
            false
        )
        .name(),
        "R3"
    );
    assert_eq!(
        route_sorted(f(), Sort::received_desc(), &[], true).name(),
        "R3"
    );
}

/// a: every Clause variant's class (exhaustive list; the source guard below
/// fails when mw-search gains a variant this list and `classify` lack).
#[test]
fn every_clause_variant_has_its_class() {
    use LeafClass::*;
    let tf = TextField::All;
    let cases: Vec<(Clause, LeafClass)> = vec![
        (
            Clause::Text {
                field: tf,
                value: "a".into(),
                phrase: false,
            },
            Content,
        ),
        (
            Clause::Fuzzy {
                field: tf,
                value: "a".into(),
                distance: 1,
            },
            Content,
        ),
        (
            Clause::Wildcard {
                field: tf,
                value: "a*".into(),
            },
            Content,
        ),
        (Clause::FromDomain("a.b".into()), Content),
        (Clause::Keyword("$seen".into()), Meta),
        (Clause::Keyword("label".into()), Meta),
        (Clause::Keyword("$list:x".into()), Derived),
        (Clause::Keyword("$supersedes:a@b".into()), Derived),
        (Clause::Keyword("$mw-mention:a@b".into()), Derived),
        (Clause::Keyword(mw_search::KW_PRIORITY_HIGH.into()), Derived),
        (Clause::Keyword(mw_search::KW_PRIORITY_LOW.into()), Derived),
        (Clause::Keyword("$priority-other".into()), Meta),
        (Clause::NotKeyword("$flagged".into()), Meta),
        (Clause::NotKeyword("$list:x".into()), Derived),
        (Clause::Mailbox("mb".into()), Either),
        (Clause::HasAttachment(true), Content),
        (Clause::Pinned(true), Meta),
        (Clause::FollowUp, Meta),
        (Clause::Mention("me".into()), Content),
        (Clause::HasCalendar(true), Content),
        (Clause::Bulk(true), Content),
        (Clause::CalendarEndAfter(1), Content),
        (
            Clause::DateRange {
                after: Some(1),
                before: None,
            },
            Either,
        ),
        (
            Clause::SizeRange {
                larger: Some(1),
                smaller: None,
            },
            Either,
        ),
        (Clause::IdSet(IdSet::new(vec![])), Resolved),
        (Clause::Account("a".into()), Resolved),
    ];
    for (c, want) in cases {
        assert_eq!(classify(&c), want, "{c:?}");
    }
}

/// The conjunction algebra: scopes intersect, NOT folds, contradictions are
/// empty, and nothing is silently dropped.
#[test]
fn conjunctions_fold_and_refuse_what_they_cannot_express() {
    let scope = MetaConj::scoped(&Scope::Folder("inbox".into()));
    let e = |t: &str| mw_search::parse_query(t).unwrap().expr;
    // in:other inside a folder scope: nothing.
    assert!(scope.try_absorb(&e("in:other is:unread")).unwrap().nothing);
    // NOT in: of the scope folder: nothing; of another: no effect.
    assert!(scope.try_absorb(&e("-in:inbox is:unread")).unwrap().nothing);
    let c = scope.try_absorb(&e("-in:other is:unread")).unwrap();
    assert_eq!(c.in_mailboxes, Some(vec!["inbox".to_owned()]));
    assert!(!c.nothing);
    // Account scope: NOT in: is kept.
    let c = MetaConj::default()
        .try_absorb(&e("-in:a -in:b tag:x"))
        .unwrap();
    assert_eq!(
        (c.not_in.clone(), c.keywords.clone()),
        (vec!["a".into(), "b".into()], vec!["x".into()])
    );
    // Double negation, NOT of a NotKeyword, pins.
    let c = MetaConj::default()
        .try_absorb(&e("NOT is:unread NOT pinned:false"))
        .unwrap();
    assert_eq!(
        (c.keywords.clone(), c.pinned),
        (vec!["$seen".into()], Some(true))
    );
    assert!(
        MetaConj::default()
            .try_absorb(&e("pinned:true pinned:false"))
            .unwrap()
            .nothing
    );
    // Follow-up marker in both directions.
    let c = MetaConj::default()
        .try_absorb(&e("tag:$mw-followup-active"))
        .unwrap();
    assert_eq!((c.follow_up_active, c.keywords.len()), (Some(true), 0));
    assert!(
        MetaConj::default()
            .try_absorb(&e("tag:$mw-followup-active -tag:$mw-followup-active"))
            .unwrap()
            .nothing
    );
    // Not expressible: OR of metadata, NOT FollowUp, NOT of a range, content.
    for t in [
        "is:unread OR is:flagged",
        "-is:followup",
        "-after:2026-01-01",
        "-larger:5",
        "is:unread alpha",
        "list:x",
        "-(is:unread is:flagged)",
    ] {
        assert!(MetaConj::default().try_absorb(&e(t)).is_none(), "{t}");
    }
    // An empty date or size range matches nothing in the index; also in SQL.
    let c = MetaConj::default()
        .try_absorb(&Expr::Clause(Clause::DateRange {
            after: None,
            before: None,
        }))
        .unwrap();
    assert!(c.nothing);
    // Ranges tighten.
    let c = MetaConj::default()
        .try_absorb(&e(
            "after:100 after:50 before:900 before:1000 larger:5 larger:7 smaller:20",
        ))
        .unwrap();
    assert_eq!(
        (c.after, c.before, c.larger, c.smaller),
        (Some(100), Some(900), Some(7), Some(20))
    );
}

/// Nested metadata under OR / NOT becomes one SQL node per maximal
/// expressible subtree; a derived keyword is the union of both sides.
#[test]
fn nested_metadata_is_rewritten_into_sql_nodes() {
    let e = |t: &str| mw_search::parse_query(t).unwrap().expr;
    let kw = |k: &str, on: bool| {
        conj(|c| {
            if on {
                c.keywords = vec![k.into()]
            } else {
                c.not_keywords = vec![k.into()]
            }
            c.meta_leaves = 1;
        })
    };
    let Node::Or(items) = rewrite(&e("is:unread OR from:x")).unwrap() else {
        panic!()
    };
    assert_eq!(items[0], Node::Meta(kw("$seen", false)));
    assert!(matches!(items[1], Node::Index(_)));
    let Node::Not(inner) = rewrite(&e("NOT (is:unread OR tag:work)")).unwrap() else {
        panic!()
    };
    assert_eq!(
        *inner,
        Node::Or(vec![
            Node::Meta(kw("$seen", false)),
            Node::Meta(kw("work", true))
        ])
    );
    let Node::Not(inner) = rewrite(&e("-is:followup")).unwrap() else {
        panic!()
    };
    assert_eq!(
        *inner,
        Node::Meta(conj(|c| {
            c.follow_up = true;
            c.meta_leaves = 1;
        }))
    );
    // Derived: index OR sql; negated: NOT index AND NOT sql.
    let Node::Or(items) = rewrite(&e("list:x")).unwrap() else {
        panic!()
    };
    assert!(matches!(&items[0], Node::Index(Expr::Clause(Clause::Keyword(k))) if k == "$list:x"));
    assert_eq!(items[1], Node::Meta(kw("$list:x", true)));
    let Node::And(items) = rewrite(&Expr::Clause(Clause::NotKeyword("$list:x".into()))).unwrap()
    else {
        panic!()
    };
    assert!(matches!(
        &items[0],
        Node::Index(Expr::Clause(Clause::NotKeyword(_)))
    ));
    assert_eq!(
        items[1],
        Node::Not(Box::new(Node::Meta(kw("$list:x", true))))
    );
    // Combined plan: top-level metadata into `top`, content and nested into nodes.
    let Route::Combined { top, nodes } = route(
        json!({"text": "(is:unread OR from:x) alpha", "hasKeyword": "work", "after": "2026-01-01T00:00:00Z"}),
    ) else {
        panic!()
    };
    let top = top.expect("top");
    assert_eq!(top.keywords, vec!["work".to_owned()]);
    assert_eq!(
        top.after,
        Some(1_767_225_600),
        "the date narrows the SQL side too"
    );
    assert_eq!(nodes.len(), 3, "the text AND is flattened: {nodes:?}");
    // assemble replaces metadata nodes in walk order.
    let mut metas = Vec::new();
    for n in &nodes {
        meta_nodes(n, &mut metas);
    }
    assert_eq!(metas, vec![&kw("$seen", false)]);
    // In a one-folder query every nested SQL condition is narrowed to it.
    let Route::Combined { nodes, .. } =
        route(json!({"inMailbox": "inbox", "text": "is:unread OR from:x"}))
    else {
        panic!()
    };
    let mut narrowed = Vec::new();
    meta_nodes(&nodes[0], &mut narrowed);
    assert_eq!(
        narrowed[0].in_mailboxes,
        Some(vec!["inbox".to_owned()]),
        "{nodes:?}"
    );
    let resolved = vec![id_set(vec!["i1".into()])];
    let mut it = resolved.into_iter();
    let expr = assemble(&nodes[0], &mut it);
    assert!(format!("{expr:?}").contains("IdSet(1 ids)"), "{expr:?}");
}

#[test]
fn single_conditions_have_a_complement() {
    let kw = conj(|c| {
        c.keywords = vec!["a".into()];
        c.meta_leaves = 1;
    });
    assert_eq!(kw.complement().unwrap().not_keywords, vec!["a".to_owned()]);
    let pin = conj(|c| {
        c.pinned = Some(true);
        c.meta_leaves = 1;
    });
    assert_eq!(pin.complement().unwrap().pinned, Some(false));
    let act = conj(|c| {
        c.follow_up_active = Some(false);
        c.meta_leaves = 1;
    });
    assert_eq!(act.complement().unwrap().follow_up_active, Some(true));
    // Within the query's folder: the complement keeps the folder.
    let in_folder = conj(|c| {
        c.keywords = vec!["a".into()];
        c.in_mailboxes = Some(vec!["m".into()]);
        c.meta_leaves = 1;
    });
    let complement = in_folder.complement().unwrap();
    assert_eq!(
        (
            complement.not_keywords.clone(),
            complement.in_mailboxes.clone()
        ),
        (vec!["a".to_owned()], Some(vec!["m".to_owned()]))
    );
    for none in [
        conj(|c| {
            c.keywords = vec!["a".into(), "b".into()];
            c.meta_leaves = 2;
        }),
        conj(|c| {
            c.keywords = vec!["a".into()];
            c.not_in = vec!["m".into()];
            c.meta_leaves = 1;
        }),
        conj(|c| {
            c.follow_up = true;
            c.meta_leaves = 1;
        }),
        conj(|c| {
            c.keywords = vec!["a".into()];
            c.after = Some(1);
            c.meta_leaves = 1;
        }),
    ] {
        assert!(none.complement().is_none(), "{none:?}");
    }
}

fn source(rel: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .replace("\r\n", "\n")
}

/// The body of `fn name` in `src`: from its signature to the first line that
/// closes at column 0 (or 4 for a method).
fn body<'a>(src: &'a str, signature: &str) -> &'a str {
    let start = src
        .find(signature)
        .unwrap_or_else(|| panic!("{signature} not found"));
    let line_start = src[..start].rfind('\n').map_or(0, |i| i + 1);
    let indent = start - line_start + (signature.len() - signature.trim_start().len());
    let close = format!("\n{}}}\n", " ".repeat(indent));
    let end = src[start..].find(&close).expect("end of fn") + start;
    &src[start..end]
}

fn enum_variants(src: &str, header: &str) -> Vec<String> {
    let start = src.find(header).unwrap_or_else(|| panic!("{header}")) + header.len();
    let end = src[start..].find("\n}\n").unwrap() + start;
    src[start..end]
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with("//") && !l.is_empty())
        .filter_map(|l| {
            let name: String = l
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            (!name.is_empty() && name.chars().next().unwrap().is_uppercase()).then_some(name)
        })
        .collect()
}

fn struct_fields(src: &str, header: &str) -> Vec<String> {
    let start = src.find(header).unwrap_or_else(|| panic!("{header}")) + header.len();
    let end = src[start..].find("\n}\n").unwrap() + start;
    src[start..end]
        .lines()
        .map(str::trim)
        .filter_map(|l| l.strip_prefix("pub "))
        .filter_map(|l| l.split(':').next())
        .map(str::to_owned)
        .collect()
}

/// f: a Clause variant or an EmailFilter field added without a route does
/// not build. rustc enforces the exhaustive `match` and the `..`-free
/// destructuring; this guard keeps them that way (no wildcard arm, no `..`)
/// and checks that every variant and field is named where it is routed.
#[test]
fn classification_stays_exhaustive_without_wildcards() {
    let plan_src = source("src/query_plan.rs");
    let variants = enum_variants(&source("../mw-search/src/query.rs"), "pub enum Clause {\n");
    assert!(variants.len() >= 18, "{variants:?}");
    for f in ["pub(crate) fn classify(", "    fn absorb_clause("] {
        let b = body(&plan_src, f);
        assert!(
            !b.contains("_ =>") && !b.contains("_ |") && !b.contains("| _"),
            "{f} has a wildcard arm"
        );
        for v in &variants {
            assert!(
                b.contains(&format!("Clause::{v}")),
                "{f} does not route Clause::{v}"
            );
        }
    }
    let sorts = enum_variants(
        &source("../mw-search/src/query.rs"),
        "pub enum SortField {\n",
    );
    let b = body(&plan_src, "pub(crate) fn sql_sort(");
    assert!(!b.contains("_ =>"), "sql_sort has a wildcard arm");
    for v in &sorts {
        assert!(
            b.contains(&format!("SortField::{v}")),
            "sql_sort does not route SortField::{v}"
        );
    }
    let fields = struct_fields(&source("src/query.rs"), "pub struct EmailFilter {\n");
    assert!(fields.len() >= 20, "{fields:?}");
    let b = body(&plan_src, "pub(crate) fn filter_conjuncts(");
    let destructure = &b[b.find("let EmailFilter {").unwrap()..b.find("} = filter;").unwrap()];
    assert!(
        !destructure.contains(".."),
        "EmailFilter destructuring uses .."
    );
    for f in &fields {
        assert!(
            destructure.contains(&format!("{f},")) || destructure.contains(&format!("{f}:")),
            "filter_conjuncts does not read EmailFilter::{f}"
        );
    }
}

/// The derived keyword set equals the one `patch_current` carries over from
/// the stored document (index_publication.rs), so P2's union covers exactly
/// the names the index keeps as content.
#[test]
fn derived_keywords_match_the_index_publication_set() {
    let publication = source("src/index_publication.rs");
    let b = body(&publication, "fn patch_current(");
    let derived = &b[..b.find(".collect();").unwrap()];
    for needle in [
        "keyword.starts_with(\"$mw-mention:\")",
        "keyword.starts_with(\"$list:\")",
        "keyword.starts_with(\"$supersedes:\")",
        "keyword.as_str() == mw_search::KW_PRIORITY_HIGH",
        "keyword.as_str() == mw_search::KW_PRIORITY_LOW",
    ] {
        assert!(
            derived.contains(needle),
            "patch_current derived set changed: {needle}"
        );
    }
    assert_eq!(
        derived.matches("starts_with").count() + derived.matches(" == ").count(),
        5,
        "{derived}"
    );
    for k in [
        "$mw-mention:a@b",
        "$list:x",
        "$supersedes:a@b",
        mw_search::KW_PRIORITY_HIGH,
        mw_search::KW_PRIORITY_LOW,
    ] {
        assert!(is_derived_keyword(k), "{k}");
    }
    for k in [
        "$seen",
        "$priority-other",
        "list:x",
        "$mw-followup-active",
        "mw-followup-completed",
    ] {
        assert!(!is_derived_keyword(k), "{k}");
    }
}
