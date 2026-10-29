// Every shipped catalog parses and formats (t32-admin §2.9, translation passes).
//
// check-catalog.mjs counts message ids line by line, so a translated file with a
// Fluent syntax error still "has" the id while the runtime parser drops it as
// junk and the user sees English (or the raw id) instead. These tests parse each
// `.ftl` of every locale the way the app does and format every message with
// numbers for its variables, so a broken placeable, a select without a default
// variant or a renamed variable fails here rather than on screen.

import { describe, it, expect } from 'vitest';
import { FluentBundle, FluentResource } from '@fluent/bundle';
import { LOCALES, SOURCE_LOCALE } from './locales.ts';

const catalogs = import.meta.glob<string>('../../locales/*/*.ftl', {
  query: '?raw',
  import: 'default',
  eager: true,
});

const MT_HEADER = '# Machine translation (t32, 2026-10). Not reviewed by a native speaker.';

/** Top-level message ids, read line by line as check-catalog.mjs does. */
function lineIds(source: string): string[] {
  return source
    .split(/\r?\n/)
    .map((line) => /^([a-zA-Z][a-zA-Z0-9_-]*)\s*=/.exec(line)?.[1])
    .filter((id): id is string => id !== undefined);
}

/** The `$variables` a message source mentions. */
function variables(source: string): string[] {
  return [...new Set([...source.matchAll(/\{\s*\$([a-zA-Z][a-zA-Z0-9_-]*)/g)].map((m) => m[1] as string))];
}

function byLocale(locale: string): Array<[string, string]> {
  return Object.entries(catalogs)
    .filter(([path]) => path.split('/').at(-2) === locale)
    .map(([path, source]) => [path.split('/').at(-1) as string, source]);
}

describe('shipped catalogs', () => {
  for (const locale of LOCALES) {
    it(`${locale}: every message parses and formats`, () => {
      const problems: string[] = [];
      for (const [file, source] of byLocale(locale)) {
        const bundle = new FluentBundle(locale, { useIsolating: false });
        const errors = bundle.addResource(new FluentResource(source));
        problems.push(...errors.map((e) => `${file}: ${String(e)}`));
        const ids = lineIds(source);
        const args = Object.fromEntries(variables(source).map((v) => [v, 2]));
        for (const id of ids) {
          const message = bundle.getMessage(id);
          if (message?.value == null) {
            problems.push(`${file}: ${id} does not parse`);
            continue;
          }
          for (const n of [0, 1, 2, 5, 21]) {
            const formatErrors: Error[] = [];
            const values = Object.fromEntries(Object.keys(args).map((k) => [k, n]));
            bundle.formatPattern(message.value, values, formatErrors);
            if (formatErrors.length > 0) problems.push(`${file}: ${id}: ${formatErrors.map(String).join('; ')}`);
          }
        }
      }
      expect(problems).toEqual([]);
    });
  }

  it('machine-translated files say so on their first line', () => {
    const unmarked: string[] = [];
    for (const locale of LOCALES) {
      if (locale === SOURCE_LOCALE) continue;
      for (const [file, source] of byLocale(locale)) {
        if (lineIds(source).length === 0) continue; // an empty placeholder stub
        if (source.split(/\r?\n/)[0] !== MT_HEADER) unmarked.push(`${locale}/${file}`);
      }
    }
    expect(unmarked).toEqual([]);
  });
});

// t39: the composer's counts read «1 parágrafos» / "1 paragraphs" until the
// messages got a plural selector.
describe('composer counts agree in number', () => {
  const format = (locale: string, id: string, count: number): string => {
    const source = byLocale(locale).find(([file]) => file === 'mail.ftl')?.[1] ?? '';
    const bundle = new FluentBundle(locale, { useIsolating: false });
    bundle.addResource(new FluentResource(source));
    return bundle.formatPattern(bundle.getMessage(id)!.value!, { count });
  };
  it.each([
    ['en', 'mail-editor-paragraphs', '1 paragraph', '3 paragraphs'],
    ['en', 'mail-editor-words', '1 word', '3 words'],
    ['en', 'mail-editor-characters', '1 character', '3 characters'],
    ['pt-PT', 'mail-editor-paragraphs', '1 parágrafo', '3 parágrafos'],
    ['pt-PT', 'mail-editor-words', '1 palavra', '3 palavras'],
    ['pt-PT', 'mail-editor-characters', '1 caráter', '3 carateres'],
  ])('%s %s', (locale, id, one, three) => {
    expect(format(locale, id, 1)).toBe(one);
    expect(format(locale, id, 3)).toBe(three);
  });
});
