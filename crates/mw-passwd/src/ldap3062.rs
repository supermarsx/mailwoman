//! [`Ldap3062`] — LDAP password-modify extended operation (RFC 3062).
//!
//! ## Why the exop is encoded here (not routed through `mw-directory`)
//! mw-directory's **frozen** `DirectorySource` trait (plan §2.2) exposes
//! `search_gal`/`expand_group`/`lookup_cert`/`lookup_photo`/`bind_auth` — and **no
//! extended-operation passthrough**. RFC 3062 password-modify is an LDAP *exop*
//! (OID `1.3.6.1.4.1.4203.1.11.1`) whose request/response *values* are BER-encoded.
//! Since we must not edit mw-directory, this module owns the RFC 3062
//! [request encoding](encode_passwd_modify_request) / [response
//! parsing](parse_passwd_modify_response) and defers the actual bind+exop round-trip
//! to an injected [`LdapExopTransport`] port that e14 backs (via `ldap3` or the
//! directory connection layer). Tests exercise the *encoding* against a mock transport.

use async_trait::async_trait;

use crate::{
    BackendKind, Ctx, PasswordChangeBackend, PasswordChangeOutcome, PasswordError, PasswordPolicy,
    Result, Secret,
};

/// The RFC 3062 LDAP Password Modify extended-operation request OID.
pub const RFC3062_PASSWD_MODIFY_OID: &str = "1.3.6.1.4.1.4203.1.11.1";

// Context-specific, primitive BER tags for the PasswdModify request fields.
const TAG_USER_IDENTITY: u8 = 0x80; // [0]
const TAG_OLD_PASSWD: u8 = 0x81; // [1]
const TAG_NEW_PASSWD: u8 = 0x82; // [2]
const TAG_SEQUENCE: u8 = 0x30;
const TAG_GEN_PASSWD: u8 = 0x80; // response [0]

/// Encode a BER definite length (short form < 128, else long form).
fn encode_len(len: usize, out: &mut Vec<u8>) {
    if len < 0x80 {
        out.push(len as u8);
    } else {
        let bytes = len.to_be_bytes();
        let first = bytes
            .iter()
            .position(|&b| b != 0)
            .unwrap_or(bytes.len() - 1);
        let sig = &bytes[first..];
        out.push(0x80 | sig.len() as u8);
        out.extend_from_slice(sig);
    }
}

fn encode_tlv(tag: u8, value: &[u8], out: &mut Vec<u8>) {
    out.push(tag);
    encode_len(value.len(), out);
    out.extend_from_slice(value);
}

/// Encode a `PasswdModifyRequestValue` (RFC 3062 §2):
///
/// ```text
/// PasswdModifyRequestValue ::= SEQUENCE {
///     userIdentity [0] OCTET STRING OPTIONAL,
///     oldPasswd    [1] OCTET STRING OPTIONAL,
///     newPasswd    [2] OCTET STRING OPTIONAL }
/// ```
///
/// Absent fields are omitted (a server applies an absent `userIdentity` to the bound
/// user, and may generate a password when `newPasswd` is absent).
#[must_use]
pub fn encode_passwd_modify_request(
    user_identity: Option<&str>,
    old: Option<&str>,
    new: Option<&str>,
) -> Vec<u8> {
    let mut body = Vec::new();
    if let Some(u) = user_identity {
        encode_tlv(TAG_USER_IDENTITY, u.as_bytes(), &mut body);
    }
    if let Some(o) = old {
        encode_tlv(TAG_OLD_PASSWD, o.as_bytes(), &mut body);
    }
    if let Some(n) = new {
        encode_tlv(TAG_NEW_PASSWD, n.as_bytes(), &mut body);
    }
    let mut out = Vec::with_capacity(body.len() + 4);
    encode_tlv(TAG_SEQUENCE, &body, &mut out);
    out
}

