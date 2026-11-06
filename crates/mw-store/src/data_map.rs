//! The data map: the single declarative registry that says where one person's
//! data lives and what happens to it.
//!
//! Every table that holds a per-person identifier (an account id, a personal
//! owner subject, an address, a principal id, an actor, a recipient...) has at
//! least one [`Rule`] here. A rule says how the rows are found for one person
//! ([`Bind`]), how they leave the system ([`How`]), which export section carries
//! them ([`Export`]), what limits how long they stay ([`Rule::retention`]) and,
//! for audit-style history, how they are pseudonymised ([`Rule::scrub`]).
//!
//! The registry is the source of truth for three things that used to be separate
//! lists: the erasure DELETE statements, the verify pass that follows an erasure
//! (it counts the rows each rule still finds, so a table that is only in the
//! registry is verified too) and the export. `tests/data_map_coverage.rs` reads
//! the live schema of a migrated database and fails when a table with a
//! per-person column is not mapped here, so a new migration cannot add personal
//! data that erasure and export silently skip.

/// Which identifiers of the person a rule's statements are run with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bind {
    /// Every account id bound to the person.
    Account,
    /// The personal owner subjects (opaque keys of the personal-data tables).
    Owner,
    /// The person's address.
    Address,
    /// Identity principal ids reached through the person's bindings and sessions.
    Principal,
    /// Account ids, owner subjects and the address (a column that holds any of
    /// them, such as the legacy `user` column).
    Any,
    /// The rule needs no identifier.
    None,
}

