//! Internal cache-use intent; this never grants permission to download a blob.
use axum::http::HeaderMap;

pub(crate) fn parse(headers: &HeaderMap) -> Result<bool, ()> {
    let mut values = headers.get_all("x-mailwoman-body-use").iter();
    match (values.next(), values.next()) {
        (None, None) => Ok(true),
        (Some(value), None) => match value.to_str().map_err(|_| ())? {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(()),
        },
        _ => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn absent_or_explicit_manual_intent_preserves_renewal() {
        let mut headers = HeaderMap::new();
        assert_eq!(parse(&headers), Ok(true));
        headers.insert("x-mailwoman-body-use", HeaderValue::from_static("true"));
        assert_eq!(parse(&headers), Ok(true));
    }

    #[test]
    fn only_explicit_false_marks_background_fetches() {
        let mut headers = HeaderMap::new();
        headers.insert("x-mailwoman-body-use", HeaderValue::from_static("false"));
        assert_eq!(parse(&headers), Ok(false));
        for invalid in [
            "",
            "0",
            "1",
            "False",
            "TRUE",
            "false, true",
            " false",
            "false ",
        ] {
            headers.insert(
                "x-mailwoman-body-use",
                HeaderValue::from_str(invalid).unwrap(),
            );
            assert!(parse(&headers).is_err(), "accepted {invalid:?}");
        }
        headers.insert(
            "x-mailwoman-body-use",
            HeaderValue::from_bytes(&[0xff]).unwrap(),
        );
        assert!(parse(&headers).is_err());
    }

    #[test]
    fn duplicate_values_are_rejected_even_when_equal() {
        let mut headers = HeaderMap::new();
        headers.append("x-mailwoman-body-use", HeaderValue::from_static("false"));
        headers.append("x-mailwoman-body-use", HeaderValue::from_static("false"));
        assert!(parse(&headers).is_err());
    }
}