/// Read a definite-length BER length at `pos`, returning `(length, header_bytes)`.
fn read_len(buf: &[u8], pos: usize) -> Result<(usize, usize)> {
    let first = *buf
        .get(pos)
        .ok_or_else(|| PasswordError::Protocol("truncated length".into()))?;
    if first < 0x80 {
        return Ok((first as usize, 1));
    }
    let n = (first & 0x7f) as usize;
    if n == 0 || n > std::mem::size_of::<usize>() {
        return Err(PasswordError::Protocol("bad long-form length".into()));
    }
    let mut len = 0usize;
    for i in 0..n {
        let b = *buf
            .get(pos + 1 + i)
            .ok_or_else(|| PasswordError::Protocol("truncated length".into()))?;
        len = (len << 8) | b as usize;
    }
    Ok((len, 1 + n))
}

/// Parse a `PasswdModifyResponseValue` (RFC 3062 §3), returning the server-generated
/// password if the response carried a `genPasswd [0]` field. An empty response value
/// (the common case for a client-supplied password) yields `None`.
///
/// ```text
/// PasswdModifyResponseValue ::= SEQUENCE { genPasswd [0] OCTET STRING OPTIONAL }
/// ```
pub fn parse_passwd_modify_response(value: &[u8]) -> Result<Option<String>> {
    if value.is_empty() {
        return Ok(None);
    }
    if value[0] != TAG_SEQUENCE {
        return Err(PasswordError::Protocol("expected SEQUENCE".into()));
    }
    let (seq_len, hdr) = read_len(value, 1)?;
    let body = value
        .get(1 + hdr..1 + hdr + seq_len)
        .ok_or_else(|| PasswordError::Protocol("truncated SEQUENCE".into()))?;
    if body.is_empty() {
        return Ok(None);
    }
    if body[0] != TAG_GEN_PASSWD {
        return Err(PasswordError::Protocol("unexpected response field".into()));
    }
    let (len, hdr) = read_len(body, 1)?;
    let gen_passwd = body
        .get(1 + hdr..1 + hdr + len)
        .ok_or_else(|| PasswordError::Protocol("truncated genPasswd".into()))?;
    Ok(Some(String::from_utf8(gen_passwd.to_vec()).map_err(
        |_| PasswordError::Protocol("non-utf8 genPasswd".into()),
    )?))
}

/// The LDAP exop transport seam: bind and issue the RFC 3062 extended operation,
/// returning the response *value* bytes. Backed at mount (e14) by `ldap3`'s
/// `extended`/`with_controls` API (OID [`RFC3062_PASSWD_MODIFY_OID`]); tests use a mock.
#[async_trait]
pub trait LdapExopTransport: Send + Sync {
    /// Send the password-modify exop with the given BER-encoded request value and
    /// return the (possibly empty) BER-encoded response value.
    async fn passwd_modify(&self, request_value: &[u8]) -> Result<Vec<u8>>;
}

/// LDAP password-modify (RFC 3062) backend.
pub struct Ldap3062<T: LdapExopTransport> {
    transport: T,
    policy: PasswordPolicy,
    /// Whether to send `userIdentity [0]` (the account's DN/authzid from `ctx.username`).
    /// When `false`, the server applies the change to the currently-bound identity.
    send_user_identity: bool,
}

impl<T: LdapExopTransport> Ldap3062<T> {
    #[must_use]
    pub fn new(transport: T, policy: PasswordPolicy) -> Self {
        Self {
            transport,
            policy,
            send_user_identity: true,
        }
    }

    /// Do not send `userIdentity` (rely on the bound identity).
    #[must_use]
    pub fn without_user_identity(mut self) -> Self {
        self.send_user_identity = false;
        self
    }
}

#[async_trait]
impl<T: LdapExopTransport> PasswordChangeBackend for Ldap3062<T> {
    async fn change(&self, ctx: &Ctx, old: Secret, new: Secret) -> Result<PasswordChangeOutcome> {
        self.policy.validate(&new)?;
        let identity = if self.send_user_identity && !ctx.username.is_empty() {
            Some(ctx.username.as_str())
        } else {
            None
        };
        let request =
            encode_passwd_modify_request(identity, Some(old.expose()), Some(new.expose()));
        let response = self.transport.passwd_modify(&request).await?;
        // A server-generated password would surface here; we supplied one, so ignore it.
        let _ = parse_passwd_modify_response(&response)?;
        Ok(PasswordChangeOutcome::changed_from(ctx))
    }

