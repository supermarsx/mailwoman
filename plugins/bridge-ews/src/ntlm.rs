//! Pure-Rust NTLMv2 message flow (MS-NLMP) for on-prem EWS `Authorization: NTLM`
//! (plan §3 e11, R2). **No GSSAPI, no C, no Kerberos** — Kerberos-SSO is a
//! documented gap (BYO reverse-proxy-auth; native post-1.0). This builds the
//! NEGOTIATE (Type 1) and AUTHENTICATE (Type 3) messages and parses the CHALLENGE
//! (Type 2), computing the NTLMv2 response over the hand-rolled [`crate::md`]
//! primitives. Correctness is pinned by the MS-NLMP §4.2.4 `NTOWFv2` vector below.
//!
//! Transport (the HTTP 401 challenge/response dance) lives in the wasm guest, which
//! runs it over the host `http-fetch` import; these functions are the pure,
//! host-testable core.

use crate::md::{hmac_md5, md4, md5, utf16le};

const SIGNATURE: &[u8; 8] = b"NTLMSSP\0";

// NTLM negotiate flags (MS-NLMP §2.2.2.5) used by the Type 1 message.
const NTLMSSP_NEGOTIATE_UNICODE: u32 = 0x0000_0001;
const NTLMSSP_REQUEST_TARGET: u32 = 0x0000_0004;
const NTLMSSP_NEGOTIATE_NTLM: u32 = 0x0000_0200;
const NTLMSSP_NEGOTIATE_ALWAYS_SIGN: u32 = 0x0000_8000;
const NTLMSSP_NEGOTIATE_EXTENDED_SESSIONSECURITY: u32 = 0x0008_0000;

const TYPE1_FLAGS: u32 = NTLMSSP_NEGOTIATE_UNICODE
    | NTLMSSP_REQUEST_TARGET
    | NTLMSSP_NEGOTIATE_NTLM
    | NTLMSSP_NEGOTIATE_ALWAYS_SIGN
    | NTLMSSP_NEGOTIATE_EXTENDED_SESSIONSECURITY;

/// `NTOWFv2 = HMAC_MD5(MD4(UTF16LE(password)), UTF16LE(UPPER(user) ++ domain))`
/// (MS-NLMP §3.3.2). The domain is NOT upper-cased (only the user).
#[must_use]
pub fn ntowf_v2(user: &str, domain: &str, password: &str) -> [u8; 16] {
    let nt_hash = md4(&utf16le(password));
    let mut identity = user.to_uppercase();
    identity.push_str(domain);
    hmac_md5(&nt_hash, &utf16le(&identity))
}

/// The NEGOTIATE (Type 1) message bytes — an anonymous, minimal negotiate with no
/// domain/workstation payload (offsets point past the fixed header).
#[must_use]
pub fn type1_message() -> Vec<u8> {
    let mut m = Vec::with_capacity(32);
    m.extend_from_slice(SIGNATURE);
    m.extend_from_slice(&1u32.to_le_bytes()); // MessageType = 1
    m.extend_from_slice(&TYPE1_FLAGS.to_le_bytes());
    // DomainName security buffer (len, maxlen, offset) — empty.
    m.extend_from_slice(&0u16.to_le_bytes());
    m.extend_from_slice(&0u16.to_le_bytes());
    m.extend_from_slice(&32u32.to_le_bytes());
    // Workstation security buffer — empty.
    m.extend_from_slice(&0u16.to_le_bytes());
    m.extend_from_slice(&0u16.to_le_bytes());
    m.extend_from_slice(&32u32.to_le_bytes());
    m
}

/// Parsed CHALLENGE (Type 2) message: the 8-byte server challenge and the raw
/// TargetInfo (AV_PAIR) blob echoed back verbatim in the Type 3 response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Challenge {
    pub server_challenge: [u8; 8],
    pub target_info: Vec<u8>,
    pub flags: u32,
}

