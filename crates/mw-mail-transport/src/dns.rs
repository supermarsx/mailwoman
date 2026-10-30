use crate::{PolicyContext, PolicyError};
use futures_util::{
    StreamExt,
    stream::{self, BoxStream},
};
use hickory_net::proto::{
    dnssec::Proof,
    op::{DnsRequest, DnsRequestOptions, DnsResponse, MessageType, Query, ResponseCode},
    rr::{Name, RData, RecordType, rdata::TLSA},
};
use hickory_net::{
    NetError, dnssec::DnssecDnsHandle, runtime::TokioRuntimeProvider, xfer::DnsHandle,
};
use mw_egress::{
    ResponseMetadata, TrafficClass,
    plan::{Request, send},
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
#[cfg(test)]
#[path = "dns_tests.rs"]
mod tests;

/// The validator's complete DNS transport. No OS resolver/UDP fallback exists.
/// AD is deliberately ignored: DnssecDnsHandle verifies signatures and DS links.
#[derive(Clone)]
pub(crate) struct Doh {
    context: PolicyContext,
    remaining: Arc<AtomicUsize>,
}
impl Doh {
    pub fn new(context: PolicyContext) -> Self {
        Self {
            context,
            remaining: Arc::new(AtomicUsize::new(96)),
        }
    }
}
impl DnsHandle for Doh {
    type Response = BoxStream<'static, Result<DnsResponse, NetError>>;
    type Runtime = TokioRuntimeProvider;
    fn send(&self, request: DnsRequest) -> Self::Response {
        let this = self.clone();
        stream::once(async move {
            if this
                .remaining
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_sub(1))
                .is_err()
            {
                return Err(NetError::Message("DNS policy query budget exhausted"));
            }
            let wire = request.to_vec()?;
            let url = reqwest::Url::parse(&this.context.doh_url)
                .map_err(|_| NetError::Message("invalid DNS transport"))?;
            let metadata = ResponseMetadata::default();
            let mut fetch = Request::post(TrafficClass::DnsAuth, "application/dns-message", &wire)
                .accept("application/dns-message")
                .response_limit(65_535)
                .capture_response(&metadata)
                .timeouts(crate::service::timeouts());
            if let Some(domain) = &this.context.egress_domain {
                fetch = fetch.for_domain(domain);
            }
            let result = send(url, &fetch).await;
            if metadata.snapshot().map(|m| m.status) != Some(200)
                || !result
                    .content_type
                    .as_deref()
                    .and_then(|s| s.split(';').next())
                    .is_some_and(|s| s.trim().eq_ignore_ascii_case("application/dns-message"))
            {
                return Err(NetError::Message("DNS policy HTTP response refused"));
            }
            let response = DnsResponse::from_buffer(
                result
                    .outcome
                    .map_err(|_| NetError::Message("DNS policy transport refused"))?,
            )?;
            if response.metadata.id != request.metadata.id
                || response.queries != request.queries
                || response.metadata.message_type != MessageType::Response
                || response.metadata.truncation
            {
                return Err(NetError::Message(
                    "DNS policy response does not match query",
                ));
            }
            Ok(response)
        })
        .boxed()
    }
}

async fn query<H: DnsHandle>(
    handle: &H,
    name: &str,
    kind: RecordType,
) -> Result<DnsResponse, PolicyError> {
    let name = Name::from_ascii(name).map_err(|_| PolicyError::Dns)?;
    let mut options = DnsRequestOptions::default();
    options.max_request_depth = 26;
    handle
        .lookup(Query::query(name, kind), options)
        .next()
        .await
        .ok_or(PolicyError::Dns)?
        .map_err(|_| PolicyError::Dnssec)
}

pub(crate) async fn txt(context: &PolicyContext, name: &str) -> Result<Vec<String>, PolicyError> {
    let response = query(&Doh::new(context.clone()), name, RecordType::TXT).await?;
    if !matches!(
        response.metadata.response_code,
        ResponseCode::NoError | ResponseCode::NXDomain
    ) {
        return Err(PolicyError::Dns);
    }
    response
        .answers
        .iter()
        .filter_map(|r| match &r.data {
            RData::TXT(t) => Some(t),
            _ => None,
        })
        .map(|t| {
            String::from_utf8(
                t.txt_data
                    .iter()
                    .flat_map(|part| part.iter().copied())
                    .collect(),
            )
            .map_err(|_| PolicyError::Dns)
        })
        .collect()
}

