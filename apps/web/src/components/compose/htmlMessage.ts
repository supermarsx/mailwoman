/** Content defaults belong in the email as well as its editing preview. */
export const HTML_MESSAGE_FONT = '14px/1.5 system-ui,sans-serif';

/** Assemble the already-sanitized editor fragment after signatures and other
 * content have been appended. Editor sandbox policies, chrome and theme colors
 * belong to the preview only; recipients receive an ordinary HTML document. */
export function htmlMessageDocument(fragment: string): string {
  const body = document.createElement('body');
  body.style.margin = '0';
  body.style.font = HTML_MESSAGE_FONT;
  // Serialize only the container. Keep the sanitized content byte-for-byte,
  // including its scoped styles and CID URLs, without parsing it again.
  const content = body.outerHTML.replace('</body>', () => fragment + '</body>');
  return '<!doctype html>\n<html><head><meta charset="utf-8">' +
    '<meta name="viewport" content="width=device-width, initial-scale=1"></head>' +
    content + '</html>';
}

/** Editor-only markers that must never reach a draft or a recipient. The
 * reply answer marker (`data-mw-native-answer`) is legacy: older builds set it
 * on the first paragraph together with spacing, which Enter then cloned. */
const LEGACY_ANSWER=/<p\b([^>]*\sdata-mw-native-answer(?:=("[^"]*"|'[^']*'|[^\s>]*))?[^>]*)>/gi;
export function stripEditorMarkers(html: string): string {
  return html
    .replace(LEGACY_ANSWER, (_all, attrs: string) => {
      const kept = attrs
        .replace(/\sdata-mw-native-answer(?:=("[^"]*"|'[^']*'|[^\s>]*))?/gi, '')
        .replace(/\sstyle=("[^"]*"|'[^']*')/gi, (_s, value: string) => {
          const css = value.slice(1, -1).split(';').filter((rule) => rule.trim() !== '' && !/^\s*(?:margin|padding|min-(?:block-size|height)|line-height)\b/i.test(rule)).join(';');
          return css === '' ? '' : ` style="${css}"`;
        });
      return `<p${kept}>`;
    })
    .replace(/\sdata-mw-(?:native-answer|native-caret|vc)(?:=("[^"]*"|'[^']*'|[^\s>]*))?/gi, '');
}
