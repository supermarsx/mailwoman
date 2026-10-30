use super::*;
use hickory_net::proto::rr::rdata::tlsa::{CertUsage, Matching, Selector};
use rcgen::{BasicConstraints, CertificateParams, IsCa, Issuer, KeyPair, KeyUsagePurpose};
use rustls_pki_types::PrivatePkcs8KeyDer;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

fn record(
    cert: &CertificateDer<'_>,
    usage: CertUsage,
    selector: Selector,
    matching: Matching,
) -> TLSA {
    let bytes = if selector == Selector::Spki {
        x509_cert::Certificate::from_der(cert.as_ref())
            .unwrap()
            .tbs_certificate()
            .subject_public_key_info()
            .to_der()
            .unwrap()
    } else {
        cert.as_ref().to_vec()
    };
    let bytes = match matching {
        Matching::Sha256 => Sha256::digest(bytes).to_vec(),
        Matching::Sha512 => Sha512::digest(bytes).to_vec(),
        _ => bytes,
    };
    TLSA::new(usage, selector, matching, bytes)
}
fn roots() -> RootCertStore {
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    roots
}
fn verifier(records: Vec<TLSA>, name: Option<&str>) -> Arc<DaneVerifier> {
    DaneVerifier::new(
        &TransportDecision {
            tlsa: records,
            sts_enforce: true,
            dane_tls: true,
            dane_name: name.map(str::to_owned),
        },
        roots(),
        Arc::new(rustls::crypto::ring::default_provider()),
    )
    .unwrap()
}

#[test]
fn ee_all_selectors_and_matching_types_ignore_pkix_names_but_reject_wrong_association() {
    let generated = rcgen::generate_simple_self_signed(vec!["different.example".into()]).unwrap();
    let cert = generated.cert.der();
    let name = ServerName::try_from("smtp.example").unwrap();
    for selector in [Selector::Full, Selector::Spki] {
        for matching in [Matching::Raw, Matching::Sha256, Matching::Sha512] {
            let t = record(cert, CertUsage::DaneEe, selector, matching);
            let v = verifier(vec![t.clone()], None);
            assert!(
                v.verify_server_cert(cert, &[], &name, &[], UnixTime::now())
                    .is_ok()
            );
            let mut bad = t;
            bad.cert_data[0] ^= 1;
            assert!(
                verifier(vec![bad], None)
                    .verify_server_cert(cert, &[], &name, &[], UnixTime::now())
                    .is_err()
            );
        }
    }
}
#[test]
fn ta_requires_a_valid_chain_and_authenticated_expanded_name() {
    let mut params = CertificateParams::new(vec![]).unwrap();
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.key_usages = vec![
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::DigitalSignature,
    ];
    let key = KeyPair::generate().unwrap();
    let ca = params.self_signed(&key).unwrap();
    let issuer = Issuer::new(params, key);
    let leaf_key = KeyPair::generate().unwrap();
    let leaf = CertificateParams::new(vec!["expanded.example".into()])
        .unwrap()
        .signed_by(&leaf_key, &issuer)
        .unwrap();
    let name = ServerName::try_from("original.example").unwrap();
    for selector in [Selector::Full, Selector::Spki] {
        for matching in [Matching::Raw, Matching::Sha256, Matching::Sha512] {
            let t = record(ca.der(), CertUsage::DaneTa, selector, matching);
            let valid = verifier(vec![t.clone()], Some("expanded.example"));
            assert!(
                valid
                    .verify_server_cert(
                        leaf.der(),
                        &[ca.der().clone()],
                        &name,
                        &[],
                        UnixTime::now()
                    )
                    .is_ok()
            );
            assert!(
                verifier(vec![t], None)
                    .verify_server_cert(
                        leaf.der(),
                        &[ca.der().clone()],
                        &name,
                        &[],
                        UnixTime::now()
                    )
                    .is_err()
            );
        }
    }
    let unrelated = rcgen::generate_simple_self_signed(vec!["expanded.example".into()]).unwrap();
    let v = verifier(
        vec![record(
            ca.der(),
            CertUsage::DaneTa,
            Selector::Full,
            Matching::Sha256,
        )],
        Some("expanded.example"),
    );
    assert!(
        v.verify_server_cert(
            unrelated.cert.der(),
            &[ca.der().clone()],
            &name,
            &[],
            UnixTime::now()
        )
        .is_err()
    );
}
#[test]
fn digest_agility_never_accepts_weaker_match_when_stronger_association_disagrees() {
    let generated = rcgen::generate_simple_self_signed(vec!["different.example".into()]).unwrap();
    let cert = generated.cert.der();
    let name = ServerName::try_from("smtp.example").unwrap();
    let weak = record(cert, CertUsage::DaneEe, Selector::Spki, Matching::Sha256);
    let mut strong = record(cert, CertUsage::DaneEe, Selector::Spki, Matching::Sha512);
    strong.cert_data[0] ^= 1;
    assert!(
        verifier(vec![weak.clone(), strong.clone()], None)
            .verify_server_cert(cert, &[], &name, &[], UnixTime::now())
            .is_err()
    );
    let full = record(cert, CertUsage::DaneEe, Selector::Spki, Matching::Raw);
    assert!(
        verifier(vec![weak, strong, full], None)
            .verify_server_cert(cert, &[], &name, &[], UnixTime::now())
            .is_ok()
    );
}
#[tokio::test]
async fn actual_tls_handshake_proves_leaf_private_key_and_rejects_wrong_pin() {
    let generated = rcgen::generate_simple_self_signed(vec!["unrelated.example".into()]).unwrap();
    let cert = generated.cert.der().clone();
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let server = rustls::ServerConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(
            vec![cert.clone()],
            PrivatePkcs8KeyDer::from(generated.signing_key.serialize_der()).into(),
        )
        .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        for _ in 0..2 {
            let (tcp, _) = listener.accept().await.unwrap();
            let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(server.clone()));
            if let Ok(mut tls) = acceptor.accept(tcp).await {
                tls.write_all(b"authenticated").await.unwrap();
            }
        }
    });
    for valid in [true, false] {
        let mut pin = record(&cert, CertUsage::DaneEe, Selector::Spki, Matching::Sha256);
        if !valid {
            pin.cert_data[0] ^= 1;
        }
        let client = rustls::ClientConfig::builder_with_provider(provider.clone())
            .with_safe_default_protocol_versions()
            .unwrap()
            .dangerous()
            .with_custom_certificate_verifier(verifier(vec![pin], None))
            .with_no_client_auth();
        let result = tokio_rustls::TlsConnector::from(Arc::new(client))
            .connect(
                ServerName::try_from("smtp.example").unwrap(),
                TcpStream::connect(addr).await.unwrap(),
            )
            .await;
        if valid {
            let mut bytes = [0; 13];
            result.unwrap().read_exact(&mut bytes).await.unwrap();
            assert_eq!(&bytes, b"authenticated");
        } else {
            assert!(result.is_err());
        }
    }
    task.await.unwrap();
}