/// Parse a CHALLENGE (Type 2) message (MS-NLMP §2.2.1.2).
pub fn parse_challenge(bytes: &[u8]) -> Result<Challenge, String> {
    if bytes.len() < 48 {
        return Err("NTLM Type 2 too short".into());
    }
    if &bytes[0..8] != SIGNATURE {
        return Err("NTLM Type 2 bad signature".into());
    }
    let msg_type = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    if msg_type != 2 {
        return Err(format!("expected NTLM Type 2, got {msg_type}"));
    }
    let flags = u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    let mut server_challenge = [0u8; 8];
    server_challenge.copy_from_slice(&bytes[24..32]);

    // TargetInfoFields at offset 40: len(2), maxlen(2), offset(4).
    let ti_len = u16::from_le_bytes([bytes[40], bytes[41]]) as usize;
    let ti_off = u32::from_le_bytes([bytes[44], bytes[45], bytes[46], bytes[47]]) as usize;
    let target_info = if ti_len == 0 {
        Vec::new()
    } else {
        let end = ti_off
            .checked_add(ti_len)
            .ok_or("NTLM Type 2 target-info overflow")?;
        if end > bytes.len() {
            return Err("NTLM Type 2 target-info out of bounds".into());
        }
        bytes[ti_off..end].to_vec()
    };

    Ok(Challenge {
        server_challenge,
        target_info,
        flags,
    })
}

/// Convert unix-millis to a Windows FILETIME (100 ns ticks since 1601-01-01 UTC).
#[must_use]
pub fn filetime_from_unix_millis(unix_millis: u64) -> u64 {
    // 11644473600 seconds between 1601-01-01 and 1970-01-01.
    unix_millis
        .wrapping_mul(10_000)
        .wrapping_add(116_444_736_000_000_000)
}

/// The NTLMv2 response pair: `NtChallengeResponse` (NTProofStr ++ blob) and
/// `LmChallengeResponse`, per MS-NLMP §3.3.2.
#[derive(Clone)]
pub struct Ntlmv2Response {
    pub nt_challenge_response: Vec<u8>,
    pub lm_challenge_response: Vec<u8>,
    pub session_base_key: [u8; 16],
}
impl std::fmt::Debug for Ntlmv2Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ntlmv2Response")
            .field("nt_response_bytes", &self.nt_challenge_response.len())
            .field("lm_response_bytes", &self.lm_challenge_response.len())
            .field("session_base_key", &"[redacted]")
            .finish()
    }
}

/// Compute the NTLMv2 response (deterministic given the client challenge +
/// timestamp ⇒ host-testable). `target_info` is the blob from the Type 2 challenge.
#[must_use]
pub fn ntlmv2_response(
    ntowf2: &[u8; 16],
    server_challenge: &[u8; 8],
    client_challenge: &[u8; 8],
    timestamp_filetime: u64,
    target_info: &[u8],
) -> Ntlmv2Response {
    // temp = Responserversion(1) ++ HiResponserversion(1) ++ Z(6) ++ Timestamp(8)
    //        ++ ClientChallenge(8) ++ Z(4) ++ TargetInfo ++ Z(4)
    let mut temp = Vec::with_capacity(28 + target_info.len() + 4);
    temp.push(0x01);
    temp.push(0x01);
    temp.extend_from_slice(&[0u8; 6]);
    temp.extend_from_slice(&timestamp_filetime.to_le_bytes());
    temp.extend_from_slice(client_challenge);
    temp.extend_from_slice(&[0u8; 4]);
    temp.extend_from_slice(target_info);
    temp.extend_from_slice(&[0u8; 4]);

    // NTProofStr = HMAC_MD5(NTOWFv2, server_challenge ++ temp)
    let mut proof_input = Vec::with_capacity(8 + temp.len());
    proof_input.extend_from_slice(server_challenge);
    proof_input.extend_from_slice(&temp);
    let nt_proof = hmac_md5(ntowf2, &proof_input);

    let mut nt_challenge_response = Vec::with_capacity(16 + temp.len());
    nt_challenge_response.extend_from_slice(&nt_proof);
    nt_challenge_response.extend_from_slice(&temp);

    // LMv2 = HMAC_MD5(NTOWFv2, server_challenge ++ client_challenge) ++ client_challenge
    let mut lm_input = Vec::with_capacity(16);
    lm_input.extend_from_slice(server_challenge);
    lm_input.extend_from_slice(client_challenge);
    let mut lm_challenge_response = hmac_md5(ntowf2, &lm_input).to_vec();
    lm_challenge_response.extend_from_slice(client_challenge);

    let session_base_key = hmac_md5(ntowf2, &nt_proof);

    Ntlmv2Response {
        nt_challenge_response,
        lm_challenge_response,
        session_base_key,
    }
}

