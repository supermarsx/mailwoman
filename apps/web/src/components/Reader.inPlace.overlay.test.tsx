import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { render, screen, waitFor } from '@solidjs/testing-library';
import { Suspense } from 'solid-js';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { Reader } from './Reader.tsx';
import { makeClient, mkEmail } from './appHarness.tsx';
import { createAppState } from '../state/store.ts';
import { AppContext } from '../state/context.ts';
import { createMessageView } from '../state/slices/messageView.ts';
import { clearSettings } from '../api/settings.ts';
import { clearFeatures } from '../state/features.ts';
import { getCryptoWorker } from '../crypto/index.ts';
import { initSync, sanitizeEmailHtmlReader } from '../wasm/mw-sanitize/mw_sanitize.js';
import type { Email } from '../api/jmap-types.ts';


vi.mock('./security/SpamScore.tsx', () => ({ SpamScore: () => null }));
vi.mock('./security/DeliveryStatus.tsx', () => ({ DeliveryStatus: () => null }));
beforeAll(() => initSync({ module: readFileSync(resolve('src/wasm/mw-sanitize/mw_sanitize_bg.wasm')) }));
const mk = (id: string, subject: string, body: string): Email => mkEmail(id, { subject, from: [{ name: 'Sender', email: 'sender@example.test' }], preview: `preview ${id}`, htmlBody: [{ partId: '1', blobId: null, size: body.length, type: 'text/html' }], bodyValues: { '1': { value: body } } });
beforeEach(() => {
  clearFeatures(); clearSettings();
  vi.spyOn(getCryptoWorker(), 'sanitizeReader').mockImplementation(async r => sanitizeEmailHtmlReader(r.html, r.blockTrackers, r.cleanLinks));
  vi.stubGlobal('fetch', vi.fn(async () => new Response('{}', { status: 404, headers: { 'content-type': 'application/json' } })));
});
afterEach(() => { vi.restoreAllMocks(); vi.unstubAllGlobals(); });

// Timeline evidence for the Firefox "Layout was forced ... frame/srcdoc" warning: it appears when a
// page-level fallback (the route Suspense overlay) shows while a body iframe is created. A message
// switch must create no iframe element after the first and insert no overlay at all.
describe('Reader message switch: iframes and overlay', () => {
  it('creates the body iframe once and never inserts the Suspense overlay across A to B to C and back', async () => {
    const emails = [mk('a', 'Subject A', '<p>A</p>'), mk('b', 'Subject B', '<p>B</p>'), mk('c', 'Subject C', '<p>C</p>')];
    const app = createAppState(makeClient({ emails }));
    await app.login({ username: 'me@example.org', password: 'p' });
    const view = createMessageView({ account: app.accountId, load: async (_a, id) => ({ email: emails.find((e) => e.id === id)!, html: `<p>${id}</p>` }), readDelay: () => null, markRead: vi.fn() });
    const timeline: string[] = [];
    const observer = new MutationObserver((records) => { for (const record of records) { for (const node of record.addedNodes) { if (!(node instanceof Element)) continue; if (node.matches('iframe') || node.querySelector('iframe')) timeline.push('iframe+'); if (node.matches('[data-testid="overlay"]')) timeline.push('overlay+'); } for (const node of record.removedNodes) if (node instanceof Element && (node.matches('iframe') || node.querySelector('iframe'))) timeline.push('iframe-'); } });
    observer.observe(document.body, { childList: true, subtree: true });
    render(() => <AppContext.Provider value={app}><Suspense fallback={<div data-testid="overlay" />}><Reader view={view} /></Suspense></AppContext.Provider>);
    const shown = async (text: string) => { await waitFor(() => expect(document.querySelector('.reader__subject')?.textContent).toBe(text)); await waitFor(() => expect(document.querySelector('iframe.reader__frame')).not.toBeNull(), { timeout: 4000 }); };
    await view.openMessage('a'); await shown('Subject A');
    const frame = document.querySelector('iframe.reader__frame');
    for (const id of ['b', 'c', 'a', 'c']) { await view.openMessage(id); await shown(`Subject ${id.toUpperCase()}`); expect(document.querySelector('iframe.reader__frame')).toBe(frame); }
    await new Promise((r) => setTimeout(r, 50));
    observer.disconnect();
    expect(timeline.filter((e) => e === 'overlay+')).toEqual([]);
    expect(timeline.filter((e) => e === 'iframe+')).toHaveLength(1);
    expect(timeline.filter((e) => e === 'iframe-')).toEqual([]);
  });
});
