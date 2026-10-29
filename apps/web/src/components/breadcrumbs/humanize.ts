// t95: the last-resort name of a breadcrumb level that has no translation.
//
// A breadcrumb never shows a raw dotted key (`mail.focusedInbox`), a kebab or
// snake id (`alias-new`, `max_bytes`) or a `+marker` segment. When no label
// exists, the segment is turned into words in sentence case instead:
// `mail.focusedInbox` -> `Mail focused inbox`, `alias-new` -> `Alias new`.
// The miss is logged once at debug level, so a missing label is visible to a
// developer without being shown to the admin.

const logged = new Set<string>();

/** Words in sentence case from a key or id. Returns '' for a segment with no letters or digits. */
export function humanize(raw: string): string {
  const words = raw
    .replace(/^[+~@#]+/, '')
    .replace(/([a-z0-9])([A-Z])/g, '$1 $2')
    .replace(/([A-Z]+)([A-Z][a-z])/g, '$1 $2')
    .split(/[\s._\-/:+]+/)
    .filter((w) => w !== '')
    .map((w) => (/^[A-Z0-9]{2,}$/.test(w) ? w : w.toLocaleLowerCase()));
  if (words.length === 0) return '';
  const text = words.join(' ');
  return text.charAt(0).toLocaleUpperCase() + text.slice(1);
}

/** Log, once per kind and key, that a breadcrumb level had no label. */
export function logMissingCrumbLabel(kind: string, key: string): void {
  const id = `${kind}:${key}`;
  if (logged.has(id)) return;
  logged.add(id);
  console.debug(`[breadcrumbs] no label for ${kind} "${key}", showing a humanized name`);
}

/** True when the value looks like an opaque id that no humanizing makes readable
 *  (a UUID, a long hex or base64url token, or a number). */
export function isOpaqueId(raw: string): boolean {
  return /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(raw)
    || /^[0-9a-f]{12,}$/i.test(raw)
    || /^\d+$/.test(raw)
    || (/^[A-Za-z0-9_-]{16,}$/.test(raw) && /\d/.test(raw) && !/[-_.]/.test(raw.slice(1, -1)) && /[a-z]/.test(raw) && /[A-Z0-9]/.test(raw));
}
