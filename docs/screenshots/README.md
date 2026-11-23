# Documentation screenshots

Every screenshot in this manual is produced by a script from a manifest entry, so
it can be taken again when the interface changes. This page explains how to add
an entry, run it, check it and use the image on a page. It is written for people
who edit the documentation; it is not part of the user manual.

## Quick start

```sh
# 1. Add an entry to your area's manifest: docs/screenshots/manifest/<area>.json
# 2. Capture it (builds the web client from source on first use, about 2 minutes)
node docs/screenshots/capture.mjs --only <area>/<id>
# 3. Put the image on its page (see "Using an image on a page")
# 4. Check images, manifests and pages
node docs/screenshots/check.mjs --area <area>
# 5. Build the site (fails on broken links)
node docs/screenshots/docs-build.mjs
```

The image is written to `docs/public/screenshots/<area>/<id>.webp` and a page uses
it as `/screenshots/<area>/<id>.webp`. Plain `node` is enough: the harness loads
Playwright from `apps/web/node_modules`. `pnpm -C apps/web exec node ../../docs/screenshots/capture.mjs ...`
works too. Do not install anything.

## Files

| Path | What it is | Who edits it |
|---|---|---|
| `capture.mjs` | runs manifest entries and writes the images | harness owner |
| `stack.mjs` | starts and stops the application with demo data | harness owner |
| `check.mjs` | checks manifests, images, pages, alt text and size budgets | harness owner |
| `docs-build.mjs` | builds the site under the shared build lock | harness owner |
| `lib/` | shared code (locators, catalogue, seeding, IMAP, browser) | harness owner |
| `manifest/_core.json` | the example entries shown on this page | harness owner |
| `manifest/<area>.json` | your entries | one writer per file |
| `demo/people.json`, `demo/mail/`, `demo/pim.json`, `demo/seeds.json` | the demo data set | harness owner |
| `demo/extra/<area>.json` | extra seeds for your area | one writer per file |
| `meta/<area>.json` | written by `capture.mjs`: hash, size and versions of each image | nobody (generated) |
| `jargon.txt` | patterns that must not appear in reader pages | harness owner |

Edit only your own `manifest/<area>.json`, `demo/extra/<area>.json` and
`docs/public/screenshots/<area>/`.

## The two stacks

Each run starts the application on free ports with a scratch database and a
private copy of the server binary (`target/debug/mailwoman`, or the file named
by `MW_SHOTS_SERVER_BIN`). The web client is built from the current source into
`%TEMP%\mw-shots\web\<hash>`; if the source does not build at that moment
(someone is mid-edit), the newest earlier build is used and the run says so.

Runs that start at the same time share builds: one run builds each source
version (under the lock `web\<hash>.lock`), the others wait and then use the
finished build; a build is moved into place only when complete. Each run prints
the build it uses (`web build <hash>: source <commit>[+local changes], built
<time>`), and `meta/<area>.json` records it per image (`web`, `webRevision`,
`webBuiltAt`, `webFallback` when an earlier build stood in). `check.mjs` warns
when the images of one area come from different builds; capture the area again
to bring them in line.

| | Stack `E` (default) | Stack `M` |
|---|---|---|
| Mail backend | Greenmail in Docker (IMAP/SMTP), server in engine mode | the in-repo mock JMAP server, server in proxy mode |
| Needs Docker | yes | no |
| Folder roles, unread counts, threads, search | real | Archive/Drafts/Spam/Trash show as plain folders, no unread counts, no threads |
| Calendar, tasks, notes, contacts | yes | no (the screens show errors) |
| Use for | everything | a quick look when Docker is unavailable |

Use stack `E` unless you have a reason not to. Its container is called
`mw-shots-greenmail-<port>` (set `MW_SHOTS_CONTAINER_PREFIX` to change the
prefix) and is removed at the end of the run.

### Accounts

