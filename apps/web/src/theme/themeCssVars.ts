// `themeCssVars()` — the theme bridge for the sanitized message iframe (e7) and
// the print pipeline (plan §3 e4: "inject the same token CSS variables into the
// sanitized message iframe + print stylesheet so mail + chrome theme together").
//
// The reader iframe is a SEPARATE document with an opaque origin
// (`sandbox=""`, no allow-same-origin), so it can NOT inherit the parent
// document's CSS custom properties, and it can NOT see the vanilla-extract
// HASHED var names. This helper therefore emits a self-contained block of
// CONCRETE values under stable `--mw-color-*` names, plus base body styling, as
// a plain CSS string e7 injects into the srcdoc `<style>`. Values come from the
// same `tokens.ts` palettes the chrome uses, so mail and chrome stay in sync.

import type { ThemeName, Density } from './contract.css.ts';
import { THEMES } from './tokens.ts';

/** Pull the concrete fallback out of an `accent()` token, or use the override. */
function resolveAccent(token: string, override?: string): string {
  if (override !== undefined && override !== '') return override;
  const m = /var\(--mw-accent,\s*([^)]+)\)/.exec(token);
  return m && m[1] !== undefined ? m[1].trim() : token;
}

const DENSITY_FONT_SIZE: Record<Density, string> = {
  compact: '13px',
  cozy: '14px',
  relaxed: '15px',
};

export interface ThemeCssVarsOptions {
  /** Inline accent override (empty/undefined = the theme default). */
  accent?: string;
  /** Density for the base font size (defaults to cozy). */
  density?: Density;
  /**
   * When true, wrap the body rules in `@media print` and drop the opaque
   * background so printers don't ink a dark page. Used by the print stylesheet.
   */
  forPrint?: boolean;
}

/**
 * Return a CSS string — a `:root{ --mw-color-*: … }` block plus base
 * `html/body/a/pre` rules — that themes an isolated document (the sanitized
 * message iframe or the print sheet) to match the given chrome theme.
 */
export function themeCssVars(theme: ThemeName, opts: ThemeCssVarsOptions = {}): string {
  const c = THEMES[theme].color;
  const accent = resolveAccent(c.accent, opts.accent);
  const reading = THEMES[theme].font.reading;
  const mono = THEMES[theme].font.mono;
  const fontSize = DENSITY_FONT_SIZE[opts.density ?? 'cozy'];

  const rootVars = [
    `--mw-color-bg: ${opts.forPrint ? '#ffffff' : c.bg}`,
    `--mw-color-surface: ${opts.forPrint ? '#ffffff' : c.surface}`,
    `--mw-color-border: ${c.border}`,
    `--mw-color-text: ${opts.forPrint ? '#000000' : c.text}`,
    `--mw-color-text-dim: ${c.textDim}`,
    `--mw-color-accent: ${accent}`,
    `--mw-color-link: ${c.link}`,
    `--mw-color-selection: ${c.selection}`,
    `--mw-font-reading: ${reading}`,
    `--mw-font-mono: ${mono}`,
  ].join('; ');

  const body = [
    'margin: 0',
    'padding: 12px 16px',
    'background: var(--mw-color-bg)',
    'color: var(--mw-color-text)',
    'font-family: var(--mw-font-reading)',
    `font-size: ${fontSize}`,
    'line-height: 1.5',
    'word-break: break-word',
  ].join('; ');

  const rules = [
    `:root { ${rootVars}; }`,
    `html, body { ${body}; }`,
    `a { color: var(--mw-color-link); }`,
    `::selection { background: var(--mw-color-selection); }`,
    `pre, code, kbd { font-family: var(--mw-font-mono); }`,
    `pre { white-space: pre-wrap; word-break: break-word; }`,
    `blockquote { border-left: 3px solid var(--mw-color-border); margin: 0.5em 0; padding-left: 0.8em; color: var(--mw-color-text-dim); }`,
    `img { max-width: 100%; height: auto; }`,
  ];

  const css = rules.join('\n');
  return opts.forPrint ? `@media print {\n${css}\n}` : css;
}
