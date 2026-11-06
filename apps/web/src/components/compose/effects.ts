// Font effects (FORMAT): double strikethrough, super and subscript, small and all
// caps, underline styles, ligatures, text scale, character spacing, outline and
// shadow. Each is plain inline CSS (or a `sup`/`sub` tag) the server sanitiser
// (`mw-sanitize`) already allows; `support` says how widely mail clients draw it.

export type EffectKind =
  | 'doubleStrike' | 'superscript' | 'subscript' | 'smallCaps' | 'allCaps'
  | 'underlineStyle' | 'ligatures' | 'scale' | 'spacing' | 'outline' | 'shadow';

export interface EffectDef {
  kind: EffectKind;
  /** The `format.*` feature id that switches it on or off. */
  feature: string;
  /** 'most': drawn by most mail apps. 'some': some apps ignore it and show the text without the effect. */
  support: 'most' | 'some';
  /** Values the effect takes (a style, a percentage, a colour); none for a plain toggle. */
  values?: readonly string[];
  defaultValue?: string;
}

export const EFFECTS: readonly EffectDef[] = [
  { kind: 'doubleStrike', feature: 'format.doubleStrikethrough', support: 'some' },
  { kind: 'superscript', feature: 'format.superscript', support: 'most' },
  { kind: 'subscript', feature: 'format.subscript', support: 'most' },
  { kind: 'smallCaps', feature: 'format.smallCaps', support: 'some' },
  { kind: 'allCaps', feature: 'format.allCaps', support: 'most' },
  { kind: 'underlineStyle', feature: 'format.underlineStyle', support: 'some', values: ['double', 'dotted', 'dashed', 'wavy', 'thick'], defaultValue: 'double' },
  { kind: 'ligatures', feature: 'format.ligatures', support: 'some', values: ['none'], defaultValue: 'none' },
  { kind: 'scale', feature: 'format.textScale', support: 'some', values: ['50', '75', '90', '110', '125', '150', '200'], defaultValue: '150' },
  { kind: 'spacing', feature: 'format.textScale', support: 'most', values: ['condensed', 'expanded'], defaultValue: 'expanded' },
  { kind: 'outline', feature: 'format.textOutline', support: 'some', values: ['#000000'], defaultValue: '#000000' },
  { kind: 'shadow', feature: 'format.textOutline', support: 'some' },
];

export const effectDef = (kind: string): EffectDef | undefined => EFFECTS.find((entry) => entry.kind === kind);

/** The tag an effect is written as (instead of a styled span). */
export function effectTag(kind: EffectKind): 'sup' | 'sub' | null {
  return kind === 'superscript' ? 'sup' : kind === 'subscript' ? 'sub' : null;
}

const OUTLINE_PIXELS: readonly [number, number][] = [[-1, -1], [1, -1], [-1, 1], [1, 1]];

/** The inline declarations of an effect, as `[property, value]`. An unknown value gives none. */
export function effectStyle(kind: EffectKind, value = ''): [string, string][] {
  switch (kind) {
    case 'doubleStrike': return [['text-decoration', 'line-through'], ['text-decoration-style', 'double']];
    case 'smallCaps': return [['font-variant', 'small-caps']];
    case 'allCaps': return [['text-transform', 'uppercase']];
    case 'underlineStyle': {
      if (!['double', 'dotted', 'dashed', 'wavy', 'thick'].includes(value)) return [];
      return value === 'thick' ? [['text-decoration', 'underline'], ['text-decoration-thickness', '2px']] : [['text-decoration', 'underline'], ['text-decoration-style', value]];
    }
    case 'ligatures': return value === 'none' ? [['font-variant', 'no-common-ligatures']] : [];
    case 'scale': {
      const percent = Number(value);
      if (!Number.isFinite(percent) || percent < 50 || percent > 200) return [];
      return [['font-stretch', `${percent}%`], ['letter-spacing', `${Math.round(((percent - 100) / 100) * 0.08 * 1000) / 1000}em`]];
    }
    case 'spacing': return value === 'condensed' ? [['letter-spacing', '-0.05em']] : value === 'expanded' ? [['letter-spacing', '0.1em']] : [];
    case 'outline': {
      if (!/^#[0-9a-f]{6}$/i.test(value)) return [];
      return [['text-shadow', OUTLINE_PIXELS.map(([x, y]) => `${x}px ${y}px 0 ${value}`).join(', ')]];
    }
    case 'shadow': return [['text-shadow', '1px 1px 2px rgba(0, 0, 0, 0.4)']];
    default: return [];
  }
}

/** The effect a parsed element carries (the first one), from its tag and its inline style. */
export function readEffect(element: HTMLElement): { kind: EffectKind; value: string } | null {
  const tag = element.tagName.toLowerCase();
  if (tag === 'sup') return { kind: 'superscript', value: '' };
  if (tag === 'sub') return { kind: 'subscript', value: '' };
  const style = element.style;
  const line = style.textDecorationLine || style.textDecoration;
  const decoStyle = style.textDecorationStyle;
  if (/line-through/.test(line) && decoStyle === 'double') return { kind: 'doubleStrike', value: '' };
  if (/underline/.test(line) && ['double', 'dotted', 'dashed', 'wavy'].includes(decoStyle)) return { kind: 'underlineStyle', value: decoStyle };
  if (/underline/.test(line) && style.getPropertyValue('text-decoration-thickness') === '2px') return { kind: 'underlineStyle', value: 'thick' };
  if (/small-caps/.test(style.fontVariant)) return { kind: 'smallCaps', value: '' };
  if (/no-common-ligatures/.test(style.fontVariant)) return { kind: 'ligatures', value: 'none' };
  if (style.textTransform === 'uppercase') return { kind: 'allCaps', value: '' };
  const stretch = /^(\d{2,3})%$/.exec(style.getPropertyValue('font-stretch'));
  if (stretch) return { kind: 'scale', value: stretch[1]! };
  const spacing = style.letterSpacing;
  if (spacing === '-0.05em') return { kind: 'spacing', value: 'condensed' };
  if (spacing === '0.1em') return { kind: 'spacing', value: 'expanded' };
  const shadow = style.textShadow;
  if (shadow) {
    const outline = /(#[0-9a-f]{6}|rgb\(\d+, \d+, \d+\))/i.exec(shadow);
    if (/^-1px -1px 0(px)? /.test(shadow) && outline) return { kind: 'outline', value: outline[1]!.startsWith('#') ? outline[1]!.toLowerCase() : '#000000' };
    return { kind: 'shadow', value: '' };
  }
  return null;
}
