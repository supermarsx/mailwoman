//! DLP config load (SPEC §7.6, plan §3 e7). The `/api/security/dlp/config`
//! endpoint surfaces the active rules so the web client can name them pre-send:
//! the deployment's `MW_DLP_RULES` and the rules stored in the admin security
//! policy ([`active_rules`]). The engine reads the SAME two sources for
//! enforcement at the `submit_email` chokepoint and for `Dlp/getRules`; both
//! deserialize into the frozen [`DlpRule`](mw_engine::DlpRule) shape, so the rules
//! shown here are the rules enforced.
//!
//! `MW_DLP_RULES` is either **inline JSON** (a `[…]` array of `DlpRule`) or a
//! **path** to a JSON file containing that array. This module only LOADS + surfaces
//! rules; detector evaluation and the redacted audit trail are engine-side (§1.8).

use std::path::Path;

use mw_engine::DlpRule;

/// Load the active DLP rules from an `MW_DLP_RULES` value (inline JSON array or a
/// path to a JSON file). Returns an empty list for an empty/whitespace source.
pub fn load_rules(source: &str) -> anyhow::Result<Vec<DlpRule>> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let json = if trimmed.starts_with('[') {
        trimmed.to_string()
    } else if Path::new(trimmed).is_file() {
        std::fs::read_to_string(trimmed)?
    } else {
        anyhow::bail!(
            "MW_DLP_RULES is neither inline JSON (starting '[') nor an existing file path"
        );
    };
    let rules: Vec<DlpRule> = serde_json::from_str(&json)?;
    Ok(rules)
}

/// The rules in force, as the engine enforces them
/// (`mw_engine::security::dlp::load_rules`): the `MW_DLP_RULES` rules, then the
/// rules stored in the admin security policy whose `id` no environment rule uses.
/// Each entry is the rule's JSON with a `source` field, `"env"` or `"admin"`;
/// an `"env"` rule cannot be changed or removed from the admin panel.
///
/// # Errors
/// When `MW_DLP_RULES` is set and does not load.
pub async fn active_rules(
    store: &mw_store::Store,
    env_source: Option<&str>,
) -> anyhow::Result<Vec<serde_json::Value>> {
    let env = match env_source {
        Some(source) => load_rules(source)?,
        None => Vec::new(),
    };
    let stored = mw_engine::security::dlp::stored_rules(store).await;
    let tagged = |rule: &DlpRule, source: &str| {
        let mut v = serde_json::to_value(rule).unwrap_or_default();
        if let Some(o) = v.as_object_mut() {
            o.insert("source".into(), source.into());
        }
        v
    };
    let mut out: Vec<serde_json::Value> = env.iter().map(|r| tagged(r, "env")).collect();
    for rule in stored.iter().filter(|s| !env.iter().any(|e| e.id == s.id)) {
        out.push(tagged(rule, "admin"));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"[
      {
        "id": "r1",
        "name": "Block card numbers",
        "enabled": true,
        "priority": 10,
        "conditions": {
          "detectors": ["pan"],
          "customRegex": null,
          "dictionaries": [],
          "attachmentTypes": [],
          "maxAttachmentSize": null,
          "recipientDomains": [],
          "recipientDomainMode": null,
          "classification": null
        },
        "action": "block",
        "message": "Messages containing card numbers cannot be sent externally."
      }
    ]"#;

    #[test]
    fn loads_inline_json() {
        let rules = load_rules(SAMPLE).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "r1");
        assert_eq!(rules[0].action, "block");
        assert_eq!(rules[0].conditions.detectors, vec!["pan".to_string()]);
    }

    #[test]
    fn loads_from_file_path() {
        let dir = std::env::temp_dir().join(format!("mw-dlp-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("rules.json");
        std::fs::write(&path, SAMPLE).unwrap();
        let rules = load_rules(path.to_str().unwrap()).unwrap();
        assert_eq!(rules.len(), 1);
    }

    #[test]
    fn empty_source_is_no_rules() {
        assert!(load_rules("   ").unwrap().is_empty());
    }

    #[test]
    fn bad_source_errors() {
        assert!(load_rules("not json, not a path").is_err());
        assert!(load_rules("[ not valid json").is_err());
    }
}
