//! Bounded package metadata. Provider payloads never become HTML or navigation URLs.
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate) struct AccountPackages {
    pub enabled: bool,
    pub packages: Vec<Package>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate) struct Package {
    pub id: String,
    pub label: String,
    pub tracking_number: String,
    pub carrier: String,
    pub provider_id: Option<String>,
    pub provider_revision: Option<String>,
    pub status: String,
    pub estimated_delivery: Option<String>,
    pub checkpoints: Vec<Checkpoint>,
    pub attempted_at: Option<i64>,
    pub updated_at: Option<i64>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate) struct Checkpoint {
    pub time: String,
    pub location: String,
    pub message: String,
}
pub(crate) fn valid_text(value:&str,max:usize)->bool {
    value.len()<=max && !value.chars().any(char::is_control)
}
pub(crate) fn valid_tracking(number:&str,carrier:&str,label:&str)->bool {
    (3..=100).contains(&number.len()) && number.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-')
        && (1..=80).contains(&carrier.len()) && carrier.bytes().all(|b|b.is_ascii_lowercase()||b.is_ascii_digit()||b==b'-')
        && valid_text(label,128)
}
fn text(value:&Value,key:&str,max:usize)->String {
    value.get(key).and_then(Value::as_str).unwrap_or("").chars().filter(|ch|!ch.is_control()).take(max).collect()
}
pub(crate) fn apply_provider(package:&mut Package,data:&Value,revision:&str,now:i64)->Result<(), &'static str> {
    // A key rotation or provider mix-up must never attach another shipment.
    if data["tracking_number"].as_str()!=Some(&package.tracking_number) || data["slug"].as_str()!=Some(&package.carrier) { return Err("providerMismatch"); }
    let id = data["id"].as_str().ok_or("providerResponse")?;
    if id.is_empty() || id.len()>128 || !id.bytes().all(|b|b.is_ascii_alphanumeric()||matches!(b,b'-'|b'_')) { return Err("providerResponse"); }
    let status = data["tag"].as_str().unwrap_or("Pending");
    if !["Pending","InfoReceived","InTransit","OutForDelivery","AttemptFail","Delivered","AvailableForPickup","Exception","Expired"].contains(&status) { return Err("providerResponse"); }
    package.provider_id=Some(id.into());
    package.provider_revision=Some(revision.into());
    package.status=status.into();
    package.estimated_delivery=data.get("courier_estimated_delivery_date").or_else(||data.get("aftership_estimated_delivery_date"))
        .map(|value|text(value,"estimated_delivery_date",40)).filter(|value|!value.is_empty());
    package.checkpoints=data["checkpoints"].as_array().map(|rows|rows.iter().rev().take(20).map(|row|Checkpoint {
        time:text(row,"checkpoint_time",40), location:text(row,"location",160),message:text(row,"message",512),
    }).collect()).unwrap_or_default();
    package.updated_at=Some(now);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tracking_values_cannot_be_urls_or_query_injection() {
        assert!(valid_tracking("RA123456789PT","portugal-ctt","Delivery"));
        for number in ["https://host/x","foo,bar","foo?x=y","abc\n123",""] { assert!(!valid_tracking(number,"ctt","")); }
        assert!(!valid_tracking("12345","ctt&slug=ups",""));
    }
    #[test]
    fn provider_response_is_identity_checked_and_bounded() {
        let mut package=Package{id:"local".into(),label:"".into(),tracking_number:"12345".into(),carrier:"ups".into(),provider_id:None,provider_revision:None,status:"Pending".into(),estimated_delivery:None,checkpoints:vec![],attempted_at:None,updated_at:None};
        let mut data=serde_json::json!({"id":"remote","tracking_number":"different","slug":"ups","tag":"Delivered"});
        assert_eq!(apply_provider(&mut package,&data,"rev",1),Err("providerMismatch"));
        assert!(package.provider_id.is_none());
        data["tracking_number"]=serde_json::json!("12345");
        data["checkpoints"]=serde_json::json!((0..100).map(|_|serde_json::json!({"message":"x".repeat(1000)})).collect::<Vec<_>>());
        apply_provider(&mut package,&data,"rev",1).unwrap();
        assert_eq!(package.checkpoints.len(),20);
        assert_eq!(package.checkpoints[0].message.len(),512);
        assert_eq!(package.status,"Delivered");
    }
}
