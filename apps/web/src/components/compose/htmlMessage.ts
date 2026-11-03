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
