//! Validated operator templates for backend authentication names.
//! Formatting never changes the application's account or policy identity.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum UsernameFormat {
    #[default]
    FullEmail,
    LocalUser,
    DomainUser {
        domain: String,
    },
    UserRealm {
        realm: String,
    },
    Custom {
        template: String,
        #[serde(default)]
        realm: Option<String>,
    },
}

fn plain(value: &str, label: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 512
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(format!(
            "{label} must be nonempty, bounded text without control characters"
        ));
    }
    Ok(())
}

impl UsernameFormat {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::FullEmail | Self::LocalUser => Ok(()),
            Self::DomainUser { domain } => {
                plain(domain, "authentication domain")?;
                if domain.contains(['\\', '@', '{', '}']) {
                    return Err(
                        "authentication domain cannot contain separators or template tokens".into(),
                    );
                }
                Ok(())
            }
            Self::UserRealm { realm } => {
                plain(realm, "authentication realm")?;
                if realm.contains(['@', '\\', '{', '}']) {
                    return Err(
                        "authentication realm cannot contain separators or template tokens".into(),
                    );
                }
                Ok(())
            }
            Self::Custom { template, realm } => {
                plain(template, "username template")?;
                if let Some(realm) = realm {
                    plain(realm, "authentication realm")?;
                }
                let mut rest = template.as_str();
                let mut identity_token = false;
                while let Some(open) = rest.find('{') {
                    if rest[..open].contains('}') {
                        return Err("unmatched username template brace".into());
                    }
                    let tail = &rest[open + 1..];
                    let close = tail.find('}').ok_or("unmatched username template brace")?;
                    match &tail[..close] {
                        "user" | "email" => identity_token = true,
                        "domain" => {}
                        "realm" if realm.is_some() => {}
                        "realm" => {
                            return Err("the realm token needs an authentication realm".into());
                        }
                        _ => return Err(
                            "allowed username tokens are {user}, {domain}, {email}, and {realm}"
                                .into(),
                        ),
                    }
                    rest = &tail[close + 1..];
                }
                if rest.contains('}') || !identity_token {
                    return Err(
                        "username templates must contain {user} or {email} and balanced braces"
                            .into(),
                    );
                }
                Ok(())
            }
        }
    }

    pub fn render(&self, address: &str) -> Result<String, String> {
        self.validate()?;
        plain(address, "login identity")?;
        let (user, domain) = address
            .rsplit_once('@')
            .ok_or("username formatting needs a full email identity")?;
        if user.is_empty() || domain.is_empty() || user.contains('@') {
            return Err("username formatting needs a full email identity".into());
        }
        let rendered = match self {
            Self::FullEmail => address.to_string(),
            Self::LocalUser => user.to_string(),
            Self::DomainUser { domain } => format!("{domain}\\{user}"),
            Self::UserRealm { realm } => format!("{user}@{realm}"),
            Self::Custom { template, realm } => {
                // Scan the template once: never interpret braces in identity data.
                let mut result = String::new();
                let mut rest = template.as_str();
                while let Some(open) = rest.find('{') {
                    result.push_str(&rest[..open]);
                    let tail = &rest[open + 1..];
                    let close = tail.find('}').ok_or("unmatched username template brace")?;
                    result.push_str(match &tail[..close] {
                        "user" => user,
                        "domain" => domain,
                        "email" => address,
                        "realm" => realm.as_deref().ok_or("missing authentication realm")?,
                        _ => return Err("invalid username token".into()),
                    });
                    rest = &tail[close + 1..];
                }
                result.push_str(rest);
                result
            }
        };
        plain(&rendered, "formatted username")?;
        Ok(rendered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_preserve_identity_case_and_explicit_alias_domain() {
        let identity = "Alice@alias.example";
        for (format, expected) in [
            (UsernameFormat::FullEmail, identity),
            (UsernameFormat::LocalUser, "Alice"),
            (
                UsernameFormat::DomainUser {
                    domain: "CORP".into(),
                },
                "CORP\\Alice",
            ),
            (
                UsernameFormat::UserRealm {
                    realm: "EXAMPLE.REALM".into(),
                },
                "Alice@EXAMPLE.REALM",
            ),
            (
                UsernameFormat::Custom {
                    template: "{realm}\\{user}+{domain}".into(),
                    realm: Some("CORP".into()),
                },
                "CORP\\Alice+alias.example",
            ),
        ] {
            assert_eq!(format.render(identity).unwrap(), expected);
        }
    }

    #[test]
    fn refuses_invalid_templates_and_never_reinterprets_user_data() {
        for template in [
            "constant",
            "{password}",
            "{user",
            "{user}}",
            "{realm}\\{user}",
            "{user}\r\nAUTH",
        ] {
            assert!(
                UsernameFormat::Custom {
                    template: template.into(),
                    realm: None
                }
                .validate()
                .is_err()
            );
        }
        let formatter = UsernameFormat::Custom {
            template: "{user}@{domain}".into(),
            realm: None,
        };
        assert_eq!(
            formatter.render("{realm}@example.test").unwrap(),
            "{realm}@example.test"
        );
        assert!(formatter.render("bare-user").is_err());
        assert!(formatter.render("user\r\n@example.test").is_err());
    }
}
