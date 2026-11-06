//! `Email::mailwoman_priority` from the sender's priority headers.

fn priority(headers: &str) -> Option<String> {
    let raw = format!("From: a@example.org\r\nSubject: s\r\n{headers}\r\nbody\r\n");
    mw_mime::parse(raw.as_bytes()).unwrap().email.mailwoman_priority
}

#[test]
fn headers_map_to_high_low_or_nothing() {
    assert_eq!(priority("X-Priority: 1 (Highest)\r\n").as_deref(), Some("high"));
    assert_eq!(priority("Importance: Low\r\n").as_deref(), Some("low"));
    assert_eq!(priority("X-MSMail-Priority: High\r\n").as_deref(), Some("high"));
    assert_eq!(priority("Priority: non-urgent\r\n").as_deref(), Some("low"));
    assert_eq!(priority("X-Priority: 3\r\n"), None);
    assert_eq!(priority("Precedence: bulk\r\n"), None);
    assert_eq!(priority(""), None);
}
