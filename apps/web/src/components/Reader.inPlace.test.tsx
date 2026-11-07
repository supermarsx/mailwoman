import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { render, screen, waitFor } from '@solidjs/testing-library';
import { onCleanup, onMount } from 'solid-js';
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

// The message switch updates the open reader in place: the chrome keeps its DOM nodes and the
// body frame component is created once. Mount/cleanup counters stand in for a dev counter.
const counts = { frameMount: 0, frameCleanup: 0 };
vi.mock('./DirectReaderFrame.tsx', () => ({ DirectReaderFrame: (props: {html:string;suspended?:boolean;documentKey?:string;onDisplayed?:(html:string,key:string|undefined)=>void}) => {
  onMount(() => { counts.frameMount++; });
  onCleanup(() => { counts.frameCleanup++; });
  return <iframe class="reader__frame" data-testid="mock-frame" data-suspended={String(props.suspended===true)} data-html={props.html} ref={() => { setTimeout(() => props.onDisplayed?.(props.html, props.documentKey), 0); }} />;
} }));
vi.mock('./security/SpamScore.tsx', () => ({ SpamScore: () => null }));
vi.mock('./security/DeliveryStatus.tsx', () => ({ DeliveryStatus: () => null }));
beforeAll(() => initSync({ module: readFileSync(resolve('src/wasm/mw-sanitize/mw_sanitize_bg.wasm')) }));
const mk = (id: string, subject: string, body: string): Email => mkEmail(id, { subject, from: [{ name: 'Sender', email: 'sender@example.test' }], preview: `preview ${id}`, htmlBody: [{ partId: '1', blobId: null, size: body.length, type: 'text/html' }], bodyValues: { '1': { value: body } } });
beforeEach(() => {
  clearFeatures(); clearSettings(); counts.frameMount = 0; counts.frameCleanup = 0;
  vi.spyOn(getCryptoWorker(), 'sanitizeReader').mockImplementation(async r => sanitizeEmailHtmlReader(r.html, r.blockTrackers, r.cleanLinks));
  vi.stubGlobal('fetch', vi.fn(async () => new Response('{}', { status: 404, headers: { 'content-type': 'application/json' } })));
});
afterEach(() => { vi.restoreAllMocks(); vi.unstubAllGlobals(); });

async function setup() {
  const emails = [mk('a', 'Subject A', '<p>Body A secret</p>'), mk('b', 'Subject B', '<p>Body B</p>'), mk('c', 'Subject C', '<p>Body C</p>')];
  const app = createAppState(makeClient({ emails }));
  await app.login({ username: 'me@example.org', password: 'p' });
  const gates = new Map<string, (value: { email: Email | null; html: string | null }) => void>();
  const view = createMessageView({
    account: app.accountId,
    load: (_a, id) => new Promise((resolve) => { gates.set(id, resolve); }),
    readDelay: () => null,
    markRead: vi.fn(),
  });
  render(() => <AppContext.Provider value={app}><Reader view={view} /></AppContext.Provider>);
  const open = async (id: string) => {
    const done = view.openMessage(id);
    await waitFor(() => expect(gates.has(id)).toBe(true));
    gates.get(id)!({ email: emails.find((e) => e.id === id)!, html: `<p>Body ${id}</p>` });
    gates.delete(id);
    await done;
  };
  return { app, view, gates, emails, open };
}
const toolbar = (): HTMLElement | null => document.querySelector('[role="toolbar"]');
const subject = (): HTMLElement | null => document.querySelector('.reader__subject');
const header = (): HTMLElement | null => document.querySelector('.reader__header');
const frameEl = (): HTMLElement | null => document.querySelector('[data-testid="mock-frame"]');
const settle = async (text: string) => { await waitFor(() => expect(subject()?.textContent).toBe(text)); await waitFor(() => expect(frameEl()?.getAttribute('data-suspended')).toBe('false')); };

describe('Reader switches messages in place', () => {
  it('keeps the toolbar, header, subject and body frame across A to B to C', async () => {
    const { open } = await setup();
    await open('a'); await settle('Subject A');
    const nodes = { toolbar: toolbar(), header: header(), subject: subject(), frame: frameEl() };
    expect(nodes.toolbar && nodes.header && nodes.subject && nodes.frame).toBeTruthy();
    await open('b'); await settle('Subject B');
    expect(toolbar()).toBe(nodes.toolbar); expect(header()).toBe(nodes.header); expect(subject()).toBe(nodes.subject);
    expect(frameEl()).toBe(nodes.frame);
    await open('c'); await settle('Subject C');
    expect(toolbar()).toBe(nodes.toolbar); expect(header()).toBe(nodes.header); expect(subject()).toBe(nodes.subject);
    expect([counts.frameMount,counts.frameCleanup]).toEqual([1,0]); expect(frameEl()).toBe(nodes.frame);
    expect(counts.frameMount).toBe(1); expect(counts.frameCleanup).toBe(0);
  });

  it('while the next message loads the chrome stays, the subject shows the list entry, and the frame is suspended', async () => {
    const { open, view, app } = await setup();
    await open('a'); await settle('Subject A');
    const nodes = { toolbar: toolbar(), subject: subject(), frame: frameEl() };
    void view.openMessage('b');
    await waitFor(() => expect(view.pendingId()).toBe('b'));
    expect(toolbar()).toBe(nodes.toolbar);
    expect(subject()).toBe(nodes.subject);
    expect(frameEl()).toBe(nodes.frame);
    expect(frameEl()?.getAttribute('data-suspended')).toBe('true');
    expect(screen.queryByText('Select a message to read')).toBeNull();
    expect(app.openEmail()).toBeNull();
  });

  it('a late response for A does not overwrite B', async () => {
    const { view, gates, emails } = await setup();
    void view.openMessage('a');
    await waitFor(() => expect(gates.has('a')).toBe(true));
    void view.openMessage('b');
    await waitFor(() => expect(gates.has('b')).toBe(true));
    gates.get('b')!({ email: emails[1]!, html: '<p>Body b</p>' });
    await settle('Subject B');
    gates.get('a')!({ email: emails[0]!, html: '<p>Body a</p>' });
    await new Promise((r) => setTimeout(r, 30));
    expect(subject()?.textContent).toBe('Subject B');
  });

  it('closing releases the chrome; the next open creates it again', async () => {
    const { open, view } = await setup();
    await open('a'); await settle('Subject A');
    view.closeMessage();
    await waitFor(() => expect(screen.getByText('Select a message to read')).toBeInTheDocument());
    expect(counts.frameCleanup).toBe(1);
    await open('b'); await settle('Subject B');
    expect(counts.frameMount).toBe(2);
  });
});
