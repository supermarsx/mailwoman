//! PQC acceptance (plan §3 e1): the hybrid X25519 + ML-KEM-768 store-seal key-wrap
//! round-trips, and rejects tampering + the wrong recipient key.

use mw_crypto::{native, pqc};

#[test]
fn hybrid_wrap_unwrap_roundtrip() {
    let kp = pqc::generate_recipient();
    let seal_key = b"this-is-a-32-byte-store-seal-key"; // 32 bytes
    let wrapped = pqc::wrap(seal_key, &kp.public).expect("wrap");
    let unwrapped = pqc::unwrap(&wrapped, &kp.secret).expect("unwrap");
    assert_eq!(unwrapped, seal_key);
}

#[test]
fn tamper_is_rejected() {
    let kp = pqc::generate_recipient();
    let wrapped = pqc::wrap(b"0123456789abcdef0123456789abcdef", &kp.public).expect("wrap");
    let mut bad = wrapped.clone();
    *bad.last_mut().unwrap() ^= 0xff;
    assert!(pqc::unwrap(&bad, &kp.secret).is_err());
}

#[test]
fn wrong_recipient_is_rejected() {
    let kp = pqc::generate_recipient();
    let other = pqc::generate_recipient();
    let wrapped = pqc::wrap(b"0123456789abcdef0123456789abcdef", &kp.public).expect("wrap");
    assert!(pqc::unwrap(&wrapped, &other.secret).is_err());
}

#[test]
fn native_facade_store_key() {
    let kp = native::generate_store_recipient();
    let seal = b"seal-key-material-32-bytes-long!!";
    let wrapped = native::wrap_store_key(seal, &kp.public).expect("wrap");
    assert_eq!(
        native::unwrap_store_key(&wrapped, &kp.secret).unwrap(),
        seal
    );
}
