// Link check for the admin "Learn more" links (t32-cfg UX4): every
// `adminDocsHref(area, id)` call under screens/Admin names a file
// `docs/admin/<area>.md` that has an `<a id="<id>"></a>` anchor. The area may be
// given as a literal or as the file's `const AREA = '<area>'`.

import { describe, expect, it } from 'vitest';
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative, resolve } from 'node:path';
import { adminDocsHref } from './docsLink.ts';

const ADMIN = resolve(process.cwd(), 'src/screens/Admin');
const DOCS = resolve(process.cwd(), '../../docs/admin');

function walk(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) return walk(p);
    return /\.tsx?$/.test(name) && !/\.test\.tsx?$/.test(name) ? [p] : [];
  });
}

interface Call {
  file: string;
  area: string;
  id: string | undefined;
}

export function docsCalls(src: string, file = ''): Call[] {
  const area = /\bconst\s+AREA\s*=\s*'([^']+)'/.exec(src)?.[1];
  const re = /adminDocsHref\(\s*(AREA|'[^']+')\s*(?:,\s*'([^']+)')?\s*\)/g;
  return [...src.matchAll(re)].map((m) => ({
    file,
    area: m[1] === 'AREA' ? (area ?? '<no AREA const>') : m[1]!.slice(1, -1),
    id: m[2],
  }));
}

describe('admin docs anchors', () => {
  it('the link keeps the docs/admin/<area>.md#<id> path a link check reads', () => {
    expect(adminDocsHref('sso', 'issuerUrl')).toMatch(/\/docs\/admin\/sso\.md#issuerUrl$/);
    expect(adminDocsHref('sso')).toMatch(/\/docs\/admin\/sso\.md$/);
  });

  it('finds literal and AREA calls', () => {
    expect(docsCalls("const AREA = 'x';\nadminDocsHref(AREA, 'a'); adminDocsHref('y', 'b'); adminDocsHref('z')")).toEqual([
      { file: '', area: 'x', id: 'a' },
      { file: '', area: 'y', id: 'b' },
      { file: '', area: 'z', id: undefined },
    ]);
  });

  it('every call names an existing file and anchor', () => {
    const calls = walk(ADMIN).flatMap((f) => docsCalls(readFileSync(f, 'utf8'), relative(ADMIN, f)));
    expect(calls.length).toBeGreaterThan(0);
    const missing = calls.filter((c) => {
      const doc = join(DOCS, `${c.area}.md`);
      if (!existsSync(doc)) return true;
      return c.id !== undefined && !readFileSync(doc, 'utf8').includes(`<a id="${c.id}"></a>`);
    });
    expect(missing.map((c) => `${c.file}: docs/admin/${c.area}.md#${c.id ?? ''}`)).toEqual([]);
  });
});
