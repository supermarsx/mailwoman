// "Learn more" targets for admin fields (t32-cfg UX4).
//
// Each admin area documents its fields in `docs/admin/<area>.md`, one
// `<a id="<field-id>"></a>` anchor per field. Open the published, themed manual;
// GitHub's Markdown view does not load the manual's theme, navigation or search.
// `docsAnchors.test.ts` still checks every target against its Markdown source.

const MANUAL = 'https://supermarsx.github.io/mailwoman/';

/** The "Learn more" URL for a field (or the whole area when `id` is left out). */
export function adminDocsHref(area: string, id?: string): string {
  return `${MANUAL}admin/${area}${id ? `#${id}` : ''}`;
}
