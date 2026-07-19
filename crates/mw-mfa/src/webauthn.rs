//! WebAuthn relying-party (server-side) verification, hand-rolled over RustCrypto.
//!
//! Scope is deliberately narrow — **attestation `"none"` only**, which is what
//! Mailwoman requests at registration. That means registration does no certificate
//! chain validation: it decodes and stores the COSE public key from `authData`, and
//! assertion later verifies a signature over `authData ‖ SHA256(clientDataJSON)`.
//!
//! Supported signature algorithms: ES256 (ECDSA/P-256/SHA-256, ASN.1 DER signature)
//! and EdDSA (Ed25519, raw 64-byte signature). RS256 is deferred.

use base64::Engine;
use sha2::{Digest, Sha256};

use crate::cbor::Reader;
use crate::cose::{CoseAlg, CoseKey};
use crate::{MfaError, ct_eq};

// authenticatorData flag bits (WebAuthn §6.1).
const FLAG_UP: u8 = 0x01; // User Present
const FLAG_UV: u8 = 0x04; // User Verified
const FLAG_AT: u8 = 0x40; // Attested credential data included

/// Policy for the user-verification requirement. "Preferred" is the Mailwoman
/// default (DQ2): UV is honoured when present but not mandated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserVerification {
    /// UV flag must be set or the ceremony is rejected.
    Required,
    /// UV is not mandated (still requires user presence).
    Preferred,
}

/// A registration ceremony to verify (`navigator.credentials.create` result).
#[derive(Debug, Clone)]
pub struct RegistrationRequest {
    /// The server-issued challenge (raw bytes) that was sent to the client.
    pub challenge: Vec<u8>,
    /// The exact expected origin, e.g. `https://mail.example.com`.
    pub origin: String,
    /// The relying-party id, e.g. `mail.example.com`.
    pub rp_id: String,
    /// CBOR `attestationObject` from the authenticator.
    pub attestation_object: Vec<u8>,
    /// Raw `clientDataJSON` bytes.
    pub client_data_json: Vec<u8>,
    /// User-verification policy for this ceremony.
    pub user_verification: UserVerification,
}

/// A credential extracted from a verified registration, ready to persist.
#[derive(Debug, Clone)]
pub struct RegisteredCredential {
    /// Raw credential id (store base64url; matched at assertion time).
    pub credential_id: Vec<u8>,
    /// Raw COSE_Key CBOR bytes — stored verbatim, re-decoded at assertion.
    pub cose_public_key: Vec<u8>,
    /// Initial signature counter reported by the authenticator.
    pub sign_count: u32,
    /// Authenticator AAGUID.
    pub aaguid: [u8; 16],
    /// Negotiated signature algorithm.
    pub alg: CoseAlg,
}

/// An assertion ceremony to verify (`navigator.credentials.get` result).
#[derive(Debug, Clone)]
pub struct AssertionRequest {
    /// The server-issued challenge (raw bytes) for this login attempt.
    pub challenge: Vec<u8>,
    /// The exact expected origin.
    pub origin: String,
    /// The relying-party id.
    pub rp_id: String,
    /// Raw `clientDataJSON` bytes.
    pub client_data_json: Vec<u8>,
    /// Raw `authenticatorData` bytes.
    pub authenticator_data: Vec<u8>,
    /// Raw signature bytes (DER for ES256, 64 bytes for EdDSA).
    pub signature: Vec<u8>,
    /// The stored COSE_Key CBOR captured at registration.
    pub cose_public_key: Vec<u8>,
    /// The last signature counter this relying party stored for the credential.
    pub stored_sign_count: u32,
    /// User-verification policy for this ceremony.
    pub user_verification: UserVerification,
}

/// The result of a successful assertion — the counter the caller must persist.
#[derive(Debug, Clone, Copy)]
pub struct AssertionOutcome {
    /// The new signature counter (already checked against regression).
    pub new_sign_count: u32,
}

/// Verify a registration ceremony (attestation `"none"`), returning the credential
/// to persist.
pub fn verify_registration(req: &RegistrationRequest) -> Result<RegisteredCredential, MfaError> {
    check_client_data(
        &req.client_data_json,
        "webauthn.create",
        &req.challenge,
        &req.origin,
    )?;

    let att = parse_attestation_object(&req.attestation_object)?;
    if att.fmt != "none" {
        return Err(MfaError::Attestation(format!(
            "only attestation \"none\" is accepted, got {:?}",
            att.fmt
        )));
    }

    let auth = parse_authenticator_data(&att.auth_data, /*expect_attested=*/ true)?;
    verify_rp_and_flags(&auth, &req.rp_id, req.user_verification)?;

    let cred = auth
        .attested
        .ok_or_else(|| MfaError::Attestation("missing attested credential data".into()))?;
    // Validate the COSE key decodes to a supported algorithm before persisting it.
    let key = CoseKey::from_cbor(&cred.cose_public_key)?;

    Ok(RegisteredCredential {
        credential_id: cred.credential_id,
        cose_public_key: cred.cose_public_key,
        sign_count: auth.sign_count,
        aaguid: cred.aaguid,
        alg: key.alg(),
    })
}

