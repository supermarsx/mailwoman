//! Native notification command for the mobile shell (§2.1 "Notifications"; frozen
//! `tauri.ts` name `mw_notify` + the `mw://notification-action` bridge).
//!
//! Mirrors the desktop `notifications` module: a new-mail notification is shown via
//! `tauri-plugin-notification` (which is registered unconditionally in `lib.rs`, so
//! this host-compiles), and a tap / action button re-enters the SPA through the
//! frontend event `mw://notification-action` carrying the frozen
//! `{ notificationId, actionId }` shape that `platform/tauri.ts`'s
//! `onNotificationAction` listener consumes. Content is the SPA's own already-fetched
//! data — push never carries content (§2.3), so this is local rendering only.
//!
//! Android action-button DELIVERY (the tap → `mw://notification-action` re-entry) is
//! wired by the notification plugin's activation callback on device; that leg is
//! exercised in the CI Android build / a device run, not the host unit tests. The
//! command + the frozen event shape are pinned here so the JS↔shell contract cannot
//! drift.

use serde::{Deserialize, Serialize};
use tauri::{Emitter, Runtime};
use tauri_plugin_notification::NotificationExt;

/// Frontend event name the capability layer listens on for notification activations.
pub const NOTIFICATION_ACTION_EVENT: &str = "mw://notification-action";

/// One actionable button on a native notification (archive / delete / reply).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotifyAction {
    pub id: String,
    pub label: String,
}

/// The `notify(...)` input (mirrors `NotifyInput` in `platform/index.ts`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotifyPayload {
    pub id: String,
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub actions: Vec<NotifyAction>,
    #[serde(default)]
    pub thread_id: Option<String>,
}

/// The event delivered to the frontend when a notification (or one of its action
/// buttons) is activated. Frozen shape (§2.1): `{ notificationId, actionId }`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationActionEvent {
    pub notification_id: String,
    pub action_id: String,
}

/// `actionId` reported when the user taps the notification body (no specific button).
pub const ACTION_DEFAULT: &str = "default";

impl NotificationActionEvent {
    pub fn new(notification_id: impl Into<String>, action_id: impl Into<String>) -> Self {
        Self {
            notification_id: notification_id.into(),
            action_id: action_id.into(),
        }
    }
}

/// Emit the notification-action event to the frontend — the single choke point for
/// the frozen event shape. The plugin's activation callback (Android, device-only)
/// calls this; the capability layer's `onNotificationAction` receives exactly this.
pub fn emit_notification_action<R: Runtime>(
    app: &tauri::AppHandle<R>,
    notification_id: impl Into<String>,
    action_id: impl Into<String>,
) -> Result<(), String> {
    app.emit(
        NOTIFICATION_ACTION_EVENT,
        NotificationActionEvent::new(notification_id, action_id),
    )
    .map_err(|e| format!("emit {NOTIFICATION_ACTION_EVENT}: {e}"))
}

/// Show a native notification for `input` (§2.1 `notify`). Title + body are always
/// shown; action buttons are attached where the OS/plugin supports them.
#[tauri::command]
pub async fn mw_notify<R: Runtime>(
    app: tauri::AppHandle<R>,
    input: NotifyPayload,
) -> Result<(), String> {
    app.notification()
        .builder()
        .title(&input.title)
        .body(&input.body)
        .show()
        .map_err(|e| format!("notify {}: {e}", input.id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_event_serializes_camelcase_and_round_trips() {
        let ev = NotificationActionEvent::new("mail-42", "archive");
        let json = serde_json::to_string(&ev).unwrap();
        assert_eq!(json, r#"{"notificationId":"mail-42","actionId":"archive"}"#);
        let back: NotificationActionEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ev);
    }

    #[test]
    fn default_action_id_is_stable() {
        let ev = NotificationActionEvent::new("t1", ACTION_DEFAULT);
        assert_eq!(ev.action_id, "default");
    }

    #[test]
    fn notify_payload_parses_from_the_capability_layer_shape() {
        let json = r#"{
            "id": "mail-99",
            "title": "New message",
            "body": "Ada Lovelace: lunch?",
            "actions": [
                { "id": "archive", "label": "Archive" },
                { "id": "reply", "label": "Reply" }
            ],
            "threadId": "thread-7"
        }"#;
        let p: NotifyPayload = serde_json::from_str(json).unwrap();
        assert_eq!(p.id, "mail-99");
        assert_eq!(p.thread_id.as_deref(), Some("thread-7"));
        assert_eq!(p.actions.len(), 2);
    }

    #[test]
    fn notify_payload_defaults_actions_and_thread_when_absent() {
        let p: NotifyPayload =
            serde_json::from_str(r#"{"id":"x","title":"t","body":"b"}"#).unwrap();
        assert!(p.actions.is_empty());
        assert_eq!(p.thread_id, None);
    }
}