/// A validated chain is necessary before a TLSA owner can replace the original
/// next-hop name. Insecure aliases disable opportunistic DANE, never invent trust.
async fn address_base<H: DnsHandle>(handle: &H, host: &str) -> Result<Option<String>, PolicyError> {
    let mut name = host.to_string();
    for _ in 0..8 {
        let response = query(handle, &name, RecordType::A).await?;
        if !matches!(
            response.metadata.response_code,
            ResponseCode::NoError | ResponseCode::NXDomain
        ) {
            return Err(PolicyError::Dns);
        }
        let owner = Name::from_ascii(&name).map_err(|_| PolicyError::Dns)?;
        let records: Vec<_> = response
            .answers
            .iter()
            .filter(|r| r.name == owner && matches!(r.data, RData::A(_) | RData::CNAME(_)))
            .collect();
        if records
            .iter()
            .any(|r| matches!(r.proof, Proof::Bogus | Proof::Indeterminate))
        {
            return Err(PolicyError::Dnssec);
        }
        if records.iter().any(|r| r.proof == Proof::Insecure) {
            return Ok(None);
        }
        if let Some(alias) = records.iter().find_map(|r| match &r.data {
            RData::CNAME(n) => Some(n.0.to_ascii()),
            _ => None,
        }) {
            name = alias;
            continue;
        }
        if records.iter().any(|r| matches!(r.data, RData::A(_))) {
            return Ok(Some(name));
        }
        let response = query(handle, &name, RecordType::AAAA).await?;
        let records: Vec<_> = response
            .answers
            .iter()
            .filter(|r| r.name == owner && matches!(r.data, RData::AAAA(_) | RData::CNAME(_)))
            .collect();
        if records
            .iter()
            .any(|r| matches!(r.proof, Proof::Bogus | Proof::Indeterminate))
        {
            return Err(PolicyError::Dnssec);
        }
        if records.iter().any(|r| r.proof == Proof::Insecure) {
            return Ok(None);
        }
        if let Some(alias) = records.iter().find_map(|r| match &r.data {
            RData::CNAME(n) => Some(n.0.to_ascii()),
            _ => None,
        }) {
            name = alias;
            continue;
        }
        if records.iter().any(|r| matches!(r.data, RData::AAAA(_))) {
            return Ok(Some(name));
        }
        return Err(PolicyError::Dns);
    }
    Err(PolicyError::Dnssec)
}
async fn tlsa_at<H: DnsHandle>(handle: &H, owner: String) -> Result<Vec<TLSA>, PolicyError> {
    let mut name = owner;
    for _ in 0..8 {
        let response = query(handle, &name, RecordType::TLSA).await?;
        if !matches!(
            response.metadata.response_code,
            ResponseCode::NoError | ResponseCode::NXDomain
        ) {
            return Err(PolicyError::Dns);
        }
        let owner = Name::from_ascii(&name).map_err(|_| PolicyError::Dns)?;
        let records: Vec<_> = response
            .answers
            .iter()
            .filter(|r| r.name == owner && matches!(r.data, RData::TLSA(_) | RData::CNAME(_)))
            .collect();
        if records
            .iter()
            .any(|r| matches!(r.proof, Proof::Bogus | Proof::Indeterminate))
        {
            return Err(PolicyError::Dnssec);
        }
        if records.iter().any(|r| r.proof == Proof::Insecure) {
            return Ok(vec![]);
        }
        if let Some(alias) = records.iter().find_map(|r| match &r.data {
            RData::CNAME(n) => Some(n.0.to_ascii()),
            _ => None,
        }) {
            name = alias;
            continue;
        }
        if records.is_empty() && !response.answers.is_empty() {
            return Err(PolicyError::Dnssec);
        }
        return Ok(records
            .iter()
            .filter_map(|r| match &r.data {
                RData::TLSA(t) => Some(t.clone()),
                _ => None,
            })
            .collect());
    }
    Err(PolicyError::Dnssec)
}
#[derive(Debug, Default)]
pub(crate) struct DaneRecords {
    pub records: Vec<TLSA>,
    pub base: Option<String>,
}
pub(crate) async fn dane(
    context: &PolicyContext,
    host: &str,
    port: u16,
) -> Result<DaneRecords, PolicyError> {
    let handle = DnssecDnsHandle::new(Doh::new(context.clone()));
    let Some(base) = address_base(&handle, host).await? else {
        return Ok(DaneRecords::default());
    };
    let records = tlsa_at(&handle, format!("_{port}._tcp.{base}")).await?;
    if !records.is_empty() || base.trim_end_matches('.').eq_ignore_ascii_case(host) {
        return Ok(DaneRecords {
            records,
            base: Some(base),
        });
    }
    Ok(DaneRecords {
        records: tlsa_at(&handle, format!("_{port}._tcp.{host}")).await?,
        base: Some(host.into()),
    })
}
