// Mail-safe column layouts: nested presentation tables with fixed pixel/percent
// widths, `valign=top` cells and the `width:100%; max-width:` pattern so clients
// that honour it collapse to one column. Pure string builders; the editors insert
// the result and the user types in the cells.

export type ColumnPreset = 'one' | 'two' | 'three' | 'sidebarLeft' | 'sidebarRight';

export interface ColumnSpec {
  /** Column widths in percent, summing to 100. */
  widths: number[];
  /** Space between columns, px. */
  gutter: number;
  /** Cell padding, px. */
  padding: number;
  /** Total content width in px (the wrapper). */
  width: number;
}

export const COLUMN_PRESETS: Record<ColumnPreset, number[]> = {
  one: [100],
  two: [50, 50],
  three: [34, 33, 33],
  sidebarLeft: [30, 70],
  sidebarRight: [70, 30],
};

export const MAX_COLUMNS = 4;

const clamp = (n: number, lo: number, hi: number): number => Math.min(hi, Math.max(lo, Math.round(Number.isFinite(n) ? n : lo)));

/** `count` equal columns whose widths sum to exactly 100. */
export function equalWidths(count: number): number[] {
  const c = clamp(count, 1, MAX_COLUMNS);
  const base = Math.floor(100 / c);
  const out = Array<number>(c).fill(base);
  out[0] = (out[0] ?? base) + (100 - base * c);
  return out;
}

export function normaliseSpec(spec: Partial<ColumnSpec> & { widths?: number[] }): ColumnSpec {
  let widths = (spec.widths ?? [50, 50]).slice(0, MAX_COLUMNS).map((w) => clamp(w, 5, 95));
  if (widths.length === 0) widths = [100];
  const sum = widths.reduce((a, b) => a + b, 0);
  if (sum !== 100) widths = widths.length === 1 ? [100] : widths.map((w) => Math.max(5, Math.round((w * 100) / sum)));
  const drift = 100 - widths.reduce((a, b) => a + b, 0);
  widths[0] = (widths[0] ?? 0) + drift;
  return { widths, gutter: clamp(spec.gutter ?? 16, 0, 48), padding: clamp(spec.padding ?? 8, 0, 48), width: clamp(spec.width ?? 600, 240, 1200) };
}

/**
 * The columns table. Each cell holds one empty paragraph (`placeholders[i]` when
 * given) for the user to type in. Gutters are cell padding on the inner edges, so
 * widths stay exact in Outlook (which ignores margins on cells).
 */
export function buildColumnsHtml(input: Partial<ColumnSpec>, placeholders: readonly string[] = []): string {
  const s = normaliseSpec(input);
  const n = s.widths.length;
  if (n === 1) {
    return `<table role="presentation" data-mw-columns="1" width="100%" cellpadding="0" cellspacing="0" border="0" style="width:100%;border-collapse:collapse"><tr><td valign="top" style="padding:${s.padding}px">${cell(placeholders[0])}</td></tr></table>`;
  }
  const half = Math.floor(s.gutter / 2);
  const cells = s.widths.map((w, i) => {
    const px = Math.floor((s.width * w) / 100);
    const left = i === 0 ? s.padding : half + s.padding;
    const right = i === n - 1 ? s.padding : half + s.padding;
    // `display:inline-block` + max-width lets capable clients wrap; the width
    // attribute keeps Outlook and table-only clients on fixed columns.
    return `<td valign="top" width="${w}%" style="width:${w}%;max-width:${px}px;padding:${s.padding}px ${right}px ${s.padding}px ${left}px;vertical-align:top">${cell(placeholders[i])}</td>`;
  });
  return `<table role="presentation" data-mw-columns="${n}" width="100%" cellpadding="0" cellspacing="0" border="0" style="width:100%;max-width:${s.width}px;border-collapse:collapse"><tr>${cells.join('')}</tr></table>`;
}

function cell(placeholder?: string): string {
  return `<p>${placeholder === undefined || placeholder === '' ? '<br>' : escapeText(placeholder)}</p>`;
}

