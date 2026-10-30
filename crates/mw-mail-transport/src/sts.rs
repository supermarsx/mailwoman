use crate::{CachedSts, PolicyError, StsPolicy, StsPolicyMode, normalize_domain};

pub const MAX_POLICY_BYTES: usize = 65536;
pub const MAX_AGE: u64 = 31_557_600;

impl StsPolicy {
    pub fn parse(bytes: &[u8]) -> Result<Self, PolicyError> {
        if bytes.len() > MAX_POLICY_BYTES {
            return Err(PolicyError::Sts);
        }
        let text = std::str::from_utf8(bytes).map_err(|_| PolicyError::Sts)?;
        let (mut version, mut mode, mut age) = (None, None, None);
        let mut mx = Vec::new();
        for line in text.lines() {
            if line.is_empty() {
                return Err(PolicyError::Sts);
            }
            let (key, value) = line.split_once(':').ok_or(PolicyError::Sts)?;
            let value = value.trim_matches([' ', '\t']);
            if value.is_empty() || value.chars().any(char::is_control) {
                return Err(PolicyError::Sts);
            }
            match key {
                "version" => {
                    if version.replace(value).is_some() || value != "STSv1" {
                        return Err(PolicyError::Sts);
                    }
                }
                "mode" => {
                    if mode.is_some() {
                        return Err(PolicyError::Sts);
                    }
                    mode = Some(match value {
                        "enforce" => StsPolicyMode::Enforce,
                        "testing" => StsPolicyMode::Testing,
                        "none" => StsPolicyMode::None,
                        _ => return Err(PolicyError::Sts),
                    });
                }
                "max_age" => {
                    if age.is_some()
                        || value.len() > 10
                        || !value.bytes().all(|b| b.is_ascii_digit())
                    {
                        return Err(PolicyError::Sts);
                    }
                    let parsed = value.parse::<u64>().map_err(|_| PolicyError::Sts)?;
                    if parsed > MAX_AGE {
                        return Err(PolicyError::Sts);
                    }
                    age = Some(parsed);
                }
                "mx" => {
                    if mx.len() >= 256 {
                        return Err(PolicyError::Sts);
                    }
                    let wildcard = value.starts_with("*.");
                    let name = normalize_domain(value.strip_prefix("*.").unwrap_or(value))
                        .map_err(|_| PolicyError::Sts)?;
                    mx.push(if wildcard { format!("*.{name}") } else { name });
                }
                _ => {
                    if key.is_empty()
                        || key.len() > 32
                        || !key.as_bytes()[0].is_ascii_alphanumeric()
                        || !key
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
                    {
                        return Err(PolicyError::Sts);
                    }
                }
            }
        }
        if version.is_none() {
            return Err(PolicyError::Sts);
        }
        let mode = mode.ok_or(PolicyError::Sts)?;
        if mode != StsPolicyMode::None && mx.is_empty() {
            return Err(PolicyError::Sts);
        }
        Ok(Self {
            mode,
            mx,
            max_age: age.ok_or(PolicyError::Sts)?,
        })
    }
    pub fn matches(&self, host: &str) -> bool {
        let Ok(host) = normalize_domain(host) else {
            return false;
        };
        self.mx
            .iter()
            .any(|pattern| match pattern.strip_prefix("*.") {
                Some(suffix) => host
                    .split_once('.')
                    .is_some_and(|(label, tail)| !label.is_empty() && tail == suffix),
                None => host == *pattern,
            })
    }
}

impl CachedSts {
    pub fn valid_at(&self, now: u64) -> bool {
        self.fetched_at <= now
            && self.expires_at > now
            && self.policy.max_age <= MAX_AGE
            && self.expires_at == self.fetched_at.saturating_add(self.policy.max_age)
            && !self.id.is_empty()
            && self.id.len() <= 32
            && self.id.bytes().all(|b| b.is_ascii_alphanumeric())
            && self.policy.mx.len() <= 256
            && (self.policy.mode == StsPolicyMode::None || !self.policy.mx.is_empty())
            && self
                .policy
                .mx
                .iter()
                .all(|name| normalize_domain(name.strip_prefix("*.").unwrap_or(name)).is_ok())
    }
}

/// TXT character-strings have already been concatenated, without inserted space.
pub(crate) fn discovery_id(records: &[String]) -> Result<Option<String>, PolicyError> {
    let mut found = None;
    for record in records {
        if !record.starts_with("v=STSv1") {
            continue;
        }
        if found.is_some() {
            return Err(PolicyError::Sts);
        }
        let mut fields = record.split(';');
        if fields.next() != Some("v=STSv1") {
            return Err(PolicyError::Sts);
        }
        let mut id = None;
        for field in fields {
            let field = field.trim_matches([' ', '\t']);
            if field.is_empty() {
                continue;
            }
            let (name, value) = field.split_once('=').ok_or(PolicyError::Sts)?;
            if name == "id" {
                if id.is_some()
                    || value.is_empty()
                    || value.len() > 32
                    || !value.bytes().all(|b| b.is_ascii_alphanumeric())
                {
                    return Err(PolicyError::Sts);
                }
                id = Some(value.to_owned());
            } else if name.is_empty()
                || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
                || value.chars().any(char::is_control)
            {
                return Err(PolicyError::Sts);
            }
        }
        found = Some(id.ok_or(PolicyError::Sts)?);
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_policy_and_single_label_wildcard() {
        let p = StsPolicy::parse(
            b"version: STSv1\r\nmode: enforce\r\nmx: *.example.org\r\nmax_age: 86400\r\n",
        )
        .unwrap();
        assert!(p.matches("MAIL.EXAMPLE.ORG"));
        assert!(!p.matches("example.org"));
        assert!(!p.matches("deep.mail.example.org"));
        assert!(!p.matches("mail.example.org.evil"));
        for bad in [
            "version: STSv1\nmode: enforce\nmax_age: 20",
            "version: STSv1\nmode: none\nmax_age: 31557601",
            "version: STSv1\nmode: none\nmode: enforce\nmax_age: 1",
            "version: STSv1\nmode: none\nmax_age: +1",
            "version: STSv1\nmode: enforce\nmx: *evil.org\nmax_age: 1",
        ] {
            assert!(StsPolicy::parse(bad.as_bytes()).is_err(), "{bad}");
        }
    }
    #[test]
    fn discovery_requires_one_well_formed_record() {
        assert_eq!(
            discovery_id(&["unrelated".into(), "v=STSv1; id=abc123".into()]).unwrap(),
            Some("abc123".into())
        );
        assert!(discovery_id(&["v=STSv1; id=a".into(), "v=STSv1; id=b".into()]).is_err());
        assert!(discovery_id(&["v=STSv1; id=a; id=b".into()]).is_err());
    }
    #[test]
    fn persisted_policy_cannot_extend_expiry_or_bypass_structure() {
        let mut value = CachedSts {
            id: "one".into(),
            policy: StsPolicy::parse(
                b"version: STSv1\nmode: enforce\nmx: smtp.example.org\nmax_age: 60",
            )
            .unwrap(),
            fetched_at: 100,
            expires_at: 160,
        };
        assert!(value.valid_at(120));
        assert!(!value.valid_at(160));
        assert!(!value.valid_at(99));
        value.expires_at = 999;
        assert!(!value.valid_at(120));
    }
}
