//! A tiny MaxMind DB writer for tests (t96). It builds small, synthetic `.mmdb`
//! files so no real location database is ever committed: an IPv6 search tree with
//! 32-bit records, IPv4 networks under `::/96`, and one data record per network.
//!
//! Not for production use: it does not merge overlapping networks and it writes
//! only the types the fixtures need (maps, UTF-8 strings, unsigned integers and an
//! array of strings).
#![doc(hidden)]

use std::net::{Ipv4Addr, Ipv6Addr};

/// One network and its country.
#[derive(Debug, Clone)]
pub struct Entry {
    bits: u128,
    prefix: u8,
    country: String,
}

impl Entry {
    /// An IPv4 network such as `89.114.0.0/16`.
    pub fn v4(cidr: &str, country: &str) -> Self {
        let (addr, len) = cidr.split_once('/').expect("cidr");
        let addr: Ipv4Addr = addr.parse().expect("ipv4");
        let len: u8 = len.parse().expect("prefix");
        assert!(len <= 32);
        Self {
            bits: u128::from(u32::from(addr)),
            prefix: 96 + len,
            country: country.into(),
        }
    }

    /// An IPv6 network such as `2a01:4f8::/32`.
    pub fn v6(cidr: &str, country: &str) -> Self {
        let (addr, len) = cidr.split_once('/').expect("cidr");
        let addr: Ipv6Addr = addr.parse().expect("ipv6");
        let len: u8 = len.parse().expect("prefix");
        assert!(len <= 128);
        Self {
            bits: u128::from(addr),
            prefix: len,
            country: country.into(),
        }
    }
}

// ── data section encoding ─────────────────────────────────────────────────────

fn control(out: &mut Vec<u8>, kind: u8, size: usize) {
    let (first, extra): (u8, Vec<u8>) = if size < 29 {
        (size as u8, vec![])
    } else if size < 29 + 256 {
        (29, vec![(size - 29) as u8])
    } else {
        let s = size - 285;
        (30, vec![(s >> 8) as u8, s as u8])
    };
    if kind <= 7 {
        out.push((kind << 5) | first);
    } else {
        out.push(first);
        out.push(kind - 7);
    }
    out.extend(extra);
}

fn string(out: &mut Vec<u8>, s: &str) {
    control(out, 2, s.len());
    out.extend_from_slice(s.as_bytes());
}

fn uint(out: &mut Vec<u8>, kind: u8, value: u64) {
    let bytes = value.to_be_bytes();
    let skip = bytes.iter().take_while(|b| **b == 0).count();
    control(out, kind, 8 - skip);
    out.extend_from_slice(&bytes[skip..]);
}

/// A value in a fixture record.
enum Value<'a> {
    Str(&'a str),
    U32(u64),
    Map(Vec<(&'a str, Value<'a>)>),
}

fn value(out: &mut Vec<u8>, v: &Value<'_>) {
    match v {
        Value::Str(s) => string(out, s),
        Value::U32(n) => uint(out, 6, *n),
        Value::Map(pairs) => {
            control(out, 7, pairs.len());
            for (k, v) in pairs {
                string(out, k);
                value(out, v);
            }
        }
    }
}

// ── the search tree ───────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum Rec {
    Empty,
    Node(usize),
    Data(usize),
}

fn build(records: &[(u128, u8, Vec<u8>)], database_type: &str, build_epoch: u64) -> Vec<u8> {
    let mut nodes: Vec<[Rec; 2]> = vec![[Rec::Empty, Rec::Empty]];
    let mut data = Vec::new();
    for (bits, prefix, record) in records {
        let offset = data.len();
        data.extend_from_slice(record);
        let mut node = 0usize;
        for i in 0..*prefix {
            let bit = ((bits >> (127 - i)) & 1) as usize;
            if i + 1 == *prefix {
                nodes[node][bit] = Rec::Data(offset);
                break;
            }
            node = match nodes[node][bit] {
                Rec::Node(next) => next,
                Rec::Empty => {
                    nodes.push([Rec::Empty, Rec::Empty]);
                    let next = nodes.len() - 1;
                    nodes[node][bit] = Rec::Node(next);
                    next
                }
                Rec::Data(_) => panic!("overlapping fixture networks"),
            };
        }
    }
    let node_count = nodes.len();
    let mut out = Vec::with_capacity(node_count * 8 + data.len() + 256);
    for pair in &nodes {
        for rec in pair {
            let v = match rec {
                Rec::Empty => node_count,
                Rec::Node(n) => *n,
                Rec::Data(off) => node_count + 16 + off,
            };
            out.extend_from_slice(&(v as u32).to_be_bytes());
        }
    }
    out.extend_from_slice(&[0u8; 16]);
    out.extend_from_slice(&data);
    out.extend_from_slice(b"\xAB\xCD\xEFMaxMind.com");
    let mut meta = Vec::new();
    control(&mut meta, 7, 9);
    string(&mut meta, "binary_format_major_version");
    uint(&mut meta, 5, 2);
    string(&mut meta, "binary_format_minor_version");
    uint(&mut meta, 5, 0);
    string(&mut meta, "build_epoch");
    uint(&mut meta, 9, build_epoch);
    string(&mut meta, "database_type");
    string(&mut meta, database_type);
    string(&mut meta, "description");
    control(&mut meta, 7, 1);
    string(&mut meta, "en");
    string(
        &mut meta,
        "Synthetic test database generated by Mailwoman's test suite",
    );
    string(&mut meta, "ip_version");
    uint(&mut meta, 5, 6);
    string(&mut meta, "languages");
    control(&mut meta, 11, 1);
    string(&mut meta, "en");
    string(&mut meta, "node_count");
    uint(&mut meta, 6, node_count as u64);
    string(&mut meta, "record_size");
    uint(&mut meta, 5, 32);
    out.extend_from_slice(&meta);
    out
}

/// A country database in the MaxMind/DB-IP schema (`country.iso_code`).
pub fn country_mmdb(entries: &[Entry], build_epoch: u64) -> Vec<u8> {
    let records: Vec<_> = entries
        .iter()
        .map(|e| {
            let mut r = Vec::new();
            value(
                &mut r,
                &Value::Map(vec![(
                    "country",
                    Value::Map(vec![("iso_code", Value::Str(&e.country))]),
                )]),
            );
            (e.bits, e.prefix, r)
        })
        .collect();
    build(&records, "Mailwoman-Test-Country", build_epoch)
}

/// A country database in the IPinfo Lite schema (flat `country_code`).
pub fn ipinfo_mmdb(entries: &[Entry], build_epoch: u64) -> Vec<u8> {
    let records: Vec<_> = entries
        .iter()
        .map(|e| {
            let mut r = Vec::new();
            value(
                &mut r,
                &Value::Map(vec![
                    ("country_code", Value::Str(&e.country)),
                    ("asn", Value::Str("AS0")),
                ]),
            );
            (e.bits, e.prefix, r)
        })
        .collect();
    build(&records, "ipinfo-lite-test", build_epoch)
}

/// An ASN-only database (no country codes), IPv4 networks.
pub fn asn_mmdb(entries: &[(&str, u64)], build_epoch: u64) -> Vec<u8> {
    let records: Vec<_> = entries
        .iter()
        .map(|(cidr, asn)| {
            let e = Entry::v4(cidr, "ZZ");
            let mut r = Vec::new();
            value(
                &mut r,
                &Value::Map(vec![("autonomous_system_number", Value::U32(*asn))]),
            );
            (e.bits, e.prefix, r)
        })
        .collect();
    build(&records, "Mailwoman-Test-ASN", build_epoch)
}
