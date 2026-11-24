import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const policy = vi.hoisted(() => ({ value: { prefetchEnabled: true } as { prefetchEnabled: boolean } | null }));
vi.mock('./performanceBootstrap.ts', () => ({ ensureWebPerformance: () => Promise.resolve(policy.value) }));
const prefetched = vi.hoisted(() => [] as string[]);
vi.mock('./chunkRetry.ts', () => ({ prefetchFile: (url: string) => { prefetched.push(url); return Promise.resolve(true); } }));

import { manifestTag, type BootManifest } from './bootManifest.ts';
import { resetWarmMailbox, warmMailbox } from './warmMailbox.ts';

const MANIFEST: BootManifest = { v: 1, mail: ['./assets/mail-A.js', './assets/mail-B.css', 'https://elsewhere.test/x.js'], login: [], fonts: {}, packs: {}, sri: {} };

function install(): void {
  const holder = document.createElement('div');
  holder.innerHTML = manifestTag(MANIFEST);
  document.head.append(holder.firstElementChild!);
}

beforeEach(() => {
  resetWarmMailbox();
  prefetched.length = 0;
  policy.value = { prefetchEnabled: true };
  install();
});
afterEach(() => {
  document.getElementById('mw-boot-manifest')?.remove();
  vi.restoreAllMocks();
});

describe('warmMailbox (sign-in screen, t52w)', () => {
  it('downloads the mailbox files of the manifest once per page', async () => {
    await warmMailbox();
    await warmMailbox();
    expect(prefetched).toEqual(['./assets/mail-A.js', './assets/mail-B.css'].map((h) => new URL(h, document.baseURI).href));
  });

  it('does nothing when prefetch is off, the policy is unknown, or the browser asks to save data', async () => {
    policy.value = { prefetchEnabled: false };
    await warmMailbox();
    resetWarmMailbox();
    policy.value = null;
    await warmMailbox();
    resetWarmMailbox();
    policy.value = { prefetchEnabled: true };
    Object.defineProperty(navigator, 'connection', { configurable: true, value: { saveData: true } });
    try { await warmMailbox(); } finally { delete (navigator as { connection?: unknown }).connection; }
    expect(prefetched).toEqual([]);
  });

  it('does nothing without a manifest (development server, tests)', async () => {
    document.getElementById('mw-boot-manifest')?.remove();
    await warmMailbox();
    expect(prefetched).toEqual([]);
  });
});