| Who | Sign-in | Password |
|---|---|---|
| Demo user | `alex@example.com` (Alex Rivera) | `demo-pass-alex` |
| Console administrator | `admin`, console at `/admin` | `demo-pass-admin` |

The harness signs both in once per stack, before the first capture, and reuses
their cookies. It then removes what its own sign-in leaves behind (the "new
browser or device" security notice mailed to the account, and the notification
entries), because they would carry today's date.

### Clock, locale and theme

- The browser clock is fixed at **Tuesday 10 March 2026, 09:30 UTC**, so "today",
  "yesterday" and the calendar week are the same on every run. The demo data is
  dated around that day. The server clock is not changed: anything the server
  stamps itself (sign-in history, request counts, "last seen") shows the real
  time. Mask it or leave it out of the crop.
- Locale `en-US`, time zone UTC, reduced motion, animations and the text caret off.
- Theme: dark. `"theme": "light"` sets the demo user's theme for that capture
  and puts it back afterwards (user screens only).
- Images are taken at 1x (1440x900 for `desktop`). The site shows images at
  their natural width, capped to the column, so a 1x crop appears at the size
  the control has in the application.

## The demo data

All people, organisations and addresses are invented. Addresses use
`example.com`, `example.org` and `example.net`; IP addresses, if you need any,
come from `192.0.2.0/24`, `198.51.100.0/24`, `203.0.113.0/24` or `2001:db8::/32`;
phone numbers use `+1 555 01xx`. No real brands, logos, people or account data.

Core seeds (`demo/seeds.json`):

| Seed | Content |
|---|---|
| `mail-basic` | 40 messages for Alex Rivera in Inbox (4 unread), Sent, Drafts, Archive, Spam, Trash and the folders Projects and Receipts; threads, a flagged message, a mailing list (City Library News), messages with PDF and PNG attachments, and one message that replaces an earlier one ("Agenda for Thursday (room change)" supersedes "Agenda for Thursday") |
| `pim-week` | stack `E` only: six events in the week of 9 March 2026, five tasks, three notes, eight contacts |
| `admin-basic` | the domains `example.com` and `example.org` |

The people are listed in `demo/people.json` (Jordan Ellis, Priya Raman, Sam Okoye,
Mei Lindqvist, Tomás Varga, Nadia Haddad, Lena Brooks, and senders for billing,
a newsletter and spam).

## A manifest entry

A manifest is either a list of entries or an object with `defaults` (merged into
every entry) and `entries`:

```json
{
  "defaults": { "stack": "E", "seed": ["mail-basic"], "page": "user/mailbox/composing.md" },
  "entries": [
    {
      "id": "compose-recipients",
      "steps": [
        { "goto": "/" },
        { "click": { "role": "button", "name": "@mail-compose" } },
        { "fill": { "role": "combobox", "name": "@mail-compose-to", "value": "priya.raman@example.com" } }
      ],
      "shot": { "element": { "role": "region", "name": "@mail-compose-header" }, "padding": 12 },
      "callouts": [
        { "n": 1, "target": { "role": "combobox", "name": "@mail-compose-to" }, "place": "bottom" }
      ],
      "alt": "The To and Subject fields of a new message, with one recipient entered."
    }
  ]
}
```