    fn policy(&self) -> PasswordPolicy {
        self.policy.clone()
    }

    fn kind(&self) -> BackendKind {
        BackendKind::Ldap3062
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn encodes_request_value_per_rfc3062() {
        // userIdentity="u", old="a", new="b"  ⇒  30 09 80 01 75 81 01 61 82 01 62
        let got = encode_passwd_modify_request(Some("u"), Some("a"), Some("b"));
        assert_eq!(
            got,
            vec![
                0x30, 0x09, 0x80, 0x01, 0x75, 0x81, 0x01, 0x61, 0x82, 0x01, 0x62
            ]
        );
    }

    #[test]
    fn omits_absent_fields() {
        // Only newPasswd present ⇒ 30 03 82 01 62
        let got = encode_passwd_modify_request(None, None, Some("b"));
        assert_eq!(got, vec![0x30, 0x03, 0x82, 0x01, 0x62]);
    }

    #[test]
    fn long_form_length_for_big_password() {
        let pw = "x".repeat(200);
        let got = encode_passwd_modify_request(None, None, Some(&pw));
        // outer SEQUENCE body = tag(1)+len-hdr(2)+value(200) = 203 ⇒ long-form 0x81 0xCB
        assert_eq!(&got[0..2], &[0x30, 0x81]);
        assert_eq!(got[2], 203);
        assert_eq!(got[3], TAG_NEW_PASSWD);
    }

    #[test]
    fn parses_generated_password_and_empty_response() {
        assert_eq!(parse_passwd_modify_response(&[]).unwrap(), None);
        // 30 03 80 01 5A ⇒ genPasswd "Z"
        assert_eq!(
            parse_passwd_modify_response(&[0x30, 0x03, 0x80, 0x01, 0x5A]).unwrap(),
            Some("Z".to_string())
        );
        // Empty SEQUENCE ⇒ None
        assert_eq!(parse_passwd_modify_response(&[0x30, 0x00]).unwrap(), None);
    }

    struct MockExop {
        seen: Mutex<Vec<u8>>,
        response: Vec<u8>,
        fail: bool,
    }
    #[async_trait]
    impl LdapExopTransport for MockExop {
        async fn passwd_modify(&self, request_value: &[u8]) -> Result<Vec<u8>> {
            *self.seen.lock().unwrap() = request_value.to_vec();
            if self.fail {
                return Err(PasswordError::Protocol("unwillingToPerform".into()));
            }
            Ok(self.response.clone())
        }
    }

    #[tokio::test]
    async fn happy_path_sends_encoded_exop() {
        let mock = MockExop {
            seen: Mutex::new(Vec::new()),
            response: vec![0x30, 0x00],
            fail: false,
        };
        let backend = Ldap3062::new(mock, PasswordPolicy::default());
        let ctx = Ctx {
            reseal_credentials: true,
            ..Ctx::new("a1", "u")
        };
        let out = backend
            .change(&ctx, Secret::new("old"), Secret::new("new-strong-password"))
            .await
            .unwrap();
        assert!(out.changed && out.reencrypt_credentials);
        // The transport received a well-formed SEQUENCE carrying all three fields.
        let seen = backend.transport.seen.lock().unwrap().clone();
        assert_eq!(seen[0], TAG_SEQUENCE);
        assert!(seen.contains(&TAG_USER_IDENTITY));
    }

    #[tokio::test]
    async fn deny_path_transport_error_propagates() {
        let mock = MockExop {
            seen: Mutex::new(Vec::new()),
            response: Vec::new(),
            fail: true,
        };
        let backend = Ldap3062::new(mock, PasswordPolicy::default());
        let err = backend
            .change(
                &Ctx::new("a1", "u"),
                Secret::new("old"),
                Secret::new("new-strong-password"),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, PasswordError::Protocol(_)));
    }
}
