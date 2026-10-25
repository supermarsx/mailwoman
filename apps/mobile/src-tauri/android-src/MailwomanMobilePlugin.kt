// Mailwoman Android mobile-capability plugin (Tauri v2 · plan t7 §3 e2).
//
// Backs the frozen `Platform` capability methods that need native Android glue:
//   * UnifiedPush registration     → registerUnifiedPush / unregisterUnifiedPush / getDistributor
//   * Share targets & file handlers → takePendingShare (+ the `shareTarget` event)
//   * Badge counts                 → setBadge
//   * New-mail notifications       → notify (+ setNotificationActionHandler: a tap or
//     an action button reaches the Rust side over a Tauri Channel, t32 f25 / D-23)
//   * Secure store (OS keychain)   → keychainGet / keychainSet / keychainDelete
//     (Android EncryptedSharedPreferences, AES-256 keyed by the Android Keystore —
//     the mobile counterpart of desktop's `keyring` DPAPI/Keychain/Secret Service)
//
// This is the counterpart the Rust `commands` plugin (src/commands/) calls via
// `run_mobile_plugin(<command>, …)`. Screen-capture protection (FLAG_SECURE) is
// a SEPARATE plugin owned by e4 (`FlagSecurePlugin.kt`) — not here.
//
// GRADLE DEPENDENCY: keychain* needs `androidx.security:security-crypto` (Apache-2.0)
// — record it in the license gate; merge step in `android-src/README.md`.
//
// BUILD NOTE (plan §1.11 / e0 probe): this Windows machine has the Android
// SDK/NDK but NO JDK, so this file cannot be compiled locally. It is compiled by
// e8's CI `android-apk` job after `tauri android init`. Merge steps + the
// required Gradle dependency (`org.unifiedpush.android:connector`) are in
// `android-src/README.md`.

package com.mailwoman.mobile

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.util.Base64
import android.webkit.WebView
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Channel
import app.tauri.plugin.JSObject
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin
import org.json.JSONArray
import org.json.JSONObject
import org.unifiedpush.android.connector.UnifiedPush

private const val BADGE_CHANNEL_ID = "mailwoman_badge"
private const val BADGE_NOTIFICATION_ID = 0x4D57 // 'MW'
// Guard: never inline more than 8 MiB of shared file bytes over the IPC bridge;
// larger shares are passed as a content URI for the SPA to stream.
private const val MAX_INLINE_BYTES = 8 * 1024 * 1024

@InvokeArg
class SetBadgeArgs {
    var count: Int = 0
}

@InvokeArg
class NotificationActionHandlerArgs {
    lateinit var handler: Channel
}

/// Posts new-mail notifications for the plugin (`notify`) and for the UnifiedPush
/// receiver (a wake that carries a notification, posted while no webview runs).
/// A tap opens the app with the notification id and action in the intent extras;
/// the plugin reads them back in `load` / `onNewIntent`.
object MailNotifications {
    private const val CHANNEL_ID = "mailwoman_mail"
    private const val EXTRA_ID = "com.mailwoman.mobile.NOTIFICATION_ID"
    private const val EXTRA_ACTION = "com.mailwoman.mobile.NOTIFICATION_ACTION"
    const val ACTION_DEFAULT = "default"

    /// Show one notification. `actions` are (id, label) pairs for the buttons;
    /// `channelName` is the user-visible name of the Android channel.
    fun post(
        context: Context,
        id: String,
        title: String,
        body: String,
        actions: List<Pair<String, String>>,
        channelName: String,
    ) {
        val nm = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            nm.createNotificationChannel(
                NotificationChannel(CHANNEL_ID, channelName, NotificationManager.IMPORTANCE_DEFAULT),
            )
        }
        val builder = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            Notification.Builder(context, CHANNEL_ID)
        } else {
            @Suppress("DEPRECATION")
            Notification.Builder(context)
        }
        builder
            .setSmallIcon(context.applicationInfo.icon)
            .setContentTitle(title)
            .setContentText(body)
            .setAutoCancel(true)
            .setContentIntent(tapIntent(context, id, ACTION_DEFAULT))
        for ((actionId, label) in actions) {
            builder.addAction(
                Notification.Action.Builder(
                    null as android.graphics.drawable.Icon?,
                    label,
                    tapIntent(context, id, actionId),
                ).build(),
            )
        }
        nm.notify(id.hashCode(), builder.build())
    }

    /// The (notification id, action id) a tap put into `intent`, or null. The
    /// extras are removed so a recreated activity does not deliver them twice.
    fun activation(intent: Intent?): Pair<String, String>? {
        val id = intent?.getStringExtra(EXTRA_ID) ?: return null
        val action = intent.getStringExtra(EXTRA_ACTION) ?: ACTION_DEFAULT
        intent.removeExtra(EXTRA_ID)
        intent.removeExtra(EXTRA_ACTION)
        return id to action
    }

    private fun tapIntent(context: Context, id: String, action: String): PendingIntent {
        val launch = context.packageManager.getLaunchIntentForPackage(context.packageName)
            ?: Intent().setPackage(context.packageName)
        launch.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_SINGLE_TOP)
        launch.putExtra(EXTRA_ID, id)
        launch.putExtra(EXTRA_ACTION, action)
        // One PendingIntent per (notification, action): the request code keeps them
        // apart, so one button's extras never replace another's.
        return PendingIntent.getActivity(
            context,
            (id + "\u0000" + action).hashCode(),
            launch,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )
    }
}

