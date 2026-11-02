/** The attribute `style` carries while markup is parsed (see `inertBody`). */
export const PARSED_STYLE_ATTRIBUTE = 'data-mw-parsed-style';
const RAW_TEXT = new Set(['script', 'style', 'textarea', 'title', 'xmp', 'iframe', 'noembed', 'noframes', 'plaintext']);

/**
 * Rename every `style` attribute in `html` to `data-mw-parsed-style` (`data-mw-style` is the paragraph preset), following the
 * HTML tokenizer closely enough that text, comments, raw-text element content
 * and other attributes' values are left exactly as they are: only attribute
 * names inside start tags are looked at.
 */
export function renameStyleAttributes(html: string): string {
  if (!/style/i.test(html)) return html;
  let out = '';
  let i = 0;
  const n = html.length;
  const space = (c: string | undefined): boolean => c === ' ' || c === '\t' || c === '\n' || c === '\r' || c === '\f';
  while (i < n) {
    const lt = html.indexOf('<', i);
    if (lt < 0) { out += html.slice(i); break; }
    out += html.slice(i, lt);
    i = lt;
    if (html.startsWith('<!--', i)) {
      const end = html.indexOf('-->', i + 4);
      const stop = end < 0 ? n : end + 3;
      out += html.slice(i, stop); i = stop; continue;
    }
    const next = html[i + 1];
    if (next === '!' || next === '?' || (next === '/' && /[a-z]/i.test(html[i + 2] ?? ''))) {
      const end = html.indexOf('>', i + 1);
      const stop = end < 0 ? n : end + 1;
      out += html.slice(i, stop); i = stop; continue;
    }
    if (!/[a-z]/i.test(next ?? '')) { out += '<'; i += 1; continue; }
    // A start tag: its name, then attributes up to '>'.
    let j = i + 1;
    while (j < n && !space(html[j]) && html[j] !== '/' && html[j] !== '>') j++;
    const tag = html.slice(i + 1, j).toLowerCase();
    out += html.slice(i, j);
    while (j < n && html[j] !== '>') {
      if (space(html[j]) || html[j] === '/') { out += html[j]; j++; continue; }
      let k = j + 1;
      while (k < n && !space(html[k]) && html[k] !== '/' && html[k] !== '>' && html[k] !== '=') k++;
      const name = html.slice(j, k);
      out += name.toLowerCase() === 'style' ? PARSED_STYLE_ATTRIBUTE : name;
      j = k;
      let m = j;
      while (m < n && space(html[m])) m++;
      if (html[m] !== '=') continue;
      m++;
      while (m < n && space(html[m])) m++;
      let end: number;
      if (html[m] === '"' || html[m] === "'") { const close = html.indexOf(html[m]!, m + 1); end = close < 0 ? n : close + 1; }
      else { end = m; while (end < n && !space(html[end]) && html[end] !== '>') end++; }
      out += html.slice(j, end); j = end;
    }
    if (j < n) { out += '>'; j++; }
    i = j;
    if (RAW_TEXT.has(tag)) {
      const close = html.toLowerCase().indexOf(`</${tag}`, i);
      const stop = close < 0 ? n : close;
      out += html.slice(i, stop); i = stop;
    }
  }
  return out;
}