/// Build the legacy fixed-header AUTHENTICATE (Type 3) shape. The production EWS
/// path uses [`protected_authenticate`] to add Version padding, MIC and binding.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn type3_message(
    user: &str,
    domain: &str,
    workstation: &str,
    resp: &Ntlmv2Response,
) -> Vec<u8> {
    let domain_b = utf16le(domain);
    let user_b = utf16le(user);
    let ws_b = utf16le(workstation);
    let lm = &resp.lm_challenge_response;
    let nt = &resp.nt_challenge_response;

    // Fixed header (no Version, no MIC): signature(8) + type(4) + 6 security
    // buffers (8 each) + flags(4) = 64 bytes; payload follows.
    const HEADER: usize = 64;
    let mut payload = Vec::new();
    let push = |buf: &[u8], payload: &mut Vec<u8>| -> (u16, u32) {
        let off = (HEADER + payload.len()) as u32;
        payload.extend_from_slice(buf);
        (buf.len() as u16, off)
    };

    let (lm_len, lm_off) = push(lm, &mut payload);
    let (nt_len, nt_off) = push(nt, &mut payload);
    let (dom_len, dom_off) = push(&domain_b, &mut payload);
    let (usr_len, usr_off) = push(&user_b, &mut payload);
    let (ws_len, ws_off) = push(&ws_b, &mut payload);
    // EncryptedRandomSessionKey — empty (no key-exchange).
    let sk_off = (HEADER + payload.len()) as u32;

    let mut m = Vec::with_capacity(HEADER + payload.len());
    m.extend_from_slice(SIGNATURE);
    m.extend_from_slice(&3u32.to_le_bytes());
    let sec = |len: u16, off: u32, m: &mut Vec<u8>| {
        m.extend_from_slice(&len.to_le_bytes());
        m.extend_from_slice(&len.to_le_bytes());
        m.extend_from_slice(&off.to_le_bytes());
    };
    sec(lm_len, lm_off, &mut m);
    sec(nt_len, nt_off, &mut m);
    sec(dom_len, dom_off, &mut m);
    sec(usr_len, usr_off, &mut m);
    sec(ws_len, ws_off, &mut m);
    sec(0, sk_off, &mut m); // session key buffer, empty
    m.extend_from_slice(&TYPE1_FLAGS.to_le_bytes());
    debug_assert_eq!(m.len(), HEADER);
    m.extend_from_slice(&payload);
    m
}

/// RFC 4121 channel-bindings serialization for absent initiator/acceptor
/// addresses and the RFC 5929 tls-server-end-point application binding.
pub fn channel_binding_hash(certificate_digest: &[u8]) -> Result<[u8; 16], String> {
    if !matches!(certificate_digest.len(), 28 | 32 | 48 | 64) {
        return Err("verified TLS certificate binding is required".into());
    }
    let mut application = b"tls-server-end-point:".to_vec();
    application.extend_from_slice(certificate_digest);
    let mut bindings = vec![0; 16];
    bindings.extend_from_slice(&(application.len() as u32).to_le_bytes());
    bindings.extend_from_slice(&application);
    Ok(md5(&bindings))
}

fn protected_target_info(
    raw: &[u8],
    binding: [u8; 16],
    target_name: &str,
    fallback_time: u64,
) -> Result<(Vec<u8>, u64), String> {
    if raw.len() > 8192
        || target_name.len() > 255
        || !target_name.starts_with("HTTP/")
        || target_name.len() <= 5
        || !target_name.bytes().all(|b| (0x21..0x7f).contains(&b))
    {
        return Err("invalid NTLM target information".into());
    }
    let mut offset = 0;
    let mut seen = std::collections::BTreeSet::new();
    let mut pairs = Vec::new();
    let mut flags = 0u32;
    let mut timestamp = fallback_time;
    let mut ended = raw.is_empty();
    while offset < raw.len() {
        let head = raw
            .get(offset..offset + 4)
            .ok_or("truncated NTLM target information")?;
        let id = u16::from_le_bytes([head[0], head[1]]);
        let len = u16::from_le_bytes([head[2], head[3]]) as usize;
        offset += 4;
        let value = raw
            .get(offset..offset + len)
            .ok_or("truncated NTLM target information")?;
        offset += len;
        if id == 0 {
            if len != 0 || raw[offset..].iter().any(|byte| *byte != 0) {
                return Err("invalid NTLM target terminator".into());
            }
            ended = true;
            break;
        }
        if seen.len() >= 128 || !seen.insert(id) {
            return Err("duplicate NTLM target information".into());
        }
        match id {
            6 => {
                if len != 4 {
                    return Err("invalid NTLM target flags".into());
                }
                flags = u32::from_le_bytes(value.try_into().unwrap());
            }
            7 => {
                if len != 8 {
                    return Err("invalid NTLM server timestamp".into());
                }
                timestamp = u64::from_le_bytes(value.try_into().unwrap());
                pairs.push((id, value.to_vec()));
            }
            9 => {}
            10 => {
                if len != 16 {
                    return Err("invalid NTLM channel binding".into());
                }
            }
            _ => pairs.push((id, value.to_vec())),
        }
    }
    if !ended {
        return Err("unterminated NTLM target information".into());
    }
    pairs.push((6, (flags | 2).to_le_bytes().to_vec()));
    pairs.push((9, utf16le(target_name)));
    pairs.push((10, binding.to_vec()));
    let mut result = Vec::new();
    for (id, value) in pairs {
        result.extend_from_slice(&id.to_le_bytes());
        result.extend_from_slice(&(value.len() as u16).to_le_bytes());
        result.extend_from_slice(&value);
    }
    result.extend_from_slice(&[0; 4]);
    Ok((result, timestamp))
}

