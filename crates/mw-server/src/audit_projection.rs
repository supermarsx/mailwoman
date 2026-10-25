//! Bounded actor resolution and fail-closed target-domain audit admission.
use crate::{
    admin_ctx::{AdminApiError, AdminCtx},
    capabilities::Capability as C,
};
use mw_admin::{ActorKind, AuditLogEntry};
use mw_store::{Store, audit_identity::AuditIdentity};
use serde_json::{Value, json};
use std::collections::HashMap;

pub(crate) struct Projection<'a> {
    store: &'a Store,
    ctx: &'a AdminCtx,
    cap: C,
    identities: HashMap<(String, String), Option<AuditIdentity>>,
    domains: HashMap<String, Option<String>>,
    remaining: usize,
}
impl<'a> Projection<'a> {
    pub(crate) fn new(store: &'a Store, ctx: &'a AdminCtx, cap: C) -> Self {
        Self {
            store,
            ctx,
            cap,
            identities: HashMap::new(),
            domains: HashMap::new(),
            remaining: 128,
        }
    }
    async fn identity(
        &mut self,
        kind: &str,
        id: &str,
    ) -> Result<Option<AuditIdentity>, AdminApiError> {
        let key = (kind.to_owned(), id.to_owned());
        if let Some(value) = self.identities.get(&key) {
            return Ok(value.clone());
        }
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        let value = self.store.audit_identity(kind, id).await?;
        self.identities.insert(key, value.clone());
        Ok(value)
    }
    async fn domain(&mut self, domain: &str) -> Option<String> {
        if let Some(value) = self.domains.get(domain) {
            return value.clone();
        }
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        let value = self.store.canonical_policy_domain(domain).await.ok();
        self.domains.insert(domain.into(), value.clone());
        value
    }
    async fn target_domain(
        &mut self,
        row: &AuditLogEntry,
    ) -> Result<Option<String>, AdminApiError> {
        let target = row.target.as_deref().unwrap_or_default();
        if let Some(domain) = target.strip_prefix("domain:") {
            return Ok(self.domain(domain).await);
        }
        // These legacy writers store an account id without a target-kind prefix.
        let account = target.strip_prefix("account:").or_else(|| {
            matches!(
                row.action.as_str(),
                "user-provisioned"
                    | "quota-changed"
                    | "sessions-revoked"
                    | "feature-flag-changed"
                    | "zero-access-toggled"
                    | "force-password-change"
                    | "remote-cache-wipe"
            )
            .then_some(target)
        });
        if let Some(account) = account {
            if let Some(identity) = self.identity("user", account).await?
                && let Some(address) = identity.address
            {
                return Ok(self
                    .domain(address.rsplit_once('@').map_or("", |(_, domain)| domain))
                    .await);
            }
            return Ok(None);
        }
        // Do not infer scope from the actor: global actions can be performed by
        // an actor who happens to belong to the caller's domain.
        Ok(None)
    }
    pub(crate) async fn decorate(
        &mut self,
        row: &AuditLogEntry,
        mut value: Value,
    ) -> Result<Option<Value>, AdminApiError> {
        if let Some(allowed) = self.ctx.domain_filter(self.cap).cloned() {
            let Some(domain) = self.target_domain(row).await? else {
                return Ok(None);
            };
            if !allowed.contains(&domain) {
                return Ok(None);
            }
        }
        let kind = match row.actor_kind {
            ActorKind::Admin => "admin",
            ActorKind::User => "user",
            _ => return Ok(Some(value)),
        };
        if let Some(identity) = self.identity(kind, &row.actor).await? {
            let mut can_name = self.ctx.has_unscoped(self.cap);
            let mut can_link = false;
            if kind == "admin" {
                can_link = self.ctx.has_unscoped(C::AdminsRead);
                can_name |= can_link;
            } else if let Some(address) = &identity.address
                && let Some(domain) = self
                    .domain(address.rsplit_once('@').map_or("", |(_, domain)| domain))
                    .await
            {
                can_name |= self.ctx.require_domain_for(self.cap, &domain).is_ok();
                can_link = self.ctx.require_domain_for(C::UsersRead, &domain).is_ok();
            }
            if can_name {
                value["actorLabel"] = json!(identity.label);
            }
            if can_name && can_link {
                value["actorLink"] =
                    json!({"section":if kind=="admin"{"admins"}else{"users"},"id":identity.id});
            }
        }
        Ok(Some(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin_ctx::DomainScope;
    fn row(target: &str) -> AuditLogEntry {
        AuditLogEntry {
            id: "1".into(),
            ts: "2026-10-25T00:00:00Z".into(),
            actor: "opaque".into(),
            actor_kind: ActorKind::User,
            action: "updated".into(),
            target: Some(target.into()),
            detail_json: "{}".into(),
            ip: None,
        }
    }
    #[tokio::test]
    async fn scoped_audit_uses_target_not_actor_and_names_only_permitted_identity() {
        let store = Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        store
            .register_successful_signin(mw_store::user_directory::SuccessfulSignin {
                account_id: "opaque",
                address: "alice@allowed.test",
                domain: "allowed.test",
                local_part: "alice",
                mode: "engine",
                now: 1,
            })
            .await
            .unwrap();
        let ctx = AdminCtx::with_grants(
            "reader",
            [
                (
                    C::AuditRead,
                    DomainScope::Domains(["allowed.test".into()].into()),
                ),
                (
                    C::UsersRead,
                    DomainScope::Domains(["allowed.test".into()].into()),
                ),
            ]
            .into(),
        );
        let mut projection = Projection::new(&store, &ctx, C::AuditRead);
        assert!(
            projection
                .decorate(&row("domain:foreign.test"), json!({}))
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            projection
                .decorate(&row("global:config"), json!({}))
                .await
                .unwrap()
                .is_none()
        );
        let value = projection
            .decorate(&row("account:opaque"), json!({}))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(value["actorLabel"], "alice@allowed.test");
        assert_eq!(value["actorLink"]["id"], "opaque");
        let mut foreign = row("domain:allowed.test");
        foreign.actor = "unknown".into();
        let value = projection
            .decorate(&foreign, json!({"actor":"unknown"}))
            .await
            .unwrap()
            .unwrap();
        assert!(value.get("actorLabel").is_none());
    }
}
