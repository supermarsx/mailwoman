# Scanner-path catalog

Two lists of paths decide which requests count as looking for something that is not there: paths of other web applications, administration panels and sensitive files that automated scanners request. The **Recognized scan paths** rule of request protection blocks a source on its first request to a path in the built-in catalog; probing protection counts a path on its editable **Probe paths** list as one miss. This page describes both lists and how a request path is matched. What happens after a match is described in [Aggregate and adaptive protection](./request-protection.md).

## The two lists

| | Built-in catalog | Probe paths |
|---|---|---|
| Used by | **Recognized scan paths** (`security.protection.probePathMode`) | probing protection, signal **A known scanner path** |
| Effect | the source is blocked on the first request | one miss; a block follows only after the grace allowance |
| Source | `crates/mw-protection/data/scan_probe_paths.json` (292 paths) plus rules in code | the setting `security.adminProbe.probePaths`, default from `crates/mw-protection/data/admin_probe_paths.json` (340 paths) |
| Editable | no | yes, on the **Probing** sub-tab |

Every path of the built-in catalog is also in the default **Probe paths** list. Editing **Probe paths** does not change the built-in catalog.

![The first two rules of the Detection and evidence group on the Probing sub-tab: the four counted misses, all selected, and the Probe paths list beginning with /+cscoe+, /.aws and /.env.](/screenshots/admin-access/scan-path-catalog-detection.webp){.mw-crop}

## What the built-in catalog covers

The catalog lists endpoints of applications that do not run on a Mailwoman origin, for example:

- content management systems: WordPress (`/wp-admin`, `/wp-login.php`, `/xmlrpc.php`, `/wp-content`), Joomla (`/administrator`, `components/com_*`, `modules/mod_*`), Drupal, TYPO3, Umbraco, Bitrix, Magento;
- webmail installers and mail administration panels;
- hosting and server panels, database tools such as phpMyAdmin;
- application-server and framework management endpoints, such as Spring `/actuator/env` or Laravel `/_ignition/execute-solution`;
- appliance and VPN endpoints, such as `/+CSCOE+`;
- exposed files: `.env` and its variants (`.env.local`, `.env.production.bak`, …), anything under `.git`, `.ssh` or `.aws`, `.htaccess`, `.gitconfig`, `wp-config.php` and backup copies (`.bak`, `.old`, `.orig`, `.save`, `~`), `config.php`, `install.php`, `setup.php`, `shell.php`;
- console guesses such as `/admin/users` and `/console`.

## How a path is matched

1. The query string and fragment are removed. Percent-encoding is decoded up to three times; a path that is still encoded after that, contains control characters or is longer than 8 KiB is not classified.
2. Backslashes become slashes, `.` and `..` segments are resolved, and the path is lower-cased.
3. A catalog entry matches when it appears as whole path segments anywhere in the path, so `/blog/wp-admin/` matches and `/wp-administration` does not. `/console` matches only as the last segment.
4. Paths that start with `/api`, `/jmap`, `/dav`, `/caldav` or `/carddav` are never classified, so a message attachment or contact called `.env` or `wp-config.php` does not count. A path containing a `cmd_sco` segment counts anywhere.
5. The console's own path is excluded. Below it, the rest of the path is matched on its own: with the console at `/mail-admin`, `/mail-admin/users` does not match, but `/mail-admin/wp-admin` does.

Generic paths such as `/login`, `/user/login`, health and metrics endpoints, OpenAPI and Swagger documents, and ordinary `/media`, `/components`, `/modules` or `/templates` folders are not in the catalog.

## Limits

- No list covers every application or renamed installation. A scanner that only requests paths outside the catalog is not blocked by this rule; probing protection and request budgets still apply.
- A match identifies a request for a foreign application on this server. It is not a claim that the request would have exploited anything.
- If you host another application under the same host name, its paths can match the catalog. Exempt the networks that use it (**Exempt addresses**), or set **Recognized scan paths** to **Observe**.

## Related

- [Aggregate and adaptive protection](./request-protection.md)
- [Crawler identification](./crawler-identification.md)
- [fail2ban](../deploy/fail2ban.md)
