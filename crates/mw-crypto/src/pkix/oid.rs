//! Object identifiers used by the certificate, CMS and revocation code, by the
//! name their RFC gives them.

use const_oid::ObjectIdentifier as O;

const fn o(s: &str) -> O {
    O::new_unwrap(s)
}

// Digests.
pub const SHA1: O = o("1.3.14.3.2.26");
pub const SHA256: O = o("2.16.840.1.101.3.4.2.1");
pub const SHA384: O = o("2.16.840.1.101.3.4.2.2");
pub const SHA512: O = o("2.16.840.1.101.3.4.2.3");

// RSA (RFC 8017, RFC 4055).
pub const RSA_ENCRYPTION: O = o("1.2.840.113549.1.1.1");
pub const MD5_WITH_RSA: O = o("1.2.840.113549.1.1.4");
pub const SHA1_WITH_RSA: O = o("1.2.840.113549.1.1.5");
pub const RSAES_OAEP: O = o("1.2.840.113549.1.1.7");
pub const MGF1: O = o("1.2.840.113549.1.1.8");
pub const RSASSA_PSS: O = o("1.2.840.113549.1.1.10");
pub const SHA256_WITH_RSA: O = o("1.2.840.113549.1.1.11");
pub const SHA384_WITH_RSA: O = o("1.2.840.113549.1.1.12");
pub const SHA512_WITH_RSA: O = o("1.2.840.113549.1.1.13");

// Elliptic curves (RFC 5480, RFC 5758, RFC 8410).
pub const EC_PUBLIC_KEY: O = o("1.2.840.10045.2.1");
pub const SECP256R1: O = o("1.2.840.10045.3.1.7");
pub const SECP384R1: O = o("1.3.132.0.34");
pub const ECDSA_SHA256: O = o("1.2.840.10045.4.3.2");
pub const ECDSA_SHA384: O = o("1.2.840.10045.4.3.3");
pub const ECDSA_SHA512: O = o("1.2.840.10045.4.3.4");
pub const X25519: O = o("1.3.101.110");
pub const ED25519: O = o("1.3.101.112");

// ECDH key agreement for CMS (RFC 5753 §7.1.4, RFC 8418). The SHA-1 KDF is what
// OpenSSL sends by default; it is read, never sent.
pub const DH_SINGLE_PASS_STD_SHA1KDF: O = o("1.3.133.16.840.63.0.2");
pub const DH_SINGLE_PASS_STD_SHA256KDF: O = o("1.3.132.1.11.1");
pub const DH_SINGLE_PASS_STD_SHA384KDF: O = o("1.3.132.1.11.2");
pub const DH_SINGLE_PASS_STD_SHA512KDF: O = o("1.3.132.1.11.3");

// AES (RFC 3565, RFC 5084, RFC 3394).
pub const AES128_CBC: O = o("2.16.840.1.101.3.4.1.2");
pub const AES256_CBC: O = o("2.16.840.1.101.3.4.1.42");
pub const AES128_GCM: O = o("2.16.840.1.101.3.4.1.6");
pub const AES256_GCM: O = o("2.16.840.1.101.3.4.1.46");
pub const AES128_WRAP: O = o("2.16.840.1.101.3.4.1.5");
pub const AES256_WRAP: O = o("2.16.840.1.101.3.4.1.45");

// CMS content types and attributes (RFC 5652, RFC 5083, RFC 8551).
pub const ID_DATA: O = o("1.2.840.113549.1.7.1");
pub const ID_SIGNED_DATA: O = o("1.2.840.113549.1.7.2");
pub const ID_ENVELOPED_DATA: O = o("1.2.840.113549.1.7.3");
pub const ID_AUTH_ENVELOPED_DATA: O = o("1.2.840.113549.1.9.16.1.23");
pub const CONTENT_TYPE: O = o("1.2.840.113549.1.9.3");
pub const MESSAGE_DIGEST: O = o("1.2.840.113549.1.9.4");
pub const SIGNING_TIME: O = o("1.2.840.113549.1.9.5");
pub const SMIME_CAPABILITIES: O = o("1.2.840.113549.1.9.15");

// Certificate extensions and key purposes (RFC 5280).
pub const SUBJECT_KEY_ID: O = o("2.5.29.14");
pub const KEY_USAGE: O = o("2.5.29.15");
pub const SUBJECT_ALT_NAME: O = o("2.5.29.17");
pub const BASIC_CONSTRAINTS: O = o("2.5.29.19");
pub const CRL_DISTRIBUTION_POINTS: O = o("2.5.29.31");
pub const AUTHORITY_KEY_ID: O = o("2.5.29.35");
pub const EXT_KEY_USAGE: O = o("2.5.29.37");
pub const ANY_EXT_KEY_USAGE: O = o("2.5.29.37.0");
pub const AUTHORITY_INFO_ACCESS: O = o("1.3.6.1.5.5.7.1.1");
pub const KP_EMAIL_PROTECTION: O = o("1.3.6.1.5.5.7.3.4");
pub const KP_OCSP_SIGNING: O = o("1.3.6.1.5.5.7.3.9");
pub const AD_OCSP: O = o("1.3.6.1.5.5.7.48.1");
pub const EMAIL_ADDRESS: O = o("1.2.840.113549.1.9.1");
pub const COMMON_NAME: O = o("2.5.4.3");
