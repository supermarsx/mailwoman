// Mailwoman iOS screen-capture DETECTION skeleton (SPEC §7.6 / plan §3 e4).
//
// HONEST POSTURE — read this first. iOS provides NO API to *prevent* a screenshot
// or screen recording of ordinary app content. (The only true exclusion is
// UITextField's `isSecureTextEntry`, scoped to a single secure text field — not a
// whole webview.) So on iOS Mailwoman does NOT claim capture prevention. What it can
// do, best-effort, is DETECT capture and react:
//   * `UIScreen.main.isCaptured` + `UIScreen.capturedDidChangeNotification` — true
//     while the screen is being recorded or AirPlay-mirrored. React by hiding
//     sensitive content behind a secure overlay until capture stops.
//   * `UIApplication.userDidTakeScreenshotNotification` — fires AFTER a screenshot
//     is taken (cannot block it); use it to inform the user / audit-log the event.
//   * App-switcher snapshot: blur/replace the window in `sceneWillResignActive` so
//     the multitasking thumbnail does not leak content.
//
// Because of this, the capability layer reports iOS as `{ supported: false }` for
// `setCaptureProtection` (so the SPA keeps the V4 watermark, same as Linux/browser),
// while this detection path is a SEPARATE, additive best-effort signal.
//
// NOT BUILT HERE. iOS needs macOS + Xcode + a paid Apple account, none of which are
// available on this Windows dev/CI machine (plan §1.9, §6 R1/R5). This file is a
// tracked skeleton to merge into the `tauri ios init`-generated Xcode project on a
// Mac; it is documented as a gap, not a V5 gate. See docs/security/screen-capture.md.

import Foundation
import UIKit

/// Best-effort screen-capture detection for the Mailwoman iOS shell.
///
/// Owns two observers (active recording + post-hoc screenshot) and exposes hooks the
/// shell wires to a secure overlay + the capability layer's audit signal. A real
/// integration would bridge `onCaptureStateChanged` to a Tauri event the SPA listens
/// on, and present/dismiss a blurred cover view over the webview.
final class ScreenCaptureDetection {
    /// Called whenever active screen recording / mirroring starts or stops.
    /// `isCaptured == true` means content is currently being captured.
    var onCaptureStateChanged: ((_ isCaptured: Bool) -> Void)?

    /// Called just after the user takes a screenshot (iOS cannot block it; this is a
    /// notify/audit signal only).
    var onScreenshotTaken: (() -> Void)?

    private var observers: [NSObjectProtocol] = []

    /// Current recording/mirroring state, per `UIScreen`.
    var isCaptured: Bool {
        UIScreen.main.isCaptured
    }

    func start() {
        let center = NotificationCenter.default

        observers.append(
            center.addObserver(
                forName: UIScreen.capturedDidChangeNotification,
                object: nil,
                queue: .main
            ) { [weak self] _ in
                guard let self else { return }
                self.onCaptureStateChanged?(UIScreen.main.isCaptured)
            }
        )

        observers.append(
            center.addObserver(
                forName: UIApplication.userDidTakeScreenshotNotification,
                object: nil,
                queue: .main
            ) { [weak self] _ in
                self?.onScreenshotTaken?()
            }
        )

        // Emit the initial state so callers can react to capture already in progress
        // at launch.
        onCaptureStateChanged?(isCaptured)
    }

    func stop() {
        let center = NotificationCenter.default
        for observer in observers {
            center.removeObserver(observer)
        }
        observers.removeAll()
    }

    deinit {
        stop()
    }
}
