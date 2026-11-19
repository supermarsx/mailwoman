use serde_json::Value;

pub const DEFAULT: &str = r#"[
{
"id": "later",
"kind": "hours",
"amount": 3,
"enabled": true
},
{
"id": "tomorrow",
"kind": "tomorrow",
"time": "09:00",
"enabled": true
},
{
"id": "next-week",
"kind": "next-week",
"time": "09:00",
"enabled": true
},
{
"id": "2-hours",
"kind": "hours",
"amount": 2,
"enabled": false
},
{
"id": "6-hours",
"kind": "hours",
"amount": 6,
"enabled": false
},
{
"id": "24-hours",
"kind": "hours",
"amount": 24,
"enabled": false
},
{
"id": "3-days",
"kind": "days",
"amount": 3,
"time": "09:00",
"enabled": false
},
{
"id": "7-days",
"kind": "days",
"amount": 7,
"time": "09:00",
"enabled": false
},
{
"id": "end-week",
"kind": "end-week",
"time": "17:00",
"enabled": false
},
{
"id": "end-month",
"kind": "end-month",
"time": "17:00",
"enabled": false
},
{
"id": "next-month",
"kind": "next-month",
"time": "09:00",
"enabled": false
},
{
"id": "next-monday",
"kind": "weekday",
"weekday": 1,
"time": "09:00",
"enabled": false
},
{
"id": "second-tuesday",
"kind": "nth-weekday",
"weekday": 2,
"ordinal": 2,
"time": "09:00",
"enabled": false
},
{
"id": "next-first",
"kind": "day-of-month",
"day": 1,
"time": "09:00",
"enabled": false
},
{
"id": "next-year",
"kind": "next-year",
"time": "09:00",
"enabled": false
}
]"#;

pub fn valid(value: &Value) -> bool {
    let Some(items) = value.as_array() else {
        return false;
    };
    if items.len() > 32 || value.to_string().len() > 16384 {
        return false;
    }
    let mut ids = std::collections::HashSet::new();
    items.iter().all(|item| {
        let Some(row) = item.as_object() else { return false; };
        if row.keys().any(|key| !matches!(key.as_str(), "id" | "kind" | "amount" | "time" | "label" | "enabled" | "weekday" | "ordinal" | "day")) { return false; }
        let Some(id) = row.get("id").and_then(Value::as_str) else { return false; };
        if id.is_empty() || id.len() > 64 || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b)) || !ids.insert(id) || !row.get("enabled").is_some_and(Value::is_boolean) { return false; }
        if let Some(label) = row.get("label") {
            let Some(label) = label.as_str() else { return false; };
            if label.trim().is_empty() || label.chars().count() > 80 || label.chars().any(|c| c.is_control() || matches!(c as u32, 0x200b..=0x200f | 0x202a..=0x202e | 0x2060..=0x206f | 0xfeff)) { return false; }
        }
        let Some(kind) = row.get("kind").and_then(Value::as_str) else { return false; };
        match kind {
            "hours" | "days" => {
                let max = if kind == "hours" { 87600 } else { 3650 };
                if !row.get("amount").and_then(Value::as_i64).is_some_and(|n| (1..=max).contains(&n)) { return false; }
            }
            "tomorrow" | "next-week" | "end-week" | "end-month" | "next-month" | "weekday" | "nth-weekday" | "day-of-month" | "next-year" => { if row.contains_key("amount") { return false; } }
            _ => return false,
        }

        for (key,needed,min,max) in [("weekday",matches!(kind,"weekday"|"nth-weekday"),0,6),("day",kind=="day-of-month",1,31)] {
            if needed { if !row.get(key).and_then(Value::as_i64).is_some_and(|n|(min..=max).contains(&n)){return false;} }else if row.contains_key(key){return false;}
        }
        if kind=="nth-weekday" {if !row.get("ordinal").and_then(Value::as_i64).is_some_and(|n|n==-1||(1..=5).contains(&n)){return false;}}else if row.contains_key("ordinal"){return false;}
        if kind == "hours" { return !row.contains_key("time"); }
        let Some(time) = row.get("time").and_then(Value::as_str) else { return false; };
        let b = time.as_bytes();
        b.len() == 5 && b[2] == b':' && [b[0], b[1], b[3], b[4]].iter().all(u8::is_ascii_digit) && &time[..2] < "24" && &time[3..] < "60"
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn snooze_presets_validate_defaults_and_reject_ambiguous_or_unbounded_rules() {
        let defaults:Value=serde_json::from_str(DEFAULT).unwrap();
        assert!(valid(&defaults));
        assert_eq!(defaults.as_array().unwrap().iter().filter(|p|p["enabled"]==true).count(),3);
        assert!(valid(&json!([])));
        for bad in [
            json!([{"id":"a","kind":"weekday","weekday":7,"time":"09:00","enabled":true}]),
            json!([{"id":"a","kind":"nth-weekday","weekday":1,"ordinal":0,"time":"09:00","enabled":true}]),
            json!([{"id":"a","kind":"day-of-month","day":32,"time":"09:00","enabled":true}]),
            json!([{"id":"a","kind":"hours","amount":0,"enabled":true}]),
            json!([{"id":"a","kind":"hours","amount":2.5,"enabled":true}]),
            json!([{"id":"a","kind":"end-month","time":"24:00","enabled":true}]),
            json!([{"id":"a","kind":"days","amount":4,"time":"12:60","enabled":true}]),
            json!([{"id":"a","kind":"hours","amount":2,"enabled":true,"label":"\n"}]),
            json!([{"id":"a","kind":"hours","amount":2,"enabled":true},{"id":"a","kind":"hours","amount":3,"enabled":true}]),
            json!([{"id":"a","kind":"hours","amount":2,"enabled":true,"script":"alert(1)"}]),
        ] {
            assert!(!valid(&bad), "{bad}");
        }
    }
}
