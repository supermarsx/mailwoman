use super::*;
fn raw() -> Vec<u8> {
    b"/* public fixture */ const value='bounded';\n".repeat(512)
}
fn variants() -> Encodings {
    prepare(&raw(), true, 1024 * 1024, |_| None)
}
fn request(value: &str) -> HeaderMap {
    let mut h = HeaderMap::new();
    h.insert(
        header::ACCEPT_ENCODING,
        HeaderValue::from_str(value).unwrap(),
    );
    h
}
#[test]
fn negotiated_quality_wildcard_identity_and_malformed_values_are_exact() {
    let v = variants();
    assert!(v.gzip.is_some() && v.brotli.is_some());
    for (header, coding) in [
        ("gzip, br", Coding::Brotli),
        ("gzip;q=1, br;q=0.5, identity;q=0", Coding::Gzip),
        ("gzip;q=0, *;q=1, identity;q=0", Coding::Brotli),
        ("identity;q=1, gzip;q=0", Coding::Identity),
        ("GZIP", Coding::Gzip),
        ("x-gzip", Coding::Gzip),
        ("gzip;q=0.5, br;q=0.25", Coding::Gzip),
        ("gzip;q=0.5, identity;q=0.8", Coding::Identity),
        ("", Coding::Identity),
    ] {
        assert_eq!(negotiate(&request(header), &v), Ok(coding), "{header}");
    }
    assert_eq!(negotiate(&HeaderMap::new(), &v), Ok(Coding::Identity));
    for header in [
        "identity;q=0, gzip;q=0, br;q=0",
        "*;q=0",
        "gzip;q=1.1",
        "gzip;q=.5",
        "gzip;q=0.5;q=1",
        "gzip;q=1,gzip;q=0",
        "gzip;q=0.1234",
    ] {
        assert!(negotiate(&request(header), &v).is_err(), "{header}");
    }
    assert!(negotiate(&request("br,identity;q=0"), &Encodings::default()).is_err());
    let excessive = (0..17)
        .map(|n| format!("unknown{n}"))
        .collect::<Vec<_>>()
        .join(",");
    assert!(negotiate(&request(&excessive), &v).is_err());
}
#[test]
fn encoded_sidecars_require_actual_complete_raw_equivalence() {
    let bytes = raw();
    for coding in [Coding::Gzip, Coding::Brotli] {
        let encoded = generate(coding, &bytes, bytes.len()).unwrap();
        assert!(sidecar_matches(coding, &encoded, &bytes));
        let mut changed = bytes.clone();
        changed[0] ^= 1;
        assert!(!sidecar_matches(coding, &encoded, &changed));
        assert!(!sidecar_matches(
            coding,
            &encoded[..encoded.len() - 1],
            &bytes
        ));
        let mut extra = encoded.clone();
        extra.extend_from_slice(b"not-another-stream");
        assert!(!sidecar_matches(coding, &extra, &bytes));
    }
    let gzip = generate(Coding::Gzip, &bytes, bytes.len()).unwrap();
    let mut concatenated = gzip.clone();
    concatenated.extend_from_slice(&gzip);
    assert!(!sidecar_matches(Coding::Gzip, &concatenated, &bytes));
}
#[test]
fn verified_sidecars_share_exact_representation_bytes_but_corrupt_ones_regenerate() {
    let bytes = raw();
    let gzip = generate(Coding::Gzip, &bytes, bytes.len()).unwrap();
    let br = generate(Coding::Brotli, &bytes, bytes.len()).unwrap();
    let good = prepare(&bytes, true, 1024 * 1024, |coding| {
        Some(Cow::Owned(if coding == Coding::Gzip {
            gzip.clone()
        } else {
            br.clone()
        }))
    });
    assert!(!good.gzip.as_ref().unwrap().generated);
    assert!(!good.brotli.as_ref().unwrap().generated);
    assert_eq!(good.gzip.as_ref().unwrap().bytes.as_ref(), gzip);
    assert_ne!(
        good.gzip.as_ref().unwrap().etag,
        good.brotli.as_ref().unwrap().etag
    );
    let repaired = prepare(&bytes, true, 1024 * 1024, |_| Some(Cow::Owned(vec![0; 16])));
    assert!(
        repaired.gzip.as_ref().unwrap().generated && repaired.brotli.as_ref().unwrap().generated
    );
    assert!(sidecar_matches(
        Coding::Gzip,
        &repaired.gzip.unwrap().bytes,
        &bytes
    ));
}
#[test]
fn encoding_output_budget_and_brotli_switch_never_expand_retained_payload() {
    let bytes = raw();
    let none = prepare(&bytes, true, 0, |_| None);
    assert_eq!(none.charge(), 0);
    assert!(none.gzip.is_none() && none.brotli.is_none());
    let v = prepare(&bytes, false, 1024 * 1024, |_| None);
    assert!(v.gzip.is_some() && v.brotli.is_none());
    assert!(v.charge() <= bytes.len());
    assert!(eligible("assets/file.ftl") && eligible("pdfjs/lib.mjs") && eligible("data.json"));
    for path in [
        "index.html",
        "console.html",
        "font.woff2",
        "photo.png",
        "source.js.map",
        "archive.zip",
    ] {
        assert!(!eligible(path));
    }
}
#[tokio::test]
async fn response_etags_vary_and_304_belong_to_selected_representation() {
    let bytes = raw();
    let mut asset = crate::public_asset_cache::Prepared::new(Cow::Owned(bytes.clone()));
    asset.compressible = true;
    asset.encodings = prepare(&bytes, true, 1024 * 1024, |_| None);
    let h = request("gzip");
    let response =
        crate::asset_response_prepared("assets/x-ABC12345.js", "text/javascript", &asset, &h);
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()[header::CONTENT_ENCODING], "gzip");
    assert_eq!(response.headers()[header::VARY], "Accept-Encoding");
    let tag = response.headers()[header::ETAG].clone();
    let length = response.headers()[header::CONTENT_LENGTH].clone();
    let encoded = axum::body::to_bytes(response.into_body(), bytes.len())
        .await
        .unwrap();
    assert_eq!(length, encoded.len().to_string());
    assert!(sidecar_matches(Coding::Gzip, &encoded, &bytes));
    let mut conditional = h.clone();
    conditional.insert(header::IF_NONE_MATCH, tag.clone());
    let unchanged = crate::asset_response_prepared(
        "assets/x-ABC12345.js",
        "text/javascript",
        &asset,
        &conditional,
    );
    assert_eq!(unchanged.status(), 304);
    assert_eq!(unchanged.headers()[header::VARY], "Accept-Encoding");
    assert_eq!(unchanged.headers()[header::CONTENT_ENCODING], "gzip");
    assert!(
        axum::body::to_bytes(unchanged.into_body(), 1)
            .await
            .unwrap()
            .is_empty()
    );
    let identity = crate::asset_response_prepared(
        "assets/x-ABC12345.js",
        "text/javascript",
        &asset,
        &request("identity"),
    );
    assert_ne!(identity.headers()[header::ETAG], tag);
    assert!(!identity.headers().contains_key(header::CONTENT_ENCODING));
    let refused = crate::asset_response_prepared(
        "assets/x-ABC12345.js",
        "text/javascript",
        &asset,
        &request("br;q=0,gzip;q=0,identity;q=0"),
    );
    assert_eq!(refused.status(), 406);
    let ftl = crate::asset_response_prepared(
        "assets/locale-ABC12345.ftl",
        "application/octet-stream",
        &asset,
        &request("gzip"),
    );
    assert_eq!(
        ftl.headers()[header::CONTENT_TYPE],
        "text/plain; charset=utf-8"
    );
}
