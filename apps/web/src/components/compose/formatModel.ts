// The Format tab's editor-independent model (FORMAT): the font size ramp, the list
// types the two list dropdowns offer, indent steps, line spacing choices, and the
// pure helpers both editors share. Nothing here touches a DOM or a schema.

/** The standard size ramp (pt) the grow and shrink buttons step through. */
export const FONT_RAMP: readonly number[] = [8, 9, 10, 10.5, 11, 12, 14, 16, 18, 20, 22, 24, 26, 28, 36, 48, 72];
export const FONT_SIZE_MIN = 1;
export const FONT_SIZE_MAX = 96;
/** The size text has when nothing says otherwise (pt). */
export const FONT_SIZE_BASE = 11;

const roundHalf = (value: number): number => Math.round(value * 10) / 10;

/**
 * The next size after `current` pt. By default the neighbouring entry of the
 * ramp (a size between two entries goes to the nearer one in the direction of
 * travel); with `fine` exactly 1 pt. The result stays inside 1 to 96 pt.
 */
export function stepFontSize(current: number | null | undefined, direction: 1 | -1, fine = false): number {
  const from = typeof current === 'number' && Number.isFinite(current) && current > 0 ? current : FONT_SIZE_BASE;
  if (fine) return Math.min(FONT_SIZE_MAX, Math.max(FONT_SIZE_MIN, roundHalf(from + direction)));
  if (direction > 0) {
    const next = FONT_RAMP.find((size) => size > from + 1e-6);
    return next ?? Math.min(FONT_SIZE_MAX, Math.max(from, FONT_RAMP[FONT_RAMP.length - 1]!));
  }
  for (let i = FONT_RAMP.length - 1; i >= 0; i -= 1) if (FONT_RAMP[i]! < from - 1e-6) return FONT_RAMP[i]!;
  return Math.max(FONT_SIZE_MIN, Math.min(from, FONT_RAMP[0]!));
}

/** `12` becomes `12pt`, `10.5` becomes `10.5pt`. */
export const pointText = (size: number): string => `${roundHalf(size)}pt`;

/** `12pt` (or `16px`) as points, or null. */
export function pointsOf(value: string | null | undefined): number | null {
  if (!value) return null;
  const match = /^(\d+(?:\.\d+)?)(pt|px)$/.exec(value.trim());
  if (!match) return null;
  const number = Number(match[1]);
  return match[2] === 'px' ? roundHalf(number * 0.75) : number;
}

/** A font-size mark value is `<number>pt` within the supported range, at most one decimal. */
export function validPointValue(value: string): boolean {
  const match = /^(\d{1,2}(?:\.\d)?)pt$/.exec(value.trim());
  if (!match) return false;
  const number = Number(match[1]);
  return number >= FONT_SIZE_MIN && number <= FONT_SIZE_MAX;
}

export type ListKind = 'bullet' | 'ordered';

/**
 * One list type. `css` is a `list-style-type` every mail client draws. A `marker`
 * type has no client-side equivalent that is dependable (an arrow bullet, `1)`,
 * a checklist, `1.1.1`), so the editor draws it and the sent HTML carries the
 * marker as text in front of each item (see `listMarkers.ts`).
 */
export interface ListStyleDef {
  id: string;
  kind: ListKind;
  css?: string;
  /** The legacy `type` attribute of an `<ol>`, which Outlook honours. */
  type?: string;
  marker?: ListMarker;
  /** Three sample markers for the dropdown's preview. */
  sample: readonly [string, string, string];
}

export type ListMarker = 'dash' | 'arrow' | 'tick' | 'check' | 'paren' | 'zero' | 'legal';

export const BULLET_STYLES: readonly ListStyleDef[] = [
  { id: 'disc', kind: 'bullet', css: 'disc', sample: ['•', '•', '•'] },
  { id: 'circle', kind: 'bullet', css: 'circle', sample: ['◦', '◦', '◦'] },
  { id: 'square', kind: 'bullet', css: 'square', sample: ['▪', '▪', '▪'] },
  { id: 'dash', kind: 'bullet', marker: 'dash', sample: ['–', '–', '–'] },
  { id: 'arrow', kind: 'bullet', marker: 'arrow', sample: ['→', '→', '→'] },
  { id: 'tick', kind: 'bullet', marker: 'tick', sample: ['✓', '✓', '✓'] },
  { id: 'check', kind: 'bullet', marker: 'check', sample: ['☐', '☑', '☐'] },
];

export const ORDERED_STYLES: readonly ListStyleDef[] = [
  { id: 'decimal', kind: 'ordered', css: 'decimal', type: '1', sample: ['1.', '2.', '3.'] },
  { id: 'paren', kind: 'ordered', marker: 'paren', sample: ['1)', '2)', '3)'] },
  { id: 'lower-alpha', kind: 'ordered', css: 'lower-alpha', type: 'a', sample: ['a.', 'b.', 'c.'] },
  { id: 'upper-alpha', kind: 'ordered', css: 'upper-alpha', type: 'A', sample: ['A.', 'B.', 'C.'] },
  { id: 'lower-roman', kind: 'ordered', css: 'lower-roman', type: 'i', sample: ['i.', 'ii.', 'iii.'] },
  { id: 'upper-roman', kind: 'ordered', css: 'upper-roman', type: 'I', sample: ['I.', 'II.', 'III.'] },
  { id: 'zero', kind: 'ordered', marker: 'zero', sample: ['01.', '02.', '03.'] },
  { id: 'legal', kind: 'ordered', marker: 'legal', sample: ['1', '1.1', '1.1.1'] },
  { id: 'outline', kind: 'ordered', css: 'decimal', type: '1', sample: ['1.', 'a.', 'i.'] },
];

