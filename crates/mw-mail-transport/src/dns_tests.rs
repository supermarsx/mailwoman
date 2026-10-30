use super::*;
use hickory_net::proto::{
    dnssec::{
        Algorithm, DigestType, SigningKey, TBS, TrustAnchors,
        crypto::Ed25519SigningKey,
        rdata::{DNSKEY, DNSSECRData, DS, NSEC, RRSIG, sig::SigInput},
    },
    op::Message,
    rr::SerialNumber,
    rr::{
        DNSClass, Record,
        rdata::{
            A,
            tlsa::{CertUsage, Matching, Selector},
        },
    },
};
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

type ZoneRecords = HashMap<(Name, RecordType), (Vec<Record>, Vec<Record>)>;
#[derive(Clone)]
struct Zone {
    records: Arc<ZoneRecords>,
    forge_ad: bool,
}
impl DnsHandle for Zone {
    type Response = BoxStream<'static, Result<DnsResponse, NetError>>;
    type Runtime = TokioRuntimeProvider;
    fn send(&self, request: DnsRequest) -> Self::Response {
        let q = &request.queries[0];
        let records = self
            .records
            .get(&(q.name().clone(), q.query_type()))
            .cloned();
        let mut message = Message::response(request.metadata.id, request.metadata.op_code);
        message.queries = request.queries.clone();
        message.metadata.authentic_data = self.forge_ad;
        if let Some((answers, authorities)) = records {
            message.answers = answers;
            message.authorities = authorities;
        }
        // Roundtrip through wire so callers cannot inject record Proof metadata.
        stream::once(async move { Ok(DnsResponse::from_buffer(message.to_vec()?)?) }).boxed()
    }
}
fn key() -> Ed25519SigningKey {
    Ed25519SigningKey::from_pkcs8(&Ed25519SigningKey::generate_pkcs8().unwrap()).unwrap()
}
fn sign(record: Record, signer: &Ed25519SigningKey, zone: &Name) -> Vec<Record> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    let public = signer.to_public_key().unwrap();
    let dnskey = DNSKEY::from_key(&public);
    let input = SigInput {
        type_covered: record.record_type(),
        algorithm: Algorithm::ED25519,
        num_labels: record.name.num_labels(),
        original_ttl: 300,
        sig_expiration: SerialNumber::new(now + 3600),
        sig_inception: SerialNumber::new(now - 60),
        key_tag: dnskey.calculate_key_tag().unwrap(),
        signer_name: zone.clone(),
    };
    let tbs =
        TBS::from_input(&record.name, DNSClass::IN, &input, std::iter::once(&record)).unwrap();
    let signature = signer.sign(&tbs).unwrap();
    let rr = Record::from_rdata(
        record.name.clone(),
        300,
        RData::DNSSEC(DNSSECRData::RRSIG(RRSIG::from_sig(input, signature))),
    );
    vec![record, rr]
}
fn fixture() -> (Zone, Arc<TrustAnchors>) {
    let root = key();
    let child = key();
    let root_name = Name::root();
    let zone = Name::from_ascii("example.").unwrap();
    let mut anchors = TrustAnchors::empty();
    anchors.insert(&root.to_public_key().unwrap());
    let mut records = HashMap::new();
    for (name, key) in [(&root_name, &root), (&zone, &child)] {
        let record = Record::from_rdata(
            name.clone(),
            300,
            RData::DNSSEC(DNSSECRData::DNSKEY(DNSKEY::from_key(
                &key.to_public_key().unwrap(),
            ))),
        );
        records.insert(
            (name.clone(), RecordType::DNSKEY),
            (sign(record, key, name), vec![]),
        );
    }
    let child_key = DNSKEY::from_key(&child.to_public_key().unwrap());
    let ds = DS::new(
        child_key.calculate_key_tag().unwrap(),
        Algorithm::ED25519,
        DigestType::SHA256,
        child_key
            .to_digest(&zone, DigestType::SHA256)
            .unwrap()
            .as_ref()
            .to_vec(),
    );
    records.insert(
        (zone.clone(), RecordType::DS),
        (
            sign(
                Record::from_rdata(zone.clone(), 300, RData::DNSSEC(DNSSECRData::DS(ds))),
                &root,
                &root_name,
            ),
            vec![],
        ),
    );
    let host = Name::from_ascii("smtp.example.").unwrap();
    records.insert(
        (host.clone(), RecordType::A),
        (
            sign(
                Record::from_rdata(host, 300, RData::A(A::new(192, 0, 2, 1))),
                &child,
                &zone,
            ),
            vec![],
        ),
    );
    let alias = Name::from_ascii("alias.example.").unwrap();
    records.insert(
        (alias.clone(), RecordType::A),
        (
            sign(
                Record::from_rdata(
                    alias,
                    300,
                    RData::CNAME(hickory_net::proto::rr::rdata::CNAME(
                        Name::from_ascii("smtp.example.").unwrap(),
                    )),
                ),
                &child,
                &zone,
            ),
            vec![],
        ),
    );
    let owner = Name::from_ascii("_25._tcp.smtp.example.").unwrap();
    let tlsa = TLSA::new(
        CertUsage::DaneEe,
        Selector::Spki,
        Matching::Sha256,
        vec![7; 32],
    );
    records.insert(
        (owner.clone(), RecordType::TLSA),
        (
            sign(
                Record::from_rdata(owner, 300, RData::TLSA(tlsa)),
                &child,
                &zone,
            ),
            vec![],
        ),
    );
    let empty = Name::from_ascii("_465._tcp.smtp.example.").unwrap();
    let nsec = Record::from_rdata(
        empty.clone(),
        300,
        RData::DNSSEC(DNSSECRData::NSEC(NSEC::new(
            Name::from_ascii("z.example.").unwrap(),
            [RecordType::NSEC, RecordType::RRSIG],
        ))),
    );
    records.insert(
        (empty, RecordType::TLSA),
        (vec![], sign(nsec, &child, &zone)),
    );
    (
        Zone {
            records: Arc::new(records),
            forge_ad: false,
        },
        Arc::new(anchors),
    )
}
#[tokio::test]
async fn validates_real_root_ds_dnskey_chain_without_ad_and_authenticated_denial() {
    let (zone, anchors) = fixture();
    let handle = DnssecDnsHandle::with_trust_anchor(zone, anchors);
    assert_eq!(
        address_base(&handle, "smtp.example.").await.unwrap(),
        Some("smtp.example.".into())
    );
    assert_eq!(
        address_base(&handle, "alias.example.").await.unwrap(),
        Some("smtp.example.".into())
    );
    let records = tlsa_at(&handle, "_25._tcp.smtp.example.".into())
        .await
        .unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].cert_data, vec![7; 32]);
    assert!(
        tlsa_at(&handle, "_465._tcp.smtp.example.".into())
            .await
            .unwrap()
            .is_empty()
    );
}
#[tokio::test]
async fn forged_ad_does_not_authenticate_a_modified_alias_chain() {
    let (mut zone, anchors) = fixture();
    zone.forge_ad = true;
    Arc::make_mut(&mut zone.records)
        .get_mut(&(Name::from_ascii("alias.example.").unwrap(), RecordType::A))
        .unwrap()
        .0[0]
        .data = RData::CNAME(hickory_net::proto::rr::rdata::CNAME(
        Name::from_ascii("attacker.example.").unwrap(),
    ));
    assert!(
        address_base(
            &DnssecDnsHandle::with_trust_anchor(zone, anchors),
            "alias.example."
        )
        .await
        .is_err()
    );
}
#[tokio::test]
async fn rejects_tampered_tlsa_and_forged_ad_unsigned_answers() {
    for unsigned in [false, true] {
        let (mut zone, anchors) = fixture();
        zone.forge_ad = true;
        let records = Arc::make_mut(&mut zone.records)
            .get_mut(&(
                Name::from_ascii("_25._tcp.smtp.example.").unwrap(),
                RecordType::TLSA,
            ))
            .unwrap();
        if unsigned {
            records.0.retain(|r| r.record_type() != RecordType::RRSIG);
        } else if let RData::TLSA(t) = &mut records.0[0].data {
            t.cert_data[0] ^= 1;
        }
        let handle = DnssecDnsHandle::with_trust_anchor(zone, anchors);
        assert!(
            tlsa_at(&handle, "_25._tcp.smtp.example.".into())
                .await
                .is_err()
        );
    }
}
#[tokio::test]
async fn rejects_unsigned_and_tampered_denial_even_with_forged_ad() {
    for unsigned in [false, true] {
        let (mut zone, anchors) = fixture();
        zone.forge_ad = true;
        let records = Arc::make_mut(&mut zone.records)
            .get_mut(&(
                Name::from_ascii("_465._tcp.smtp.example.").unwrap(),
                RecordType::TLSA,
            ))
            .unwrap();
        if unsigned {
            records.1.clear();
        } else {
            records.1[0].ttl = 299;
            records.1[0].name = Name::from_ascii("other.example.").unwrap();
        }
        let handle = DnssecDnsHandle::with_trust_anchor(zone, anchors);
        assert!(
            tlsa_at(&handle, "_465._tcp.smtp.example.".into())
                .await
                .is_err()
        );
    }
}