/// NTLMv2 with transcript MIC, TLS channel binding and service binding. There is
/// no weaker retry: malformed challenges or unavailable verified TLS bindings
/// are authentication failures. No KEY_EXCH was offered, so the exported key
/// is the NTLMv2 session base key (MS-NLMP 3.4.5.1 / 3.1.5.1.2).
#[allow(clippy::too_many_arguments)]
pub fn protected_authenticate(
    user: &str,
    domain: &str,
    workstation: &str,
    password: &str,
    type1: &[u8],
    type2: &[u8],
    client_challenge: &[u8; 8],
    fallback_filetime: u64,
    tls_certificate_digest: &[u8],
    target_name: &str,
) -> Result<Vec<u8>, String> {
    if [user, domain, workstation]
        .iter()
        .any(|value| value.len() > 320 || value.chars().any(char::is_control))
        || type1.len() < 32
        || type1.len() > 1024
        || &type1[..8] != SIGNATURE
        || type1[8..12] != 1u32.to_le_bytes()
    {
        return Err("invalid NTLM negotiate message or identity".into());
    }
    let offered = u32::from_le_bytes(type1[12..16].try_into().unwrap());
    let challenge = parse_challenge(type2)?;
    let flags = offered & challenge.flags;
    let required = NTLMSSP_NEGOTIATE_UNICODE
        | NTLMSSP_NEGOTIATE_NTLM
        | NTLMSSP_NEGOTIATE_EXTENDED_SESSIONSECURITY;
    if flags & required != required || offered & 0x40000000 != 0 {
        return Err("NTLMv2 extended session security is required".into());
    }
    let (target_info, timestamp) = protected_target_info(
        &challenge.target_info,
        channel_binding_hash(tls_certificate_digest)?,
        target_name,
        fallback_filetime,
    )?;
    let mut response = ntlmv2_response(
        &ntowf_v2(user, domain, password),
        &challenge.server_challenge,
        client_challenge,
        timestamp,
        &target_info,
    );
    response.lm_challenge_response.clear();
    let original = type3_message(user, domain, workstation, &response);
    let mut message = original[..64].to_vec();
    message.extend_from_slice(&[0; 24]);
    message.extend_from_slice(&original[64..]);
    for offset in [16, 24, 32, 40, 48, 56] {
        let previous = u32::from_le_bytes(message[offset..offset + 4].try_into().unwrap());
        message[offset..offset + 4].copy_from_slice(&(previous + 24).to_le_bytes());
    }
    // VERSION is not offered by the minimal Type1, so its 8 bytes remain zero.
    message[60..64].copy_from_slice(&flags.to_le_bytes());
    let mut transcript = Vec::with_capacity(type1.len() + type2.len() + message.len());
    transcript.extend_from_slice(type1);
    transcript.extend_from_slice(type2);
    transcript.extend_from_slice(&message);
    message[72..88].copy_from_slice(&hmac_md5(&response.session_base_key, &transcript));
    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::md::hex;

    fn protected_challenge() -> Vec<u8> {
        let mut target = Vec::new();
        target.extend_from_slice(&7u16.to_le_bytes());
        target.extend_from_slice(&8u16.to_le_bytes());
        target.extend_from_slice(&12345678u64.to_le_bytes());
        target.extend_from_slice(&[0; 4]);
        let mut result = vec![0; 48];
        result[..8].copy_from_slice(SIGNATURE);
        result[8..12].copy_from_slice(&2u32.to_le_bytes());
        result[20..24].copy_from_slice(&TYPE1_FLAGS.to_le_bytes());
        result[24..32].copy_from_slice(&[0x42; 8]);
        result[40..42].copy_from_slice(&(target.len() as u16).to_le_bytes());
        result[42..44].copy_from_slice(&(target.len() as u16).to_le_bytes());
        result[44..48].copy_from_slice(&48u32.to_le_bytes());
        result.extend_from_slice(&target);
        result
    }
    #[test]
    fn extended_protection_contains_verified_tls_service_binding_timestamp_and_transcript_mic() {
        let type1 = type1_message();
        let type2 = protected_challenge();
        let certificate = (0..32).collect::<Vec<u8>>();
        assert_eq!(
            hex(&channel_binding_hash(&certificate).unwrap()),
            "8f1214c9c9cab8dc3bf866da9aba57a7"
        );
        let message = protected_authenticate(
            "User",
            "Domain",
            "WS",
            "Password",
            &type1,
            &type2,
            &[0x24; 8],
            999,
            &certificate,
            "HTTP/exchange.test",
        )
        .unwrap();
        assert_eq!(&message[64..72], &[0; 8]);
        assert_eq!(&message[12..16], &[0; 4], "LM response omitted");
        let offset = u32::from_le_bytes(message[24..28].try_into().unwrap()) as usize;
        let length = u16::from_le_bytes(message[20..22].try_into().unwrap()) as usize;
        let response = &message[offset..offset + length];
        let ntowf = ntowf_v2("User", "Domain", "Password");
        let mut proof_input = vec![0x42; 8];
        proof_input.extend_from_slice(&response[16..]);
        assert_eq!(&response[..16], &hmac_md5(&ntowf, &proof_input));
        assert_eq!(
            u64::from_le_bytes(response[24..32].try_into().unwrap()),
            12345678,
            "use server timestamp"
        );
        let mut av = std::collections::BTreeMap::new();
        let mut at = 44;
        while at + 4 <= response.len() {
            let id = u16::from_le_bytes(response[at..at + 2].try_into().unwrap());
            let size = u16::from_le_bytes(response[at + 2..at + 4].try_into().unwrap()) as usize;
            at += 4;
            if id == 0 {
                break;
            }
            av.insert(id, response[at..at + size].to_vec());
            at += size;
        }
        assert_eq!(
            av.get(&10),
            Some(&channel_binding_hash(&certificate).unwrap().to_vec())
        );
        assert_eq!(av.get(&9), Some(&utf16le("HTTP/exchange.test")));
        assert_eq!(av.get(&6), Some(&2u32.to_le_bytes().to_vec()));
        let key = hmac_md5(&ntowf, &response[..16]);
        let mut unsigned = message.clone();
        unsigned[72..88].fill(0);
        let mut transcript = type1.clone();
        transcript.extend_from_slice(&type2);
        transcript.extend_from_slice(&unsigned);
        assert_eq!(&message[72..88], &hmac_md5(&key, &transcript));
        transcript[16] ^= 1;
        assert_ne!(
            &message[72..88],
            &hmac_md5(&key, &transcript),
            "transcript tampering must be observable"
        );
        let changed = protected_authenticate(
            "User",
            "Domain",
            "WS",
            "Password",
            &type1,
            &type2,
            &[0x24; 8],
            999,
            &[0xff; 32],
            "HTTP/exchange.test",
        )
        .unwrap();
        assert_ne!(
            message, changed,
            "a different TLS peer changes the proof and MIC"
        );
    }
    #[test]
    fn extended_protection_refuses_missing_bindings_downgrades_and_ambiguous_target_information() {
        let type1 = type1_message();
        let mut type2 = protected_challenge();
        assert!(
            protected_authenticate(
                "u",
                "d",
                "",
                "p",
                &type1,
                &type2,
                &[1; 8],
                1,
                &[],
                "HTTP/exchange.test"
            )
            .is_err()
        );
        type2[20..24].copy_from_slice(&0u32.to_le_bytes());
        assert!(
            protected_authenticate(
                "u",
                "d",
                "",
                "p",
                &type1,
                &type2,
                &[1; 8],
                1,
                &[2; 32],
                "HTTP/exchange.test"
            )
            .is_err()
        );
        assert!(
            protected_target_info(
                &[6, 0, 4, 0, 0, 0, 0, 0, 6, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [0; 16],
                "HTTP/exchange.test",
                1
            )
            .is_err()
        );
        assert!(
            protected_target_info(&[6, 0, 4, 0, 0, 0], [0; 16], "HTTP/exchange.test", 1).is_err()
        );
        assert!(protected_target_info(&[0, 0, 0, 0], [0; 16], "HTTP/evil\r\nHost", 1).is_err());
    }

    #[test]
    fn ntowf_v2_ms_nlmp_4_2_4_vector() {
        // MS-NLMP §4.2.4.1.1: User="User", Domain="Domain", Password="Password".
        // Expected NTOWFv2 = 0c868a403bfd7a93a3001ef22ef02e3f.
        let n = ntowf_v2("User", "Domain", "Password");
        assert_eq!(hex(&n), "0c868a403bfd7a93a3001ef22ef02e3f");
    }

    #[test]
    fn type1_message_shape() {
        let t1 = type1_message();
        assert_eq!(&t1[0..8], SIGNATURE);
        assert_eq!(u32::from_le_bytes([t1[8], t1[9], t1[10], t1[11]]), 1);
        assert_eq!(t1.len(), 32);
    }

    #[test]
    fn parse_challenge_extracts_server_challenge_and_target_info() {
        // Hand-built minimal Type 2: header + server challenge 0x0123..ef + a 2-byte
        // target-info blob (EOL AV pair) placed at offset 48.
        let mut m = Vec::new();
        m.extend_from_slice(SIGNATURE);
        m.extend_from_slice(&2u32.to_le_bytes()); // type 2
        // TargetNameFields (8) — empty.
        m.extend_from_slice(&[0u8; 8]);
        m.extend_from_slice(&0u32.to_le_bytes()); // flags
        m.extend_from_slice(&[0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]); // server chal
        m.extend_from_slice(&[0u8; 8]); // reserved
        // TargetInfoFields: len=4, maxlen=4, offset=48.
        m.extend_from_slice(&4u16.to_le_bytes());
        m.extend_from_slice(&4u16.to_le_bytes());
        m.extend_from_slice(&48u32.to_le_bytes());
        assert_eq!(m.len(), 48);
        m.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // MsvAvEOL

        let c = parse_challenge(&m).unwrap();
        assert_eq!(
            c.server_challenge,
            [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]
        );
        assert_eq!(c.target_info, vec![0, 0, 0, 0]);
    }

    #[test]
    fn ntlmv2_response_and_type3_round_trip() {
        let ntowf2 = ntowf_v2("User", "Domain", "Password");
        let server_challenge = [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
        let client_challenge = [0xaa; 8];
        let target_info = [0u8, 0, 0, 0];
        let resp = ntlmv2_response(
            &ntowf2,
            &server_challenge,
            &client_challenge,
            0,
            &target_info,
        );

        // NTProofStr is the first 16 bytes; the blob follows.
        assert_eq!(
            resp.nt_challenge_response.len(),
            16 + 28 + target_info.len() + 4
        );
        // LMv2 is always 24 bytes (16-byte HMAC ++ 8-byte client challenge).
        assert_eq!(resp.lm_challenge_response.len(), 24);

        let t3 = type3_message("User", "Domain", "WS", &resp);
        assert_eq!(&t3[0..8], SIGNATURE);
        assert_eq!(u32::from_le_bytes([t3[8], t3[9], t3[10], t3[11]]), 3);
        // The NT response security buffer (offset 20) must point inside the payload.
        let nt_len = u16::from_le_bytes([t3[20], t3[21]]) as usize;
        let nt_off = u32::from_le_bytes([t3[24], t3[25], t3[26], t3[27]]) as usize;
        assert_eq!(nt_len, resp.nt_challenge_response.len());
        assert_eq!(
            &t3[nt_off..nt_off + nt_len],
            &resp.nt_challenge_response[..]
        );
    }

    #[test]
    fn filetime_conversion_anchors_at_epoch_delta() {
        assert_eq!(filetime_from_unix_millis(0), 116_444_736_000_000_000);
    }
}