function escapeText(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

/** The column tables directly in `root` (nested ones are the user's own). */
export function columnTables(root: ParentNode): HTMLTableElement[] {
  return Array.from(root.querySelectorAll<HTMLTableElement>('table[data-mw-columns]'));
}

/** Replace a column table by its cells' content, in order (Merge into one column). */
export function mergeColumns(table: HTMLTableElement): void {
  const doc = table.ownerDocument;
  const frag = doc.createDocumentFragment();
  for (const td of Array.from(table.querySelectorAll<HTMLTableCellElement>(':scope > tbody > tr > td, :scope > tr > td'))) {
    while (td.firstChild) frag.appendChild(td.firstChild);
  }
  // Drop the empty placeholder paragraphs left by unused columns.
  for (const p of Array.from(frag.querySelectorAll('p'))) {
    if (p.childNodes.length === 1 && p.firstChild?.nodeName === 'BR') p.remove();
  }
  if (!frag.firstChild) frag.appendChild(doc.createElement('p')).appendChild(doc.createElement('br'));
  table.replaceWith(frag);
}

/** Change the column count/widths of an existing table, keeping the content of the first cells. */
export function reflowColumns(table: HTMLTableElement, input: Partial<ColumnSpec>): void {
  const tds = Array.from(table.querySelectorAll<HTMLTableCellElement>(':scope > tbody > tr > td, :scope > tr > td'));
  const next = new DOMParser().parseFromString(buildColumnsHtml(input), 'text/html').querySelector('table')!;
  const newTds = Array.from(next.querySelectorAll<HTMLTableCellElement>(':scope > tbody > tr > td'));
  newTds.forEach((td, i) => {
    const source = tds[i];
    if (source) td.innerHTML = source.innerHTML;
  });
  // Surplus old cells fold into the last new cell so nothing typed is lost.
  const last = newTds[newTds.length - 1];
  for (const extra of tds.slice(newTds.length)) if (last) last.insertAdjacentHTML('beforeend', extra.innerHTML);
  table.replaceWith(table.ownerDocument.importNode(next, true));
}

/** Content-width wrapper: one centred or left-aligned table with a `max-width`. */
export function wrapContentWidth(innerHtml: string, widthPx: number | 'full', align: 'center' | 'left' = 'center'): string {
  if (widthPx === 'full') return innerHtml;
  const w = clamp(widthPx, 240, 1200);
  const a = align === 'center' ? ' align="center"' : '';
  return `<table role="presentation" data-mw-width="${w}"${a} width="100%" cellpadding="0" cellspacing="0" border="0" style="width:100%;max-width:${w}px;${align === 'center' ? 'margin:0 auto;' : ''}border-collapse:collapse"><tr><td valign="top">${innerHtml}</td></tr></table>`;
}

/**
 * Columns tables in `html`: tables we built (`data-mw-columns`), or, after an
 * editor round trip dropped the marker, a borderless one-row table of 2-4 cells
 * that all carry a percentage width and `vertical-align:top`.
 */
export function isColumnsLike(table: HTMLTableElement): boolean {
  if (table.hasAttribute('data-mw-columns')) return true;
  const rows = table.querySelectorAll(':scope > tbody > tr, :scope > tr');
  if (rows.length !== 1) return false;
  const cells = Array.from(rows[0]!.children) as HTMLElement[];
  if (cells.length < 2 || cells.length > MAX_COLUMNS) return false;
  return cells.every((c) => /%/.test(c.style.width || c.getAttribute('width') || '') && (c.style.verticalAlign === 'top' || c.getAttribute('valign') === 'top') && c.style.borderTopWidth === '');
}

/** Merge every columns table in `html` into one column; returns the new HTML and how many were merged. */
export function mergeColumnsInHtml(html: string): { html: string; merged: number } {
  const body = new DOMParser().parseFromString(`<body>${html}</body>`, 'text/html').body;
  let merged = 0;
  for (const table of Array.from(body.querySelectorAll<HTMLTableElement>('table'))) {
    if (!body.contains(table) || !isColumnsLike(table)) continue;
    mergeColumns(table);
    merged += 1;
  }
  return { html: body.innerHTML, merged };
}
