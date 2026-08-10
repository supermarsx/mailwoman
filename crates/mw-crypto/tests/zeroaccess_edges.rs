//! Rejection paths of the zero-access hierarchy (SPEC §9), reached through the
//! public API only.
//!
//! The crate's inline tests cover the happy hierarchy — derive, wrap, seal, pair,
//! recover. What they leave uncovered is what happens when the *inputs* are
//! wrong: a short salt, a truncated blob, a mistyped recovery word, a pairing
//! public point of the wrong length. Those are the paths a corrupted database row
//! or a hostile relay actually produces, and every one of them must be a typed
//! error rather than a panic or a silently different key.

use mw_crypto::zeroaccess as za;

fn root() -> [u8; za::KEY_LEN] {
    za::derive_root_key(
        b"correct horse battery staple",
        b"0123456789abcdef-salt",
        &za::ArgonParams {
            m_cost: 8,
            t_cost: 1,
            p_cost: 1,
        },
    )
    .expect("root key")
}

/// The salt floor is enforced, not assumed. A short salt is the shape a
/// truncated `zeroaccess_accounts` row would present.
#[test]
fn a_short_salt_is_refused() {
    let params = za::ArgonParams::default();
    for salt in [b"".as_slice(), b"short", &[0u8; 15]] {
        assert!(
            za::derive_root_key(b"secret", salt, &params).is_err(),
            "accepted a {}-byte salt",
            salt.len()
        );
    }
    // Exactly 16 is the boundary and must be accepted.
    assert!(za::derive_root_key(b"secret", &[0u8; 16], &params).is_ok());
}

/// Argon2 cost parameters that the KDF cannot honour are refused rather than
/// silently substituted.
#[test]
fn impossible_argon_parameters_are_refused() {
    let bad = za::ArgonParams {
        m_cost: 0,
        t_cost: 0,
        p_cost: 0,
    };
    assert!(za::derive_root_key(b"secret", &[0u8; 16], &bad).is_err());
}

/// The recorded interactive defaults are what `Default` gives — the params are
/// persisted next to the wrapped root key, so a divergence between the two would
/// make a stored account underivable.
#[test]
fn default_params_are_the_interactive_params() {
    assert_eq!(za::ArgonParams::default(), za::ArgonParams::interactive());
    let p = za::ArgonParams::interactive();
    assert_eq!((p.m_cost, p.t_cost, p.p_cost), (19_456, 2, 1));
}

/// Unwrapping something shorter than a nonce is a typed error, not an index
/// panic — this is the shape a truncated blob column has.
#[test]
fn truncated_blobs_are_refused_rather_than_panicking() {
    let kek = za::derive_kek(&root());
    let data_key = za::generate_data_key();
    let wrapped = za::wrap_key(&kek, &data_key).expect("wrap");

    for len in [0usize, 1, 12, 23] {
        assert!(
            za::unwrap_key(&kek, &wrapped[..len]).is_err(),
            "accepted a {len}-byte wrapped blob"
        );
    }
    // A blob that is long enough to have a nonce but carries no valid tag.
    assert!(za::unwrap_key(&kek, &[0u8; 40]).is_err());

    let aad = za::row_aad("notes", "row-1", 3);
    let sealed = za::seal_row(&data_key, b"plaintext", &aad).expect("seal");
    for len in [0usize, 23] {
        assert!(za::open_row(&data_key, &sealed[..len], &aad).is_err());
    }
    assert_eq!(
        za::open_row(&data_key, &sealed, &aad).unwrap(),
        b"plaintext"
    );
}

/// A wrapped data key cannot be replayed as a row ciphertext, or vice versa —
/// the two use different AAD by construction.
#[test]
fn a_wrapped_key_and_a_row_are_not_interchangeable() {
    let kek = za::derive_kek(&root());
    let data_key = za::generate_data_key();
    let wrapped = za::wrap_key(&kek, &data_key).expect("wrap");
    let aad = za::row_aad("notes", "row-1", 3);
    let sealed = za::seal_row(&kek, b"plaintext", &aad).expect("seal");

    assert!(
        za::open_row(&kek, &wrapped, &aad).is_err(),
        "wrap read as a row"
    );
    assert!(za::unwrap_key(&kek, &sealed).is_err(), "row read as a wrap");
}

