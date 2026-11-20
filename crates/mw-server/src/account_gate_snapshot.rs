//! Optional fresh final-session gate. Unsupported/cold cases use the old reader.
use super::*;
use mw_store::final_gate_snapshot::{MAX_FINAL_GATE_KEYS, MAX_FINAL_GATE_PARAMETER_BYTES};

struct PreparedName {
    typed: String,
    identity: crate::domain_config::QualifiedIdentityInput,
}
struct Prepared {
    names: Vec<PreparedName>,
    domains: Vec<String>,
    ordered_subjects: Vec<String>,
    keys: Vec<String>,
}
impl Prepared {
    fn new(ids: &[&str], names: &[&str]) -> Option<Self> {
        if names.is_empty() || names.len() > MAX_FINAL_GATE_KEYS || ids.len() > MAX_FINAL_GATE_KEYS
        {
            return None;
        }
        // Eligibility must precede folding, URL parsing and extra key copies.
        let mut raw_bytes = 0usize;
        for value in ids.iter().chain(names) {
            if value.len() > 1024 {
                return None;
            }
            raw_bytes = raw_bytes.checked_add(value.len())?;
            if raw_bytes > MAX_FINAL_GATE_PARAMETER_BYTES {
                return None;
            }
        }
        let (extra, scans) = spellings(&subjects(names));
        if !scans.is_empty() {
            return None;
        }
        let all = ids.iter().chain(names).copied().collect::<Vec<_>>();
        let mut ordered_subjects = subjects(&all);
        for subject in extra {
            if !ordered_subjects.contains(&subject) {
                ordered_subjects.push(subject);
            }
        }
        // The existing reader normalizes again before exact lookup. Preserve
        // exotic non-idempotent normalization with its unchanged fallback.
        if ordered_subjects
            .iter()
            .any(|subject| flag_subject(subject) != *subject)
        {
            return None;
        }
        let keys = ordered_subjects
            .iter()
            .map(|subject| flags_key(subject))
            .collect::<Vec<_>>();
        if keys.len() > MAX_FINAL_GATE_KEYS {
            return None;
        }
        let mut domains = Vec::new();
        let mut prepared = Vec::new();
        for name in names {
            let typed = name.trim();
            if typed.len() > 1024 || typed.chars().any(char::is_control) {
                return None;
            }
            let (_, raw_domain) = typed.rsplit_once('@')?;
            if !raw_domain.is_ascii() {
                return None;
            }
            let identity = crate::domain_config::QualifiedIdentityInput::parse(typed).ok()??;
            if !domains.contains(&identity.domain) {
                domains.push(identity.domain.clone());
            }
            prepared.push(PreparedName {
                typed: typed.into(),
                identity,
            });
        }
        let bytes = serde_json::to_vec(&domains)
            .ok()?
            .len()
            .checked_add(serde_json::to_vec(&keys).ok()?.len())?;
        if bytes > MAX_FINAL_GATE_PARAMETER_BYTES {
            return None;
        }
        Some(Self {
            names: prepared,
            domains,
            ordered_subjects,
            keys,
        })
    }
}

pub(super) async fn try_gate(
    store: &Store,
    ids: &[&str],
    names: &[&str],
) -> Result<Option<AccountGate>, StoreError> {
    let Some(prepared) = Prepared::new(ids, names) else {
        return Ok(None);
    };
    let mut snapshot = store
        .final_account_gate_snapshot(&prepared.domains, &prepared.keys)
        .await?;
    let Some(def) = crate::settings_registry::lookup("signin.domainMode") else {
        return Ok(None);
    };
    if def.scopes != crate::settings_registry::Scopes::Global
        || !store.try_empty_global_resolution_ready("signin.domainMode", &snapshot.revision)?
    {
        return Ok(None);
    }
    let subject = mw_store::settings_policy::SettingSubject {
        personal_owner: None,
        domain: "",
        account_id: "",
        address: "",
    };
    let policy = std::mem::replace(&mut snapshot.global_policy, Ok(None))?;
    let listed = crate::settings_registry::resolve_loaded(
        def,
        "signin.domainMode",
        &subject,
        policy.as_ref(),
        &[],
        &crate::settings_registry::process_env,
    )
    .value
        == "listed";
    finish_gate(prepared, snapshot, listed)
}

fn finish_gate(
    prepared: Prepared,
    mut snapshot: mw_store::final_gate_snapshot::FinalGateSnapshot,
    listed: bool,
) -> Result<Option<AccountGate>, StoreError> {
    let mut gate = AccountGate::default();
    let mut addresses = Vec::new();
    // Decode/evaluate in original name order; never parse flags ahead of holds.
    let mut domains = std::collections::HashMap::new();
    for name in &prepared.names {
        if !domains.contains_key(&name.identity.domain) {
            let row = snapshot
                .domains
                .remove(&name.identity.domain)
                .ok_or_else(|| StoreError::Corrupt("final gate domain missing".into()))??;
            if row.second_alias
                || row.legacy_read_required
                || !crate::domain_config::normalize_domain(&row.canonical_domain)
                    .is_ok_and(|normalized| normalized == row.canonical_domain)
            {
                return Ok(None);
            }
            domains.insert(name.identity.domain.clone(), row);
        }
        let row = &domains[&name.identity.domain];
        addresses.push(flag_subject(&name.typed));
        addresses.push(
            name.identity
                .canonical(row.canonical_domain.clone())
                .address,
        );
        match row.enabled {
            Some(false) => gate.disabled = true,
            None if listed => gate.disabled = true,
            _ => {}
        }
    }
    // The old directory reader folds all supplied/canonical names before its
    // count check, then rejects 5001 even if an early row would match.
    let folded = addresses
        .iter()
        .map(|address| mw_store::user_lifecycle::final_gate_access_identity(address))
        .collect::<Result<std::collections::HashSet<_>, _>>()?;
    if snapshot.held_addresses.len() > 5000 {
        return Err(StoreError::Corrupt(
            "directory access holds exceed bounded lookup limit".into(),
        ));
    }
    for row in snapshot.held_addresses {
        if folded.contains(&mw_store::user_lifecycle::final_gate_access_identity(
            &row?,
        )?) {
            gate.disabled = true;
            break;
        }
    }
    for subject in prepared.ordered_subjects {
        let flags = match snapshot.exact_flags.remove(&flags_key(&subject)) {
            Some(raw) => parse_flags(&raw?)?,
            None => UserFeatureFlags::default(),
        };
        gate.disabled |= flags.disabled;
        gate.password_change_required |= flags.force_password_change;
    }
    Ok(Some(gate))
}

#[cfg(test)]
#[path = "account_gate_snapshot_tests.rs"]
mod tests;
