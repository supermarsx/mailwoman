// Mailwoman iOS mobile-capability plugin — BEST-EFFORT SKELETON (plan t7 §3 e2, §1.9).
//
// ⚠️ DOCUMENTED GAP, NOT A V5 GATE. iOS needs macOS + Xcode + a paid Apple
// account to build/run, none of which exist on this Windows dev/CI machine. This
// file is a *skeleton* so the Rust `ios_plugin_binding!(init_plugin_mailwoman_mobile)`
// in `src/commands/mod.rs` has a documented counterpart; it is NOT compiled or
// verified here. The committed live push paths are UnifiedPush (Android) and Web
// Push (desktop/web); iOS/APNs is best-effort + mocked in CI (plan §1.7/§5, §9).
//
// To activate later (on a Mac): after `tauri ios init`, drop this under the
// generated `gen/apple/Sources/…`, add the APNs entitlement, and implement the
// command bodies. The command names must match the Rust `run_mobile_plugin`
// calls: getDistributor, registerUnifiedPush, unregisterUnifiedPush,
// takePendingShare, setBadge.

import Tauri
import UIKit
import UserNotifications

class MailwomanMobilePlugin: Plugin {
    // Share sheet / file open captured from the app's launch or an open-URL.
    private var pendingShare: JSObject?

    // iOS uses APNs, not UnifiedPush; `getDistributor` returns null so the
    // capability layer selects the APNs transport (mocked in CI).
    @objc public func getDistributor(_ invoke: Invoke) {
        invoke.resolve(["distributor": NSNull()])
    }

    // Register for remote notifications (APNs). The device token is delivered to
    // AppDelegate.didRegisterForRemoteNotificationsWithDeviceToken; a real impl
    // forwards it as the endpoint. Skeleton: report "pending".
    @objc public func registerUnifiedPush(_ invoke: Invoke) {
        DispatchQueue.main.async {
            UIApplication.shared.registerForRemoteNotifications()
        }
        invoke.resolve(["endpoint": NSNull(), "appId": "apns"])
    }

    @objc public func unregisterUnifiedPush(_ invoke: Invoke) {
        DispatchQueue.main.async {
            UIApplication.shared.unregisterForRemoteNotifications()
        }
        invoke.resolve()
    }

    @objc public func takePendingShare(_ invoke: Invoke) {
        let payload = pendingShare
        pendingShare = nil
        invoke.resolve(["payload": payload ?? NSNull()])
    }

    // iOS shows the app-icon badge natively.
    @objc public func setBadge(_ invoke: Invoke) {
        let count = (invoke.getObject()?["count"] as? Int) ?? 0
        DispatchQueue.main.async {
            UIApplication.shared.applicationIconBadgeNumber = max(0, count)
        }
        invoke.resolve(["supported": true])
    }
}

@_cdecl("init_plugin_mailwoman_mobile")
func initPlugin() -> Plugin {
    return MailwomanMobilePlugin()
}