| Field | Default | Meaning |
|---|---|---|
| `id` | required | lowercase letters, digits and dashes; the file name of the image |
| `page` | required | the page (relative to `docs/`) that shows the image; `check.mjs` requires the reference |
| `alt` | required | one sentence saying what the image shows; never "Screenshot of ..." |
| `stack` | `"E"` | `"E"` or `"M"` |
| `seed` | `[]` | named seeds, applied in order (see "Seeding extra data") |
| `session` | `"user"` | `"user"` (signed in as Alex), `"admin"` (console), `"none"` (signed out, for the sign-in page) |
| `viewport` | `"desktop"` | `"desktop"` 1440x900, `"narrow"` 1024x768, `"phone"` 390x844, or `{ "width", "height" }` |
| `theme` | `"dark"` | `"dark"` or `"light"` |
| `steps` | `[]` | what to do before the capture (see "Steps"); without a `goto` the harness opens `/` (or the console) |
| `shot` | required | `"full"`, `{ "element": ... }` or `{ "clip": ... }` (see "Shots") |
| `callouts` | `[]` | numbered markers (see "Callouts") |
| `mask` | `[]` | locators painted over in grey (volatile values) |
| `hide` | `[]` | locators made invisible (a toast that is not the subject) |
| `settle` | `800` | milliseconds to wait after the steps |
| `clock` | 2026-03-10T09:30:00Z | another fixed browser time (ISO), or `false` for the real time |
| `scale` | `1` | device pixel ratio |
| `quality` | `0.9` | WebP quality to start from; lowered in steps of 0.05 until the image fits its budget |
| `watch` | `[]` | repository globs (`apps/web/src/screens/Settings/**`); `--changed-since` re-captures the entry when one of them changed |
| `isolate` | `false` | `true` gives the entry a fresh stack (use when its steps change server state that other entries would see) |
| `note` | | free text for other writers |

Entries in one run that share `stack` and `seed` share one stack and run in file
order, each in a new browser context. Steps that change server data (sending,
deleting, saving a setting) affect the entries after them; put such an entry
last or give it `"isolate": true`. The `setting` step undoes itself.

## Locators

A locator is an object that names one element. Prefer what the user sees:

1. **Role and name**: `{ "role": "button", "name": "@mail-reply" }`
2. **Label** of a form field: `{ "label": "@mail-compose-subject" }`
3. **Text**: `{ "text": "Re: Venue options for the launch evening" }`
4. `{ "placeholder": ... }`, `{ "title": ... }`, `{ "alt": ... }`
5. Fallbacks: `{ "testid": "..." }`, `{ "css": "..." }`. Avoid class names: the
   application's class names are generated and change between builds.

A value that starts with `@` is a key of the English catalogue
(`apps/web/locales/en/*.ftl`); the harness replaces it with the English text.
Use keys for every label of the interface, so the image breaks loudly when a
label changes and the page text (which quotes the same catalogue string) is
updated with it. Message subjects, names and other demo data are plain text.

To find a key, search the catalogue for the label as it appears on screen:

```sh
rg -n "= Reply all$" apps/web/locales/en/
# mail.ftl:138:mail-reply-all = Reply all
```

- `"@key.attr"` uses an attribute of the message.
- A message with variables (`Idle timeout in { $time }`) matches any value; pass
  `"args": { "time": "29:58" }` to fill them in instead.
- `"exact": false` matches a substring (exact is the default).

Narrowing:

| Option | Example | Effect |
|---|---|---|
| `within` | `{ "role": "button", "name": "@mail-reply", "within": { "role": "toolbar", "name": "@mail-reader-actions" } }` | search inside another element |
| `hasText` | `{ "role": "listitem", "hasText": "Invoice 2026-031" }` | keep elements containing this text |
| `nth`, `first`, `last` | `{ "text": "Inbox", "first": true }` | pick one of several matches |
| `closest` | `{ "role": "heading", "name": "@reader-card-text", "closest": "section" }` | the nearest enclosing `section` (or `fieldset`, `form`, `li`, ...) |

Playwright refuses a locator that matches several elements; the error names
them, and you add `within`, `hasText` or `first`.

To see roles and names, run the entry with `--hold` and use the browser's
accessibility tree, or print one: `--trace` saves a Playwright trace you can open
with `pnpm -C apps/web exec playwright show-trace <file>`.

## Steps