/// Verify an assertion ceremony, returning the counter to persist. Rejects a
/// signature-counter regression (a possible cloned authenticator).
pub fn verify_assertion(req: &AssertionRequest) -> Result<AssertionOutcome, MfaError> {
    check_client_data(
        &req.client_data_json,
        "webauthn.get",
        &req.challenge,
        &req.origin,
    )?;

    let auth = parse_authenticator_data(&req.authenticator_data, /*expect_attested=*/ false)?;
    verify_rp_and_flags(&auth, &req.rp_id, req.user_verification)?;

    // The signed message is authenticatorData ‖ SHA256(clientDataJSON).
    let client_hash = Sha256::digest(&req.client_data_json);
    let mut message = Vec::with_capacity(req.authenticator_data.len() + client_hash.len());
    message.extend_from_slice(&req.authenticator_data);
    message.extend_from_slice(&client_hash);

    let key = CoseKey::from_cbor(&req.cose_public_key)?;
    verify_signature(&key, &message, &req.signature)?;

    // Signature-counter regression check (WebAuthn §7.2 step 21). If both the new
    // and stored counters are zero the authenticator does not support a counter and
    // the check is skipped; otherwise the new counter MUST strictly exceed the
    // stored one.
    let new = auth.sign_count;
    let stored = req.stored_sign_count;
    if !(new == 0 && stored == 0) && new <= stored {
        return Err(MfaError::CounterRegression);
    }

    Ok(AssertionOutcome {
        new_sign_count: new,
    })
}

// ── clientDataJSON ───────────────────────────────────────────────────────────

fn check_client_data(
    client_data_json: &[u8],
    expected_type: &str,
    expected_challenge: &[u8],
    expected_origin: &str,
) -> Result<(), MfaError> {
    #[derive(serde::Deserialize)]
    struct ClientData {
        #[serde(rename = "type")]
        ty: String,
        challenge: String,
        origin: String,
    }
    let cd: ClientData = serde_json::from_slice(client_data_json)
        .map_err(|e| MfaError::ClientData(format!("parse: {e}")))?;

    if cd.ty != expected_type {
        return Err(MfaError::ClientData(format!(
            "type {:?} != {:?}",
            cd.ty, expected_type
        )));
    }
    // challenge is base64url (no padding) of the raw server challenge.
    let got = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(cd.challenge.as_bytes())
        .map_err(|_| MfaError::ClientData("challenge not valid base64url".into()))?;
    if !ct_eq(&got, expected_challenge) {
        return Err(MfaError::Challenge);
    }
    if cd.origin != expected_origin {
        return Err(MfaError::Origin);
    }
    Ok(())
}

// ── attestationObject ────────────────────────────────────────────────────────

struct AttestationObject {
    fmt: String,
    auth_data: Vec<u8>,
}

fn parse_attestation_object(bytes: &[u8]) -> Result<AttestationObject, MfaError> {
    let mut r = Reader::new(bytes);
    let n = r.map_len()?;
    let mut fmt: Option<String> = None;
    let mut auth_data: Option<Vec<u8>> = None;
    for _ in 0..n {
        let key = r.text()?;
        match key {
            "fmt" => fmt = Some(r.text()?.to_string()),
            "authData" => auth_data = Some(r.bytes()?.to_vec()),
            // attStmt (and anything else) is not needed for attestation "none".
            _ => r.skip()?,
        }
    }
    Ok(AttestationObject {
        fmt: fmt.ok_or_else(|| MfaError::Attestation("missing fmt".into()))?,
        auth_data: auth_data.ok_or_else(|| MfaError::Attestation("missing authData".into()))?,
    })
}

// ── authenticatorData ────────────────────────────────────────────────────────

struct AttestedCredential {
    aaguid: [u8; 16],
    credential_id: Vec<u8>,
    cose_public_key: Vec<u8>,
}

struct AuthenticatorData {
    rp_id_hash: [u8; 32],
    flags: u8,
    sign_count: u32,
    attested: Option<AttestedCredential>,
}

