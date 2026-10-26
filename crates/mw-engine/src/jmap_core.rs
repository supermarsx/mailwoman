//! RFC8620 request admission shared by the native HTTP adapter and engine.
use serde_json::{Value, json};
pub const MAX_SIZE_REQUEST: usize = 2 * 1024 * 1024;
pub const MAX_CALLS: usize = 64;
pub const MAX_OBJECTS: usize = 5000;
pub const CORE: &str = "urn:ietf:params:jmap:core";

pub fn problem(kind: &str, limit: Option<&str>) -> Value {
    let mut value = json!({"type":format!("urn:ietf:params:jmap:error:{kind}"),"status":400});
    if let Some(limit) = limit {
        value["limit"] = json!(limit);
    }
    value
}

/// HTTP callers require `using`; legacy in-process callers may omit it.
/// Unknown extension properties are deliberately ignored (RFC8620 section3.3).
pub fn validate(request: &Value, size: usize, require_using: bool) -> Result<(), Value> {
    if size > MAX_SIZE_REQUEST {
        return Err(problem("limit", Some("maxSizeRequest")));
    }
    let invalid = || problem("notRequest", None);
    let object = request.as_object().ok_or_else(invalid)?;
    let calls = object
        .get("methodCalls")
        .and_then(Value::as_array)
        .ok_or_else(invalid)?;
    if calls.len() > MAX_CALLS {
        return Err(problem("limit", Some("maxCallsInRequest")));
    }
    for call in calls {
        let call = call
            .as_array()
            .filter(|call| call.len() == 3)
            .ok_or_else(invalid)?;
        if !call[0].is_string() || !call[1].is_object() || !call[2].is_string() {
            return Err(invalid());
        }
    }
    if let Some(ids) = object.get("createdIds").filter(|ids| !ids.is_null())
        && !ids
            .as_object()
            .is_some_and(|ids| ids.values().all(Value::is_string))
    {
        return Err(invalid());
    }
    match object.get("using") {
        Some(using) => {
            let caps = using.as_array().ok_or_else(invalid)?;
            if !caps.iter().all(Value::is_string) {
                return Err(invalid());
            }
            let session = super::session_json("", "");
            for cap in caps {
                let cap = cap.as_str().expect("validated string");
                if session["capabilities"].get(cap).is_none()
                    && cap != crate::sieve_jmap::CAPABILITY
                {
                    return Err(problem("unknownCapability", None));
                }
            }
        }
        None if require_using => return Err(invalid()),
        None => {}
    }
    Ok(())
}

pub fn method_too_large(name: &str, args: &Value) -> bool {
    if name.ends_with("/get") {
        return args
            .get("ids")
            .and_then(Value::as_array)
            .is_some_and(|ids| ids.len() > MAX_OBJECTS);
    }
    if name.ends_with("/set") {
        let count = ["create", "update"]
            .into_iter()
            .map(|key| {
                args.get(key)
                    .and_then(Value::as_object)
                    .map_or(0, |items| items.len())
            })
            .sum::<usize>()
            + args
                .get("destroy")
                .and_then(Value::as_array)
                .map_or(0, |items| items.len());
        return count > MAX_OBJECTS;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn request_shape_capabilities_and_limits_are_checked_before_dispatch() {
        for invalid in [
            json!(null),
            json!({}),
            json!({"using":[],"methodCalls":[["Core/echo",[],"x"]]}),
            json!({"using":[],"methodCalls":[["Core/echo",{},42]]}),
        ] {
            assert_eq!(
                validate(&invalid, 0, true).unwrap_err()["type"],
                "urn:ietf:params:jmap:error:notRequest"
            );
        }
        let good =
            json!({"using":[CORE],"methodCalls":[["Core/echo",{},"x"]],"futureExtension":true});
        assert!(validate(&good, 0, true).is_ok());
        assert_eq!(
            validate(&good, MAX_SIZE_REQUEST + 1, true).unwrap_err()["limit"],
            "maxSizeRequest"
        );
        assert_eq!(
            validate(&json!({"using":["unknown"],"methodCalls":[]}), 0, true).unwrap_err()["type"],
            "urn:ietf:params:jmap:error:unknownCapability"
        );
        assert_eq!(validate(&json!({"using":[CORE],"methodCalls":vec![json!(["Core/echo",{},"x"]);MAX_CALLS+1]}),0,true).unwrap_err()["limit"],"maxCallsInRequest");
    }
}
