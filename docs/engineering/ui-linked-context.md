# Linked mailbox: request routing audit

Source of truth: `apps/web/src/api/linked-scope.json` (read by the web client and, through `include_str!`, by
`crates/mw-server/src/linked_accounts/scope.rs`). `apiFetch` (`apps/web/src/api/http.ts`) classifies every request
to the app's own server while a linked mailbox is active:

- **mailbox**: the link selector (`x-mailwoman-link`, or `?mwLink=` on a GET `resource`) is attached; the server runs the
  handler as the linked child. A selector with any other path is refused with 403 `linkedContextUnsupported`.
- **primary / public / signed**: sent without a selector, on purpose (account list, analytics consent, public config, signed image tokens).
- **anything else**: `LinkedContextUnsupportedError` is thrown in the browser. The request is never sent, so an unlisted call cannot
  silently run against the primary account.
- `blob:` and `data:` URLs name bytes the page already holds. They bypass this check.

Page-level navigations that do not go through `apiFetch` (`<a href>`, `window.location.assign`) cannot send a header, so they must
embed `?mwLink=` through `linkedResourceUrl`. Audited: attachment download/preview URLs (`buildDownloadUrl`, already did), folder ZIP
(`exportUrl`: the large-archive `window.location.assign` was the one navigation that would have run WITHOUT the selector, fixed now and
`/api/export/folder/<id>.zip` added to the mailbox scope as a resource).

## Findings

- No call silently targets the primary account while a linked mailbox is open. The only primary-scoped paths are the intentional ones in the table.
- Fixed: `/api/export/folder/*.zip` threw in a linked mailbox (folder ZIP export impossible) and its browser-level download had no selector.
- Fixed: `blob:`/`data:` URLs were classed unsupported (every attachment preview failed).
- Genuinely not available in a linked mailbox (by design, see `refused` in the manifest): password, 2FA, sessions, passkeys, zero-access, masked addresses,
  websocket/event-source (live mail comes through the accounts channel).
- Unsupported-and-reachable features, failing with reason `linked` where the caller uses `recoverRequest`; other callers see the thrown error (not individually verified):
  cookies/consent, privacy requests, retention, focused senders, list-unsubscribe, API keys, web version, features profile, client-error reports.
  Not yet given a bespoke in-place message: tracked as follow-ups, none of them can reach the primary.

## Call sites (admin console excluded: it has no linked context)

Paths are the literals found per file; `X` is an interpolated segment. Method defaults to GET when none is visible next to the literal.

