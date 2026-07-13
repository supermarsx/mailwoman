// Mailwoman Android screen-capture protection plugin (SPEC §7.6 / plan §3 e0 → e4).
//
// The REAL native version of V4's honest web watermark: setting FLAG_SECURE on the
// activity window makes the OS exclude it from screenshots, screen recording, and
// the recents thumbnail. There is NO core Tauri API for this, so it lives in a
// small custom Kotlin plugin the capability layer's `setCaptureProtection` invokes.
//
// e0 ships the SKELETON; e4 fills the command registration + the capability wiring
// (and the honest-degrade matrix: Linux/iOS/browser fall back to the watermark).

package com.mailwoman.mobile

import android.view.WindowManager
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke

@TauriPlugin
class FlagSecurePlugin(private val activity: android.app.Activity) : Plugin(activity) {
    /// Enable/disable FLAG_SECURE on the activity window. e4 wires this to the
    /// frozen `setCaptureProtection(enabled)` capability method (§2.1).
    @Command
    fun setCaptureProtection(invoke: Invoke) {
        val enabled = invoke.parseArgs(Args::class.java).enabled
        activity.runOnUiThread {
            if (enabled) {
                activity.window.setFlags(
                    WindowManager.LayoutParams.FLAG_SECURE,
                    WindowManager.LayoutParams.FLAG_SECURE,
                )
            } else {
                activity.window.clearFlags(WindowManager.LayoutParams.FLAG_SECURE)
            }
        }
        invoke.resolve()
    }

    class Args {
        var enabled: Boolean = false
    }
}
