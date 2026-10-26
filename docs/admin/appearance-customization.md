# Personal appearance customization

Mailbox accounts and signed-in administrators have separate saved customization documents. Open the mailbox Appearance settings or the administrator’s My preferences page. Custom accent colors are saved with ordinary appearance preferences and shaded against the current theme to retain visible controls and readable accent text.

Two global settings, `appearance.customCssEnabled` and `appearance.customJsEnabled`, default to false. Owners may save drafts while execution is disabled. Custom CSS is paused whenever an administrator holds any mailbox appearance field. Scripts cannot replace held fields, including indirectly through a mode change. Administrator-personal appearance is separate from mailbox appearance policy.

CSS accepts up to 16 KiB, 128 rules and 512 declarations. The parser allows simple selectors and visual properties. Selectors are scoped to the owner’s workspace, including `body`, `html` and `:root`. It rejects at-rules, network resources, attribute selectors, positioning, custom-property definitions and `!important`. It does not style message iframes, other owners, or dialogs outside the workspace root.

Scripts accept up to 16 KiB and run in an opaque-origin frame’s dedicated worker, with a 500 ms execution deadline and at most 32 updates. They receive only `context.dark` and `context.reducedMotion`. Call `appearance.set` with an object containing supported choices: `accent` (a named accent or six-digit hex), `density` (`compact`, `cozy`, `relaxed`), `font` (a supported interface font), or `mode` (`light`, `dark`). These effects are temporary; the script source is what is saved. This is not general application JavaScript: no mail, account identifiers, application DOM, network access or app-origin storage is provided. Browser isolation is not a promise of a hard browser-process memory quota.

The shell CSP is unchanged. Only the fixed public sandbox response has a purpose-specific CSP; user code is sent as data over a private message port and never interpolated into the frame HTML. Server-rendered bootstrap hashes normalize HTML line endings. CSS is applied through the stylesheet API, without allowing inline style text.

Documents use owner-bound requests and ETag compare-and-swap. A conflict keeps the editor draft until the owner explicitly reloads or resolves it. Runtime revalidation occurs on visibility changes and at most every 30 seconds while visible; failed or timed-out revalidation removes applied customization. Account changes remove styles and terminate workers immediately.

To recover from an unwanted customization, add `appearanceSafe=1` to the page query (before any `#` route). The workspace opens without custom CSS or scripts so they can be reset. This does not remove administrator appearance locks.