/// What happens to the rows at erasure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum How {
    /// A fixed DELETE statement with the identifier as `?1`.
    Delete(&'static str),
    /// The rows go with a parent row (a foreign key cascade or the parent's
    /// DELETE); the rule's `count` proves it.
    Cascade,
    /// The rows are kept; the reason is shown in the completion record and in
    /// `docs/admin/erasure-coverage.md`.
    Keep(&'static str),
}

/// Where the rows appear in an export.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Export {
    /// Not part of an export.
    None,
    /// Selected columns. `cols` is `name:s|i|j` (text, integer, JSON text) per
    /// output column, in the order the SELECT returns them.
    Rows {
        section: &'static str,
        select: &'static str,
        cols: &'static str,
    },
    /// Sealed or secret rows: the export says they exist and are not exported.
    Marker { section: &'static str },
    /// Exported by code that is not table-driven (mail bodies, files, calendars).
    Code { section: &'static str },
}

/// How a retained history row is pseudonymised by the per-person scrub.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scrub {
    /// Columns holding the person's identifiers.
    pub columns: &'static str,
    pub note: &'static str,
}

/// One entry of the registry.
#[derive(Clone, Copy, Debug)]
pub struct Rule {
    pub table: &'static str,
    pub bind: Bind,
    pub how: How,
    /// `SELECT COUNT(*) ... ?1`: rows still found for the person. Empty for a
    /// kept table.
    pub count: &'static str,
    pub export: Export,
    /// Name of the retention control that bounds how long the rows stay.
    pub retention: &'static str,
    pub scrub: Option<Scrub>,
    /// Run after every other rule (the parent of cascades).
    pub last: bool,
}

impl Rule {
    const BASE: Rule = Rule {
        table: "",
        bind: Bind::None,
        how: How::Cascade,
        count: "",
        export: Export::None,
        retention: "",
        scrub: None,
        last: false,
    };
}

macro_rules! del {
    ($t:literal, $c:literal, $b:expr) => {
        Rule {
            table: $t,
            bind: $b,
            how: How::Delete(concat!("DELETE FROM ", $t, " WHERE ", $c, "=?1")),
            count: concat!("SELECT COUNT(*) FROM ", $t, " WHERE ", $c, "=?1"),
            ..Rule::BASE
        }
    };
    ($t:literal, $c:literal, $b:expr, export: $e:expr) => {
        Rule {
            export: $e,
            ..del!($t, $c, $b)
        }
    };
}

/// A DELETE with its own WHERE clause (children reached through a parent).
macro_rules! sub {
    ($t:literal, $w:literal, $b:expr) => {
        Rule {
            table: $t,
            bind: $b,
            how: How::Delete(concat!("DELETE FROM ", $t, " WHERE ", $w)),
            count: concat!("SELECT COUNT(*) FROM ", $t, " WHERE ", $w),
            ..Rule::BASE
        }
    };
    ($t:literal, $w:literal, $b:expr, export: $e:expr) => {
        Rule {
            export: $e,
            ..sub!($t, $w, $b)
        }
    };
}

/// Rows removed with a parent: `count` finds them through the parent.
macro_rules! child {
    ($t:literal, $w:literal, $b:expr) => {
        Rule {
            table: $t,
            bind: $b,
            how: How::Cascade,
            count: concat!("SELECT COUNT(*) FROM ", $t, " WHERE ", $w),
            ..Rule::BASE
        }
    };
    ($t:literal, $w:literal, $b:expr, export: $e:expr) => {
        Rule {
            export: $e,
            ..child!($t, $w, $b)
        }
    };
}

macro_rules! keep {
    ($t:literal, $reason:expr) => {
        Rule {
            table: $t,
            how: How::Keep($reason),
            ..Rule::BASE
        }
    };
}

macro_rules! rows {
    ($s:literal, $sql:literal, $cols:literal) => {
        Export::Rows {
            section: $s,
            select: $sql,
            cols: $cols,
        }
    };
}

const STAFF: &str = "operator account of the admin console; removed with the administrator account, not by a mail user's erasure";
const GLOBAL: &str = "server-wide configuration with no per-person key";

/// Export sections the registry feeds, beyond the ones built by code.
pub const TABLE_SECTIONS: &[&str] = &[
    "mailstate",
    "tasks",
    "security",
    "integrations",
    "delegation",
    "usage",
    "records",
];

/// The registry. Rules run in this order, those marked `last` after the rest.
pub const RULES: &[Rule] = &[
    // ---- credentials and sign-in ----------------------------------------
    del!("totp_secrets", "account_id", Bind::Account, export: Export::Marker { section: "security" }),
    del!("webauthn_credentials", "account_id", Bind::Account, export: rows!("security",
        "SELECT credential_id,label,created_at,sign_count FROM webauthn_credentials WHERE account_id=?1",
        "credentialId:s,label:s,createdAt:i,signCount:i")),
    del!("recovery_codes", "account_id", Bind::Account, export: Export::Marker { section: "security" }),
    del!("remembered_device", "account_id", Bind::Account, export: Export::Code { section: "sessions" }),
    del!("native_sessions", "account_id", Bind::Account, export: Export::Code { section: "sessions" }),
    del!("signin_event", "account_id", Bind::Account, export: Export::Code { section: "sessions" }),
    del!("signin_identity", "account_id", Bind::Account, export: rows!("security",
        "SELECT address,seen_at FROM signin_identity WHERE account_id=?1", "address:s,seenAt:i")),
    del!("password_change_audit", "account_id", Bind::Account, export: rows!("security",
        "SELECT ts,backend,outcome FROM password_change_audit WHERE account_id=?1", "ts:i,backend:s,outcome:s")),
    del!("api_keys", "account_id", Bind::Account, export: rows!("security",
        "SELECT id,key_prefix,scopes,expires_at,created_at,last_used_at,revoked_at FROM api_keys WHERE account_id=?1",
        "id:s,keyPrefix:s,scopes:s,expiresAt:i,createdAt:i,lastUsedAt:i,revokedAt:i")),
    del!("oauth_tokens", "account_id", Bind::Account, export: rows!("security",
        "SELECT client_id,scopes,kind,expires_at,created_at,revoked_at FROM oauth_tokens WHERE account_id=?1",
        "clientId:s,scopes:s,kind:s,expiresAt:i,createdAt:i,revokedAt:i")),
    del!("account_oauth", "account_id", Bind::Account, export: Export::Marker { section: "integrations" }),
    del!("passwd_config", "account_id", Bind::Account),
    Rule {
        retention: "window expiry",
        ..sub!("auth_failure", "key_kind='account' AND key=?1", Bind::Any)
    },
    keep!("auth_source_failure", "keyed by source address and surface, not by person; expires on its own (expires_at)"),
    del!("user_activity", "account_id", Bind::Account, export: rows!("usage",
        "SELECT last_seen_at FROM user_activity WHERE account_id=?1", "lastSeenAt:i")),
    // ---- personal settings, identities, signatures ----------------------
    del!("signatures", "account_id", Bind::Account),
    del!("notification_rules", "account_id", Bind::Account),
    del!("identities", "account_id", Bind::Account),
    del!("masked_email", "account_id", Bind::Account, export: rows!("mailstate",
        "SELECT id,alias_addr,target_desc,state,created_at,last_used_at FROM masked_email WHERE account_id=?1",
        "id:s,aliasAddr:s,targetDesc:s,state:s,createdAt:i,lastUsedAt:i")),
    del!("account_draft", "account_id", Bind::Account, export: Export::Marker { section: "mailstate" }),
    del!("account_draft", "owner", Bind::Owner),
    del!("personal_setting_value", "owner_subject", Bind::Owner),
    del!("personal_signature", "owner_subject", Bind::Owner),
    del!("personal_saved_search", "owner_subject", Bind::Owner, export: rows!("mailstate",
        "SELECT id,name,query_json,as_folder,query_text FROM personal_saved_search WHERE owner_subject=?1",
        "id:s,name:s,query:j,asFolder:i,queryText:s")),
    del!("personal_identity", "owner_subject", Bind::Owner),
    del!("personal_notification_rules", "owner_subject", Bind::Owner),
    del!("personal_push_subscription", "owner_subject", Bind::Owner),
    del!("focused_sender", "owner_subject", Bind::Owner, export: rows!("mailstate",
        "SELECT sender,tab,updated_at FROM focused_sender WHERE owner_subject=?1", "sender:s,tab:s,updatedAt:i")),
    del!("push_subscriptions", "account_id", Bind::Account),
    del!("remote_image_grants", "account_id", Bind::Account, export: rows!("mailstate",
        "SELECT scope_kind,scope_value,granted_at,revoked FROM remote_image_grants WHERE account_id=?1",
        "scopeKind:s,scopeValue:s,grantedAt:i,revoked:i")),
    del!("mdn_sender_rule", "account_id", Bind::Account, export: rows!("mailstate",
        "SELECT address,decision,updated_at FROM mdn_sender_rule WHERE account_id=?1", "address:s,decision:s,updatedAt:i")),
    del!("receipt_status", "account_id", Bind::Account, export: rows!("mailstate",
        "SELECT sent_email_id,recipient,kind,status,reported_at FROM receipt_status WHERE account_id=?1",
        "sentEmailId:s,recipient:s,kind:s,status:s,reportedAt:i")),
    del!("plugin_kv", "account_id", Bind::Account, export: rows!("records",
        "SELECT plugin_id,key,size,updated_at FROM plugin_kv WHERE account_id=?1", "pluginId:s,key:s,size:i,updatedAt:i")),
    del!("plugin_grants", "account_id", Bind::Account),
    del!("inbound_webhooks", "account_id", Bind::Account, export: rows!("integrations",
        "SELECT id,label,actions_json,created_at,last_used_at FROM inbound_webhooks WHERE account_id=?1",
        "id:s,label:s,actions:j,createdAt:i,lastUsedAt:i")),
    del!("webhooks", "account_id", Bind::Account, export: rows!("integrations",
        "SELECT id,url,label,event_filter,created_at FROM webhooks WHERE account_id=?1",
        "id:s,url:s,label:s,eventFilter:s,createdAt:i")),
    del!("tags", "\"user\"", Bind::Any, export: rows!("mailstate",
        "SELECT id,name,color,icon FROM tags WHERE \"user\"=?1", "id:s,name:s,color:s,icon:s")),
    del!("saved_searches", "\"user\"", Bind::Any, export: rows!("mailstate",
        "SELECT id,name,query_json,as_folder,query_text FROM saved_searches WHERE \"user\"=?1",
        "id:s,name:s,query:j,asFolder:i,queryText:s")),
    // ---- integrations with sealed credentials ---------------------------
    del!("ews_account_cred", "account_id", Bind::Account, export: rows!("integrations",
        "SELECT endpoint_host,enabled,created_at FROM ews_account_cred WHERE account_id=?1",
        "endpointHost:s,enabled:i,createdAt:i")),
    del!("dav_account", "account_id", Bind::Account, export: rows!("integrations",
        "SELECT enabled,username,last_sync_at,created_at FROM dav_account WHERE account_id=?1",
        "enabled:i,username:s,lastSyncAt:i,createdAt:i")),
    sub!("bridge_oauth_tokens", "bridge_account_id IN (SELECT bridge_id FROM bridge_accounts WHERE account_id=?1) OR bridge_account_id=?1", Bind::Account,
        export: Export::Marker { section: "integrations" }),
    del!("bridge_accounts", "account_id", Bind::Account, export: Export::Marker { section: "integrations" }),
    del!("account_provider", "account_id", Bind::Account, export: rows!("integrations",
        "SELECT provider,endpoint_url,login_address,created_at FROM account_provider WHERE account_id=?1",
        "provider:s,endpointUrl:s,loginAddress:s,createdAt:i")),
    del!("child_account", "owner_account_id", Bind::Account, export: rows!("integrations",
        "SELECT account_id,provider_id,display_name,added_at FROM child_account WHERE owner_account_id=?1",
        "accountId:s,providerId:s,displayName:s,addedAt:i")),
    del!("nextcloud_account_link", "account_id", Bind::Account, export: Export::Marker { section: "integrations" }),
    del!("nextcloud_pim_binding", "account_id", Bind::Account),
    del!("package_account", "account_id", Bind::Account),
    // ---- linked mailboxes and identity ----------------------------------
    sub!("linked_credential", "owner_principal_id=?1", Bind::Principal, export: Export::Marker { section: "integrations" }),
    sub!("linked_grant", "link_id IN (SELECT id FROM linked_account WHERE owner_principal_id=?1)", Bind::Principal),
    sub!("linked_context", "link_id IN (SELECT id FROM linked_account WHERE owner_principal_id=?1)", Bind::Principal),
    del!("linked_layout", "owner_principal_id", Bind::Principal),
    del!("linked_primary", "owner_principal_id", Bind::Principal),
    del!("linked_account", "owner_principal_id", Bind::Principal, export: rows!("integrations",
        "SELECT id,display_name,created_at FROM linked_account WHERE owner_principal_id=?1", "id:s,displayName:s,createdAt:i")),
    del!("linked_account", "child_principal_id", Bind::Principal),
    del!("oidc_consent", "principal_id", Bind::Principal, export: rows!("security",
        "SELECT client_id,scopes_json,updated_at FROM oidc_consent WHERE principal_id=?1", "clientId:s,scopes:j,updatedAt:i")),
    del!("oidc_grant", "principal_id", Bind::Principal),
    // feature_user_subject references identity_binding, so it goes first.
    del!("feature_user_subject", "account_id", Bind::Account),
    del!("session_principal", "principal_id", Bind::Principal),
    del!("identity_binding", "principal_id", Bind::Principal),
    keep!("identity_principal", "an opaque random id with no identifier once its bindings are deleted"),
    // ---- delegation, groups, directory ----------------------------------
    del!("delegation_grant", "owner_account_id", Bind::Account, export: rows!("delegation",
        "SELECT id,'owner' AS role,owner_account_id,delegate_owner_account_id,scope_kind,permissions_json,expires_at,created_at,revoked_at FROM delegation_grant WHERE owner_account_id=?1",
        "id:s,role:s,ownerAccountId:s,delegateAccountId:s,scopeKind:s,permissions:j,expiresAt:i,createdAt:i,revokedAt:i")),
    del!("delegation_grant", "delegate_owner_account_id", Bind::Account, export: rows!("delegation",
        "SELECT id,'delegate' AS role,owner_account_id,delegate_owner_account_id,scope_kind,permissions_json,expires_at,created_at,revoked_at FROM delegation_grant WHERE delegate_owner_account_id=?1",
        "id:s,role:s,ownerAccountId:s,delegateAccountId:s,scopeKind:s,permissions:j,expiresAt:i,createdAt:i,revokedAt:i")),
    del!("shared_mailbox", "account_id", Bind::Account, export: rows!("delegation",
        "SELECT account_id,domain,name,created_at FROM shared_mailbox WHERE account_id=?1",
        "accountId:s,domain:s,name:s,createdAt:i")),
    del!("user_group_member", "account_id", Bind::Account, export: rows!("delegation",
        "SELECT group_id FROM user_group_member WHERE account_id=?1", "groupId:s")),
    del!("usage_counter", "address", Bind::Address, export: rows!("usage",
        "SELECT metric,day,value FROM usage_counter WHERE address=?1", "metric:s,day:s,value:i")),
    del!("usage_storage", "address", Bind::Address, export: rows!("usage",
        "SELECT blob_bytes,blob_count,content_bytes,cache_bytes,cache_messages,measured_at FROM usage_storage WHERE address=?1",
        "blobBytes:i,blobCount:i,contentBytes:i,cacheBytes:i,cacheMessages:i,measuredAt:i")),
    del!("quotas", "account_id", Bind::Account, export: rows!("usage",
        "SELECT bytes_limit,msg_limit FROM quotas WHERE account_id=?1", "bytesLimit:i,msgLimit:i")),
    Rule {
        export: Export::Code { section: "profile" },
        ..keep!("mail_user", "the directory row is closed (status deleted), not removed, so the address cannot be re-enrolled without an administrator")
    },
    del!("mail_user_account", "account_id", Bind::Account),
    keep!("user_group", "group definitions belong to the directory, not to one member"),
    keep!("client_error_group", "browser error groups: a scrubbed error name, message, boundary and route template with counts; no person identifiers"),
    keep!("client_error_event", "pseudonymous diagnostics: scrubbed stack frames and a coarse browser, OS and viewport class per occurrence; account_hash is a salted one-way hash, empty unless the administrator enabled it, and events expire with retention.clientErrors.eventDays"),
    keep!("metrics_bucket", "aggregate metrics, no person identifiers: counts and quantile sketches per instance, metric and time bucket, with fixed role-class dimensions only"),
    keep!("data_export", "legacy export log; rows are removed with the request retention"),
    // ---- mail, calendars, contacts, notes --------------------------------
    del!("sieve_script", "account_id", Bind::Account, export: Export::Code { section: "rules" }),
    child!("tasks", "list_id IN (SELECT id FROM calendars WHERE account_id=?1)", Bind::Account, export: rows!("tasks",
        "SELECT id,uid,due_utc,start_utc,priority,percent_complete,status,ical_raw FROM tasks WHERE list_id IN (SELECT id FROM calendars WHERE account_id=?1) ORDER BY id",
        "id:s,uid:s,dueUtc:s,startUtc:s,priority:i,percentComplete:i,status:s,ical:s")),
    child!("event_instances", "event_id IN (SELECT id FROM events WHERE calendar_id IN (SELECT id FROM calendars WHERE account_id=?1))", Bind::Account),
    child!("events", "calendar_id IN (SELECT id FROM calendars WHERE account_id=?1)", Bind::Account, export: Export::Code { section: "calendars" }),
    child!("calendar_shares", "calendar_id IN (SELECT id FROM calendars WHERE account_id=?1)", Bind::Account),
    del!("calendars", "account_id", Bind::Account, export: Export::Code { section: "calendars" }),
    child!("contact_groups", "address_book_id IN (SELECT id FROM address_books WHERE account_id=?1)", Bind::Account),
    child!("contacts", "address_book_id IN (SELECT id FROM address_books WHERE account_id=?1)", Bind::Account, export: Export::Code { section: "contacts" }),
    del!("address_books", "account_id", Bind::Account, export: Export::Code { section: "contacts" }),
    del!("notes", "account_id", Bind::Account, export: Export::Code { section: "notes" }),
    del!("notebooks", "account_id", Bind::Account, export: Export::Code { section: "notes" }),
    del!("message_embeddings", "account_id", Bind::Account),
    del!("uploaded_blobs", "account_id", Bind::Account, export: rows!("records",
        "SELECT blob_id,content_type,size,created_at FROM uploaded_blobs WHERE account_id=?1", "blobId:s,contentType:s,size:i,createdAt:i")),
    sub!("open_events", "token_hash IN (SELECT token_hash FROM open_tracking WHERE account_id=?1)", Bind::Account, export: rows!("records",
        "SELECT at FROM open_events WHERE token_hash IN (SELECT token_hash FROM open_tracking WHERE account_id=?1)", "at:s")),
    del!("open_tracking", "account_id", Bind::Account, export: rows!("records",
        "SELECT created_at,open_count,first_open_at,last_open_at FROM open_tracking WHERE account_id=?1",
        "createdAt:i,openCount:i,firstOpenAt:i,lastOpenAt:i")),
    del!("submissions", "account_id", Bind::Account, export: rows!("mailstate",
        "SELECT id,email_id,identity_id,send_at,undo_status,created_at,attempts FROM submissions WHERE account_id=?1",
        "id:s,emailId:s,identityId:s,sendAt:s,undoStatus:s,createdAt:s,attempts:i")),
    child!("message_meta", "stable_id IN (SELECT stable_id FROM messages WHERE account_id=?1)", Bind::Account, export: rows!("mailstate",
        "SELECT stable_id,pinned,snoozed_until,follow_up_at FROM message_meta WHERE stable_id IN (SELECT stable_id FROM messages WHERE account_id=?1)",
        "stableId:s,pinned:i,snoozedUntil:s,followUpAt:s")),
    del!("threads", "account_id", Bind::Account),
    del!("sender_controls", "account_id", Bind::Account, export: rows!("mailstate",
        "SELECT address,thread_id,action,at FROM sender_controls WHERE account_id=?1", "address:s,threadId:s,action:s,at:s")),
    del!("security_verdicts", "account_id", Bind::Account),
    del!("sync_state", "account_id", Bind::Account),
    del!("pop3_uidl", "account_id", Bind::Account),
    del!("changes", "account_id", Bind::Account),
    del!("pim_changes", "account_id", Bind::Account),
    del!("crypto_changes", "account_id", Bind::Account),
    child!("bodies", "account_id=?1", Bind::Account, export: Export::Code { section: "mail" }),
    child!("messages", "account_id=?1", Bind::Account, export: Export::Code { section: "mail" }),
    child!("mailboxes", "account_id=?1", Bind::Account),
    // ---- keys ---------------------------------------------------------------
    del!("key_associations", "account_id", Bind::Account, export: rows!("security",
        "SELECT address,crypto_key_id,seen_at FROM key_associations WHERE account_id=?1", "address:s,cryptoKeyId:s,seenAt:s")),
    del!("crypto_keys", "account_id", Bind::Account, export: rows!("security",
        "SELECT id,kind,is_own,addresses_json,fingerprint,key_id,algorithm,created_at,expires_at,trust,source,CASE WHEN encrypted_private_backup IS NULL THEN 0 ELSE 1 END AS has_private_backup FROM crypto_keys WHERE account_id=?1",
        "id:s,kind:s,isOwn:i,addresses:j,fingerprint:s,keyId:s,algorithm:s,createdAt:s,expiresAt:s,trust:s,source:s,hasPrivateBackup:i")),
    del!("zeroaccess_accounts", "account_id", Bind::Account, export: Export::Marker { section: "security" }),
    // ---- history that is scrubbed rather than deleted ----------------------
    Rule {
        scrub: Some(Scrub { columns: "account_id", note: "rows are removed with the account; a foreign key cannot be re-keyed, so a scrub-only run leaves them" }),
        retention: "audit.retention",
        ..del!("dlp_audit", "account_id", Bind::Account, export: rows!("records",
            "SELECT at,rule_name,action,blocked FROM dlp_audit WHERE account_id=?1", "at:s,ruleName:s,action:s,blocked:i"))
    },
    Rule {
        scrub: Some(Scrub { columns: "actor", note: "the actor is replaced with the keyed pseudonym" }),
        retention: "audit.retention",
        ..keep!("assist_audit", "capability audit kept for the audit retention period; the actor is pseudonymised by the scrub")
    },
    Rule {
        scrub: Some(Scrub { columns: "actor,target,detail_json,ip", note: "chain-preserving: rows are re-sealed and an audit_rechain anchor is written" }),
        retention: "audit.retention",
        export: Export::Code { section: "audit" },
        ..keep!("audit_log", "security and administrative audit trail kept for the audit retention period; identifiers are pseudonymised by the scrub")
    },
    Rule {
        scrub: Some(Scrub { columns: "subject_hash", note: "the subject is already a hash; the scrub re-keys it to the pseudonym" }),
        retention: "audit.retention",
        ..keep!("sso_login_audit", "sign-in audit kept for the audit retention period; the subject is pseudonymised by the scrub")
    },
    Rule {
        scrub: Some(Scrub { columns: "summary", note: "identifiers inside the summary are replaced" }),
        retention: "audit.retention",
        ..keep!("alert_event", "alert history kept for the audit retention period; identifiers in the summary are pseudonymised by the scrub")
    },
    Rule {
        retention: "requestProtection.retention",
        ..keep!("request_protection_event", "keyed by a one-way hash of a network source, not by person; bounded by its retention and not linkable to an address, so a per-person scrub has nothing to match")
    },
    Rule {
        retention: "requestProtection.retention",
        ..keep!("request_protection_subject", "keyed by a one-way hash of a network source, not by person; expires on its own")
    },
    // ---- restriction, consent, retention rules, archive, breach register ----
    del!("account_restriction", "account_id", Bind::Account, export: rows!("records",
        "SELECT id,scopes_json,mail_delivery,reason,source,created_at,expires_at,lifted_at FROM account_restriction WHERE account_id=?1",
        "id:s,scopes:j,mailDelivery:s,reason:s,source:s,createdAt:i,expiresAt:i,liftedAt:i")),
    del!("consent_record", "account_id", Bind::Account, export: rows!("records",
        "SELECT id,domain,purpose,decision,version,basis,source,created_at FROM consent_record WHERE account_id=?1",
        "id:s,domain:s,purpose:s,decision:s,version:i,basis:s,source:s,createdAt:i")),
    del!("user_retention_rule", "account_id", Bind::Account, export: rows!("records",
        "SELECT id,name,enabled,sort_order,template,body_json,created_at,updated_at FROM user_retention_rule WHERE account_id=?1",
        "id:s,name:s,enabled:i,sortOrder:i,template:s,rule:j,createdAt:i,updatedAt:i")),
    del!("user_retention_account", "account_id", Bind::Account, export: rows!("records",
        "SELECT enabled,trash_first,undo_days,paused_until,folder_defaults_json,last_run_at,updated_at FROM user_retention_account WHERE account_id=?1",
        "enabled:i,trashFirst:i,undoDays:i,pausedUntil:i,folderDefaults:j,lastRunAt:i,updatedAt:i")),
    del!("user_pim_retention", "account_id", Bind::Account, export: rows!("records",
        "SELECT kind,item_ref,config_json,due_at,updated_at FROM user_pim_retention WHERE account_id=?1",
        "kind:s,itemRef:s,config:s,dueAt:i,updatedAt:i")),
    del!("user_retention_run", "account_id", Bind::Account, export: rows!("records",
        "SELECT id,started_at,finished_at,dry_run,triggered_by,matched,acted,errors FROM user_retention_run WHERE account_id=?1",
        "id:s,startedAt:i,finishedAt:i,dryRun:i,triggeredBy:s,matched:i,acted:i,errors:i")),
    del!("retention_archive_index", "account_id", Bind::Account),
    keep!("retention_archive_segment", "append-only sealed archive files hold rows of many people; an erased person's index entries are removed, and the segment content goes when its segment expires"),
    Rule {
        scrub: Some(Scrub { columns: "recipient,detail", note: "the notified address is replaced by the keyed pseudonym" }),
        retention: "breach.retention",
        ..keep!("breach_delivery", "delivery log of breach notices kept as part of the breach documentation; the recipient is pseudonymised by the scrub")
    },
    Rule {
        scrub: Some(Scrub { columns: "actor,detail_json", note: "identifiers in the trail are replaced by the keyed pseudonym" }),
        retention: "breach.retention",
        ..keep!("breach_event", "trail of a breach incident kept as part of the breach documentation; identifiers are pseudonymised by the scrub")
    },
    keep!("breach_incident", "breach documentation kept for the regulatory record; holds no per-person key"),
    del!("brand_asset", "owner_account", Bind::Account),
    keep!("domain_signature_template", "domain-wide template; the matched column is an editing flag"),
    keep!("egress_proxy", "operator-configured proxy; the username is a proxy credential"),
    keep!("ssh_tunnel", "operator-configured tunnel; the username is a tunnel credential"),
    keep!("ip_block", "abuse-prevention block keyed by network, expires on its own"),
    keep!("scheduled_task", "the lease owner is a server node name"),
    keep!("setting_policy", "the matched column is an override flag"),
    keep!("template_version", "template content (a mail subject) owned by the template scope"),
    // ---- bookkeeping of erasure itself --------------------------------------
    keep!("erasure_tombstone", "holds only a keyed hash of the erased subject, written so a restore can erase the person again"),
    keep!("scrub_key", "the keyed-pseudonym secrets; destroyed after the configured retention"),
    keep!("audit_rechain", "anchor records of chain-preserving pseudonymisation; hold hashes only"),
    keep!("backup_register", "backup manifests: label, time and tombstone watermark, no identifiers"),
    // ---- not per person -----------------------------------------------------
    keep!("account_ban", "abuse-prevention record keyed by a value (address or network); expires on its own and is lifted by an administrator"),
    keep!("admin_account", STAFF),
    keep!("admin_account_role", STAFF),
    keep!("admin_capability_override", STAFF),
    keep!("admin_recovery_codes", STAFF),
    keep!("admin_saved_query", STAFF),
    keep!("admin_totp", STAFF),
    keep!("admin_webauthn_credentials", STAFF),
    keep!("admin_sessions", STAFF),
    keep!("admin_users", STAFF),
    keep!("push_config", "the server's own web-push key pair, not a person's data"),
    keep!("sso_config", GLOBAL),
    keep!("oidc_authorization", "short-lived authorization request; expires on its own and is removed with its session"),
    keep!("oidc_code", "short-lived authorization code; expires on its own"),
    keep!("oidc_token", "tokens expire on their own and are removed with their grant"),
    keep!("identity_verification", "one-time identity verification tokens (hash only); expire on their own"),
    keep!("diag_probe_account", "operator-configured probe mailbox, not a person's account"),
    keep!("signin_allow", "operator allow-list entries for sign-in patterns"),
    keep!("template", "templates are owned by a scope (global, domain or user) and are managed by the template editor"),
    keep!("org_view", "organisation-wide saved views, not a person's data"),
    keep!("linked_challenge", "short-lived sign-in challenge bound to a session; removed with the session"),
    // accounts runs last: its cascade removes the cached mail and PIM rows.
    Rule { last: true, ..del!("accounts", "id", Bind::Account) },
    Rule {
        table: "sessions",
        bind: Bind::Account,
        how: How::Delete("DELETE FROM sessions WHERE account_id=?1"),
        count: "SELECT COUNT(*) FROM sessions WHERE account_id=?1",
        export: Export::Code { section: "sessions" },
        ..Rule::BASE
    },
    // User-scope setting rows are matched by scope id (an account id or the address).
    Rule {
        table: "setting_value",
        bind: Bind::Any,
        how: How::Delete("DELETE FROM setting_value WHERE scope_kind='user' AND scope_id=?1"),
        count: "SELECT COUNT(*) FROM setting_value WHERE scope_kind='user' AND scope_id=?1",
        export: Export::Code { section: "settings" },
        ..Rule::BASE
    },
    del!("cookie_consent", "account_id", Bind::Account, export: Export::Code { section: "consent" }),
    Rule {
        retention: "dsr.retention",
        export: Export::Code { section: "requests" },
        ..keep!("dsr_request", "the request and its trail are kept for the request retention period as proof of handling")
    },
    keep!("dsr_event", "the trail of a request, kept and removed with it"),
    keep!("dsr_part", "stored export parts, removed when the export expires or the person is erased"),
];

/// Rules whose export is table-driven, for one section.
pub fn export_rules(section: &str) -> impl Iterator<Item = &'static Rule> + '_ {
    RULES.iter().filter(move |r| match r.export {
        Export::Rows { section: s, .. } | Export::Marker { section: s } => s == section,
        _ => false,
    })
}

/// Rules that erase rows (delete or cascade), in execution order: everything
/// except `last`, then the `last` ones.
pub fn erase_order() -> impl Iterator<Item = &'static Rule> {
    RULES
        .iter()
        .filter(|r| !r.last)
        .chain(RULES.iter().filter(|r| r.last))
}

/// Every table named in the registry.
pub fn mapped_tables() -> std::collections::BTreeSet<&'static str> {
    RULES.iter().map(|r| r.table).collect()
}
