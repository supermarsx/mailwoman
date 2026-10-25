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
// PRIVACY (§2.3): a plain wake carries NO content — only "something changed,
// wake + fetch"; it triggers a foreground JMAP `/changes` refetch. When the
// server's notification rules decide a new message should be announced (t32 e48),
// the message is the JSON `{"mw":1,"notify":{"id","title","body"[,"channelName"]}}`,
// whose text the server has already limited to what its content policy allows;
// that notification is posted here (also while no webview runs) and the wake is
// forwarded as before. Anything that is not that JSON (e.g. the `mw-wake` marker)
// is a wake only. Message bytes are never parsed as mail.
//
// Registered in the manifest (see `android-src/manifest-intents.xml`). Requires
// the `org.unifiedpush.android:connector` Gradle dependency (see README).

package com.mailwoman.mobile

import android.content.Context
import android.content.SharedPreferences
import org.json.JSONObject
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
        wakeNotification(message)?.let { n ->
            MailNotifications.post(context, n.id, n.title, n.body, emptyList(), n.channelName)
        }
        PushBridge.onWake()
    }
}

/// The notification a wake asks for (see the PRIVACY note above).
data class WakeNotification(
    val id: String,
    val title: String,
    val body: String,
    val channelName: String,
)

/// Parse `{"mw":1,"notify":{...}}`; null for a plain wake or anything else.
fun wakeNotification(message: ByteArray): WakeNotification? {
    if (message.isEmpty() || message[0] != '{'.code.toByte()) return null
    return try {
        val root = JSONObject(String(message, Charsets.UTF_8))
        if (root.optInt("mw") != 1) return null
        val n = root.optJSONObject("notify") ?: return null
        val id = n.optString("id")
        val title = n.optString("title")
        if (id.isEmpty() || title.isEmpty()) return null
        WakeNotification(id, title, n.optString("body"), n.optString("channelName", "Mail"))
    } catch (e: Exception) {
        null
    }
}