/// Row AAD binds table, row id and schema version. Changing any one of the three
/// must make the row unopenable — that is the §9.3 tamper-detection property.
#[test]
fn every_row_aad_component_is_binding() {
    let data_key = za::generate_data_key();
    let sealed =
        za::seal_row(&data_key, b"secret note", &za::row_aad("notes", "r1", 3)).expect("seal");

    for wrong in [
        za::row_aad("notes2", "r1", 3),
        za::row_aad("notes", "r2", 3),
        za::row_aad("notes", "r1", 4),
    ] {
        assert!(
            za::open_row(&data_key, &sealed, &wrong).is_err(),
            "{wrong:?}"
        );
    }
    assert!(za::open_row(&data_key, &sealed, &za::row_aad("notes", "r1", 3)).is_ok());
}

/// The separator makes the AAD unambiguous: `("note", "sr1")` and
/// `("notes", "r1")` must not collide, or a row could be moved between tables.
#[test]
fn row_aad_components_cannot_be_confused_by_concatenation() {
    assert_ne!(za::row_aad("note", "sr1", 3), za::row_aad("notes", "r1", 3));
    let aad = za::row_aad("notes", "r1", 3);
    assert_eq!(aad.iter().filter(|&&b| b == 0x1F).count(), 2);
    assert!(aad.ends_with(b"3"));
}

/// Recovery-phrase rejection: wrong word count, an unknown word in the key part,
/// an unknown checksum word, and a valid-but-wrong checksum are each refused.
/// A phrase that decodes to a *different* key would be far worse than one that
/// fails to decode.
#[test]
fn recovery_phrase_rejects_every_kind_of_mistyping() {
    let r = root();
    let phrase = za::recovery_phrase(&r);
    let words: Vec<&str> = phrase.split_whitespace().collect();
    assert_eq!(words.len(), za::KEY_LEN + 1);
    assert_eq!(za::root_key_from_phrase(&phrase).unwrap(), r);

    // Too few / too many words.
    assert!(za::root_key_from_phrase("").is_err());
    assert!(za::root_key_from_phrase(&words[..words.len() - 1].join(" ")).is_err());
    assert!(za::root_key_from_phrase(&format!("{phrase} baba")).is_err());

    // An unknown word in the key part (right length, not in the syllable list).
    let mut bad = words.clone();
    bad[0] = "zzzz";
    assert!(za::root_key_from_phrase(&bad.join(" ")).is_err());

    // A word of the wrong length.
    let mut short = words.clone();
    short[5] = "ba";
    assert!(za::root_key_from_phrase(&short.join(" ")).is_err());

    // An unknown checksum word.
    let mut bad_sum = words.clone();
    *bad_sum.last_mut().unwrap() = "qqqq";
    assert!(za::root_key_from_phrase(&bad_sum.join(" ")).is_err());

    // A valid checksum word that is the wrong one for this key.
    let mut wrong_sum = words.clone();
    let last = *wrong_sum.last().unwrap();
    *wrong_sum.last_mut().unwrap() = if last == "baba" { "bade" } else { "baba" };
    assert!(za::root_key_from_phrase(&wrong_sum.join(" ")).is_err());

    // Extra whitespace is tolerated — a user retyping from paper.
    let spaced = words.join("  ");
    assert_eq!(
        za::root_key_from_phrase(&format!("  {spaced}\n")).unwrap(),
        r
    );
}

/// A single mistyped syllable is caught by the checksum rather than yielding a
/// silently wrong root key.
#[test]
fn a_one_word_typo_is_caught_by_the_checksum() {
    let r = root();
    let phrase = za::recovery_phrase(&r);
    let mut words: Vec<String> = phrase.split_whitespace().map(str::to_string).collect();
    let original = words[3].clone();
    words[3] = if original == "baba" { "bade" } else { "baba" }.to_string();
    match za::root_key_from_phrase(&words.join(" ")) {
        Err(_) => {}
        Ok(k) => panic!("a typo decoded to a key: {k:?} (was {original})"),
    }
}

