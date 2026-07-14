# F-Droid recipe

`metadata/com.mailwoman.mobile.yml` is the fdroiddata recipe for the Android shell.
The file name is fixed to the applicationId `com.mailwoman.mobile`.

## Why F-Droid-friendly (SPEC §16)

Push uses **UnifiedPush**, not Firebase Cloud Messaging, so the app has **no Google
Play Services dependency** and builds cleanly in F-Droid's buildserver. The shell
carries no proprietary code — it is a verified web-bundle wrapper.

## Local lint (autonomous, no account)

```sh
# With the fdroidserver toolchain installed:
fdroid readmeta   # parse the metadata
fdroid lint com.mailwoman.mobile
```

CI validates the YAML parses (the fdroidserver toolchain is not on the runner).

## HUMAN-gated (before F-Droid presence)

- **Submission:** merge request to `gitlab.com/fdroid/fdroiddata` adding this
  metadata; F-Droid maintainers review the build recipe and anti-features.
- **Reproducible build:** the recipe must build from the tagged source commit in
  F-Droid's clean buildserver — verify `Builds[].commit`, the NDK version, and the
  prebuild (SPA build + `emit-bundle-hash` + `tauri android init` + `merge.py`) all
  succeed there.
- **Signing:** F-Droid signs with its own key; the recipe outputs the **unsigned**
  APK. (A separate Play Store track needs the Play upload key — see
  `docs/deploy/packaging.md`.)
