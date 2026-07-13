#!/usr/bin/env python3
"""CI-only: merge these tracked Android source templates + Gradle deps into the
`tauri android init`-generated gen/ project (plan t7 §3 e2/e4/e8; the human-readable
steps are in this directory's `README.md`).

`gen/android/` is generated (git-ignored) and needs a JDK — unavailable on the dev
machine (plan §1.11 / risk #1) — so this runs only in the `android-apk` CI job, which
is `continue-on-error`. It is best-effort and idempotent: each step is skipped if it
already applied, so a re-run (or a partially-merged tree) does not double-inject.

Steps:
  1. copy the custom Kotlin plugins into the app's java package
     (MailwomanMobilePlugin = e2, UnifiedPushReceiver = e2, FlagSecurePlugin = e4),
  2. add the Gradle deps (UnifiedPush connector + androidx.security-crypto),
  3. merge AndroidManifest.xml (share/file-handler/deep-link <intent-filter>s into
     <activity>, the UnifiedPush <receiver> into <application>, the runtime
     <uses-permission>s into <manifest>).

The injected manifest/Gradle snippets are kept in sync with the human-readable source
of truth `manifest-intents.xml` + `README.md` in this directory.
"""

from __future__ import annotations

import shutil
import sys
from pathlib import Path

# This script lives in apps/mobile/src-tauri/android-src/; the generated project is a
# sibling gen/android/ under src-tauri/.
SRC = Path(__file__).resolve().parent
GEN = SRC.parent / "gen" / "android"
JAVA_DIR = GEN / "app" / "src" / "main" / "java" / "com" / "mailwoman" / "mobile"
MANIFEST = GEN / "app" / "src" / "main" / "AndroidManifest.xml"
GRADLE = GEN / "app" / "build.gradle.kts"

KOTLIN_FILES = [
    "MailwomanMobilePlugin.kt",
    "UnifiedPushReceiver.kt",
    "FlagSecurePlugin.kt",
]

GRADLE_DEPS = [
    'implementation("org.unifiedpush.android:connector:2.4.0")',
    'implementation("androidx.security:security-crypto:1.1.0-alpha06")',
]

ACTIVITY_INTENT_FILTERS = """
        <!-- Mailwoman share target + file handlers + deep links (t7-e2, merged by android-src/merge.py). -->
        <intent-filter>
            <action android:name="android.intent.action.SEND" />
            <category android:name="android.intent.category.DEFAULT" />
            <data android:mimeType="text/plain" />
            <data android:mimeType="message/rfc822" />
            <data android:mimeType="*/*" />
        </intent-filter>
        <intent-filter>
            <action android:name="android.intent.action.VIEW" />
            <category android:name="android.intent.category.DEFAULT" />
            <category android:name="android.intent.category.BROWSABLE" />
            <data android:scheme="content" />
            <data android:scheme="file" />
            <data android:mimeType="message/rfc822" />
            <data android:mimeType="text/calendar" />
            <data android:mimeType="text/vcard" />
            <data android:mimeType="application/vnd.ms-outlook" />
        </intent-filter>
        <intent-filter android:autoVerify="false">
            <action android:name="android.intent.action.VIEW" />
            <category android:name="android.intent.category.DEFAULT" />
            <category android:name="android.intent.category.BROWSABLE" />
            <data android:scheme="mailto" />
            <data android:scheme="mailwoman" />
        </intent-filter>
"""

APPLICATION_RECEIVER = """
        <!-- UnifiedPush receiver (t7-e2, merged by android-src/merge.py). -->
        <receiver
            android:name="com.mailwoman.mobile.UnifiedPushReceiver"
            android:exported="true">
            <intent-filter>
                <action android:name="org.unifiedpush.android.connector.MESSAGE" />
                <action android:name="org.unifiedpush.android.connector.UNREGISTERED" />
                <action android:name="org.unifiedpush.android.connector.NEW_ENDPOINT" />
                <action android:name="org.unifiedpush.android.connector.REGISTRATION_FAILED" />
            </intent-filter>
        </receiver>
"""

MANIFEST_PERMISSIONS = """
    <!-- Mailwoman runtime permissions (t7-e2, merged by android-src/merge.py). -->
    <uses-permission android:name="android.permission.POST_NOTIFICATIONS" />
    <uses-permission android:name="android.permission.USE_BIOMETRIC" />
"""

# A stable marker so every injection is idempotent across re-runs.
MARKER = "android-src/merge.py"


def fail(msg: str) -> None:
    print(f"[android-merge] ERROR: {msg}", file=sys.stderr)
    sys.exit(1)


def copy_kotlin() -> None:
    print("[android-merge] 1/3 copying custom Kotlin plugins…")
    JAVA_DIR.mkdir(parents=True, exist_ok=True)
    for name in KOTLIN_FILES:
        src = SRC / name
        if not src.exists():
            fail(f"missing tracked Kotlin template {src}")
        shutil.copy2(src, JAVA_DIR / name)
        print(f"    copied {name}")


def add_gradle_deps() -> None:
    print("[android-merge] 2/3 adding Gradle dependencies…")
    if not GRADLE.exists():
        fail(f"{GRADLE} not found — did 'tauri android init' run?")
    text = GRADLE.read_text(encoding="utf-8")
    missing = [d for d in GRADLE_DEPS if d not in text]
    if not missing:
        print("    already present — skipping")
        return
    idx = text.rfind("dependencies {")
    if idx < 0:
        fail("no `dependencies {` block in build.gradle.kts")
    insert_at = text.index("\n", idx) + 1
    block = "".join(f"    {d}\n" for d in missing)
    GRADLE.write_text(text[:insert_at] + block + text[insert_at:], encoding="utf-8")
    for d in missing:
        print(f"    added {d}")


def _inject_before(text: str, closing_tag: str, snippet: str, what: str) -> str:
    idx = text.rfind(closing_tag)
    if idx < 0:
        fail(f"no {closing_tag} in AndroidManifest.xml")
    print(f"    injected {what} before {closing_tag}")
    return text[:idx] + snippet + text[idx:]


def merge_manifest() -> None:
    print("[android-merge] 3/3 merging AndroidManifest.xml…")
    if not MANIFEST.exists():
        fail(f"{MANIFEST} not found — did 'tauri android init' run?")
    text = MANIFEST.read_text(encoding="utf-8")
    if MARKER in text:
        print("    already merged — skipping")
        return
    text = _inject_before(text, "</activity>", ACTIVITY_INTENT_FILTERS, "intent-filters")
    text = _inject_before(text, "</application>", APPLICATION_RECEIVER, "UnifiedPush receiver")
    # Permissions go just before <application>, which is inside <manifest>.
    idx = text.find("<application")
    if idx < 0:
        fail("no <application> in AndroidManifest.xml")
    text = text[:idx] + MANIFEST_PERMISSIONS + "\n    " + text[idx:]
    print("    injected uses-permission entries")
    MANIFEST.write_text(text, encoding="utf-8")


def main() -> None:
    if not GEN.exists():
        fail(f"{GEN} not found — run `tauri android init` before this script")
    copy_kotlin()
    add_gradle_deps()
    merge_manifest()
    print("[android-merge] done — the tracked Kotlin + manifest + Gradle deps are merged.")


if __name__ == "__main__":
    main()
