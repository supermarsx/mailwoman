#!/bin/sh
# stamp-version.sh — single-source the release version across every packaging
# manifest and native-shell config.
#
# The ONE source of truth is `[workspace.package] version` in the root Cargo.toml.
# This script reads it and writes it into: the winget manifests, the Flatpak
# AppStream release, the F-Droid metadata (versionName + numeric versionCode +
# CurrentVersion), both Tauri configs, and the desktop/mobile package.json.
#
# Idempotent: running it twice produces no diff (every replacement rewrites the
# whole version token, so re-running with an already-stamped tree is a no-op).
#
# The coordinator runs this after bumping the workspace version at release time;
# any developer can run it to prove the manifests are drift-free:
#
#     sh scripts/stamp-version.sh            # stamp to the current workspace version
#     git diff --exit-code -- packaging apps # must be clean if already single-sourced
#
# POSIX sh; no non-stdlib tools beyond sed/mktemp.
set -eu

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# --- read the single source of truth: [workspace.package] version ---
VER="$(sed -n '/^\[workspace\.package\]/,/^\[/{s/^version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p;}' Cargo.toml | head -n1)"
if [ -z "${VER:-}" ]; then
  echo "stamp-version: could not read [workspace.package] version from Cargo.toml" >&2
  exit 1
fi

# Android versionCode: a monotonic integer derived from the rolling YY.N.P version
# (major*10000 + minor*100 + patch), e.g. 26.11.0 -> 261100.
MAJOR="${VER%%.*}"
REST="${VER#*.}"
MINOR="${REST%%.*}"
PATCH="${REST#*.}"
PATCH="${PATCH%%.*}"
CODE=$(( MAJOR * 10000 + MINOR * 100 + PATCH ))

echo "stamp-version: workspace version = $VER (android versionCode $CODE)"

# stamp FILE EXPR...  — apply sed expressions to FILE via a temp file (portable
# across GNU/BSD sed, no in-place-flag divergence).
stamp() {
  f="$1"; shift
  if [ ! -f "$f" ]; then
    echo "stamp-version: missing $f" >&2
    exit 1
  fi
  tmp="$(mktemp)"
  sed "$@" "$f" > "$tmp"
  mv "$tmp" "$f"
  echo "  stamped $f"
}

# --- winget (three manifests + the doc comment path) ---
stamp packaging/winget/Mailwoman.Mailwoman.yaml \
  -e "s/^PackageVersion: .*/PackageVersion: $VER/" \
  -e "s#Mailwoman/Mailwoman/[0-9][^/]*/#Mailwoman/Mailwoman/$VER/#"
stamp packaging/winget/Mailwoman.Mailwoman.locale.en-US.yaml \
  -e "s/^PackageVersion: .*/PackageVersion: $VER/"
stamp packaging/winget/Mailwoman.Mailwoman.installer.yaml \
  -e "s/^PackageVersion: .*/PackageVersion: $VER/" \
  -e "s#desktop/[0-9][^/]*/Mailwoman_[0-9][^_]*_#desktop/$VER/Mailwoman_${VER}_#g"

# --- Flatpak AppStream release ---
stamp packaging/flatpak/com.mailwoman.Mailwoman.metainfo.xml \
  -e "s#<release version=\"[^\"]*\"#<release version=\"$VER\"#"

# --- F-Droid metadata (versionName + versionCode + commit + CurrentVersion) ---
stamp packaging/fdroid/metadata/com.mailwoman.mobile.yml \
  -e "s/^\(  - versionName: \).*/\1$VER/" \
  -e "s/^\(    versionCode: \).*/\1$CODE/" \
  -e "s/^\(    commit: \).*/\1'$VER'/" \
  -e "s/^\(CurrentVersion: \).*/\1$VER/" \
  -e "s/^\(CurrentVersionCode: \).*/\1$CODE/"

# --- Helm chart (appVersion tracks the app; the chart `version` is independent
#     and is deliberately NOT stamped) + the README example image tags ---
stamp packaging/helm/mailwoman/Chart.yaml \
  -e "s/^appVersion: .*/appVersion: \"$VER\"/"
stamp packaging/helm/README.md \
  -e "s#mailwoman:[0-9][^[:space:]]*#mailwoman:$VER#g" \
  -e "s#--set image.tag=[0-9][^[:space:]]*#--set image.tag=$VER#g"

# --- native shells: both Tauri configs + both package.json ---
# (tauri.conf.json files are owned outside this task's commit scope, but stamping
#  them keeps the single-source guarantee whole; they are already at $VER so this
#  is a no-op unless the workspace version moves.)
for conf in apps/desktop/src-tauri/tauri.conf.json apps/mobile/src-tauri/tauri.conf.json; do
  stamp "$conf" -e "s/\"version\": \"[^\"]*\"/\"version\": \"$VER\"/"
done
for pkg in apps/desktop/package.json apps/mobile/package.json; do
  stamp "$pkg" -e "s/\"version\": \"[^\"]*\"/\"version\": \"$VER\"/"
done

echo "stamp-version: done — all manifests at $VER"
