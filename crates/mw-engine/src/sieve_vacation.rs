//! VacationResponse's generated blocks in the active script.
use super::*;

const META_BEGIN: &str = "# mailwoman:vacation metadata begin\n";
const META_END: &str = "# mailwoman:vacation metadata end\n";
const BODY_BEGIN: &str = "\n# mailwoman:vacation action begin\n";
const BODY_END: &str = "# mailwoman:vacation action end\n";

fn region<'a>(source: &'a str, begin: &str, end: &str) -> Result<Option<&'a str>, Value> {
    let Some(start) = source.find(begin) else {
        return if source.contains(end) {
            Err(err("invalidSieve"))
        } else {
            Ok(None)
        };
    };
    if start > 0 && !begin.starts_with('\n') && !source[..start].ends_with('\n') {
        return Err(err("invalidSieve"));
    }
    let rest = &source[start + begin.len()..];
    let Some(stop) = rest.find(end) else {
        return Err(err("invalidSieve"));
    };
    let stop = start + begin.len() + stop + end.len();
    if rest.contains(begin) || source[stop..].contains(end) {
        return Err(err("invalidSieve"));
    }
    Ok(Some(&source[start..stop]))
}
pub(super) fn protected(source: &str) -> Result<(Option<&str>, Option<&str>), Value> {
    Ok((
        region(source, META_BEGIN, META_END)?,
        region(source, BODY_BEGIN, BODY_END)?,
    ))
}
fn default_object() -> Value {
    json!({"id":"singleton","isEnabled":false,"fromDate":null,"toDate":null,"subject":null,"textBody":null,"htmlBody":null})
}
fn object(source: &str) -> Result<Value, Value> {
    let (meta, body) = protected(source)?;
    match (meta, body) {
        (None, None) => Ok(default_object()),
        (Some(meta), Some(_)) => {
            let line = meta
                .lines()
                .nth(1)
                .and_then(|l| l.strip_prefix("# "))
                .ok_or_else(|| err("invalidSieve"))?;
            serde_json::from_str(line).map_err(|_| err("invalidSieve"))
        }
        _ => Err(err("invalidSieve")),
    }
}
fn quote(value: &str) -> String {
    // Quoted Sieve strings support literal newlines, but backslash and quotes
    // must be escaped. JSON's \n escapes are not Sieve string escapes.
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
fn merge(source: &str, obj: &Value) -> Result<String, Value> {
    let (meta, body) = protected(source)?;
    let mut manual = source.to_string();
    for block in [meta, body].into_iter().flatten() {
        manual = manual.replacen(block, "", 1);
    }
    if obj["isEnabled"] != true {
        return Ok(format!(
            "{META_BEGIN}# {obj}\n{META_END}{manual}{BODY_BEGIN}{BODY_END}"
        ));
    }
    let mut required = vec!["vacation"];
    let mut tests = Vec::new();
    for (key, comparator) in [("fromDate", "ge"), ("toDate", "lt")] {
        if let Some(value) = obj[key].as_str() {
            let parsed = chrono::DateTime::parse_from_rfc3339(value)
                .map_err(|_| err("invalidProperties"))?;
            let utc = parsed
                .with_timezone(&chrono::Utc)
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            tests.push(format!(
                "currentdate :zone \"+0000\" :value \"{comparator}\" \"iso8601\" {}",
                quote(&utc)
            ));
        }
    }
    if !tests.is_empty() {
        required.extend(["date", "relational"]);
    }
    let mut action = "vacation :days 7".to_string();
    if let Some(subject) = obj["subject"].as_str() {
        action.push_str(&format!(" :subject {}", quote(subject)));
    }
    // Prefer text. HTML-only responses are a MIME text/html entity.
    let reason = if let Some(text) = obj["textBody"].as_str() {
        text.to_string()
    } else if let Some(html) = obj["htmlBody"].as_str() {
        action.push_str(" :mime");
        format!(
            "Content-Type: text/html; charset=utf-8\r\nContent-Transfer-Encoding: 8bit\r\n\r\n{html}"
        )
    } else {
        return Err(err("invalidProperties"));
    };
    action.push_str(&format!(" {};\n", quote(&reason)));
    if !tests.is_empty() {
        action = format!("if allof ({}) {{\n{action}}}\n", tests.join(", "));
    }
    Ok(format!(
        "{META_BEGIN}# {obj}\nrequire [{}];\n{META_END}{manual}{BODY_BEGIN}{action}{BODY_END}",
        required
            .iter()
            .map(|v| quote(v))
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

impl Engine {
    pub(super) async fn managed_vacation(
        &self,
        account: &str,
        method: &str,
        args: &Value,
        rows: Vec<SieveScriptRow>,
        ctx: &mut SieveContext,
    ) -> Value {
        let source = rows
            .iter()
            .find(|s| s.active)
            .map_or("", |s| s.content.as_str());
        let mut obj = match object(source) {
            Ok(obj) => obj,
            Err(error) => return error,
        };
        if method == "VacationResponse/get" {
            let wanted = args["ids"].as_array();
            let list = if wanted.is_none_or(|ids| ids.iter().any(|id| id == "singleton")) {
                vec![obj]
            } else {
                vec![]
            };
            let not_found: Vec<_> = wanted
                .into_iter()
                .flatten()
                .filter(|id| *id != "singleton")
                .cloned()
                .collect();
            return json!({"accountId":account,"state":state(&rows),"list":list,"notFound":not_found});
        }
        let mut not_created = Map::new();
        let mut not_destroyed = Map::new();
        let mut not_updated = Map::new();
        let mut updated = Map::new();
        for key in args["create"]
            .as_object()
            .into_iter()
            .flat_map(|m| m.keys())
        {
            not_created.insert(key.clone(), err("singleton"));
        }
        for key in args["destroy"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            not_destroyed.insert(key.into(), err("singleton"));
        }
        for (id, patch) in args["update"].as_object().into_iter().flatten() {
            if id != "singleton" {
                not_updated.insert(id.clone(), err("notFound"));
                continue;
            }
            let Some(patch) = patch.as_object() else {
                not_updated.insert(id.clone(), err("invalidProperties"));
                continue;
            };
            let invalid = patch.iter().any(|(k, v)| match k.as_str() {
                "isEnabled" => !v.is_boolean(),
                "fromDate" | "toDate" => {
                    !(v.is_null()
                        || v.as_str()
                            .is_some_and(|s| chrono::DateTime::parse_from_rfc3339(s).is_ok()))
                }
                "subject" => {
                    !(v.is_null() || v.as_str().is_some_and(|s| !s.chars().any(char::is_control)))
                }
                "textBody" | "htmlBody" => !(v.is_null() || v.is_string()),
                _ => true,
            });
            if invalid {
                not_updated.insert(id.clone(), err("invalidProperties"));
                continue;
            }
            for (key, value) in patch {
                obj[key] = value.clone();
            }
            let merged = match merge(source, &obj) {
                Ok(source) => source,
                Err(error) => {
                    not_updated.insert(id.clone(), error);
                    continue;
                }
            };
            match self
                .persist_managed_source(account, rows.clone(), &merged, ctx)
                .await
            {
                Ok(()) => {
                    updated.insert(id.clone(), Value::Null);
                }
                Err(error) => {
                    not_updated.insert(id.clone(), error);
                }
            }
        }
        json!({"accountId":account,"updated":updated,"notUpdated":not_updated,"created":{},"notCreated":not_created,"destroyed":[],"notDestroyed":not_destroyed})
    }
}
