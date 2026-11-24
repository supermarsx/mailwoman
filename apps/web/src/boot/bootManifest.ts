// t52w: the build manifest the boot script preloads from (src/boot/boot.js).
//
// Written by vite.config.ts `bootScript()` into dist/index.html as a JSON data
// block, `<script type="application/json" id="mw-boot-manifest">`, placed BEFORE
// the boot script so the parser has it when boot.js runs. A data block is not
// executed (no `script-src` question, no Trusted Types sink). It sits in the
// shell rather than in boot.js because the shell changes with every build
// anyway, while a manifest inside boot.js would rename boot.js on every
// one-line change anywhere in the app.
//
// Pure: imported by vite.config.ts at build time and by the tests; it imports
// nothing from the app.

/** Id of the data block in index.html. */
export const BOOT_MANIFEST_ID = 'mw-boot-manifest';

/**
 * UI font slugs that can be preloaded: the first family of each UI font stack
 * (theme/appearance.ts FONT_STACKS, theme/tokens.ts font.ui), in `fontSlug` form.
 * boot.test.ts checks this against FONT_STACKS.
 */
export const UI_FONT_SLUGS: readonly string[] = ['jost', 'inter', 'roboto', 'open-sans', 'manrope', 'newsreader', 'jetbrains-mono'];

/** `"Open Sans Variable", system-ui` -> `open-sans`; the same rule boot.js applies. */
export function fontSlug(stack: string): string {
  const family = (stack.split(',')[0] ?? '').trim().replace(/^["']|["']$/g, '');
  return family.toLowerCase().replace(/ variable$/, '').replace(/\s+/g, '-');
}

export interface BootManifest {
  v: 1;
  /** The mailbox screen's files beyond the entry's own (JS and CSS). */
  mail: string[];
  /** The sign-in screen's files beyond the entry's own. */
  login: string[];
  /** UI font slug -> its latin woff2 (the subset a Latin UI always uses). */
  fonts: Record<string, string>;
  /** Locale -> its app translation pack. */
  packs: Record<string, string>;
  /** Subresource Integrity for the JS files, filled after the files are written. */
  sri: Record<string, string>;
  /** Files the idle fill downloads after the mailbox is up (sw/offlineTiers.ts), filled with `sri`. */
  idle?: string[];
}

/** What the manifest needs from one Rollup output item. */
export interface ManifestChunk {
  type: 'chunk' | 'asset';
  fileName: string;
  name?: string | undefined;
  isEntry?: boolean;
  imports?: readonly string[];
  importedCss?: readonly string[];
}

const href = (fileName: string): string => `./${fileName}`;

/**
 * The manifest for one build. `screen` sets are the static closure of the
 * named manual chunk (`mail`, `login`: scripts/chunkPlan.ts) minus the entry's
 * closure, which index.html already loads; each set lists scripts first.
 */
export function buildBootManifest(items: readonly ManifestChunk[]): BootManifest {
  const chunks = new Map(items.filter((i) => i.type === 'chunk').map((i) => [i.fileName, i] as const));
  const closure = (start: ManifestChunk | undefined): { js: Set<string>; css: Set<string> } => {
    const js = new Set<string>();
    const css = new Set<string>();
    const stack = start ? [start.fileName] : [];
    while (stack.length > 0) {
      const file = stack.pop()!;
      if (js.has(file)) continue;
      const chunk = chunks.get(file);
      if (!chunk) continue;
      js.add(file);
      for (const sheet of chunk.importedCss ?? []) css.add(sheet);
      stack.push(...(chunk.imports ?? []));
    }
    return { js, css };
  };
  const entry = closure([...chunks.values()].find((c) => c.isEntry === true));
  const screen = (name: string): string[] => {
    const chunk = [...chunks.values()].find((c) => c.name === name && c.isEntry !== true);
    if (!chunk) throw new Error(`boot manifest: no "${name}" chunk in the build`);
    const own = closure(chunk);
    return [
      ...[...own.js].filter((f) => !entry.js.has(f)).sort(),
      ...[...own.css].filter((f) => !entry.css.has(f)).sort(),
    ].map(href);
  };
  const fonts: Record<string, string> = {};
  const packs: Record<string, string> = {};
  for (const item of items) {
    const font = /^assets\/([a-z0-9-]+)-latin-wght-normal-[A-Za-z0-9_-]+\.woff2$/.exec(item.fileName);
    if (font && UI_FONT_SLUGS.includes(font[1]!)) fonts[font[1]!] = href(item.fileName);
    const pack = /^assets\/catalog\.([A-Za-z-]+)\.app-[A-Za-z0-9_-]+\.json$/.exec(item.fileName);
    if (pack) packs[pack[1]!] = href(item.fileName);
  }
  return { v: 1, mail: screen('mail'), login: screen('login'), fonts: sortKeys(fonts), packs: sortKeys(packs), sri: {} };
}

function sortKeys(value: Record<string, string>): Record<string, string> {
  return Object.fromEntries(Object.entries(value).sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)));
}

/** The data block as written into index.html (`<` escaped so no text can close it). */
export function manifestTag(manifest: BootManifest): string {
  return `<script type="application/json" id="${BOOT_MANIFEST_ID}">${JSON.stringify(manifest).replace(/</g, '\\u003c')}</script>`;
}

/** The manifest the page carries; `null` outside a production page (dev server, tests). */
export function pageManifest(doc: Document = document): BootManifest | null {
  try {
    const value: unknown = JSON.parse(doc.getElementById(BOOT_MANIFEST_ID)?.textContent ?? 'null');
    return value !== null && typeof value === 'object' && (value as { v?: unknown }).v === 1 ? (value as BootManifest) : null;
  } catch {
    return null;
  }
}

/** Read the data block back out of a built index.html (null when absent). */
export function readManifestTag(html: string): { manifest: BootManifest; tag: string } | null {
  const match = new RegExp(`<script type="application/json" id="${BOOT_MANIFEST_ID}">([^<]*)</script>`).exec(html);
  if (!match) return null;
  return { manifest: JSON.parse(match[1]!) as BootManifest, tag: match[0] };
}
