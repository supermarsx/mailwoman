//! UnifiedPush registration for Android (§2.1 `pushSubscribe`, §2.3, plan §1.7).
//!
//! UnifiedPush is the **self-hostable, no-Google** Android push path: the app
//! asks the on-device **distributor** (ntfy, NextPush, …) to allocate an
//! endpoint URL; the Mailwoman server (e5) later POSTs opaque wakes to that URL.
//! Content never transits push — the wake only triggers a foreground JMAP
//! `/changes` refetch (frozen §2.3).
//!
//! Flow (frozen for e6's `tauri.ts`):
//!   1. `push_register` asks the native plugin to `registerApp` with the
//!      distributor. If an endpoint was already granted (cached in Kotlin) it is
//!      returned immediately as a [`PushSubscriptionInfo`]; otherwise `null` is
//!      returned and the endpoint arrives asynchronously.
//!   2. When the distributor grants/rotates an endpoint, the Kotlin
//!      `UnifiedPushReceiver` triggers the `unifiedpush://new-endpoint` plugin
//!      event carrying a `PushSubscriptionInfo`; `tauri.ts` listens and POSTs it
//!      to `/api/push/subscribe`.
//!   3. `push_unregister` tells the distributor to release the endpoint.
//!
//! On the desktop **host** build there is no distributor, so every command
//! degrades to `Ok(None)` / no-op — keeping the crate `cargo check`-clean
//! without the Android toolchain.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

#[cfg(mobile)]
use crate::commands::MobileBridge;
#[cfg(mobile)]
use tauri::Manager;

/// The frozen push subscription shape exchanged with `/api/push/subscribe`
/// (§2.3). For UnifiedPush: `transport = "unifiedpush"`, `keys = null`,
/// `app_id` = the UnifiedPush instance id. camelCase over the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushSubscriptionInfo {
    pub transport: String,
    pub endpoint: String,
    /// Web Push only — always `None` for UnifiedPush.
    pub keys: Option<PushKeys>,
    /// UnifiedPush/APNs instance id.
    pub app_id: Option<String>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushKeys {
    pub p256dh: String,
    pub auth: String,
}

/// Reply from the native `registerUnifiedPush` command: the endpoint if the
/// distributor has already granted one, else `null` (it arrives via the
/// `unifiedpush://new-endpoint` event).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // fields consumed only on the mobile target
struct RegisterReply {
    endpoint: Option<String>,
    app_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct DistributorReply {
    distributor: Option<String>,
}

/// The user-visible id of the currently selected UnifiedPush distributor, if one
/// is installed and saved (§2.1 — informs whether UnifiedPush is available).
#[tauri::command]
pub fn push_get_distributor<R: Runtime>(app: AppHandle<R>) -> Result<Option<String>, String> {
    #[cfg(mobile)]
    {
        let reply: DistributorReply = app.state::<MobileBridge<R>>().run("getDistributor", ())?;
        Ok(reply.distributor)
    }
    #[cfg(not(mobile))]
    {
        let _ = app;
        Ok(None)
    }
}

/// Register with the on-device UnifiedPush distributor. Returns the endpoint
/// immediately if already granted, otherwise `null` (it arrives via the
/// `unifiedpush://new-endpoint` event). §2.1 `pushSubscribe`.
#[tauri::command]
pub fn push_register<R: Runtime>(
    app: AppHandle<R>,
) -> Result<Option<PushSubscriptionInfo>, String> {
    #[cfg(mobile)]
    {
        let reply: RegisterReply = app
            .state::<MobileBridge<R>>()
            .run("registerUnifiedPush", ())?;
        Ok(reply.endpoint.map(|endpoint| PushSubscriptionInfo {
            transport: "unifiedpush".into(),
            endpoint,
            keys: None,
            app_id: reply.app_id,
            expires_at: None,
        }))
    }
    #[cfg(not(mobile))]
    {
        let _ = app;
        Ok(None)
    }
}

/// Release the UnifiedPush endpoint with the distributor (§2.1 `pushUnsubscribe`).
#[tauri::command]
pub fn push_unregister<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    #[cfg(mobile)]
    {
        let _: serde_json::Value = app
            .state::<MobileBridge<R>>()
            .run("unregisterUnifiedPush", ())?;
        Ok(())
    }
    #[cfg(not(mobile))]
    {
        let _ = app;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unifiedpush_subscription_serializes_camelcase_no_keys() {
        let info = PushSubscriptionInfo {
            transport: "unifiedpush".into(),
            endpoint: "https://ntfy.example/up?id=abc".into(),
            keys: None,
            app_id: Some("default".into()),
            expires_at: None,
        };
        let v = serde_json::to_value(&info).unwrap();
        assert_eq!(v["transport"], "unifiedpush");
        assert_eq!(v["appId"], "default");
        assert!(v["keys"].is_null());
        assert!(v.get("expiresAt").is_some());
    }

    #[test]
    fn register_reply_parses_pending_endpoint() {
        let r: RegisterReply =
            serde_json::from_str(r#"{"endpoint":null,"appId":"default"}"#).unwrap();
        assert!(r.endpoint.is_none());
        assert_eq!(r.app_id.as_deref(), Some("default"));
    }
}
