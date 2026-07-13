// Mailwoman Android screen-capture protection plugin (SPEC §7.6 / plan §3 e4).
//
// The REAL native version of V4's honest web watermark: setting FLAG_SECURE on the
// activity window makes the OS exclude it from screenshots, screen recording, and
// the recents thumbnail. There is NO core Tauri API for this, so it lives in this
// small custom Kotlin plugin the capability layer's `setCaptureProtection` invokes.
//
// Wiring (documented for e7 / the generated Android project):
//   1. After `tauri android init`, copy this file to
//      `gen/android/app/src/main/java/com/mailwoman/mobile/FlagSecurePlugin.kt`.
//   2. Register it in the generated `MainActivity` (Tauri v2 registers plugins in
//      Kotlin via the RustPlugin/`onCreate` hook or `tauri.conf.json` `plugins`):
//         override fun onCreate(savedInstanceState: Bundle?) {
//             super.onCreate(savedInstanceState)
//         }
//      and add `FlagSecurePlugin::class` to the app's plugin registry (the
//      generated `generatedPlugins` list). e7 owns the final registration; this file
//      owns the command logic only.
//
// Contract: the capability method is `setCaptureProtection(enabled)` and it resolves
// `{ supported: true }` on Android — FLAG_SECURE is a genuine OS control here, unlike
// Linux/iOS/browser which return `{ supported: false }` and keep the watermark.

package com.mailwoman.mobile

import android.app.Activity
import android.view.WindowManager
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

@InvokeArg
internal class SetCaptureProtectionArgs {
    var enabled: Boolean = false
}

@TauriPlugin
class FlagSecurePlugin(private val activity: Activity) : Plugin(activity) {
    /**
     * Enable or disable FLAG_SECURE on the activity window, backing the frozen
     * `Platform.setCaptureProtection(enabled)` capability (§2.1). Window flags must
     * be mutated on the UI thread. Resolves `{ supported: true }` because Android
     * genuinely enforces the exclusion.
     */
    @Command
    fun setCaptureProtection(invoke: Invoke) {
        val args = invoke.parseArgs(SetCaptureProtectionArgs::class.java)
        activity.runOnUiThread {
            if (args.enabled) {
                activity.window.setFlags(
                    WindowManager.LayoutParams.FLAG_SECURE,
                    WindowManager.LayoutParams.FLAG_SECURE,
                )
            } else {
                activity.window.clearFlags(WindowManager.LayoutParams.FLAG_SECURE)
            }
        }
        val result = JSObject()
        result.put("supported", true)
        invoke.resolve(result)
    }
}
