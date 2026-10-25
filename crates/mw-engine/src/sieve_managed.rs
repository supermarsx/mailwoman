//! Server-generated builder edits share the authoritative script transaction.
use super::*;

impl Engine {
    pub(crate) async fn handle_managed_sieve(
        &self,
        account: &str,
        method: &str,
        args: &Value,
        ctx: &mut SieveContext,
    ) -> Value {
        if args.get("accountId").and_then(Value::as_str) != Some(account) {
            return err("accountNotFound");
        }
        if ctx.unavailable {
            return err("serverUnavailable");
        }
        let lock = account_lock(account);
        let _guard = lock.lock().await;
        let rows = match self.sieve_snapshot(account, ctx).await {
            Ok(rows) => rows,
            Err(error) => {
                ctx.unavailable = true;
                return error;
            }
        };
        let old_state = state(&rows);
        if method.ends_with("/set")
            && args
                .get("ifInState")
                .filter(|v| !v.is_null())
                .is_some_and(|v| v.as_str() != Some(&old_state))
        {
            return err("stateMismatch");
        }
        let mut result = match method {
            "MailRule/get" => self.mail_rule_get(account, args).await,
            "MailRule/set" => {
                self.mail_rule_set_managed(account, None, args, Some(ctx))
                    .await
            }
            "VacationResponse/get" | "VacationResponse/set" => {
                self.managed_vacation(account, method, args, rows, ctx)
                    .await
            }
            _ => return err("unknownMethod"),
        };
        if result.get("type").is_none() {
            if method == "MailRule/get"
                && let Some(list) = result["list"].as_array_mut()
            {
                for rule in list {
                    rule["runsAt"] = if ctx.client.is_some() {
                        "server-sieve"
                    } else {
                        "engine"
                    }
                    .into();
                }
            }
            if method.ends_with("/get") {
                result["state"] = old_state.into();
            } else {
                result["oldState"] = old_state.into();
                match self.store().list_sieve_scripts(account).await {
                    Ok(rows) => result["newState"] = state(&rows).into(),
                    Err(_) => return err("serverFail"),
                }
            }
        }
        result
    }

    /// Caller holds the account lock. Only this server-generated merge receives
    /// Builder privilege; client-supplied markers never bypass raw-edit policy.
    pub(crate) async fn persist_managed_rules(
        &self,
        account: &str,
        rules: &[mw_sieve::Rule],
        ctx: &mut SieveContext,
    ) -> Result<(), Value> {
        let rows = self.sieve_snapshot(account, ctx).await?;
        let active = rows.iter().find(|s| s.active);
        let source = mw_sieve::managed::merge(active.map_or("", |s| s.content.as_str()), rules)
            .map_err(|_| err("invalidSieve"))?;
        self.persist_managed_source(account, rows, &source, ctx)
            .await
    }

    pub(super) async fn persist_managed_source(
        &self,
        account: &str,
        rows: Vec<SieveScriptRow>,
        source: &str,
        ctx: &mut SieveContext,
    ) -> Result<(), Value> {
        let active = rows.iter().find(|s| s.active);
        let source = if source.is_empty() {
            "# Mailwoman managed rules\n"
        } else {
            source
        };
        let blob = self
            .store_upload(account, "application/sieve", source.as_bytes())
            .await
            .map_err(|_| err("serverFail"))?;
        let args = if let Some(active) = active {
            json!({"accountId":account,"ifInState":state(&rows),"update":{active.id():{"blobId":blob}},"onSuccessActivateScript":active.id()})
        } else {
            let mut name = "mailwoman".to_string();
            let mut n = 1;
            while rows.iter().any(|s| s.name == name) {
                name = format!("mailwoman-{n}");
                n += 1;
            }
            json!({"accountId":account,"ifInState":state(&rows),"create":{"managed":{"name":name,"blobId":blob}},"onSuccessActivateScript":"#managed"})
        };
        ctx.edit_kind = EditKind::Builder;
        let result = self
            .sieve_set(account, &args, rows, ctx, &mut HashMap::new())
            .await;
        ctx.edit_kind = EditKind::Raw;
        if result.get("type").is_some() {
            return Err(result);
        }
        for field in ["notCreated", "notUpdated"] {
            if let Some(error) = result[field].as_object().and_then(|m| m.values().next()) {
                return Err(error.clone());
            }
        }
        Ok(())
    }
}
