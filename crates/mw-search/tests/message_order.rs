use mw_search::{Index, IndexDoc, KW_PRIORITY_HIGH, KW_PRIORITY_LOW, SearchQuery, Sort, SortField};

fn fixture() -> Index {
    let index = Index::open_in_ram().unwrap();
    let docs: Vec<_> = (0..60)
        .map(|i| IndexDoc {
            stable_id: format!("m{i:02}"),
            account_id: "acct".into(),
            mailbox_id: "inbox".into(),
            subject: format!("Subject {i:02}"),
            from: format!("Sender {:02}", 59 - i),
            date: i,
            sent_at: Some(60 - i),
            size: i as u64 * 100,
            keywords: match i % 3 {
                0 => vec![KW_PRIORITY_HIGH.into()],
                1 => vec![KW_PRIORITY_LOW.into()],
                _ => vec![],
            },
            ..IndexDoc::default()
        })
        .collect();
    index.upsert_batch(&docs).unwrap();
    index
}

#[test]
fn sent_received_size_and_text_directions_sort_the_whole_result() {
    let index = fixture();
    for (field, ascending, first, last) in [
        (SortField::ReceivedAt, false, "m59", "m00"),
        (SortField::ReceivedAt, true, "m00", "m59"),
        (SortField::SentAt, false, "m00", "m59"),
        (SortField::SentAt, true, "m59", "m00"),
        (SortField::Size, false, "m59", "m00"),
        (SortField::Size, true, "m00", "m59"),
        (SortField::From, false, "m00", "m59"),
        (SortField::From, true, "m59", "m00"),
        (SortField::Subject, false, "m59", "m00"),
        (SortField::Subject, true, "m00", "m59"),
    ] {
        let ids = index
            .search(
                &SearchQuery::all().with_sort(Sort {
                    field,
                    ascending,
                    time_zone: None,
                }),
                0,
            )
            .unwrap();
        assert_eq!(ids.len(), 60);
        assert_eq!(ids.first().unwrap(), first);
        assert_eq!(ids.last().unwrap(), last);
    }
}

#[test]
fn importance_groups_honor_secondary_order_before_a_page_is_cut() {
    let index = fixture();
    for ascending in [false, true] {
        let query = SearchQuery::all().with_sort(Sort {
            field: SortField::Importance,
            ascending,
            time_zone: None,
        });
        let result = index
            .search_hits_ordered(
                &query,
                7,
                &[Sort {
                    field: SortField::Size,
                    ascending: true,
                    time_zone: None,
                }],
            )
            .unwrap();
        let remainder = if ascending { 1 } else { 0 };
        assert_eq!(
            result.ids,
            (0..7)
                .map(|i| format!("m{:02}", i * 3 + remainder))
                .collect::<Vec<_>>()
        );
        assert!(result.truncated);
    }
}

#[test]
fn missing_sent_time_falls_back_to_received_without_changing_normal_importance() {
    let index = Index::open_in_ram().unwrap();
    index
        .upsert_batch(&[
            IndexDoc {
                stable_id: "undated".into(),
                date: 20,
                ..IndexDoc::default()
            },
            IndexDoc {
                stable_id: "dated".into(),
                date: 30,
                sent_at: Some(10),
                ..IndexDoc::default()
            },
        ])
        .unwrap();
    assert_eq!(
        index
            .search(
                &SearchQuery::all().with_sort(Sort::for_field(SortField::SentAt)),
                0
            )
            .unwrap(),
        ["undated", "dated"]
    );
}

#[test]
fn grouping_uses_the_same_sender_subject_and_size_buckets_before_page_cutting() {
    for (field, ascending, docs, expected) in [
        (
            SortField::GroupFrom,
            true,
            vec![
                ("a", "Ann <z@example.org>", "Z", 30),
                ("b", "Bob <a@example.org>", "A", 20),
                ("c", "Zoe <z@example.org>", "B", 10),
            ],
            vec!["b", "c", "a"],
        ),
        (
            SortField::GroupSubject,
            true,
            vec![
                ("a", "a@example.org", "A", 30),
                ("b", "a@example.org", "B", 20),
                ("c", "a@example.org", " Re: Fwd: A ", 10),
            ],
            vec!["c", "a", "b"],
        ),
        (
            SortField::GroupSize,
            false,
            vec![
                ("a", "a@example.org", "A", 1000),
                ("b", "a@example.org", "Z", 25000),
                ("c", "a@example.org", "B", 30000),
            ],
            vec!["c", "a", "b"],
        ),
    ] {
        let index = Index::open_in_ram().unwrap();
        index
            .upsert_batch(
                &docs
                    .into_iter()
                    .map(|(id, from, subject, size)| IndexDoc {
                        stable_id: id.into(),
                        from: from.into(),
                        subject: subject.into(),
                        size,
                        ..IndexDoc::default()
                    })
                    .collect::<Vec<_>>(),
            )
            .unwrap();
        let query = SearchQuery::all().with_sort(Sort {
            field,
            ascending,
            time_zone: None,
        });
        let secondary = if field == SortField::GroupSize {
            SortField::Subject
        } else {
            SortField::Size
        };
        let then = [Sort {
            field: secondary,
            ascending: true,
            time_zone: None,
        }];
        assert_eq!(
            index.search_hits_ordered(&query, 0, &then).unwrap().ids,
            expected
        );
        let page = index.search_hits_ordered(&query, 2, &then).unwrap();
        assert_eq!(page.ids, expected[..2]);
        assert!(page.truncated);
    }
}

#[test]
fn calendar_day_buckets_honor_display_timezone_and_secondary_order_at_dst_boundaries() {
    let index = Index::open_in_ram().unwrap();
    let docs: Vec<_> = [
        ("a", "A", "2026-03-09T03:00:00Z"),
        ("b", "Z", "2026-03-09T01:00:00Z"),
        ("c", "B", "2026-03-09T05:00:00Z"),
    ]
    .into_iter()
    .map(|(id, subject, date)| {
        let stamp = chrono::DateTime::parse_from_rfc3339(date)
            .unwrap()
            .timestamp();
        IndexDoc {
            stable_id: id.into(),
            subject: subject.into(),
            date: stamp,
            sent_at: Some(stamp),
            ..IndexDoc::default()
        }
    })
    .collect();
    index.upsert_batch(&docs).unwrap();
    for field in [SortField::GroupReceived, SortField::GroupSent] {
        for (zone, expected) in [
            (chrono_tz::America::New_York, vec!["c", "a", "b"]),
            (chrono_tz::UTC, vec!["a", "c", "b"]),
        ] {
            let query = SearchQuery::all().with_sort(Sort {
                field,
                ascending: false,
                time_zone: Some(zone),
            });
            let result = index
                .search_hits_ordered(
                    &query,
                    0,
                    &[Sort {
                        field: SortField::Subject,
                        ascending: true,
                        time_zone: None,
                    }],
                )
                .unwrap();
            assert_eq!(result.ids, expected, "{field:?} {zone}");
        }
    }
}
