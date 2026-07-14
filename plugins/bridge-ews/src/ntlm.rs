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

use crate::md::{hmac_md5, md4, utf16le};

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
#[derive(Debug, Clone)]
pub struct Ntlmv2Response {
    pub nt_challenge_response: Vec<u8>,
    pub lm_challenge_response: Vec<u8>,
    pub session_base_key: [u8; 16],
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

/// Build the AUTHENTICATE (Type 3) message (MS-NLMP §2.2.1.3). No Version/MIC block
/// (extended-session-security auth without signing is what EWS-over-TLS needs).
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::md::hex;

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
