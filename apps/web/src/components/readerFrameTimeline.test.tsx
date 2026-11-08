import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { Suspense } from 'solid-js';
import { render, waitFor } from '@solidjs/testing-library';
import { Reader } from './Reader.tsx';
import { makeClient, mkEmail } from './appHarness.tsx';
import { createAppState } from '../state/store.ts';
import { AppContext } from '../state/context.ts';
import { createMessageView } from '../state/slices/messageView.ts';
import { clearSettings } from '../api/settings.ts';
import { clearFeatures } from '../state/features.ts';
import type { Email } from '../api/jmap-types.ts';

const mk = (id: string, subject: string, body: string): Email => mkEmail(id, { subject, from: [{ name: 'Sender', email: 'sender@example.test' }], preview: `preview ${id}`, htmlBody: [{ partId: '1', blobId: null, size: body.length, type: 'text/html' }], bodyValues: { '1': { value: body } } });
beforeEach(() => { clearFeatures(); clearSettings(); });
afterEach(() => { vi.restoreAllMocks(); });

async function setup() {
  const emails = [mk('a', 'Subject A', '<p>Body A secret</p>'), mk('b', 'Subject B', '<p>Body B</p>')];
  const app = createAppState(makeClient({ emails }));
  await app.login({ username: 'me@example.org', password: 'p' });
  const gates = new Map<string, (value: { email: Email; html: string }) => void>();
  const view = createMessageView({
    account: app.accountId,
    load: (_a, id) => new Promise((resolve) => { gates.set(id, resolve); }),
    readDelay: () => null,
    markRead: vi.fn(),
  });
  const shell = render(() => <Suspense fallback={<div data-shell-fallback="">shell</div>}><AppContext.Provider value={app}><Reader view={view} /></AppContext.Provider></Suspense>);
  return { app, view, gates, emails, shell };
}


// Timeline: every overlay/loading container and every iframe insertion, removal and srcdoc change,
// with a monotonic timestamp. Overlay and frame remount must not coincide with opening or switching a message.
type Ev = { t: number; what: string };
function record(root: HTMLElement): { events: Ev[]; stop: () => void } {
  const events: Ev[] = [];
  const t0 = performance.now();
  const note = (what: string) => events.push({ t: Math.round(performance.now() - t0), what });
  const isLoading = (n: Node): boolean => n instanceof HTMLElement && (n.matches('[data-shell-fallback],[data-testid="async-pending"],.async-state--pending') || n.querySelector('[data-shell-fallback],[data-testid="async-pending"],.async-state--pending') !== null);
  const frames = (n: Node): HTMLIFrameElement[] => n instanceof HTMLIFrameElement ? [n] : n instanceof HTMLElement ? [...n.querySelectorAll('iframe')] : [];
  const mo = new MutationObserver((list) => {
    for (const m of list) {
      m.addedNodes.forEach((n) => { if (isLoading(n)) note('overlay+'); frames(n).forEach(() => note('iframe+')); });
      m.removedNodes.forEach((n) => { if (isLoading(n)) note('overlay-'); frames(n).forEach(() => note('iframe-')); });
      if (m.type === 'attributes' && m.target instanceof HTMLIFrameElement && m.attributeName === 'srcdoc') note('iframe srcdoc');
    }
  });
  mo.observe(root, { childList: true, subtree: true, attributes: true, attributeFilter: ['srcdoc'] });
  return { events, stop: () => mo.disconnect() };
}

describe('reader open/switch timeline', () => {
  it('never shows a full-page fallback or loading card, and keeps one frame across a switch', async () => {
    const { view, gates, emails, shell } = await setup();
    const rec = record(shell.container);
    void view.openMessage('a');
    gates.get('a')!({ email: emails[0]!, html: '<p>Body A</p>' });
    await waitFor(() => expect(view.openEmail()?.id).toBe('a'));
    const frame = document.querySelector('iframe');
    void view.openMessage('b');
    await waitFor(() => expect(view.pendingId()).toBe('b'));
    gates.get('b')!({ email: emails[1]!, html: '<p>Body B</p>' });
    await waitFor(() => expect(view.openEmail()?.id).toBe('b'));
    rec.stop();
    const overlays = rec.events.filter((e) => e.what.startsWith('overlay'));
    expect(overlays, JSON.stringify(rec.events)).toEqual([]);
    const afterFirst = rec.events.filter((e) => e.what === 'iframe-');
    expect(afterFirst, JSON.stringify(rec.events)).toEqual([]);
    if (frame) expect(document.querySelector('iframe')).toBe(frame);
  });
});
