use crate::{PolicyError, TransportDecision};
use der::{Decode, Encode};
use hickory_net::proto::rr::rdata::TLSA;
use rustls::{
    DigitallySignedStruct, RootCertStore, SignatureScheme,
    client::{
        WebPkiServerVerifier,
        danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    },
    crypto::{CryptoProvider, verify_tls12_signature, verify_tls13_signature},
};
use rustls_pki_types::{CertificateDer, ServerName, UnixTime};
use sha2::{Digest, Sha256, Sha512};
use std::sync::Arc;
#[cfg(test)]
#[path = "tlsa_tests.rs"]
mod tests;

pub(crate) fn usable(t: &TLSA) -> bool {
    matches!(u8::from(t.cert_usage), 2 | 3)
        && matches!(u8::from(t.selector), 0 | 1)
        && match u8::from(t.matching) {
            0 => !t.cert_data.is_empty() && t.cert_data.len() <= 16384,
            1 => t.cert_data.len() == 32,
            2 => t.cert_data.len() == 64,
            _ => false,
        }
}
fn matches(t: &TLSA, cert: &CertificateDer<'_>) -> bool {
    if !usable(t) {
        return false;
    }
    let selected = match u8::from(t.selector) {
        0 => cert.as_ref().to_vec(),
        1 => match x509_cert::Certificate::from_der(cert.as_ref())
            .ok()
            .and_then(|c| c.tbs_certificate().subject_public_key_info().to_der().ok())
        {
            Some(v) => v,
            None => return false,
        },
        _ => return false,
    };
    match u8::from(t.matching) {
        0 => selected == t.cert_data,
        1 => Sha256::digest(&selected).as_slice() == t.cert_data,
        2 => Sha512::digest(&selected).as_slice() == t.cert_data,
        _ => false,
    }
}

/// This verifier can only be constructed from a decision produced by local
/// DNSSEC validation. DNS responses/AD bits cannot directly create an override.
#[derive(Debug)]
pub struct DaneVerifier {
    records: Vec<TLSA>,
    dane_name: Option<String>,
    pkix: Arc<WebPkiServerVerifier>,
    provider: Arc<CryptoProvider>,
}
impl DaneVerifier {
    pub fn new(
        decision: &TransportDecision,
        roots: RootCertStore,
        provider: Arc<CryptoProvider>,
    ) -> Result<Arc<Self>, PolicyError> {
        let pkix = WebPkiServerVerifier::builder_with_provider(Arc::new(roots), provider.clone())
            .build()
            .map_err(|_| PolicyError::Configuration)?;
        let records = decision
            .tlsa
            .iter()
            .filter(|record| {
                usable(record)
                    && (u8::from(record.matching) != 1
                        || !decision.tlsa.iter().any(|other| {
                            usable(other)
                                && other.cert_usage == record.cert_usage
                                && other.selector == record.selector
                                && u8::from(other.matching) == 2
                        }))
            })
            .cloned()
            .collect();
        Ok(Arc::new(Self {
            records,
            dane_name: decision.dane_name.clone(),
            pkix,
            provider,
        }))
    }
    fn verify_dane(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        // DANE-EE binds the leaf key/certificate directly. Validity dates and a
        // PKIX hostname are intentionally irrelevant, but CertificateVerify is
        // still checked below with the negotiated TLS algorithm.
        if self
            .records
            .iter()
            .any(|t| u8::from(t.cert_usage) == 3 && matches(t, end_entity))
        {
            return Ok(ServerCertVerified::assertion());
        }
        let dane_name = self
            .dane_name
            .as_deref()
            .and_then(|n| ServerName::try_from(n.trim_end_matches('.')).ok());
        let dane_name = dane_name.as_ref().unwrap_or(server_name);
        for record in self
            .records
            .iter()
            .filter(|t| u8::from(t.cert_usage) == 2 && usable(t))
        {
            let mut anchors: Vec<CertificateDer<'_>> = intermediates
                .iter()
                .filter(|cert| matches(record, cert))
                .cloned()
                .collect();
            if u8::from(record.selector) == 0 && u8::from(record.matching) == 0 {
                anchors.push(CertificateDer::from(record.cert_data.clone()));
            }
            for anchor in anchors {
                let mut roots = RootCertStore::empty();
                if roots.add(anchor).is_err() {
                    continue;
                }
                if let Ok(verifier) = WebPkiServerVerifier::builder_with_provider(
                    Arc::new(roots),
                    self.provider.clone(),
                )
                .build()
                    && let Ok(verified) =
                        verifier.verify_server_cert(end_entity, intermediates, dane_name, ocsp, now)
                {
                    return Ok(verified);
                }
            }
        }
        Err(rustls::Error::General(
            "SMTP DANE certificate authentication failed".into(),
        ))
    }
}
impl ServerCertVerifier for DaneVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        if self.records.is_empty() {
            return self
                .pkix
                .verify_server_cert(end_entity, intermediates, server_name, ocsp, now);
        }
        self.verify_dane(end_entity, intermediates, server_name, ocsp, now)
    }
    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls12_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }
    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls13_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.provider
            .signature_verification_algorithms
            .supported_schemes()
    }
}
