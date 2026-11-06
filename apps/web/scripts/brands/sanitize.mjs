// Sanitiser for user-supplied official logos (src/brands/custom/<id>.svg).
// Only <svg>, <g>, <path>, <circle>, <rect> and <polygon> with plain geometry attributes are accepted.
// Anything else (script, style, href, foreignObject, event handlers, transforms, entities, comments)
// is rejected, never stripped. The result is one path string on a 24x24 viewBox.

const TAGS = {
  svg: ['xmlns', 'viewbox', 'width', 'height'],
  g: [],
  path: ['d'],
  circle: ['cx', 'cy', 'r'],
  rect: ['x', 'y', 'width', 'height'],
  polygon: ['points'],
};
const NUM = /^-?(\d+\.?\d*|\.\d+)(e-?\d+)?$/i;
const PATH_DATA = /^[MmLlHhVvCcSsQqTtAaZz0-9eE.,\s+-]+$/;

export function sanitizeSvg(text) {
  let src = String(text).trim();
  if (src.charCodeAt(0) === 0xfeff) src = src.slice(1).trim();
  src = src.replace(/^<\?xml[^>]*\?>\s*/, '');
  if (/<!|&|<\?/.test(src)) throw new Error('doctype, comment, entity or processing instruction not allowed');
  const tokens = src.match(/<[^>]*>|[^<]+/g) ?? [];
  const stack = [];
  const shapes = [];
  let sawSvg = false;
  for (const token of tokens) {
    if (!token.startsWith('<')) {
      if (token.trim() !== '') throw new Error('text content not allowed');
      continue;
    }
    const close = /^<\/\s*([a-zA-Z]+)\s*>$/.exec(token);
    if (close) {
      if (stack.pop() !== close[1]) throw new Error('unbalanced tags');
      continue;
    }
    const open = /^<([a-zA-Z]+)((?:\s+[a-zA-Z:-]+\s*=\s*"[^"]*")*)\s*(\/?)>$/.exec(token);
    if (!open) throw new Error(`unsupported markup: ${token.slice(0, 40)}`);
    const [, name, attrText, selfClose] = open;
    const allowed = TAGS[name];
    if (!allowed) throw new Error(`element <${name}> not allowed`);
    if (name === 'svg') {
      if (sawSvg || stack.length) throw new Error('nested svg');
      sawSvg = true;
    } else if (!stack.includes('svg')) throw new Error('shape outside svg');
    const attrs = {};
    for (const m of attrText.matchAll(/([a-zA-Z:-]+)\s*=\s*"([^"]*)"/g)) {
      const key = m[1].toLowerCase();
      if (!allowed.includes(key)) throw new Error(`attribute ${m[1]} not allowed on <${name}>`);
      attrs[key] = m[2];
    }
    if (name === 'svg') {
      const vb = (attrs.viewbox ?? '').trim().split(/[\s,]+/).join(' ');
      if (vb !== '0 0 24 24') throw new Error('viewBox must be "0 0 24 24"');
      if (attrs.xmlns !== undefined && attrs.xmlns !== 'http://www.w3.org/2000/svg') throw new Error('bad xmlns');
    } else if (name === 'path') {
      if (!PATH_DATA.test(attrs.d ?? '')) throw new Error('bad path data');
      shapes.push(attrs.d.trim());
    } else if (name === 'circle') {
      const [cx, cy, r] = ['cx', 'cy', 'r'].map(k => attrs[k] ?? '');
      if (![cx, cy, r].every(v => NUM.test(v)) || Number(r) <= 0) throw new Error('bad circle');
      const R = Number(r);
      shapes.push(`M${Number(cx) - R} ${cy}a${R} ${R} 0 1 0 ${2 * R} 0a${R} ${R} 0 1 0 ${-2 * R} 0z`);
    } else if (name === 'rect') {
      const [x, y, w, h] = ['x', 'y', 'width', 'height'].map(k => attrs[k] ?? (k === 'x' || k === 'y' ? '0' : ''));
      if (![x, y, w, h].every(v => NUM.test(v)) || Number(w) <= 0 || Number(h) <= 0) throw new Error('bad rect');
      shapes.push(`M${x} ${y}h${w}v${h}h${-Number(w)}z`);
    } else if (name === 'polygon') {
      const nums = (attrs.points ?? '').trim().split(/[\s,]+/);
      if (nums.length < 6 || nums.length % 2 || !nums.every(v => NUM.test(v))) throw new Error('bad polygon');
      shapes.push(`M${nums[0]} ${nums[1]}${nums.slice(2).reduce((acc, v, i) => acc + (i % 2 ? ` ${v}` : `L${v}`), '')}z`);
    }
    if (!selfClose) stack.push(name);
  }
  if (stack.length) throw new Error('unclosed element');
  if (!sawSvg || shapes.length === 0) throw new Error('no drawable shape');
  const d = shapes.join('');
  if (!PATH_DATA.test(d)) throw new Error('bad geometry');
  return d;
}
