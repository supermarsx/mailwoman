// "Learn more" targets for admin fields (t32-cfg UX4).
//
// Each admin area documents its fields in `docs/admin/<area>.md`, one
// `<a id="<field-id>"></a>` anchor per field. The console is not served with the
// docs, so the link opens the file in the project repository; the path inside
// the URL stays `docs/admin/<area>.md#<field-id>`, which is what a link check
// reads (`docsAnchors.test.ts` resolves every call in the admin screens).

const REPO_DOCS = 'https://github.com/supermarsx/mailwoman/blob/master/';

/** The "Learn more" URL for a field (or the whole area when `id` is left out). */
export function adminDocsHref(area: string, id?: string): string {
  return `${REPO_DOCS}docs/admin/${area}.md${id ? `#${id}` : ''}`;
}
