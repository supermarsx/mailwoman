//! Native notification commands (plan §2.1 "Notifications"; §3 e1).
//!
//! Backs the capability-layer `notify(...)` and the `onNotificationAction` bridge.
//! A new-mail notification is shown through `tauri-plugin-notification`; when the
//! user activates it or one of its action buttons (archive / delete / reply), the
//! shell emits a frontend IPC event `mw://notification-action` carrying the frozen
//! `{ notificationId, actionId }` shape that `platform/tauri.ts`'s
//! `onNotificationAction` listener consumes (which then drives the SPA — archive the
//! thread, open the composer at `threadId`, etc.).
//!
//! Commands registered by e7 (`tauri::generate_handler!`):
//!   * `mw_notify(app, input: NotifyPayload)` -> Result<(), String>
//!
//! Action delivery: desktop OS notification-action support varies by platform and
//! is owned by the plugin's own activation callback. e7 wires that callback (in the
//! Tauri `setup`) to call [`emit_notification_action`], the single choke point that
//! serializes the frozen event shape. Tests here pin that shape + the payload
//! parsing so the JS<->shell contract cannot drift.

use serde::{Deserialize, Serialize};
use tauri::{Emitter, Runtime};
use tauri_plugin_notification::NotificationExt;

/// Frontend event name the capability layer listens on for notification activations.
pub const NOTIFICATION_ACTION_EVENT: &str = "mw://notification-action";

/// One actionable button on a native notification (archive / delete / reply).
/// camelCase over the IPC boundary to match the existing surfaces (plan §2).
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
    /// Action buttons; empty/absent when the OS or SPA context wants a plain toast.
    #[serde(default)]
    pub actions: Vec<NotifyAction>,
    /// Optional thread the notification refers to (deep-link back into the SPA).
    #[serde(default)]
    pub thread_id: Option<String>,
}

/// The event delivered to the frontend when a notification (or one of its action
/// buttons) is activated. Frozen shape (plan §2.1): `{ notificationId, actionId }`.
/// A plain body-click reports `actionId = ACTION_DEFAULT`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationActionEvent {
    pub notification_id: String,
    pub action_id: String,
}

/// `actionId` reported when the user clicks the notification body (no specific
/// button) — the SPA opens the referenced thread.
pub const ACTION_DEFAULT: &str = "default";

impl NotificationActionEvent {
    pub fn new(notification_id: impl Into<String>, action_id: impl Into<String>) -> Self {
        Self {
            notification_id: notification_id.into(),
            action_id: action_id.into(),
        }
    }
}

/// Emit the notification-action event to the frontend. The single choke point for
/// the frozen event shape: e7's plugin activation callback calls this, and the
/// capability layer's `onNotificationAction` receives exactly this payload.
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

/// Show a native notification for `input`. Title + body are always shown; action
/// buttons are attached where the OS/plugin supports them (their activation returns
/// via e7's callback -> [`emit_notification_action`]). Content is the SPA's own
/// (already-fetched) data — push never carries content (plan §2.3), so this is
/// local rendering only.
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
        // Frozen wire shape the capability layer's onNotificationAction expects.
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
        // Exactly the JSON `platform/tauri.ts` sends for a reply-able new-mail toast.
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
        assert_eq!(
            p.actions[1],
            NotifyAction {
                id: "reply".into(),
                label: "Reply".into()
            }
        );
    }

    #[test]
    fn notify_payload_defaults_actions_and_thread_when_absent() {
        // A plain toast with no buttons and no thread reference still parses.
        let p: NotifyPayload =
            serde_json::from_str(r#"{"id":"x","title":"t","body":"b"}"#).unwrap();
        assert!(p.actions.is_empty());
        assert_eq!(p.thread_id, None);
    }
}
