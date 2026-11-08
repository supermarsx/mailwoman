// Guard for docs/ui-loading.md: the full-page loading card (AsyncPending) and
// cover-the-screen overlays are for the app boot, account switch and the first
// visit of a top-level screen only. Everything else uses InlineLoading.
import { describe, expect, it } from 'vitest';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative, sep } from 'node:path';

const SRC = join(__dirname, '..');
function walk(dir: string, out: string[] = []): string[] {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) walk(p, out);
    else if (/\.(tsx?|css)$/.test(name) && !/\.test\./.test(name)) out.push(p);
  }
  return out;
}
const files = walk(SRC).map((p) => ({ rel: relative(SRC, p).split(sep).join('/'), text: readFileSync(p, 'utf8') }));

// Each entry says why it may use the skeleton card.
const ASYNC_PENDING_ALLOWED: Record<string, string> = {
  'App.tsx': 'boot / authentication handshake',
  'components/ErrorBoundary.tsx': 'lazy-route chunk of a top-level screen, first visit',
  'components/AsyncState.tsx': 'definition',
  'components/MessageList.tsx': 'list-region row skeleton while the folder has no rows yet',
  'components/FolderPane.tsx': 'list-region skeleton while the folder has no rows yet',
  'screens/Mailbox.tsx': 'folder-tree rows before the first mailbox list; first visit of a module screen',
};

describe('loading policy', () => {
  it('references AsyncPending only from the allowlist', () => {
    const offenders = files.filter((f) => /\bAsyncPending\b/.test(f.text) && !(f.rel in ASYNC_PENDING_ALLOWED) && /.tsx?$/.test(f.rel)).map((f) => f.rel);
    expect(offenders).toEqual([]);
  });
  it('has no loading-named element covering its parent with a backdrop or blur', () => {
    const bad = files.filter((f) => /\.css(\.ts)?$/.test(f.rel)).flatMap((f) => {
      const hits: string[] = [];
      for (const m of f.text.matchAll(/(?:export const|globalStyle\(\s*['"`])\s*([\w.\-_ >\[\]="]*(?:loading|pending|busy)[\w.\-_ >\[\]="]*)[\s\S]{0,400}?\}\)/gi)) {
        const block = m[0];
        if (/inset:\s*(0|'0')\b/.test(block) && /(backdrop|blur|background)/.test(block) && !/pointerEvents:\s*'none'/.test(block)) hits.push(`${f.rel}: ${m[1]}`);
      }
      return hits;
    });
    expect(bad).toEqual([]);
  });
  it('draws the reader frame wait as a label, not a cover', () => {
    const css = files.find((f) => f.rel === 'components/readerPane.css.ts')!.text;
    const line = css.split('\n').find((l) => l.includes('.reader__frame-loading'))!;
    expect(line).not.toMatch(/inset:\s*0/);
    expect(line).toMatch(/pointerEvents:\s*'none'/);
  });
});