| Step | Example |
|---|---|
| `goto` | `{ "goto": "/" }`, `{ "goto": "/#/calendar" }`, `{ "goto": "{admin}/users" }` |
| `click`, `dblclick`, `rightclick`, `hover`, `focus` | `{ "click": { "role": "button", "name": "@mail-compose" } }` |
| `fill` | `{ "fill": { "label": "@mail-compose-subject", "value": "Booking page" } }` |
| `type` | like `fill`, key by key (for fields that react to typing) |
| `keys` | `{ "keys": "Hello Priya" }` types into whatever has focus (the message editor) |
| `press` | `{ "press": "Escape" }` or `{ "press": { "role": "combobox", "name": "@mail-compose-to", "key": "Enter" } }` |
| `check`, `uncheck`, `select` | `{ "select": { "label": "...", "value": "..." } }` |
| `waitFor`, `waitForHidden` | `{ "waitFor": { "text": "Print proof review" } }` |
| `scroll` | `{ "scroll": { "text": "Loading" } }`; add `"block": "start"` or `"center"` to align it |
| `wait` | `{ "wait": 500 }` (last resort; prefer `waitFor`) |
| `setting` | `{ "setting": { "key": "reader.zoom", "value": 125 } }` sets a personal setting for this capture only |
| `reload` | `{ "reload": true }` |
| `eval` | JavaScript run in the page; avoid, and say why in `note` |

Useful addresses in the application: `/` (Inbox), `/#/calendar`, `/#/tasks`,
`/#/notes`, `/#/contacts`, `/#/keys`, `/#/templates`, `/#/notifications`,
`/#/attachments`, `/#/outbox`. Settings has no address: click
`{ "role": "button", "name": "@mail-nav-settings" }` and then the section, for
example `{ "click": { "text": "@reader-settings-title", "first": true } }`.
The console is at `{admin}`.

End with a `waitFor` on something that only exists once the screen has loaded
(a heading, a row of demo data). Before the capture the harness moves the
pointer to the bottom right corner, so hover effects disappear, unless a step
is a `hover`.

## Shots

| Shot | Use | Budget |
|---|---|---|
| `"full"` | orientation shot of a whole screen (the first image on a screen page) | 220 KB |
| `{ "element": <locator>, "padding": 8 }` | a crop around one control, card or panel | 80 KB |
| `{ "element": [<locator>, <locator>] }` | a crop around several elements | 80 KB |
| `{ "clip": { "x", "y", "width", "height", "relativeTo": <locator> } }` | a fixed rectangle, in CSS pixels, from the top left of the window or of an element | 80 KB |

An element crop paints its padding in the page background, so no slivers of
neighbouring controls appear; `"clean": false` in `shot` keeps them. A crop
grows to include its callouts.

## Callouts

```json
"callouts": [
  { "n": 1, "target": { "role": "button", "name": "@mail-reply" }, "place": "bottom" },
  { "n": 2, "target": { "role": "button", "name": "@mail-reply-all" }, "place": "bottom", "box": true }
]
```

A callout is a yellow disc with a black number, drawn beside its target.
`place` is `right` (default), `left`, `top`, `bottom`, `top-left`, `top-right`,
`inside-left` or `inside-right`; `dx` and `dy` move it by a few pixels; `"box": true`
also outlines the target. Every image with callouts is followed on its page by a
legend (see below); `check.mjs` counts its items.

## Seeding extra data

When your screens need state the core seeds do not have, add a named seed to
`demo/extra/<area>.json` and list it in your entries' `seed`:

```json
{
  "seeds": {
    "settings-large-text": {
      "include": ["mail-basic"],
      "userSettings": { "reader.fontSize": 18 },
      "admin": [
        { "method": "PUT", "path": "/domains/example.net", "body": {} }
      ]
    }
  }
}
```

Keys of a seed, applied in this order:

| Key | Effect |
|---|---|
| `include` | other seeds applied first |
| `stacks` | `["E"]` if the seed only works on stack E |
| `mail` | mail data files under `demo/` (same format as `demo/mail/basic.json`), loaded before the server starts |
| `adminSettings` | `{ "<setting key>": value }`, written as the console administrator |
| `admin` | `[{ "method", "path", "body" }]` requests to the console API; `path` is under the console path (`/domains/example.net`) |
| `userSettings` | `{ "<setting key>": value }`, written as the demo user |
| `user` | `[{ "method", "path", "body" }]` requests as the demo user (`/api/...`) |
| `jmap` | `[{ "using": [...], "calls": [[method, args, id], ...] }]` JMAP calls as the demo user |
| `pim` | calendar/task/note/contact files under `demo/` (format of `demo/pim.json`) |

