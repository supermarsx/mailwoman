// The Styles gallery's model (FORMAT): the built-in styles of each style set, the
// user's own styles, and how a style is read from and written to JSON. A style is
// a block type (a paragraph, a heading, a quote, a code block, a task item), a
// paragraph look and the text look of the block's text. Applying one writes plain
// tags and inline CSS, so it renders the same in every mail client.

import { composerFontStack } from './fontCatalog.ts';
import { PARAGRAPH_SPACING } from './richFormatting.ts';
import { validPointValue } from './formatModel.ts';

export type StyleBlock = 'p' | 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6' | 'quote' | 'code' | 'task' | 'inline';

export interface StyleMarks {
  fontFamily?: string | undefined;
  /** `12pt` */
  fontSize?: string | undefined;
  /** `#rrggbb` */
  color?: string | undefined;
  bold?: boolean | undefined;
  italic?: boolean | undefined;
  underline?: boolean | undefined;
  highlight?: string | undefined;
}

export interface StyleAttrs {
  align?: 'left' | 'center' | 'right' | 'justify' | undefined;
  lineHeight?: string | undefined;
  /** pt, one of the paragraph spacing steps */
  before?: number | undefined;
  after?: number | undefined;
  /** 40 px steps */
  indent?: number | undefined;
}

export interface StyleDef {
  id: string;
  /** Shown for a custom style; a built-in one uses `mail-editor-style-<id>` or `cfmt-style-<id>`. */
  name?: string;
  block: StyleBlock;
  attrs: StyleAttrs;
  marks: StyleMarks;
}

export const BUILTIN_STYLE_IDS = [
  'normal', 'no-spacing', 'heading1', 'heading2', 'heading3', 'heading4', 'heading5', 'heading6', 'title', 'subtitle',
  'strong', 'emphasis', 'intense-emphasis', 'quote', 'intense-quote', 'list-paragraph', 'task-item', 'code-inline', 'code', 'caption',
] as const;
export type BuiltinStyleId = (typeof BUILTIN_STYLE_IDS)[number];

/** The ones shown as tiles in the ribbon; the rest are in the all-styles list. */
export const QUICK_STYLE_IDS: readonly string[] = ['normal', 'no-spacing', 'heading1', 'heading2'];

export const STYLE_SETS = ['default', 'modern', 'classic', 'compact'] as const;
export type StyleSet = (typeof STYLE_SETS)[number];
export const isStyleSet = (value: unknown): value is StyleSet => (STYLE_SETS as readonly unknown[]).includes(value);

interface SetLook {
  heading: readonly number[];
  title: number;
  subtitle: number;
  family?: string;
  headingColor?: string;
  subtitleColor?: string;
  before: 0 | 6 | 12;
  after: 0 | 6 | 12;
}

const stack = (id: string): string => composerFontStack(id) ?? '';
const LOOKS: Record<StyleSet, SetLook> = {
  default: { heading: [20, 16, 14, 12, 11, 10], title: 28, subtitle: 18, subtitleColor: '#595959', before: 12, after: 6 },
  modern: { heading: [22, 18, 15, 13, 11, 10], title: 30, subtitle: 16, family: stack('inter'), headingColor: '#1f4e79', subtitleColor: '#595959', before: 12, after: 6 },
  classic: { heading: [20, 16, 14, 12, 11, 10], title: 26, subtitle: 16, family: stack('georgia'), subtitleColor: '#404040', before: 12, after: 6 },
  compact: { heading: [16, 14, 12, 11, 10, 9], title: 22, subtitle: 14, subtitleColor: '#595959', before: 6, after: 0 },
};

const MONO = stack('courier-new');
const ACCENT = '#1f4e79';
const GREY = '#595959';

const pt = (size: number): string => `${size}pt`;
const compact = <T extends object>(value: T): T => Object.fromEntries(Object.entries(value).filter(([, v]) => v !== undefined)) as T;

/** One built-in style, as the given set draws it. */
export function builtinStyle(set: StyleSet, id: string): StyleDef | undefined {
  const look = LOOKS[set];
  const heading = /^heading([1-6])$/.exec(id);
  if (heading) {
    const level = Number(heading[1]);
    return { id, block: `h${level}` as StyleBlock, attrs: { before: look.before, after: look.after }, marks: compact({ fontSize: pt(look.heading[level - 1]!), bold: true, fontFamily: look.family, color: look.headingColor }) };
  }
  switch (id) {
    case 'normal': return { id, block: 'p', attrs: {}, marks: {} };
    case 'no-spacing': return { id, block: 'p', attrs: { before: 0, after: 0, lineHeight: '1' }, marks: {} };
    case 'title': return { id, block: 'p', attrs: { before: 0, after: 6 }, marks: compact({ fontSize: pt(look.title), fontFamily: look.family, color: look.headingColor }) };
    case 'subtitle': return { id, block: 'p', attrs: { before: 0, after: 6 }, marks: compact({ fontSize: pt(look.subtitle), color: look.subtitleColor, fontFamily: look.family, italic: set === 'classic' }) };
    case 'strong': return { id, block: 'inline', attrs: {}, marks: { bold: true } };
    case 'emphasis': return { id, block: 'inline', attrs: {}, marks: { italic: true } };
    case 'intense-emphasis': return { id, block: 'inline', attrs: {}, marks: { bold: true, italic: true, color: ACCENT } };
    case 'quote': return { id, block: 'quote', attrs: {}, marks: { italic: true } };
    case 'intense-quote': return { id, block: 'quote', attrs: { align: 'center' }, marks: { italic: true, bold: true, color: ACCENT } };
    case 'list-paragraph': return { id, block: 'p', attrs: { indent: 1 }, marks: {} };
    case 'task-item': return { id, block: 'task', attrs: {}, marks: {} };
    case 'code-inline': return { id, block: 'inline', attrs: {}, marks: compact({ fontFamily: MONO, highlight: '#f0f0f0' }) };
    case 'code': return { id, block: 'code', attrs: {}, marks: {} };
    case 'caption': return { id, block: 'p', attrs: { before: 0, after: 6 }, marks: { fontSize: pt(9), italic: true, color: GREY } };
    default: return undefined;
  }
}

