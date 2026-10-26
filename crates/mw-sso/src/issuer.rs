//! Fixed-algorithm OIDC issuer signing, using the existing OpenID Connect library.
//! This module owns no HTTP/session authority and never logs key or token bytes.
use openidconnect::core::{
    CoreGenderClaim, CoreJsonWebKeySet, CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm,
    CoreRsaPrivateSigningKey,
};
use openidconnect::{
    AccessToken, AdditionalClaims, ClientId, IdToken, IdTokenClaims, IdTokenVerifier, IssuerUrl,
    JsonWebKeyId, PrivateSigningKey,
};
use rsa::pkcs1::EncodeRsaPrivateKey;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtraClaims {
    sid: String,
}
impl AdditionalClaims for ExtraClaims {}
type Token = IdToken<
    ExtraClaims,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm,
>;

#[derive(Debug, thiserror::Error)]
#[error("OIDC signing or validation failed")]
pub struct IssuerCryptoError;
/// Generate an RSA3072 key. Call from a bounded blocking worker.
pub fn generate_rs256_key(kid: &str) -> Result<(String, Value), IssuerCryptoError> {
    let key = rsa::RsaPrivateKey::new(&mut rand10::rng(), 3072).map_err(|_| IssuerCryptoError)?;
    let pem = key
        .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
        .map_err(|_| IssuerCryptoError)?
        .to_string();
    let jwk = public_jwk(&pem, kid)?;
    Ok((pem, jwk))
}
pub fn public_jwk(pem: &str, kid: &str) -> Result<Value, IssuerCryptoError> {
    let key = CoreRsaPrivateSigningKey::from_pem(pem, Some(JsonWebKeyId::new(kid.into())))
        .map_err(|_| IssuerCryptoError)?;
    let mut value =
        serde_json::to_value(key.as_verification_key()).map_err(|_| IssuerCryptoError)?;
    value["use"] = Value::String("sig".into());
    value["alg"] = Value::String("RS256".into());
    Ok(value)
}
pub fn sign_id_token(
    pem: &str,
    kid: &str,
    claims: Value,
    access_token: &str,
) -> Result<String, IssuerCryptoError> {
    let claims: IdTokenClaims<ExtraClaims, CoreGenderClaim> =
        serde_json::from_value(claims).map_err(|_| IssuerCryptoError)?;
    let key = CoreRsaPrivateSigningKey::from_pem(pem, Some(JsonWebKeyId::new(kid.into())))
        .map_err(|_| IssuerCryptoError)?;
    Token::new(
        claims,
        &key,
        CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256,
        Some(&AccessToken::new(access_token.into())),
        None,
    )
    .map(|token| token.to_string())
    .map_err(|_| IssuerCryptoError)
}
/// Validate a logout hint against exact configured issuer/client and retained
/// public keys. Expired hints are rejected; the caller can offer a separate
/// explicit local logout confirmation without trusting the rejected hint.
pub fn verify_id_token(
    token: &str,
    issuer: &str,
    client: &str,
    jwks: Value,
    nonce: Option<&str>,
) -> Result<Value, IssuerCryptoError> {
    let token: Token = token.parse().map_err(|_| IssuerCryptoError)?;
    let keys: CoreJsonWebKeySet = serde_json::from_value(jwks).map_err(|_| IssuerCryptoError)?;
    let verifier = IdTokenVerifier::new_public_client(
        ClientId::new(client.into()),
        IssuerUrl::new(issuer.into()).map_err(|_| IssuerCryptoError)?,
        keys,
    )
    .set_allowed_algs([CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256]);
    let claims = token
        .claims(
            &verifier,
            |actual: Option<&openidconnect::Nonce>| match nonce {
                Some(expected) if actual.map(|value| value.secret().as_str()) != Some(expected) => {
                    Err("nonce mismatch".into())
                }
                _ => Ok(()),
            },
        )
        .map_err(|_| IssuerCryptoError)?;
    serde_json::to_value(claims).map_err(|_| IssuerCryptoError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn issued_token_is_library_verified_and_bound_to_issuer_audience_nonce() {
        let (pem, jwk) = generate_rs256_key("test-key").unwrap();
        assert!(jwk.get("d").is_none());
        assert_eq!(jwk["alg"], "RS256");
        let now = chrono::Utc::now().timestamp();
        let claims = json!({"iss":"https://issuer.example/oidc","sub":"principal-id","aud":["relying-party"],"iat":now,"exp":now+300,"auth_time":now-20,"nonce":"nonce-proof","sid":"grant-id"});
        let token = sign_id_token(&pem, "test-key", claims, "identity-access-token").unwrap();
        let jwks = json!({"keys":[jwk]});
        let valid = verify_id_token(
            &token,
            "https://issuer.example/oidc",
            "relying-party",
            jwks.clone(),
            Some("nonce-proof"),
        )
        .unwrap();
        assert_eq!(valid["sub"], "principal-id");
        assert_eq!(valid["sid"], "grant-id");
        assert!(valid.get("at_hash").is_some());
        assert!(
            verify_id_token(
                &token,
                "https://other.example/oidc",
                "relying-party",
                jwks.clone(),
                Some("nonce-proof")
            )
            .is_err()
        );
        assert!(
            verify_id_token(
                &token,
                "https://issuer.example/oidc",
                "other-client",
                jwks.clone(),
                Some("nonce-proof")
            )
            .is_err()
        );
        assert!(
            verify_id_token(
                &token,
                "https://issuer.example/oidc",
                "relying-party",
                jwks.clone(),
                Some("wrong")
            )
            .is_err()
        );
        let mut corrupted = token.into_bytes();
        let last = corrupted.len() - 10;
        corrupted[last] = if corrupted[last] == b'A' { b'B' } else { b'A' };
        assert!(
            verify_id_token(
                std::str::from_utf8(&corrupted).unwrap(),
                "https://issuer.example/oidc",
                "relying-party",
                jwks,
                None
            )
            .is_err()
        );
    }
}