Strings may contain these placeholders:

::: v-pre
| Placeholder | Becomes |
|---|---|
| `{{me}}` | `alex@example.com` |
| `{{admin}}` | the console path |
| `{{person:priya}}` | an address from `demo/people.json` |
| `{{date:+3d}}`, `{{date:-2h}}`, `{{date:+30m}}` | an ISO time relative to the fixed clock |
:::

(The `{admin}` in a `goto` step is the same console path, with single braces.)

Setting keys and their values are listed in `crates/mw-settings/src/settings_registry.rs`
and on the [settings reference](/admin/settings). Console API routes are
registered in `crates/mw-server/src/admin_router.rs` and the modules it calls
(look for `admin_endpoint(Method::..., "/path", ...)`). The browser sends the
CSRF token for you.

Need a person who is not in `demo/people.json`? Ask the harness owner to add
one, so every area uses the same cast.

Rules for extra data: only invented people and the reserved domains and number
ranges above; reuse the people in `demo/people.json` where you can; no real
company or product names other than Mailwoman. A mail or calendar file of your
own goes next to your seeds, as `demo/extra/<area>-mail.json` (format of
`demo/mail/basic.json`) or `demo/extra/<area>-pim.json` (format of
`demo/pim.json`), and the seed points at it: `"mail": "extra/<area>-mail.json"`.
A file in `demo/extra/` that has no `seeds` key is data, not seeds.

## Running and debugging

```sh
node docs/screenshots/capture.mjs --manifest <area>             # every entry of one area
node docs/screenshots/capture.mjs --only <area>/<id>[,<id>...]  # some entries
node docs/screenshots/capture.mjs --all                         # every manifest
node docs/screenshots/capture.mjs --all --changed-since <git rev>
node docs/screenshots/capture.mjs --manifest <area> --dry-run   # validate and list
```

`--changed-since <rev>` captures entries that are new or changed since that
commit, whose image is missing, or whose `watch` globs match a changed file.

When an entry fails, the run prints the step that failed and saves the page as it
was to `%TEMP%\mw-shots\failures\<area>-<id>.png`; the other entries continue and
the exit code is 1. To look around:

| Option | Effect |
|---|---|
| `--headed` | show the browser window |
| `--hold` | keep the stack and the browser open after the last entry (Ctrl-C ends); the address is printed |
| `--trace` | save a Playwright trace per entry to `%TEMP%\mw-shots\traces\` |
| `--keep-run-dirs` | keep the scratch run folder (server and mock logs, database) |
| `--web-dir <dir>` | use this web build instead of building one |

`node docs/screenshots/stack.mjs E --seed mail-basic,pim-week` starts a stack on
its own, applies seeds and prints the address and accounts, so you can explore
the application in your own browser. Ctrl-C stops it.

Only four capture runs may run at once on this machine (each starts a server, a
container and a browser). The harness takes one of four slots in
`%TEMP%\mw-shots\slots` and waits when all are taken.

## Using an image on a page

```md
![The mailbox: folders on the left, the Inbox message list in the middle, and an opened message on the right.](/screenshots/core/mailbox.webp){.mw-shot}
```

- `{.mw-shot}` for full shots, `{.mw-crop}` for crops. `check.mjs` checks the class.
- The alt text says what the image shows. Use the entry's `alt`.
- The orientation shot goes at the top of a screen's page; a crop goes directly
  after the step it explains, never without text around it.
- After an image with callouts, add the legend: an ordered list, then
  `{.mw-callouts}` on the line after it:

```md
![The message actions toolbar, with markers 1 to 3 on Reply, Reply all and Forward.](/screenshots/core/reader-actions.webp){.mw-crop}