/// Pairing rejects a public point that is the wrong length or not on the curve,
/// and an envelope that is too short — the three things a hostile relay can send.
#[test]
fn pairing_refuses_malformed_material() {
    let r = root();
    let new_device = za::pair_generate();
    assert_eq!(new_device.public.len(), 33, "compressed SEC1 point");

    // Wrong length is refused up front; a right-length blob that is not a
    // decodable point is refused by the SEC1 decode. (A right-length blob that
    // *is* a decodable point is accepted by design — sealing to the wrong peer
    // is the MITM case, and it is the SAS comparison below, not the decode, that
    // authenticates the channel.)
    for bad_public in [
        vec![],
        vec![0u8; 32],
        vec![0u8; 34],
        vec![0u8; 33],  // right length, SEC1 identity tag
        vec![0x04; 33], // right length, uncompressed tag with too few bytes
    ] {
        assert!(
            za::pair_seal(&r, &bad_public).is_err(),
            "sealed to a {}-byte public {:#04x}…",
            bad_public.len(),
            bad_public.first().copied().unwrap_or(0)
        );
    }

    let sealed = za::pair_seal(&r, &new_device.public).expect("seal");
    for len in [0usize, 33, 40, 56] {
        assert!(
            za::pair_open(
                &sealed.envelope[..len.min(sealed.envelope.len())],
                &new_device.secret
            )
            .is_err(),
            "opened a {len}-byte envelope"
        );
    }
    // A well-formed envelope with a garbage secret, and with a secret of the
    // wrong length.
    assert!(za::pair_open(&sealed.envelope, &[]).is_err());
    assert!(za::pair_open(&sealed.envelope, &[0u8; 31]).is_err());

    // An envelope whose embedded public point is corrupted.
    let mut corrupt = sealed.envelope.clone();
    corrupt[1] ^= 0xff;
    assert!(za::pair_open(&corrupt, &new_device.secret).is_err());

    // The control case still works and both sides agree on the SAS.
    let opened = za::pair_open(&sealed.envelope, &new_device.secret).expect("open");
    assert_eq!(opened.root_key, r);
    assert_eq!(opened.sas_words, sealed.sas_words);
    assert_eq!(opened.sas_words.len(), 6);
}

/// Tampering with the sealed payload fails the AEAD; it does not yield a
/// different root key.
#[test]
fn pairing_envelope_tampering_is_rejected() {
    let r = root();
    let device = za::pair_generate();
    let sealed = za::pair_seal(&r, &device.public).expect("seal");

    for offset in [33, 40, sealed.envelope.len() - 1] {
        let mut bad = sealed.envelope.clone();
        bad[offset] ^= 0x01;
        assert!(
            za::pair_open(&bad, &device.secret).is_err(),
            "tamper at {offset} was accepted"
        );
    }
}

/// Two pairings of the same root key produce different envelopes and different
/// SAS words — the ephemeral key is genuinely per-ceremony, so a replayed
/// envelope is visible to the user comparing words.
#[test]
fn each_pairing_ceremony_is_fresh() {
    let r = root();
    let device = za::pair_generate();
    let a = za::pair_seal(&r, &device.public).expect("seal");
    let b = za::pair_seal(&r, &device.public).expect("seal");
    assert_ne!(a.envelope, b.envelope);
    assert_ne!(a.sas_words, b.sas_words);
}

/// Subkey derivation is domain-separated: different labels give different keys,
/// and the KEK is not the root key.
#[test]
fn subkey_labels_are_domain_separated() {
    let r = root();
    let kek = za::derive_kek(&r);
    assert_ne!(kek, r);

    let classes = ["message-cache", "search", "notes", "attachment"];
    let keys: Vec<_> = classes.iter().map(|l| za::derive_subkey(&kek, l)).collect();
    for (i, a) in keys.iter().enumerate() {
        assert_ne!(*a, kek, "{} equals its parent", classes[i]);
        for (j, b) in keys.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "{} and {} collide", classes[i], classes[j]);
            }
        }
    }
    // Empty and near-miss labels are still distinct.
    assert_ne!(
        za::derive_subkey(&kek, ""),
        za::derive_subkey(&kek, "notes")
    );
    assert_ne!(
        za::derive_subkey(&kek, "notes"),
        za::derive_subkey(&kek, "note")
    );
}

/// Generated data keys are random — a constant would make every account's rows
/// readable with one key.
#[test]
fn generated_data_keys_are_distinct() {
    let a = za::generate_data_key();
    let b = za::generate_data_key();
    assert_ne!(a, b);
    assert_ne!(a, [0u8; za::KEY_LEN]);
}
