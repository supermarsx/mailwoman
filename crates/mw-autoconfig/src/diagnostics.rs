//! Read-only DNS diagnostics for an already-authorized operator request.
use hickory_resolver::proto::rr::RData;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Records<T> {
    pub records: Vec<T>,
    pub error: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct Mx {
    pub preference: u16,
    pub host: String,
}
#[derive(Debug, Serialize)]
pub struct Srv {
    pub priority: u16,
    pub weight: u16,
    pub port: u16,
    pub host: String,
}
#[derive(Debug, Serialize)]
pub struct Txt {
    pub present: bool,
    pub records: Vec<String>,
    pub error: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct Service {
    pub service: String,
    pub records: Vec<Srv>,
    pub error: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct DnsReport {
    pub domain: String,
    pub mx: Records<Mx>,
    pub spf: Txt,
    pub dmarc: Txt,
    pub srv: Vec<Service>,
}
fn error() -> Option<String> {
    Some("DNS lookup failed or timed out".into())
}
fn text(record: &hickory_resolver::proto::rr::rdata::TXT) -> String {
    String::from_utf8_lossy(
        &record
            .txt_data
            .iter()
            .flat_map(|part| part.iter().copied())
            .take(4096)
            .collect::<Vec<_>>(),
    )
    .into_owned()
}
fn policy_txt(records: Vec<String>, prefix: &str, error: Option<String>) -> Txt {
    let records: Vec<_> = records
        .into_iter()
        .filter(|r| {
            r.to_ascii_lowercase()
                .strip_prefix(prefix)
                .is_some_and(|tail| {
                    tail.chars()
                        .next()
                        .is_none_or(|c| c.is_ascii_whitespace() || c == ';')
                })
        })
        .collect();
    Txt {
        present: !records.is_empty(),
        records,
        error,
    }
}
pub async fn diagnose(domain: &str) -> Result<DnsReport, String> {
    let resolver = hickory_resolver::TokioResolver::builder_tokio()
        .map_err(|_| "DNS resolver unavailable")?
        .build()
        .map_err(|_| "DNS resolver unavailable")?;
    let mx = match resolver.mx_lookup(format!("{domain}.")).await {
        Ok(lookup) => Records {
            records: lookup
                .answers()
                .iter()
                .filter_map(|record| match &record.data {
                    RData::MX(mx) => Some(Mx {
                        preference: mx.preference,
                        host: mx.exchange.to_utf8(),
                    }),
                    _ => None,
                })
                .take(100)
                .collect(),
            error: None,
        },
        Err(e) if e.is_no_records_found() => Records {
            records: vec![],
            error: None,
        },
        Err(_) => Records {
            records: vec![],
            error: error(),
        },
    };
    let mut policies = Vec::new();
    for (host, prefix) in [
        (format!("{domain}."), "v=spf1"),
        (format!("_dmarc.{domain}."), "v=dmarc1"),
    ] {
        let (records, err) = match resolver.txt_lookup(host).await {
            Ok(lookup) => (
                lookup
                    .answers()
                    .iter()
                    .filter_map(|record| match &record.data {
                        RData::TXT(txt) => Some(text(txt)),
                        _ => None,
                    })
                    .take(100)
                    .collect(),
                None,
            ),
            Err(e) if e.is_no_records_found() => (vec![], None),
            Err(_) => (vec![], error()),
        };
        policies.push(policy_txt(records, prefix, err));
    }
    let mut srv = Vec::new();
    for service in ["_jmap._tcp", "_imaps._tcp", "_submission._tcp"] {
        let (records, err) = match resolver.srv_lookup(format!("{service}.{domain}.")).await {
            Ok(lookup) => (
                lookup
                    .answers()
                    .iter()
                    .filter_map(|record| match &record.data {
                        RData::SRV(s) => Some(Srv {
                            priority: s.priority,
                            weight: s.weight,
                            port: s.port,
                            host: s.target.to_utf8(),
                        }),
                        _ => None,
                    })
                    .take(100)
                    .collect(),
                None,
            ),
            Err(e) if e.is_no_records_found() => (vec![], None),
            Err(_) => (vec![], error()),
        };
        srv.push(Service {
            service: service.into(),
            records,
            error: err,
        });
    }
    let dmarc = policies.pop().unwrap();
    let spf = policies.pop().unwrap();
    Ok(DnsReport {
        domain: domain.into(),
        mx,
        spf,
        dmarc,
        srv,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unrelated_txt_is_not_misreported_as_a_policy_and_errors_are_distinct() {
        let spf = policy_txt(
            vec!["site-verification=abc".into(), "v=spf1 -all".into()],
            "v=spf1",
            None,
        );
        assert!(spf.present);
        assert_eq!(spf.records, ["v=spf1 -all"]);
        let missing = policy_txt(vec!["site-verification=abc".into()], "v=dmarc1", None);
        assert!(!missing.present);
        assert!(missing.error.is_none());
        assert!(policy_txt(vec![], "v=spf1", error()).error.is_some());
    }
}