export const LIST_STYLES: readonly ListStyleDef[] = [...BULLET_STYLES, ...ORDERED_STYLES];

export function listStyleDef(id: string | null | undefined): ListStyleDef | undefined {
  return LIST_STYLES.find((entry) => entry.id === id);
}

/** The default list style of a kind. */
export const DEFAULT_LIST_STYLE: Readonly<Record<ListKind, string>> = { bullet: 'disc', ordered: 'decimal' };

/** The style a numbered list has at nesting `depth` (0 is the outermost) under the outline type. */
const OUTLINE: readonly string[] = ['decimal', 'lower-alpha', 'lower-roman'];
export const outlineStyle = (depth: number): string => OUTLINE[((depth % OUTLINE.length) + OUTLINE.length) % OUTLINE.length]!;
/** The style a nested numbered list takes when its parent is `parent`: decimal, then a., then i., around. */
export function nestedOrderedStyle(parent: string | null | undefined): string {
  const index = OUTLINE.indexOf(parent ?? 'decimal');
  return index < 0 ? (parent ?? 'decimal') : OUTLINE[(index + 1) % OUTLINE.length]!;
}
/** The style a nested bullet list takes under a parent of the standard three: disc, circle, square. */
const BULLET_CYCLE: readonly string[] = ['disc', 'circle', 'square'];
export function nestedBulletStyle(parent: string | null | undefined): string {
  const index = BULLET_CYCLE.indexOf(parent ?? 'disc');
  return index < 0 ? (parent ?? 'disc') : BULLET_CYCLE[(index + 1) % BULLET_CYCLE.length]!;
}

export const CHECK_OFF = '☐';
export const CHECK_ON = '☑';

/** One indent step in px (the sent HTML says `margin-left:Npx`). */
export const INDENT_STEP = 40;
export const INDENT_MAX = 10;

export const LINE_SPACING_CHOICES: readonly string[] = ['1', '1.15', '1.5', '2'];
/** A custom line spacing is a multiplier between 0.5 and 5, two decimals at most. */
export function validLineHeight(value: unknown): value is string {
  if (typeof value !== 'string' || !/^\d(?:\.\d{1,2})?$/.test(value)) return false;
  const number = Number(value);
  return number >= 0.5 && number <= 5;
}

export type ClearVariant = 'all' | 'text' | 'paragraph' | 'everything';
export const CLEAR_VARIANTS: readonly ClearVariant[] = ['all', 'text', 'paragraph', 'everything'];

export type PasteMode = 'keep' | 'merge' | 'text';
export const PASTE_MODES: readonly PasteMode[] = ['keep', 'merge', 'text'];

/** What the format painter has taken: shown as a chip (`Arial 11 · bold · #333`). */
export interface PaintSummary {
  family?: string | null;
  size?: number | null;
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  strike?: boolean;
  color?: string | null;
  highlight?: string | null;
}

export function paintSummaryText(summary: PaintSummary): string {
  const parts: string[] = [];
  const first = [summary.family ? summary.family.split(',')[0]!.replace(/["']/g, '').trim() : '', summary.size ? String(roundHalf(summary.size)) : ''].filter(Boolean).join(' ');
  if (first) parts.push(first);
  if (summary.bold) parts.push('bold');
  if (summary.italic) parts.push('italic');
  if (summary.underline) parts.push('underline');
  if (summary.strike) parts.push('strike');
  if (summary.color) parts.push(summary.color);
  if (summary.highlight) parts.push(`bg ${summary.highlight}`);
  return parts.join(' · ');
}

/** Keys a ribbon control answers to (Ctrl on Windows and Linux, Cmd on Apple). */
export const isGrowKey = (event: Pick<KeyboardEvent, 'key' | 'shiftKey' | 'ctrlKey' | 'metaKey' | 'altKey'>): boolean =>
  (event.ctrlKey || event.metaKey) && !event.altKey && event.shiftKey && (event.key === '>' || event.key === '.');
export const isShrinkKey = (event: Pick<KeyboardEvent, 'key' | 'shiftKey' | 'ctrlKey' | 'metaKey' | 'altKey'>): boolean =>
  (event.ctrlKey || event.metaKey) && !event.altKey && event.shiftKey && (event.key === '<' || event.key === ',');
/** Ctrl+Space or Ctrl+\ clears formatting. */
export const isClearKey = (event: Pick<KeyboardEvent, 'key' | 'shiftKey' | 'ctrlKey' | 'metaKey' | 'altKey'>): boolean =>
  (event.ctrlKey || event.metaKey) && !event.altKey && !event.shiftKey && (event.key === ' ' || event.key === '\\');

/** The paste modes' HTML: `merge` keeps emphasis and structure but drops the source's font, size, colour and spacing. */
const MERGE_DROP = ['font-family', 'font-size', 'color', 'background', 'background-color', 'line-height', 'margin', 'margin-top', 'margin-bottom', 'margin-left', 'margin-right', 'padding', 'text-indent', 'mso-ansi-font-size'];
export function mergeFormatting(root: ParentNode): void {
  for (const font of Array.from(root.querySelectorAll('font'))) font.replaceWith(...Array.from(font.childNodes));
  for (const element of Array.from(root.querySelectorAll<HTMLElement>('[style]'))) {
    // Through the CSSOM, never the attribute (the page CSP refuses inline style text).
    for (const property of MERGE_DROP) element.style.removeProperty(property);
    if (element.getAttribute('style') === '') element.removeAttribute('style');
  }
  for (const element of Array.from(root.querySelectorAll('span'))) if (element.attributes.length === 0) element.replaceWith(...Array.from(element.childNodes));
}
