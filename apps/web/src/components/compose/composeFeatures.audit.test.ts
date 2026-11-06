// Every composer feature flag needs a label in the English and Portuguese
// catalogs, and a control that consults it. Entries without either are dead
// switches in Settings and the admin panel.
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { expect, it, vi } from 'vitest';

vi.mock('../../api/settings.ts', () => ({ settings: () => ({}), loadSettings: vi.fn(async () => undefined) }));

import { COMPOSE_FEATURE_CATALOGS, COMPOSE_FEATURES } from './composeFeatures.ts';

const root = process.cwd();

function ftlIds(locale: string): Set<string> {
  const ids = new Set<string>();
  for (const cat of COMPOSE_FEATURE_CATALOGS) {
    const text = readFileSync(join(root, 'locales', locale, `${cat}.ftl`), 'utf8');
    for (const m of text.matchAll(/^([a-z][A-Za-z0-9_-]*)\s*=/gm)) ids.add(m[1]!);
  }
  return ids;
}

function sources(dir: string, out: string[] = []): string[] {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) sources(p, out);
    else if (/\.(ts|tsx)$/.test(name) && !/\.test\.tsx?$/.test(name) && name !== 'composeFeatures.ts') out.push(p);
  }
  return out;
}

it('every feature has a label (and description) in en and pt-PT', () => {
  for (const locale of ['en', 'pt-PT']) {
    const ids = ftlIds(locale);
    const missing = COMPOSE_FEATURES.flatMap((f) => [f.labelKey, ...(f.descKey ? [f.descKey] : [])]).filter((k) => !ids.has(k));
    expect(missing, `${locale} catalog lacks`).toEqual([]);
  }
});

it('every feature id is consulted by some control', () => {
  const text = sources(join(root, 'src')).map((p) => readFileSync(p, 'utf8')).join('\n');
  const unused = COMPOSE_FEATURES.map((f) => f.id).filter((id) => !text.includes(`'${id}'`) && !text.includes(`"${id}"`) && !text.includes(`\`${id}\``));
  expect(unused, 'no control reads these ids').toEqual([]);
});