/** Every style the gallery lists: the built-ins of `set` (a custom style of the same id replaces one), then the custom ones. */
export function allStyles(set: StyleSet, custom: readonly StyleDef[]): StyleDef[] {
  const overrides = new Map(custom.map((style) => [style.id, style]));
  const built = BUILTIN_STYLE_IDS.map((id) => overrides.get(id) ?? builtinStyle(set, id)!);
  return [...built, ...custom.filter((style) => !(BUILTIN_STYLE_IDS as readonly string[]).includes(style.id))];
}

export const resolveStyle = (set: StyleSet, custom: readonly StyleDef[], id: string): StyleDef | undefined => allStyles(set, custom).find((style) => style.id === id);

export const isBuiltinId = (id: string): boolean => (BUILTIN_STYLE_IDS as readonly string[]).includes(id);

/** Custom styles that count against `compose.styles.maxCustom` (a built-in's override does not). */
export const customCount = (custom: readonly StyleDef[]): number => custom.filter((style) => !isBuiltinId(style.id)).length;

// ---- JSON ------------------------------------------------------------------

const BLOCKS: readonly string[] = ['p', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'quote', 'code', 'task', 'inline'];
const HEX = /^#[0-9a-f]{6}$/i;

function cleanMarks(value: unknown): StyleMarks {
  const input = (value && typeof value === 'object' ? value : {}) as Record<string, unknown>;
  const out: StyleMarks = {};
  if (typeof input['fontFamily'] === 'string' && input['fontFamily'].length < 200) out.fontFamily = input['fontFamily'];
  if (typeof input['fontSize'] === 'string' && validPointValue(input['fontSize'])) out.fontSize = input['fontSize'];
  if (typeof input['color'] === 'string' && HEX.test(input['color'])) out.color = input['color'].toLowerCase();
  if (typeof input['highlight'] === 'string' && HEX.test(input['highlight'])) out.highlight = input['highlight'].toLowerCase();
  for (const flag of ['bold', 'italic', 'underline'] as const) if (input[flag] === true) out[flag] = true;
  return out;
}

function cleanAttrs(value: unknown): StyleAttrs {
  const input = (value && typeof value === 'object' ? value : {}) as Record<string, unknown>;
  const out: StyleAttrs = {};
  if (['left', 'center', 'right', 'justify'].includes(input['align'] as string)) out.align = input['align'] as StyleAttrs['align'];
  if (typeof input['lineHeight'] === 'string' && /^\d(\.\d{1,2})?$/.test(input['lineHeight'])) out.lineHeight = input['lineHeight'];
  for (const side of ['before', 'after'] as const) if (typeof input[side] === 'number' && (PARAGRAPH_SPACING as readonly number[]).includes(input[side])) out[side] = input[side];
  if (typeof input['indent'] === 'number' && Number.isInteger(input['indent']) && input['indent'] >= 1 && input['indent'] <= 10) out.indent = input['indent'];
  return out;
}

/** Read the stored custom styles. Anything unreadable is dropped, so a bad value never breaks the gallery. */
export function parseCustomStyles(text: string, limit = 100): StyleDef[] {
  if (!text) return [];
  let raw: unknown;
  try { raw = JSON.parse(text); } catch { return []; }
  if (!Array.isArray(raw)) return [];
  const seen = new Set<string>();
  const out: StyleDef[] = [];
  for (const entry of raw) {
    if (!entry || typeof entry !== 'object') continue;
    const { id, name, block } = entry as Record<string, unknown>;
    if (typeof id !== 'string' || !/^[a-z0-9-]{1,40}$/.test(id) || seen.has(id) || !BLOCKS.includes(block as string)) continue;
    if (!isBuiltinId(id) && (typeof name !== 'string' || name.trim() === '' || name.length > 40)) continue;
    seen.add(id);
    out.push({ id, ...(typeof name === 'string' ? { name: name.trim().slice(0, 40) } : {}), block: block as StyleBlock, attrs: cleanAttrs((entry as Record<string, unknown>)['attrs']), marks: cleanMarks((entry as Record<string, unknown>)['marks']) });
    if (out.length >= limit) break;
  }
  return out;
}

export function serializeCustomStyles(styles: readonly StyleDef[]): string {
  return styles.length === 0 ? '' : JSON.stringify(styles);
}

/** An id for a new custom style from its name. */
export function newStyleId(name: string, taken: readonly string[]): string {
  const base = `c-${name.toLowerCase().normalize('NFKD').replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '').slice(0, 30) || 'style'}`;
  let id = base;
  for (let n = 2; taken.includes(id); n += 1) id = `${base}-${n}`;
  return id;
}

/** Create or replace `style` in `custom`; false when a new custom style would pass `max`. */
export function saveCustomStyle(custom: readonly StyleDef[], style: StyleDef, max: number): StyleDef[] | null {
  const at = custom.findIndex((entry) => entry.id === style.id);
  if (at >= 0) return custom.map((entry, index) => (index === at ? style : entry));
  if (!isBuiltinId(style.id) && customCount(custom) >= max) return null;
  return [...custom, style];
}
