use serde_json::Value;
pub const DEFAULT:&str=r#"{"enabled":true,"convertImages":true,"maxBytes":67108864,"types":{},"imageFormats":["*"],"textExtensions":[],"tools":{}}"#;
pub fn valid(value:&Value)->bool{
 let Some(row)=value.as_object() else{return false;};
 if row.keys().any(|key|!matches!(key.as_str(),"enabled"|"convertImages"|"maxBytes"|"types"|"imageFormats"|"textExtensions"|"tools"))||!row.get("enabled").is_some_and(Value::is_boolean)||!row.get("convertImages").is_some_and(Value::is_boolean)||!row.get("maxBytes").and_then(Value::as_u64).is_some_and(|n|(1024..=268435456).contains(&n)){return false;}
 let Some(types)=row.get("types").and_then(Value::as_object) else{return false;};
 if types.iter().any(|(key,value)|!["image","pdf","text","html","code","audio","video","message","calendar"].contains(&key.as_str())||!value.is_boolean()){return false;}
 for key in ["imageFormats","textExtensions"]{
  let Some(items)=row.get(key).and_then(Value::as_array) else{return false;};
  if items.len()>128||items.iter().any(|item|!item.as_str().is_some_and(|s|s=="*"||(!s.is_empty()&&s.len()<=40&&(s.as_bytes()[0].is_ascii_lowercase()||s.as_bytes()[0].is_ascii_digit())&&s.bytes().all(|c|c.is_ascii_lowercase()||c.is_ascii_digit()||b"+_-".contains(&c))))){return false;}
 }
 let Ok(known)=serde_json::from_str::<Vec<Value>>(include_str!("../../../apps/web/src/viewers/previewTools.json")) else{return false;};
 let Some(tools)=row.get("tools").and_then(Value::as_object) else{return false;};
 tools.iter().all(|(key,value)|known.iter().any(|tool|tool["id"].as_str()==Some(key.as_str()))&&value.as_str().is_some_and(|value|["toolbar","menu","hidden"].contains(&value)))
}

#[cfg(test)]
mod tests {
 use super::*;use serde_json::json;
 #[test] fn bounded_known_preview_configuration(){
  let default:Value=serde_json::from_str(DEFAULT).unwrap();assert!(valid(&default));
  for (key,bad) in [("maxBytes",json!(0)),("maxBytes",json!(268435457)),("types",json!({"exe":true})),("tools",json!({"download":"execute"})),("tools",json!({"unknown":"menu"})),("imageFormats",json!(["../png"])),("convertImages",json!("yes"))]{let mut value=default.clone();value[key]=bad;assert!(!valid(&value),"{value}");}
  let mut value=default;value["tools"]=json!({"download":"menu","pdf.print":"hidden"});value["types"]=json!({"html":false});assert!(valid(&value));
 }
}
