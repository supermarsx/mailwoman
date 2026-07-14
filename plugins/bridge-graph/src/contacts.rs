//! Contacts + GAL (directory) over Graph — backs the `addrbook-source` export and
//! the engine's recipient resolution. Personal contacts (`/me/contacts`), relevance-
//! ranked people (`/me/people`), and the organization directory (`/users`) are all
//! folded into a single de-duplicated `"Name <email>"` list.

use crate::graph::{GraphClient, Result, Transport};
use crate::model::{ContactsResponse, PeopleResponse, UsersResponse};

fn format_entry(name: Option<&str>, email: &str) -> String {
    match name {
        Some(n) if !n.is_empty() => format!("{n} <{email}>"),
        _ => email.to_string(),
    }
}

/// Percent-encode a query value for a Graph `$search` / `$filter` string. Minimal:
/// escapes the characters that would break the OData query, no external crate.
fn encode_query(q: &str) -> String {
    let mut out = String::with_capacity(q.len());
    for b in q.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// `GET /me/contacts` filtered client-side by `query` → personal contact addresses.
pub fn list_contacts<T: Transport>(
    client: &GraphClient<'_, T>,
    query: &str,
) -> Result<Vec<String>> {
    let resp: ContactsResponse = client.get_json("/me/contacts?$top=100")?;
    let q = query.to_ascii_lowercase();
    let mut out = Vec::new();
    for c in resp.value {
        let name = c.display_name.clone().unwrap_or_default();
        for e in c.email_addresses {
            if let Some(addr) = e.address {
                if q.is_empty()
                    || name.to_ascii_lowercase().contains(&q)
                    || addr.to_ascii_lowercase().contains(&q)
                {
                    out.push(format_entry(Some(&name), &addr));
                }
            }
        }
    }
    Ok(out)
}

/// `GET /me/people?$search` → relevance-ranked people (mixes contacts + GAL).
pub fn search_people<T: Transport>(
    client: &GraphClient<'_, T>,
    query: &str,
) -> Result<Vec<String>> {
    let path = format!("/me/people?$search=\"{}\"&$top=25", encode_query(query));
    let resp: PeopleResponse = client.get_json(&path)?;
    let mut out = Vec::new();
    for p in resp.value {
        for e in p.scored_email_addresses {
            if let Some(addr) = e.address {
                out.push(format_entry(p.display_name.as_deref(), &addr));
            }
        }
    }
    Ok(out)
}

/// `GET /users?$search` → the organization directory (the Global Address List).
pub fn search_gal<T: Transport>(client: &GraphClient<'_, T>, query: &str) -> Result<Vec<String>> {
    let path = format!(
        "/users?$search=\"displayName:{q}\" OR \"mail:{q}\"&$top=25",
        q = encode_query(query)
    );
    let resp: UsersResponse = client.get_json(&path)?;
    let mut out = Vec::new();
    for u in resp.value {
        let email = u.mail.or(u.user_principal_name);
        if let Some(addr) = email {
            out.push(format_entry(u.display_name.as_deref(), &addr));
        }
    }
    Ok(out)
}

/// The `addrbook-source::search` implementation: personal contacts + people + GAL,
/// de-duplicated, preserving first-seen order (contacts, then people, then GAL).
pub fn addrbook_search<T: Transport>(
    client: &GraphClient<'_, T>,
    query: &str,
) -> Result<Vec<String>> {
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for src in [
        list_contacts(client, query)?,
        search_people(client, query)?,
        search_gal(client, query)?,
    ] {
        for entry in src {
            let key = entry.to_ascii_lowercase();
            if seen.insert(key) {
                out.push(entry);
            }
        }
    }
    Ok(out)
}
