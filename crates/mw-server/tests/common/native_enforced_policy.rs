//! Opt-in benchmark security profile. No limits, exemptions or defaults change.
use mw_store::{Store, settings_policy::SettingPolicyRow};
use serde_json::{Value, json};
use std::{net::{IpAddr, Ipv4Addr}, time::Duration};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile { Defaults, Enforced }
impl Profile {
    pub fn from_environment(value: Result<String, std::env::VarError>) -> Result<Self, &'static str> {
        match value {
            Ok(value) => Self::parse(Some(&value)),
            Err(std::env::VarError::NotPresent) => Ok(Self::Defaults),
            Err(std::env::VarError::NotUnicode(_)) => Err("non-Unicode native security profile"),
        }
    }
    pub fn parse(value: Option<&str>) -> Result<Self, &'static str> {
        match value { None | Some("defaults") => Ok(Self::Defaults), Some("enforced") => Ok(Self::Enforced), _ => Err("unknown native security profile") }
    }
}
pub async fn install(store: &Store) {
    for suffix in ["rateMode", "adaptiveMode", "probePathMode", "userAgentMode"] {
        let key = format!("security.protection.{suffix}");
        let definition = mw_server::settings_registry::lookup(&key).expect("registered policy");
        let value = mw_server::settings_registry::validate(definition, &json!("enforce"), "globalValue").expect("validated enforcement mode");
        store.put_setting_policy(&SettingPolicyRow {key, global_value:Some(value.to_string()),
            allow_user_override:false, global_locked:true, updated_by:"native-enforced-fixture".into(),
            updated_at:chrono::Utc::now().timestamp()}).await.unwrap();
    }
}
const BROWSER: &str = "Mozilla/5.0 Chrome/140.0 Safari/537.36";
async fn limited(response: reqwest::Response) {
    assert_eq!(response.status(),429);
    assert!(response.headers().contains_key("retry-after"));
    assert_eq!(response.json::<Value>().await.unwrap()["code"],"requestProtectionLimited");
}
async fn controls(store: &Store, client: &reqwest::Client, base: &str) -> Value {
    let policy = mw_server::fixture_profile::protection_policy_witness(store).await.expect("actual parsed protection policy");
    assert_eq!(policy["defaultsExceptModes"],true,"only four mode changes are permitted");
    assert_eq!(policy["noExemptions"],true,"enforced profile cannot use exemptions");
    let parsed = &policy["parsed"];
    for key in ["rateMode","adaptiveMode","probePathMode"] { assert_eq!(parsed[key],"enforce"); }
    assert_eq!(parsed["userAgent"]["mode"],"enforce");
    assert_eq!(parsed["storageFailure"],"closed");
    assert_eq!(parsed["clientRequests"],600);
    assert_eq!(parsed["accountRequests"],1800);
    assert_eq!(parsed["adminRequests"],1200);
    assert_eq!(parsed["rateWindowSecs"],60);
    assert_eq!(parsed["userAgent"]["scope"],"anonymous");
    assert_eq!(parsed["userAgent"]["blockBots"],true);
    // All negative sources are outside the measured 198.18.0.1..198.18.39.16 range.
    let health = client.get(format!("{base}/healthz")).header("x-forwarded-for","198.19.255.240").send().await.unwrap();
    assert_eq!(health.status(),404); let _=health.bytes().await.unwrap();
    let untrusted = reqwest::Client::builder().no_proxy().timeout(Duration::from_secs(5))
        .local_address(IpAddr::V4(Ipv4Addr::new(127,0,0,2))).build().unwrap();
    let health = untrusted.get(format!("{base}/healthz")).header("x-forwarded-for","198.19.255.240").send().await.unwrap();
    assert_eq!(health.status(),200,"untrusted peer cannot assert external XFF"); let _=health.bytes().await.unwrap();
    limited(client.get(format!("{base}/api/me")).header("x-forwarded-for","198.19.255.241").header("user-agent","curl/8.0").send().await.unwrap()).await;
    limited(client.get(format!("{base}/.git/config")).header("x-forwarded-for","198.19.255.242").header("user-agent",BROWSER).send().await.unwrap()).await;
    limited(client.get(format!("{base}/api/me")).header("x-forwarded-for","198.19.255.242").header("user-agent",BROWSER).send().await.unwrap()).await;
    // The normal source quota is exhausted without lowering it or adding exemptions.
    // Production uses UTC-aligned minute buckets. Start in the first quarter;
    // never retry a partially consumed quota or confuse a reset with admission.
    while chrono::Utc::now().timestamp().rem_euclid(60) > 15 {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let rate_bucket=chrono::Utc::now().timestamp().div_euclid(60);
    for _ in 0..600 {
        let response=client.get(format!("{base}/api/me")).header("x-forwarded-for","198.19.255.243").header("user-agent",BROWSER).send().await.unwrap();
        assert_eq!(response.status(),401); let _=response.bytes().await.unwrap();
    }
    limited(client.get(format!("{base}/api/me")).header("x-forwarded-for","198.19.255.243").header("user-agent",BROWSER).send().await.unwrap()).await;
    assert_eq!(rate_bucket,chrono::Utc::now().timestamp().div_euclid(60),"rate control crossed a bucket; evidence is unqualified, not retried");
    let after=mw_server::fixture_profile::protection_policy_witness(store).await.unwrap();
    assert_eq!(policy,after,"security preflight cannot mutate policy");
    json!({"kind":"native-enforced-policy-v1","policy":policy,"controls":{"trustedExternalHealth":404,"untrustedForwardedHealth":200,"automation":429,"scanner":429,"scannerFollowup":429,"rateAllowed":600,"rateRefused":429},"adaptiveControl":"separate-real-router-test-required","outsideMeasurement":true,"physicalSourceDiversityProven":false})
}
pub async fn preflight(store:&Store, client:&reqwest::Client, base:&str)->Value {
    tokio::time::timeout(Duration::from_secs(120),controls(store,client,base)).await.expect("bounded security preflight; no weakened acceptance on timeout")
}
#[test]
fn native_security_profile_names_are_explicit_and_strict() {
    assert_eq!(Profile::from_environment(Err(std::env::VarError::NotPresent)),Ok(Profile::Defaults));
    assert_eq!(Profile::from_environment(Ok("enforced".into())),Ok(Profile::Enforced));
    assert!(Profile::from_environment(Err(std::env::VarError::NotUnicode(std::ffi::OsString::from("invalid")))).is_err());
    assert_eq!(Profile::parse(None),Ok(Profile::Defaults));
    assert_eq!(Profile::parse(Some("defaults")),Ok(Profile::Defaults));
    assert_eq!(Profile::parse(Some("enforced")),Ok(Profile::Enforced));
    for value in ["", "observe", "off", "ENFORCED", "enforced "] { assert!(Profile::parse(Some(value)).is_err()); }
}
