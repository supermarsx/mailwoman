# Session completion ledger

Updated 2026-10-26. This ledger covers the requests made in this session. Assignment is not completion. A checkpoint commit preserves work; it does not establish runtime acceptance. Root coordinates shared files, reviews evidence and commits each coherent change. No production deployment or push has been performed in this continuation.

States: **Verified** means the bounded behavior listed has passing local acceptance evidence; **Partial** means implementation or acceptance remains; **Audit** means existing code must be checked against the full request before a completion claim. Live provider/deployment acceptance is separate from mock/local tests.

## Verified bounded deliveries

| Request | State and evidence | Remaining limits |
|---|---|---|
| SpamAssassin/Rspamd score widget | Verified, e512f80; parser/UI tests and Chromium/Firefox/phone | Header-reported scores, not independent scanning |
| Attachment tabs, EML/TNEF preview, file information and fingerprints | Verified bounded implementation, 3da161a/c272855/95cb42a; live browser matrix | 4 MiB message-preview and 64 MiB fingerprint bounds; more policy work assigned below |
| Outlook-style reply/forward quoting, user/admin configuration | Verified tests, 706bb6e | Further live compose acceptance remains in the final regression queue |
| External-recipient warning/confirmation and exact trusted domains | Verified, 1747c3c; 39 tests and three live browser layouts | Composer safeguard, not a server delivery block |
| Preferred-domain hint hidden by default, configurable opt-in, Email or username label, spacing | Verified, 3633f19/0abe18c; domain HTTP tests | Production deployment pending |
| Configurable initial-login cooldown and no automatic credential retries | Verified, d94472d/7bc4b05; native/UI checks | Cooldown is process-local; production alias timeout needs actual deployment diagnosis if still reproducible |
| Self-hosted anti-bot challenge | Verified local backend and six browser flows, 3533a9e/d5bb7eb | Physical native-shell acceptance unverified; off by default |
| Global branding editor, name/notice/logo/favicon/background | Verified, 8858770; store/API/browser checks | PWA icon variants, custom CSS and domain branding not included |
| Atomic conditional settings/branding writes | Verified, 6fe9983; SQLite/PostgreSQL and HTTP concurrency tests | Further settings usability audit below |
| Themed tooltips and retained mail surfaces/transitions | Verified bounded fixes, 89158d8/ea3af85 | Whole-app UX audit still required |
| Session activity renewal up to 15 days; strict admin sessions | Verified bounded implementation, 0b7b28e | Final integrated regression still required |
| Docker vendored dependency inclusion | Verified source stage, Cargo manifest loading and full runtime-static image, 2f04439; local container healthcheck and HTTP health/SPA 200 passed | Build captured source before the latest parallel feature changes; not a production deployment |

## Remaining work and ownership