fn parse_authenticator_data(
    bytes: &[u8],
    expect_attested: bool,
) -> Result<AuthenticatorData, MfaError> {
    // rpIdHash(32) ‖ flags(1) ‖ signCount(4) ‖ [attestedCredentialData] ‖ [extensions]
    if bytes.len() < 37 {
        return Err(MfaError::Malformed("authenticatorData too short".into()));
    }
    let mut rp_id_hash = [0u8; 32];
    rp_id_hash.copy_from_slice(&bytes[0..32]);
    let flags = bytes[32];
    let sign_count = u32::from_be_bytes([bytes[33], bytes[34], bytes[35], bytes[36]]);

    let attested = if flags & FLAG_AT != 0 {
        // attestedCredentialData: aaguid(16) ‖ credIdLen(2 BE) ‖ credId ‖ COSE_Key
        let rest = &bytes[37..];
        if rest.len() < 18 {
            return Err(MfaError::Malformed(
                "attested credential data too short".into(),
            ));
        }
        let mut aaguid = [0u8; 16];
        aaguid.copy_from_slice(&rest[0..16]);
        let cred_id_len = u16::from_be_bytes([rest[16], rest[17]]) as usize;
        let cred_start: usize = 18;
        let cred_end = cred_start
            .checked_add(cred_id_len)
            .ok_or_else(|| MfaError::Malformed("credential id length overflow".into()))?;
        if cred_end > rest.len() {
            return Err(MfaError::Malformed("credential id runs past buffer".into()));
        }
        let credential_id = rest[cred_start..cred_end].to_vec();

        // The COSE_Key is the next CBOR item; measure exactly how many bytes it uses
        // so a trailing extensions map (if any) is not swept into the stored key.
        let cose_bytes = &rest[cred_end..];
        let mut r = Reader::new(cose_bytes);
        CoseKey::decode(&mut r)?;
        let cose_public_key = cose_bytes[..r.pos].to_vec();

        Some(AttestedCredential {
            aaguid,
            credential_id,
            cose_public_key,
        })
    } else {
        None
    };

    if expect_attested && attested.is_none() {
        return Err(MfaError::Attestation(
            "authenticatorData is missing attested credential data (AT flag not set)".into(),
        ));
    }

    Ok(AuthenticatorData {
        rp_id_hash,
        flags,
        sign_count,
        attested,
    })
}

fn verify_rp_and_flags(
    auth: &AuthenticatorData,
    rp_id: &str,
    uv: UserVerification,
) -> Result<(), MfaError> {
    let expected = Sha256::digest(rp_id.as_bytes());
    if !ct_eq(&auth.rp_id_hash, &expected) {
        return Err(MfaError::RpIdHash);
    }
    if auth.flags & FLAG_UP == 0 {
        return Err(MfaError::UserPresence);
    }
    if uv == UserVerification::Required && auth.flags & FLAG_UV == 0 {
        return Err(MfaError::UserVerification);
    }
    Ok(())
}

// ── signature verification ───────────────────────────────────────────────────

fn verify_signature(key: &CoseKey, message: &[u8], signature: &[u8]) -> Result<(), MfaError> {
    match key {
        CoseKey::Es256 { x, y } => {
            use p256::ecdsa::signature::Verifier;
            // Uncompressed SEC1 point: 0x04 ‖ X ‖ Y.
            let mut sec1 = [0u8; 65];
            sec1[0] = 0x04;
            sec1[1..33].copy_from_slice(x);
            sec1[33..65].copy_from_slice(y);
            let vk = p256::ecdsa::VerifyingKey::from_sec1_bytes(&sec1)
                .map_err(|e| MfaError::Verify(format!("bad P-256 key: {e}")))?;
            // WebAuthn ES256 signatures are ASN.1 DER, not raw r‖s.
            let sig = p256::ecdsa::DerSignature::try_from(signature)
                .map_err(|e| MfaError::Verify(format!("bad DER signature: {e}")))?;
            vk.verify(message, &sig)
                .map_err(|_| MfaError::Verify("ES256 signature did not verify".into()))
        }
        CoseKey::EdDsa { x } => {
            let vk = ed25519_dalek::VerifyingKey::from_bytes(x)
                .map_err(|e| MfaError::Verify(format!("bad Ed25519 key: {e}")))?;
            let sig_bytes: [u8; 64] = signature
                .try_into()
                .map_err(|_| MfaError::Verify("Ed25519 signature must be 64 bytes".into()))?;
            let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);
            vk.verify_strict(message, &sig)
                .map_err(|_| MfaError::Verify("EdDSA signature did not verify".into()))
        }
    }
}
