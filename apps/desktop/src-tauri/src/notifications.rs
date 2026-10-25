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
//! Action delivery (t32 f25, D-23): `tauri-plugin-notification`'s desktop builder
//! shows a toast and drops the result, so a click did nothing. [`mw_notify`] shows
//! the toast through `notify-rust` (the crate the plugin itself uses) and keeps its
//! handle: a thread waits for the OS response (Windows toast activation, macOS
//! click, the XDG `ActionInvoked` signal), brings the main window to the front and
//! calls [`emit_notification_action`], the single choke point that serializes the
//! frozen event shape. The wait ends when the toast is dismissed or expires; a
//! click on an expired entry in the OS notification centre only brings the app up
//! (Windows starts or focuses it through the app id). Tests here pin the event
//! shape, the payload parsing and the response mapping.

use notify_rust::NotificationResponse;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, Runtime};

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

/// The `actionId` the SPA receives for an OS response, or `None` when the toast was
/// closed without being used. A body click is [`ACTION_DEFAULT`]; a button is its
/// id; a macOS inline reply opens the composer (`reply`). The XDG body click
/// arrives as the action key `default`, and notify-rust's Windows backend can
/// report a dismissal as `__closed`; both are mapped so every platform agrees.
pub fn activation_action(response: &NotificationResponse) -> Option<&str> {
    match response {
        NotificationResponse::Default => Some(ACTION_DEFAULT),
        NotificationResponse::Action(key) if key == "__closed" || key.is_empty() => None,
        NotificationResponse::Action(key) => Some(key.as_str()),
        NotificationResponse::Reply(_) => Some("reply"),
        NotificationResponse::Closed(_) => None,
    }
}

/// Bring the main window to the front after a notification was used.
fn focus_main<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// The OS notification for `input`, set up the way `tauri-plugin-notification`
/// sets up its own: on Windows the app id only for the installed app (a dev build
/// under `target/` has no registered app id and keeps the default), on macOS the
/// bundle id except in dev.
fn os_notification<R: Runtime>(
    app: &tauri::AppHandle<R>,
    input: &NotifyPayload,
) -> notify_rust::Notification {
    let mut n = notify_rust::Notification::new();
    n.summary(&input.title).body(&input.body).auto_icon();
    // XDG reports a body click only when a `default` action is offered.
    #[cfg(all(unix, not(target_os = "macos")))]
    n.action(ACTION_DEFAULT, &input.title);
    for action in &input.actions {
        n.action(&action.id, &action.label);
    }
    let identifier = app.config().identifier.clone();
    #[cfg(windows)]
    {
        let sep = std::path::MAIN_SEPARATOR;
        let installed = tauri::utils::platform::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|dir| dir.display().to_string()))
            .is_some_and(|dir| {
                !(dir.ends_with(&format!("{sep}target{sep}debug"))
                    || dir.ends_with(&format!("{sep}target{sep}release")))
            });
        if installed {
            n.app_id(&identifier);
        }
    }
    #[cfg(target_os = "macos")]
    {
        let _ = notify_rust::set_application(if tauri::is_dev() {
            "com.apple.Terminal"
        } else {
            &identifier
        });
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    let _ = identifier;
    n
}

/// Show a native notification for `input`. Title + body are always shown; action
/// buttons are attached where the OS supports them. A click on the toast or one of
/// its buttons focuses the app and reaches the SPA as `mw://notification-action`
/// (`{ notificationId: input.id, actionId }`). Content is the SPA's own
/// (already-fetched) data — push never carries content (plan §2.3), so this is
/// local rendering only.
#[tauri::command]
pub async fn mw_notify<R: Runtime>(
    app: tauri::AppHandle<R>,
    input: NotifyPayload,
) -> Result<(), String> {
    let notification = os_notification(&app, &input);
    let (shown_tx, shown_rx) = std::sync::mpsc::channel::<Result<(), String>>();
    // `show` blocks on D-Bus/WinRT and the response wait blocks until the toast is
    // used or closed, so both run on a thread of their own.
    std::thread::spawn(move || match notification.show() {
        Ok(handle) => {
            let _ = shown_tx.send(Ok(()));
            let _ = handle.wait_for_response(|response: &NotificationResponse| {
                if let Some(action) = activation_action(response) {
                    focus_main(&app);
                    let _ = emit_notification_action(&app, input.id.clone(), action);
                }
            });
        }
        Err(e) => {
            let _ = shown_tx.send(Err(format!("notify {}: {e}", input.id)));
        }
    });
    tauri::async_runtime::spawn_blocking(move || shown_rx.recv())
        .await
        .map_err(|e| format!("notify: {e}"))?
        .unwrap_or_else(|_| Err("notify: the notification thread stopped".to_string()))
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
    fn a_body_click_and_a_button_reach_the_spa_and_a_dismissal_does_not() {
        use notify_rust::CloseReason;
        let cases = [
            (NotificationResponse::Default, Some(ACTION_DEFAULT)),
            (
                NotificationResponse::Action("archive".into()),
                Some("archive"),
            ),
            // The XDG body click arrives as the `default` action key.
            (
                NotificationResponse::Action("default".into()),
                Some(ACTION_DEFAULT),
            ),
            (
                NotificationResponse::Reply("on my way".into()),
                Some("reply"),
            ),
            (NotificationResponse::Action("__closed".into()), None),
            (NotificationResponse::Action(String::new()), None),
            (NotificationResponse::Closed(CloseReason::Dismissed), None),
            (NotificationResponse::Closed(CloseReason::Expired), None),
        ];
        for (response, expected) in cases {
            assert_eq!(activation_action(&response), expected, "{response:?}");
        }
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