| Request group (all constituent requests remain in scope) | Owner | State / next acceptance |
|---|---|---|
| Deployment: pull latest Git, dirty-tree handling, Compose wait compatibility, colored steps/commit information, Docker build speed | Root | Git/TTY/color and Compose2.30 resume/restart regression suites pass. POSIX sh invocation now delegates to Bash. Full static image, local health/SPA smoke and the real Docker lifecycle suite pass (2026-10-26): strict-CSRF admin/browser, snapshots/restore, corrupt backup refusal, health-failure rollback, exposure/network transitions, resource limits, GPG encryption, hooks/pruning and systemd unit validation. Age encryption was unavailable; signature flow uses a stand-in cosign, not production signing verification. Cache-speed acceptance remains. |
| Global brutalist/Bauhaus controls: inputs, dropdowns, multi-select, checkboxes, buttons, margins, field/button alignment, width/layout behavior | Root | Audit all screens; fix shared primitives and verify desktop/mobile/dark/reduced-motion |
| Dedicated editor pages/tabs: roles, domains, identities, signatures, SSO and other editable entities | Root | Audit; no residual inline entity editors where user requested dedicated views |
| Admin personal configuration/TOTP, global/admin/user configurability, effective configuration UX | Security + Root UI | Audit persistence, policy locks, navigation and account isolation |
| Domain list table, quick/guided add, aliases, enable/disable/edit/delete/clone, bulk/filter actions | Integrations + Root UI | Audit existing implementation and live flows |
| Per-domain multiple backends/priorities, IMAP/JMAP/EWS/365/Gmail and supported technologies | Integrations | Audit actual support; surface capabilities honestly; no unsupported MAPI claims |
| Domain discovery, authentication modes/username formats, auto/priority TLS, certificate checks, IP/dynamic hosts and SSH tunnels | Integrations | Audit runtime endpoints, alias inheritance and connection diagnostics |
| Automatic folder mapping including spam, admin per-domain/user overrides, pin/order/custom icons | Integrations + Root UI | Audit persistence and real IMAP mapping behavior |
| Diagnostics: domains, DNS, security, network, services/pipes, capabilities, server information, copyable report | Integrations + Security | Audit and expand; redact secrets in copied output |
| DLP: Portuguese identity cards, NIF, NISS, password/common keywords and configurable policies | Security | Audit validators, positives/negatives and enforcement |
| Fail2ban, Webmin/Virtualmin/Hestia domain/mailbox management, individual configuration and SSH tunneling | Security | Audit and implement real adapters; panel users signing in is out of scope per user choice |
| Security centre deployment checks/table, geo fencing, per-client/per-user limits, failed-password limits, session revoke/IP block/user alerts | Security | Audit configuration plus actual enforcement and admin tables |
| Login/audit logs, friendly linked actors, bounded retention, user-directory enrollment, admin session tables | Security | Audit real errors and rendering; preserve account-enumeration protection |
| Full passkeys and admin self-service TOTP | Security | Audit browser/platform flows, enrollment/recovery and policy enforcement |
| Dashboard data/configurability, settings table width/actions/side popover, privacy statistics tables | Root | Audit endpoint availability, responsive tables and scroll-aware UI |
| Message display dark adaptation/inversion, loaders and global/user controls | Root | Audit rendering isolation and persistence across themes |
| Retained message/compose/settings tabs, tiling, compact icon ribbon, collapsible/resizable columns and icon-only folders | Root | Audit all paths, state persistence, keyboard and narrow layouts |
| Trash empty, spam, archive, read/unread, pinning, informative toasts, bulk move/tag/forward/download | Root | Audit real backend effects, selection, undo semantics and feedback |
| Large folders, bounded/infinite/paged lists, message scrolling, fixed sidebar/viewport | Root | Stress-test large real/mock folders and prevent vertical overflow |
| Global search, full threading, Outlook-like filters, saved search folders | Root + Integrations | Audit feature depth and backend behavior, not only controls |
| Embedded OpenID Connect identity provider for other apps | Security | Audit existing provider work; complete protocol/security and relying-party acceptance |
| Linked child accounts with own identities, signatures, mappings and top-bar selector | Integrations | Audit parent/child authorization, isolated account state and switching |
| Nextcloud per-account files/attachments, contacts and calendars, admin/user controls | Integrations | Partial; 265999f/2e888ea/ed9aed3 provide sealed account-isolated links, per-instance private CA, guarded bounded file operations and dedicated admin/user configuration. TLS/HTTP authority, rejected-link invalidation, UI and browser acceptance pass. Multiple-instance selection, Reader saving, private-host grants and actual contacts/calendar synchronization remain. |
| Server-backed user settings/drafts and cross-device read/move state synchronization | Root + Integrations | Audit account identity, drafts and realtime conflict handling |
| CalDAV discovery, PWA/offline, browser storage permissions | Integrations + Security | Audit discovery, offline boundaries, permission UX and sync recovery |
| Extensive HTTP/2 and HTTP/3/QUIC deployment support | Integrations | Audit server/proxy configuration and real negotiated protocols |
| Open-source telemetry, extensive controls, off by default | Root | Verified bounded operational telemetry: persisted admin editor with revision/step-up, visible environment pins and restart state, bounded sampling/queues/timeouts; native3/HTTP1/browser3 and actual local OTel Collector0.163.2 ingestion pass. Request counts/durations and fixed request spans export; private query/event markers do not. Production collector validation remains deployment-specific. |
| Optional open-source analytics, off by default | Integrations | Verified local runtime, 40b827b/74a8d0b; native, store and three browser layouts pass. Real provider ingestion remains deployment-specific. |
| Quick replies, off by default and user/admin configurable | Root | Verified compact reply tabs, af880dc; 52 tests, TypeScript/lint and Chromium/Firefox/phone acceptance passed. Reuses full composer protections. |
| IMAP quota/usage/folder item and unread indicators | Integrations + Root UI | Audit real capabilities and per-user/admin toggles |
| Contact photos/profile lookup or generated avatars with privacy controls | Root | Audit local/default behavior and opt-in external access |
| Suspicious/external link protection and trusted-domain controls | Security + Root UI | Audit navigation interception, URL parsing and isolated previews |
| Vacation/auto reply where supported, admin/user controls | Integrations | Audit server capability detection and real persistence |
| Word/Outlook-like rich/plain/HTML editor, compact ribbon, Cc/Bcc, identity From and format-appropriate signatures | Root | Audit full compose/send/draft paths; encrypted Bcc limitations must remain explicit |
| Read receipts and RFC2369/list actions/latest relevant email standards | Mail processing + Integrations | Audit incoming/outgoing interpretation and opt-in/privacy behavior; enumerate standards rather than claim all |
| All requested JMAP RFC support and tagging aligned with JMAP | Integrations + Root tags | Build explicit RFC/capability matrix and acceptance tests; no blanket conformance claim |
| Incoming DSN interpretation | Mail processing | Verified bounded engine/proxy support, 961e507; MIME120, account isolation, HTTP authorization/limits, UI30 and six real-browser cases pass. Delivery reports do not imply a recipient read the message. |
| Email/folder tagging, extensive import/export, printing/PDF | Root + Mail processing | Audit tags/import/export; print dialog exists but dedicated acceptance pending |
| Attachment policies, suspicious types, isolated image/text/audio/video/PDF tabs, admin extension controls | Security | Verified bounded policy, 35b5ebc; stored metadata authority, hot CID policy, preview/download and whole-message export guards; native/HTTP and browser3 pass. Nextcloud export integration remains in its lane. |
| Notifications, push permission and per-user/admin controls | Security | Audit browser/native delivery, click actions, privacy and persistence |
| Package tracking, disabled by default, user/admin controls | Root | Pending implementation and privacy-aware provider policy |
| Sender/receiver timezone helpers for mail and events | Root | Pending implementation/verification; do not invent missing sender zones |
| Native consistent icons, 15 selectable folder icons, self-hosted fonts, extensive shortcuts and controls | Root | Keyboard bindings verified, fcbe710: 14 actions, dedicated editors, account persistence/admin locks, conflict prevention and retained-tab dispatch. Unit/native registry and browser3 pass. Icon/font inventory audit remains. |
| Custom CSS/user-directory drafts | Root | Partial, 3032a58; inherited modules remain unwired and not accepted |

## Completion rule

Each row needs evidence of implemented runtime behavior, reachable/configurable UI where requested, account/domain authorization and persistence where applicable, relevant tests, and honest external-service limits. Final closure requires integrated build/regression checks, a complete review of this ledger against the conversation, and explicit reporting of any deployment-only blockers. Commit continuously; do not silently turn a pending request into a deferred or completed one.