| File | Endpoint | Method | Scope | In a linked mailbox |
|---|---|---|---|---|
| `api/accountDrafts.ts` | `/api/account/draftsX` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/accountPreferences.ts` | `/api/account/preferences` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/acl-types.ts` | `/jmap/api` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/analytics.ts` | `/api/analytics/status` | GET | primary | works as the SIGN-IN account by design (no selector; person-level data) |
| `api/analytics.ts` | `/api/analytics/events` | POST | primary | works as the SIGN-IN account by design (no selector; person-level data) |
| `api/analyticsTagConsent.ts` | `/api/analytics/status` | GET | primary | works as the SIGN-IN account by design (no selector; person-level data) |
| `api/appearanceCustomization.ts` | `/api/account/appearance/customization` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/attachmentPolicy.ts` | `/jmap/download` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/client.ts` | `/api/login` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/client.ts` | `/api/me` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/client.ts` | `/jmap/session` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/client.ts` | `/jmap/api` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/client.ts` | `/api/sanitize` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/client.ts` | `/api/logout` | POST | primary | works as the SIGN-IN account by design (no selector; person-level data) |
| `api/dataRequests.ts` | `/api/privacy/requestsX` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/focusedSenders.ts` | `/api/account/focused-senders` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/http.ts` | `/api/login` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/http.ts` | `/api/...` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/http.ts` | `/api/login/2fa` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/http.ts` | `/api/login/passkey/finish` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/http.ts` | `/api/me` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/http.ts` | `/jmap/session` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/http.ts` | `/api/security/step-up` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/jmap.ts` | `/jmap/upload/{accountId}` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/linkedAccounts.ts` | `/api/accountsX` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/mailBulk.ts` | `/api/export/${encodeURIComponent` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/mailTransfer.ts` | `/api/import/formats` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/mailTransfer.ts` | `/api/import/X` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/mailTransfer.ts` | `/api/export/${encodeURIComponent` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/mailingList.ts` | `/api/mail/list-unsubscribe` | POST | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/nextcloudAccounts.ts` | `/api/account/nextcloud` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/nextcloudAccounts.ts` | `/api/account/nextcloud/links/${encodeURIComponent` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/nextcloudPim.ts` | `/api/account/nextcloud/pimX` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/packages.ts` | `/api/account/packagesX` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/policy.ts` | `/api/policy` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/prefs.ts` | `/api/account/` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/prefs.ts` | `/api/account/appearance` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/privacy.ts` | `/api/privacy/overview` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/privacy.ts` | `/api/privacy/restriction` | POST | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/privacy.ts` | `/api/privacy/consents/${encodeURIComponent` | POST | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/privacy.ts` | `/api/account/sessions` | POST | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/privacy.ts` | `/api/account/sessions/revoke` | POST | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/privacy.ts` | `/api/keys` | POST | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/privacy.ts` | `/api/keys/${encodeURIComponent` | POST | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/receipts.ts` | `/api/account/mdn-policy` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/receipts.ts` | `/api/account/mdn-decision/${encodeURIComponent` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/receipts.ts` | `/api/account/mdn-senders` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/remote-content.ts` | `/api/rc/policy` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/remote-content.ts` | `/api/rc/sign` | POST | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/remote-content.ts` | `/api/rc/X/` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/remote-content.ts` | `/api/rc/stats` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/remote-content.ts` | `/api/rc/batch` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/remote-images.ts` | `/api/remote-images/X` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/remote-images.ts` | `/api/remote-images/grants` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/remote-images.ts` | `/api/image-proxy` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/retention.ts` | `/api/retentionX` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/senderResolution.ts` | `/api/account/sender/resolve` | POST | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/sessionActivity.ts` | `/api/account/session/activity` | POST | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/sessionCache.ts` | `/jmap/session` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/sessionCache.ts` | `/jmap/api` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/settings.ts` | `/api/settings` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/settings.ts` | `/api/settings/${encodeURIComponent` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `api/tracking.ts` | `/api/track/open` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `api/webVersion.ts` | `/api/version` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `components/PersonAvatar.tsx` | `/api/account/contact-photo` | POST | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `components/messageAttachmentDownload.ts` | `/jmap/download` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `components/messageMenuActions.ts` | `/api/export/${encodeURIComponent` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `components/compose/accountInsert.ts` | `/jmap/api` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `components/compose/accountInsert.ts` | `/jmap/session` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `components/compose/accountInsert.ts` | `/api/export/${encodeURIComponent` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `components/exportDialog/exportRun.ts` | `/api/export/${encodeURIComponent` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `components/properties/BulkProperties.tsx` | `/api/export/${encodeURIComponent` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `components/properties/MessageProperties.tsx` | `/api/export/${encodeURIComponent` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `components/properties/folderExport.ts` | `/api/export/folder/${encodeURIComponent` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `cookies/runtime.ts` | `/api/cookies/config` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `cookies/runtime.ts` | `/api/cookies/consent/account` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `cookies/runtime.ts` | `/api/cookies/consent` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `cookies/runtime.ts` | `/api/cookies/detected` | POST | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `modules/apikeys/service.ts` | `/api/keys` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `modules/apikeys/service.ts` | `/api/keys/${encodeURIComponent` | POST | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `modules/assist/service.ts` | `/api/assist/` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `modules/assist/service.ts` | `/api/assist/config` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `modules/assist/service.ts` | `/api/assist/invoke` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `modules/assist/service.ts` | `/api/assist/transcribe` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `modules/calendar/feeds.ts` | `/api/calendar/subscribe` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `modules/calendar/feeds.ts` | `/api/calendar/refresh` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `modules/directory/DirectorySearch.tsx` | `/api/directory/` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `modules/directory/service.ts` | `/api/directory/` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `modules/directory/service.ts` | `/api/directory/search` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `modules/directory/service.ts` | `/api/directory/group/${encodeURIComponent` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `modules/directory/service.ts` | `/api/directory/cert` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `modules/directory/service.ts` | `/api/directory/photo` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `modules/nextcloud/service.ts` | `/api/nextcloud/` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `modules/nextcloud/service.ts` | `/api/account/nextcloud` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `modules/nextcloud/service.ts` | `/api/nextcloud/files` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `modules/nextcloud/service.ts` | `/api/nextcloud/list` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `modules/nextcloud/service.ts` | `/api/nextcloud/attach` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `modules/nextcloud/service.ts` | `/api/nextcloud/save` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `modules/nextcloud/service.ts` | `/api/nextcloud/share-link` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `modules/nextcloud/service.ts` | `/api/nextcloud/upload` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `modules/nextcloud/service.ts` | `/api/nextcloud/upload/${encodeURIComponent` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `modules/passwd/service.ts` | `/api/password` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `modules/passwd/service.ts` | `/api/password/policy` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `modules/zeroaccess/pairing.ts` | `/api/zeroaccess/pair/offer` | POST | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `modules/zeroaccess/pairing.ts` | `/api/zeroaccess/pair/envelope` | POST | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `modules/zeroaccess/pairing.ts` | `/api/zeroaccess/pair/envelope/${encodeURIComponent` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `modules/zeroaccess/service.ts` | `/api/zeroaccess` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `modules/zeroaccess/service.ts` | `/api/zeroaccess/enable` | POST | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `modules/zeroaccess/service.ts` | `/api/zeroaccess/disable` | POST | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `observability/clientErrors.ts` | `/api/observability/client-errors` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `perf/clientTiming.ts` | `/api/policy` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `perf/clientTiming.ts` | `/api/client-timing` | GET | primary | works as the SIGN-IN account by design (no selector; person-level data) |
| `realtime/pushClient.ts` | `/jmap/ws` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `realtime/pushClient.ts` | `/jmap/eventsource` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `realtime/pushClient.ts` | `/jmap/session` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `screens/IdentityConsent/index.tsx` | `/oidc/consent` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `screens/Settings/BrowserPermissions.tsx` | `/api/push/permission` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `screens/Settings/BrowserPermissions.tsx` | `/api/push/subscribe` | POST | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `screens/Settings/BrowserPermissions.tsx` | `/api/push/unsubscribe` | POST | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `screens/Settings/SigninGeography.tsx` | `/api/security/signin-geography/history` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `screens/Settings/service.ts` | `/api/account/2fa` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `screens/Settings/service.ts` | `/api/account/2fa/totp/begin` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `screens/Settings/service.ts` | `/api/account/2fa/totp/confirm` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `screens/Settings/service.ts` | `/api/account/2fa/totp/disable` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `screens/Settings/service.ts` | `/api/account/2fa/passkey/begin` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `screens/Settings/service.ts` | `/api/account/2fa/passkey/finish` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `screens/Settings/service.ts` | `/api/account/2fa/passkey/remove` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `screens/Settings/service.ts` | `/api/account/2fa/recovery/regenerate` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `screens/Settings/service.ts` | `/api/login/2fa` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `screens/Settings/service.ts` | `/api/account/sessions` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `screens/Settings/service.ts` | `/api/account/sessions/revoke` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `screens/Settings/service.ts` | `/api/account/signatures` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `screens/Settings/service.ts` | `/api/account/signatures/${encodeURIComponent` | DELETE | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `screens/Settings/service.ts` | `/api/account/identities` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `screens/Settings/service.ts` | `/api/account/identities/${encodeURIComponent` | DELETE | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `screens/Settings/service.ts` | `/api/account/notifications` | PUT | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `screens/Settings/service.ts` | `/api/account/saved-searches` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `screens/Settings/service.ts` | `/api/account/saved-searches/${encodeURIComponent` | DELETE | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `state/features.ts` | `/api/features` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
| `state/slices/mail.ts` | `/api/policy` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `state/slices/mail.ts` | `/api/me` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `state/slices/nextcloud.ts` | `/api/account/nextcloud` | GET | mailbox | works: sent with the link selector (header; or ?mwLink for GET resources) and runs as the linked mailbox |
| `viewers/sandbox.ts` | `/api/rc/i/` | GET | unsupported | throws LinkedContextUnsupportedError before any request (fail closed; never reaches the primary) |
