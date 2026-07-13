// Mailwoman UnifiedPush receiver (Tauri v2 · plan t7 §3 e2, §2.3).
//
// UnifiedPush delivers distributor events (endpoint grant/rotation, wake
// messages) as broadcasts to this receiver — INDEPENDENTLY of whether the
// webview/plugin is alive. It:
//   * caches the endpoint (SharedPreferences) so `registerUnifiedPush` can
//     return it synchronously and the app survives restarts, and
//   * forwards live events to `MailwomanMobilePlugin` (via `PushBridge`) when the
//     app is foregrounded, so `tauri.ts` can POST the endpoint to
//     `/api/push/subscribe` and refetch `/changes` on a wake.
//
// PRIVACY (§2.3): a wake message carries NO content — only "something changed,
// wake + fetch". We never read message bytes as mail content; we trigger a
// foreground JMAP `/changes` refetch (or a minimal notification when backgrounded).
//
// Registered in the manifest (see `android-src/manifest-intents.xml`). Requires
// the `org.unifiedpush.android:connector` Gradle dependency (see README).

package com.mailwoman.mobile

import android.content.Context
import android.content.SharedPreferences
import org.unifiedpush.android.connector.MessagingReceiver

private const val PREFS = "mailwoman_push"
private const val KEY_ENDPOINT = "up_endpoint"

/// Process-wide bridge between the (always-on) receiver and the (lifecycle-bound)
/// Tauri plugin. Holds a reference to the live plugin plus the cached endpoint.
object PushBridge {
    /// UnifiedPush instance id — a single instance per app is sufficient here.
    const val instance: String = "default"

    @Volatile
    var plugin: MailwomanMobilePlugin? = null

    private fun prefs(context: Context): SharedPreferences =
        context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)

    fun cachedEndpoint(context: Context): String? =
        prefs(context).getString(KEY_ENDPOINT, null)

    fun onEndpoint(context: Context, endpoint: String, instance: String) {
        prefs(context).edit().putString(KEY_ENDPOINT, endpoint).apply()
        plugin?.emitEndpoint(endpoint, instance)
    }

    fun clearEndpoint(context: Context) {
        prefs(context).edit().remove(KEY_ENDPOINT).apply()
    }

    fun onWake() {
        // Ask a live SPA to refetch. When the app is backgrounded the plugin is
        // null; the OS-level notification (posted below) is the fallback nudge.
        plugin?.emitWake()
    }
}

class UnifiedPushReceiver : MessagingReceiver() {
    override fun onNewEndpoint(context: Context, endpoint: String, instance: String) {
        PushBridge.onEndpoint(context, endpoint, instance)
    }

    override fun onRegistrationFailed(context: Context, instance: String) {
        // Leave the cached endpoint in place; the SPA falls back to WS/SSE while
        // foregrounded. e6 surfaces the failure in the push settings UI.
    }

    override fun onUnregistered(context: Context, instance: String) {
        PushBridge.clearEndpoint(context)
    }

    override fun onMessage(context: Context, message: ByteArray, instance: String) {
        // Opaque wake ONLY — message content is never treated as mail (§2.3).
        PushBridge.onWake()
    }
}
