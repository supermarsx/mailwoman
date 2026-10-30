/** Display-only substitutions. Callers must retain the original message for
 * replies, forwarding, exports, searching and stored data. */
const replacements: Readonly<Record<string, string>> = {
  a: '4', b: '8', e: '3', g: '9', i: '1', o: '0', s: '5', t: '7', z: '2',
};

export function leetspeakText(text: string): string {
  return text.replace(/[abegiostzáàâãäéèêëíìîïóòôõö]/giu, character => {
    const letter = character.normalize('NFD')[0]!.toLowerCase();
    return replacements[letter] ?? character;
  });
}

/** Transform only the sanitized display copy, never markup, destinations,
 * resource URLs, CSP, style rules or message source. */
export function leetspeakHtml(html: string): string {
  const document = new DOMParser().parseFromString(html, 'text/html');
  const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
  const nodes: Text[] = [];
  while (walker.nextNode()) {
    const node = walker.currentNode as Text;
    if (!node.parentElement?.closest('script,style,noscript,template')) nodes.push(node);
  }
  for (const node of nodes) node.data = leetspeakText(node.data);
  for (const element of document.body.querySelectorAll('[alt], [title], [aria-label]')) {
    for (const attribute of ['alt', 'title', 'aria-label']) {
      const value = element.getAttribute(attribute);
      if (value !== null) element.setAttribute(attribute, leetspeakText(value));
    }
  }
  return '<!doctype html>' + document.documentElement.outerHTML;
}
