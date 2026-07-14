# winget recipe

Three-file winget manifest (schema 1.6.0):

- `Mailwoman.Mailwoman.yaml` — version manifest
- `Mailwoman.Mailwoman.installer.yaml` — installer manifest (x64 + arm64 NSIS)
- `Mailwoman.Mailwoman.locale.en-US.yaml` — default-locale manifest

## Local validate (autonomous, no account)

```powershell
winget validate --manifest packaging\winget
# or, against a full install test:
winget install --manifest packaging\winget
```

CI validates the three files parse as YAML. Full `winget validate` needs Windows +
the winget client and is a local/release step.

## HUMAN-gated (before winget presence)

- **Signed installer:** the `InstallerUrl` must serve an **Authenticode-signed** NSIS
  installer (see `bundle.windows.certificateThumbprint` in `tauri.conf.json`), and
  `InstallerSha256` must be its real hash. winget rejects unsigned/hash-mismatched
  installers.
- **ProductCode:** fill `ProductCode` from the built NSIS installer.
- **Submission:** PR to `github.com/microsoft/winget-pkgs` under
  `manifests/m/Mailwoman/Mailwoman/26.8.0/`; the Microsoft validation pipeline +
  a maintainer review gate the merge.