1. **Reply** answers the sender.
2. **Reply all** answers the sender and every other recipient.
3. **Forward** sends the message on to someone else.
{.mw-callouts}
```

- Pages that describe a screen set `screen: true` in their front matter;
  `check.mjs` then requires an orientation shot on them.

## Checks

```sh
node docs/screenshots/check.mjs                # every area
node docs/screenshots/check.mjs --area <area>  # yours
```

It reports: invalid manifest fields; unknown seeds; an entry without an image or
whose page does not show it; an image a page shows without an entry; an image
without an entry; missing alt text; the wrong class; a callout image without a
complete legend; a `screen: true` page without an orientation shot; images over
their budget (full 220 KB, crop 80 KB, all images together 30 MB).

Pages must also pass the wording check:

```sh
rg -n -f docs/screenshots/jargon.txt <your pages>
```

## Building the site

A plain `pnpm -C docs build` writes to `docs/.vitepress/dist` and
`docs/.vitepress/cache`, which every writer shares. Build with a private output
and cache folder instead, either through the helper:

```sh
node docs/screenshots/docs-build.mjs              # output in a scratch folder, removed afterwards
node docs/screenshots/docs-build.mjs --out <dir>  # keep the output
```

or directly, from `docs/`:

```sh
node node_modules/vitepress/dist/node/cli.js build . --outDir <scratch>/out --cacheDir <scratch>/cache
```

The helper uses its own output and cache folders and also holds the lock
`docs/.vitepress/.docs-build.lock` (a folder with the owner's process id inside;
a second build waits, and a lock left by a process that no longer runs is taken
over), so at most one build runs at a time on this machine. Do not run
`pnpm -C docs build` while others are writing.

## Things you will see

- **"DMARC not present"** on every opened demo message (stack E): the engine
  checks DKIM, SPF and DMARC of incoming mail, and the demo messages are not
  signed. That is correct behaviour; describe it if your page is about it, or
  crop around it.
- **Server-stamped times** (sign-in history, sessions, request history, idle
  timers) show the real date and change between runs: crop them out or `mask` them.
- **"Loading..."**: add a `waitForHidden` on it, or a `waitFor` on the content.
- A run repeats an image exactly when nothing on screen changed; areas you mask
  may differ.

## Examples

These are the entries of `manifest/_core.json`.

The mailbox (`mailbox`, full shot, stack E, seed `mail-basic`):

![The mailbox: folders on the left, the Inbox message list in the middle, and an opened message with a PDF attachment on the right.](/screenshots/core/mailbox.webp){.mw-shot}

The message actions toolbar with callouts (`reader-actions`, element crop):

![The message actions toolbar above an opened message, with markers 1 to 3 on Reply, Reply all and Forward.](/screenshots/core/reader-actions.webp){.mw-crop}

1. **Reply**
2. **Reply all**
3. **Forward**
{.mw-callouts}

A new message (`compose`, full shot after `click`, `fill` and `press` steps):

![A new message with one recipient and a subject, the editor ribbon open on Format text, and an empty message body.](/screenshots/core/compose.webp){.mw-shot}

One settings card (`settings-text-colors`, element crop found with `closest`):

![The Text and colors card of the Message display settings: message colors, reading font size, line height, reading width and message zoom.](/screenshots/core/settings-text-colors.webp){.mw-crop}

The console (`admin-dashboard`, `session: "admin"`, seed `admin-basic`, a `clip`
of the top of the window, which leaves out the server's request counts):

![The top of the administration console: the navigation on the left, and on the dashboard the setup checklist and the account and session counts.](/screenshots/core/admin-dashboard.webp){.mw-crop}

The calendar (`calendar-week`, seed `pim-week`, stack E):

![The calendar in week view for 8 to 14 March 2026 with the week's events, and the month overview and calendar list on the left.](/screenshots/core/calendar-week.webp){.mw-shot}