@InvokeArg
class KeychainGetArgs {
    lateinit var service: String
    lateinit var key: String
}

@InvokeArg
class KeychainSetArgs {
    lateinit var service: String
    lateinit var key: String
    lateinit var value: String
}

@InvokeArg
class KeychainDeleteArgs {
    lateinit var service: String
    lateinit var key: String
}

@TauriPlugin
class MailwomanMobilePlugin(private val activity: android.app.Activity) : Plugin(activity) {

    // A payload captured from a share/open intent, awaiting `takePendingShare`.
    @Volatile
    private var pendingShare: JSObject? = null

    // Where notification taps go (set by the Rust side at start), and a tap that
    // arrived before it was set (the app was started by the tap).
    private var notificationActionHandler: Channel? = null
    private var pendingNotificationAction: Pair<String, String>? = null

    override fun load(webView: WebView) {
        super.load(webView)
        PushBridge.plugin = this
        // The launch intent may itself be a share / file-open, or a notification tap.
        captureShareIntent(activity.intent)
        captureNotificationIntent(activity.intent)
    }

    // Tauri forwards new intents (e.g. a share while the app is already running).
    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        captureShareIntent(intent)
        captureNotificationIntent(intent)
    }

    // ── New-mail notifications ───────────────────────────────────────────────

    @Command
    fun notify(invoke: Invoke) {
        try {
            val args = invoke.getArgs()
            val actions = mutableListOf<Pair<String, String>>()
            val list = args.optJSONArray("actions")
            if (list != null) {
                for (i in 0 until list.length()) {
                    val a = list.optJSONObject(i) ?: continue
                    actions.add(a.optString("id") to a.optString("label"))
                }
            }
            MailNotifications.post(
                activity,
                args.optString("id"),
                args.optString("title"),
                args.optString("body"),
                actions,
                args.optString("channelName", "Mail"),
            )
            invoke.resolve(JSObject())
        } catch (e: Exception) {
            invoke.reject("notify: ${e.message}")
        }
    }

    @Command
    fun setNotificationActionHandler(invoke: Invoke) {
        val args = invoke.parseArgs(NotificationActionHandlerArgs::class.java)
        val pending = synchronized(this) {
            notificationActionHandler = args.handler
            pendingNotificationAction.also { pendingNotificationAction = null }
        }
        pending?.let { (id, action) -> sendNotificationAction(args.handler, id, action) }
        invoke.resolve(JSObject())
    }

    private fun captureNotificationIntent(intent: Intent?) {
        val (id, action) = MailNotifications.activation(intent) ?: return
        val handler = synchronized(this) {
            notificationActionHandler ?: run {
                pendingNotificationAction = id to action
                null
            }
        }
        handler?.let { sendNotificationAction(it, id, action) }
    }

    private fun sendNotificationAction(handler: Channel, id: String, action: String) {
        handler.send(JSObject().apply {
            put("notificationId", id)
            put("actionId", action)
        })
    }

    // ── Share targets + file handlers ────────────────────────────────────────

    @Command
    fun takePendingShare(invoke: Invoke) {
        val payload = pendingShare
        pendingShare = null
        // Tauri's `resolve` needs a JSObject, so wrap in `{ payload: <obj|null> }`;
        // the Rust side reads `.payload` and maps JSON null → None.
        invoke.resolve(JSObject().apply { put("payload", payload ?: JSONObject.NULL) })
    }

    private fun captureShareIntent(intent: Intent?) {
        if (intent == null) return
        val payload = when (intent.action) {
            Intent.ACTION_SEND -> shareFromSend(intent)
            Intent.ACTION_VIEW -> shareFromView(intent)
            else -> null
        } ?: return
        pendingShare = payload
        // Also push it to a live webview so an already-open SPA reacts immediately.
        trigger("shareTarget", payload)
    }

    private fun shareFromSend(intent: Intent): JSObject? {
        val out = JSObject()
        intent.getStringExtra(Intent.EXTRA_SUBJECT)?.let { out.put("title", it) }
        intent.getStringExtra(Intent.EXTRA_TEXT)?.let { out.put("text", it) }
        @Suppress("DEPRECATION")
        val stream: Uri? = intent.getParcelableExtra(Intent.EXTRA_STREAM)
        stream?.let { uri ->
            readFile(uri)?.let { out.put("files", JSONArray().put(it)) }
        }
        return if (out.length() == 0) null else out
    }

    private fun shareFromView(intent: Intent): JSObject? {
        val uri = intent.data ?: return null
        val file = readFile(uri) ?: return null
        return JSObject().apply { put("files", JSONArray().put(file)) }
    }

    // Read a content:// or file:// URI into a `{ name, mime, bytesB64 }` object.
    // e6's `tauri.ts` decodes `bytesB64` → Uint8Array for the frozen
    // ShareTargetPayload. Oversized files carry `contentUri` instead of bytes.
    private fun readFile(uri: Uri): JSObject? {
        val resolver = activity.contentResolver
        val mime = resolver.getType(uri) ?: guessMime(uri)
        val name = queryDisplayName(uri) ?: uri.lastPathSegment ?: "shared"
        val obj = JSObject().apply {
            put("name", name)
            put("mime", mime)
        }
        return try {
            resolver.openInputStream(uri)?.use { input ->
                val bytes = input.readBytes()
                if (bytes.size <= MAX_INLINE_BYTES) {
                    obj.put("bytesB64", Base64.encodeToString(bytes, Base64.NO_WRAP))
                } else {
                    obj.put("contentUri", uri.toString())
                }
                obj
            }
        } catch (e: Exception) {
            obj.put("contentUri", uri.toString())
            obj
        }
    }

    private fun queryDisplayName(uri: Uri): String? {
        return try {
            activity.contentResolver.query(uri, null, null, null, null)?.use { c ->
                val idx = c.getColumnIndex(android.provider.OpenableColumns.DISPLAY_NAME)
                if (idx >= 0 && c.moveToFirst()) c.getString(idx) else null
            }
        } catch (e: Exception) {
            null
        }
    }

    private fun guessMime(uri: Uri): String = when (uri.toString().substringAfterLast('.').lowercase()) {
        "eml" -> "message/rfc822"
        "ics" -> "text/calendar"
        "vcf" -> "text/vcard"
        "msg" -> "application/vnd.ms-outlook"
        else -> "application/octet-stream"
    }

    // ── Badge ────────────────────────────────────────────────────────────────
    //
    // Android has no universal launcher-badge API. We update a low-importance
    // notification carrying `setNumber(count)`; whether it renders as a numeric
    // badge, a dot, or nothing is launcher-dependent (honest — reported as
    // `supported = true` only in that the API call succeeded). count == 0 clears.

    @Command
    fun setBadge(invoke: Invoke) {
        val args = invoke.parseArgs(SetBadgeArgs::class.java)
        val count = args.count.coerceAtLeast(0)
        val nm = activity.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                val ch = NotificationChannel(
                    BADGE_CHANNEL_ID,
                    "Unread badge",
                    NotificationManager.IMPORTANCE_MIN,
                )
                ch.setShowBadge(true)
                nm.createNotificationChannel(ch)
            }
            if (count == 0) {
                nm.cancel(BADGE_NOTIFICATION_ID)
            } else {
                val builder = android.app.Notification.Builder(activity, BADGE_CHANNEL_ID)
                    .setSmallIcon(activity.applicationInfo.icon)
                    // The app label, which Tauri generates from productName
                    // (MW_SHELL_PRODUCT_NAME at build time, t32).
                    .setContentTitle(activity.applicationInfo.loadLabel(activity.packageManager))
                    .setContentText("$count unread")
                    .setNumber(count)
                    .setOngoing(false)
                nm.notify(BADGE_NOTIFICATION_ID, builder.build())
            }
            invoke.resolve(JSObject().apply { put("supported", true) })
        } catch (e: Exception) {
            invoke.resolve(JSObject().apply { put("supported", false) })
        }
    }

    // ── Secure store (OS keychain) ───────────────────────────────────────────
    //
    // EncryptedSharedPreferences (AES-256-GCM value encryption, keyed by a MasterKey
    // in the hardware-backed Android Keystore) is the mobile counterpart of desktop's
    // `keyring` (DPAPI / Keychain / Secret Service). The `service` namespaces the
    // store into a distinct prefs file (`mailwoman.session` vs `mailwoman.secure`) so
    // the bearer token and the key-vault-passphrase wrap never collide; `key` is the
    // item within it. The Rust `commands::keychain` module calls these via
    // `run_mobile_plugin`.

    private fun securePrefs(service: String): android.content.SharedPreferences {
        // Sanitize the service into a safe prefs file name.
        val fileName = "mw_" + service.replace(Regex("[^A-Za-z0-9_.-]"), "_")
        val masterKey = MasterKey.Builder(activity)
            .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
            .build()
        return EncryptedSharedPreferences.create(
            activity,
            fileName,
            masterKey,
            EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
            EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
        )
    }

    @Command
    fun keychainGet(invoke: Invoke) {
        val args = invoke.parseArgs(KeychainGetArgs::class.java)
        try {
            val value = securePrefs(args.service).getString(args.key, null)
            invoke.resolve(JSObject().apply {
                put("value", value ?: JSONObject.NULL)
            })
        } catch (e: Exception) {
            invoke.reject("keychain get ${args.service}/${args.key}: ${e.message}")
        }
    }

    @Command
    fun keychainSet(invoke: Invoke) {
        val args = invoke.parseArgs(KeychainSetArgs::class.java)
        try {
            securePrefs(args.service).edit().putString(args.key, args.value).apply()
            invoke.resolve(JSObject())
        } catch (e: Exception) {
            invoke.reject("keychain set ${args.service}/${args.key}: ${e.message}")
        }
    }

    @Command
    fun keychainDelete(invoke: Invoke) {
        val args = invoke.parseArgs(KeychainDeleteArgs::class.java)
        try {
            securePrefs(args.service).edit().remove(args.key).apply()
            invoke.resolve(JSObject())
        } catch (e: Exception) {
            invoke.reject("keychain delete ${args.service}/${args.key}: ${e.message}")
        }
    }

    // ── UnifiedPush ──────────────────────────────────────────────────────────

    @Command
    fun getDistributor(invoke: Invoke) {
        val distributor = UnifiedPush.getDistributor(activity)
        invoke.resolve(JSObject().apply {
            put("distributor", if (distributor.isNullOrEmpty()) JSONObject.NULL else distributor)
        })
    }

    @Command
    fun registerUnifiedPush(invoke: Invoke) {
        // Ensure a distributor is selected. If none is saved, auto-pick the only
        // installed one (a UI picker for the multi-distributor case is an e6/e7
        // follow-up; a single self-hosted distributor is the common path).
        if (UnifiedPush.getDistributor(activity).isNullOrEmpty()) {
            val available = UnifiedPush.getDistributors(activity)
            if (available.isEmpty()) {
                invoke.reject("no UnifiedPush distributor installed")
                return
            }
            UnifiedPush.saveDistributor(activity, available.first())
        }
        UnifiedPush.registerApp(activity)
        // The endpoint is delivered asynchronously to UnifiedPushReceiver, which
        // caches it and fires `unifiedpush://new-endpoint`. Return any cached one.
        val cached = PushBridge.cachedEndpoint(activity)
        invoke.resolve(JSObject().apply {
            put("endpoint", cached ?: JSONObject.NULL)
            put("appId", PushBridge.instance)
        })
    }

    @Command
    fun unregisterUnifiedPush(invoke: Invoke) {
        UnifiedPush.unregisterApp(activity)
        PushBridge.clearEndpoint(activity)
        invoke.resolve(JSObject())
    }

    // Called by UnifiedPushReceiver (via PushBridge) when an endpoint arrives.
    fun emitEndpoint(endpoint: String, instance: String) {
        trigger("newEndpoint", JSObject().apply {
            put("transport", "unifiedpush")
            put("endpoint", endpoint)
            put("keys", JSONObject.NULL)
            put("appId", instance)
            put("expiresAt", JSONObject.NULL)
        })
    }

    // Called on an opaque push wake — the SPA responds with a JMAP `/changes`
    // refetch (frozen §2.3). No content is carried.
    fun emitWake() {
        trigger("pushWake", JSObject())
    }
}
