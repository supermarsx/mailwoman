import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { cleanup, render, waitFor, fireEvent, screen } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { MessageList } from './MessageList.tsx';
import { makeClient, mkEmail } from './appHarness.tsx';
import { createAppState, type AppState } from '../state/store.ts';
import { AppContext } from '../state/context.ts';
import { clearSettings, loadSettings } from '../api/settings.ts';
import type { JmapRequest, JmapResponse } from '../api/jmap-types.ts';
import { observeListViewport } from './listViewport.ts';

vi.mock('./initialSync.ts', () => ({ createInitialSync: () => ({ pending: () => false, running: () => false, failed: () => false, retry: () => undefined }) }));
let height = 600;
let resize: () => void;
let disconnects = 0;
beforeEach(async () => {
  localStorage.clear(); clearSettings(); height = 600; disconnects = 0;
  vi.spyOn(HTMLElement.prototype, 'clientHeight', 'get').mockImplementation(function(this: HTMLElement) { return this.classList.contains('list__scroll') || this.classList.contains('list') ? height : 0; });
  vi.stubGlobal('ResizeObserver', class { constructor(callback: () => void) { resize = callback; } observe() {} disconnect() { disconnects++; } });
  await loadSettings({ refresh: true, fetcher: async () => new Response(JSON.stringify({ settings: { 'mail.list.pageSize': { value: 20, locked: true, source: 'global' } } })) });
});
afterEach(() => { cleanup(); clearSettings(); vi.restoreAllMocks(); vi.unstubAllGlobals(); });
const flush = async (): Promise<void> => { for (let i = 0; i < 20; i++) await Promise.resolve(); };
async function mount(count = 200, options: { repeat?: boolean; held?: boolean; failed?: boolean; threaded?: boolean; hidden?: boolean } = {}) {
  if(options.threaded)await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{'mail.list.pageSize':{value:20,locked:true,source:'global'},'mail.conversations.enabled':{value:true,locked:false,source:'user'}}}))});
  const client = makeClient(); const base = client.jmap;
  const calls: { position: number; limit: number; box: string; signal?: AbortSignal }[] = [];
  let release: (() => void) | undefined;
  client.jmap = vi.fn(async (request: JmapRequest, call?: { signal?: AbortSignal }): Promise<JmapResponse> => {
    const query = request.methodCalls.find(([method]) => method === 'Email/query'); if (!query) return base(request, call);
    const args = query[1] as { position?: number; limit?: number; filter?: { inMailbox?: string }; calculateTotal?: boolean };
    const position = args.position ?? 0, limit = args.limit ?? 0, box = args.filter?.inMailbox ?? 'inbox';
    calls.push({ position, limit, box, ...(call?.signal ? { signal: call.signal } : {}) });
    if (position > 0 && options.held) await new Promise<void>(resolve => { release = resolve; });
    if (position > 0 && options.failed) throw new Error('page refused');
    const start = options.repeat ? 0 : position;
    const emails = Array.from({ length: Math.max(0, Math.min(limit, count - start)) }, (_, i) => mkEmail(`${box}-${start+i}`, { mailboxIds: { [box]: true }, ...(options.threaded ? { threadId: 'one-giant-thread' } : {}), ...(options.hidden ? { 'header:Expires:asText': '2001-01-01T00:00:00Z' } : {}) }));
    return { methodResponses: [['Email/query', { accountId: 'acct1', queryState: 'q', ids: emails.map(email => email.id), position, ...(args.calculateTotal ? { total: count } : {}) }, 'q'], ['Email/get', { accountId: 'acct1', state: 's', list: emails, notFound: [] }, 'g']], sessionState: 's' };
  });
  let app!: AppState; const [online, setOnline] = createSignal(true);
  const result = render(() => { app = createAppState(client); return <AppContext.Provider value={{ ...app, online }}><MessageList /></AppContext.Provider>; });
  await app.login({ username: 'me@example.org', password: 'p' });
  await waitFor(() => expect(app.messages().length).toBeGreaterThan(0)); await flush();
  return { app, result, calls, setOnline, release: () => release?.() };
}
it('measures a tall initial viewport and fetches exact bounded continuations without increasing the admin page limit', async () => {
  height = 2400; const { app, result, calls } = await mount();
  await waitFor(() => expect(app.messages()).toHaveLength(40)); await flush();
  expect(calls.map(call => [call.position, call.limit])).toEqual([[0,20],[20,20]]);
  expect(result.container.querySelectorAll('.list__row').length).toBeGreaterThan(30);
  expect(result.container.querySelector('[data-index="33"]')).not.toBeNull();
  expect(app.messages().length * 72).toBeGreaterThan(height);
});
it('fills again on container resize and density change, then stops as soon as the small buffer is present', async () => {
  const { app, calls } = await mount(); expect(calls.map(call => call.position)).toEqual([0]);
  height = 2400; resize(); await waitFor(() => expect(app.messages()).toHaveLength(40));
  app.setDensity('compact'); await waitFor(() => expect(app.messages()).toHaveLength(60)); await flush();
  expect(calls.map(call => [call.position, call.limit])).toEqual([[0,20],[20,20],[40,20]]);
  await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{'mail.list.pageSize':{value:20,source:'global',locked:true},'mail.messageListStyle':{value:'table',source:'user',locked:false}}}))});
  await waitFor(() => expect(app.messages()).toHaveLength(80)); expect(calls.map(call => call.position)).toEqual([0,20,40,60]);
  height = 300; resize(); await flush(); expect(calls).toHaveLength(4);
});
it('exhausts a short folder and stops a duplicate continuation without looping', async () => {
  height = 2400; const first = await mount(25); await waitFor(() => expect(first.app.messages()).toHaveLength(25)); await flush(); expect(first.calls).toHaveLength(2); expect(first.app.hasMore()).toBe(false);
  first.result.unmount();
  const repeated = await mount(200, { repeat: true }); await waitFor(() => expect(repeated.app.hasMore()).toBe(false)); await flush(); expect(repeated.calls).toHaveLength(2); expect(repeated.app.messages()).toHaveLength(20);
});
it('bounds pages without visual growth in a giant collapsed conversation and offers explicit continuation', async () => {
  height = 2400; const { app, calls } = await mount(100000, { threaded: true });
  const more = await screen.findByRole('button', { name: 'Load more messages' }); await flush();
  expect(app.messages()).toHaveLength(100); expect(calls.map(call => call.position)).toEqual([0,20,40,60,80]);
  expect(document.querySelectorAll('.list__row')).toHaveLength(1); expect(more).toBeEnabled();
  // Showing the continuation control may shrink the list; that layout-only
  // observer update must not reset its own budget and restart automatic paging.
  height = 2300; resize(); await flush(); height = 2400; resize(); await flush(); expect(calls).toHaveLength(5);
  fireEvent.click(more); await waitFor(() => expect(app.messages()).toHaveLength(180)); await flush();
  expect(calls.map(call => call.position)).toEqual([0,20,40,60,80,100,120,140,160]);
  expect(screen.getByRole('button', { name: 'Load more messages' })).toBeEnabled();
});
it('bounds entirely hidden expired pages and offers continuation without claiming to still load', async () => {
  await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{'mail.list.pageSize':{value:20,source:'global',locked:true},'mail.expiry.hideExpired':{value:'hide',source:'user',locked:false}}}))});
  height = 2400; const { calls, app } = await mount(100000, { hidden: true });
  expect(await screen.findByRole('button', {name:'Load more messages'})).toBeEnabled(); await flush();
  expect(calls).toHaveLength(5); expect(app.messages()).toHaveLength(100); expect(document.querySelectorAll('.list__row')).toHaveLength(0);
  expect(screen.queryByText('Loading…')).not.toBeInTheDocument(); expect(document.querySelector('section.list')).toHaveAttribute('aria-busy','false');
});
it('does not issue automatic requests offline and resumes filling on reconnection', async () => {
  const { app, calls, setOnline } = await mount(); setOnline(false); height = 2400; resize(); await flush(); expect(calls).toHaveLength(1);
  setOnline(true); await waitFor(() => expect(app.messages()).toHaveLength(40)); expect(calls).toHaveLength(2);
});
it('does not fill a hidden retained list and measures again when it becomes visible', async () => {
  const { app, calls } = await mount(); height = 0; resize(); await flush();
  app.setDensity('compact'); await flush(); expect(calls).toHaveLength(1);
  height = 2400; resize(); await waitFor(() => expect(app.messages()).toHaveLength(60)); expect(calls).toHaveLength(3);
});
it('does not loop after a failed continuation', async () => {
  height = 2400; const { app, calls } = await mount(200, { failed: true }); await waitFor(() => expect(app.loadMoreFailed()).toBe(true)); await flush();
  // The slice owns one immediate automatic retry; viewport notifications must
  // not turn that bounded recovery into another series of requests.
  resize(); await flush(); expect(calls).toHaveLength(3);
});
it('keeps an old folder response from continuing or filling the replacement folder', async () => {
  height = 2400; const state = await mount(200, { held: true }); await waitFor(() => expect(state.calls).toHaveLength(2));
  height = 300; await state.app.selectMailbox('archive'); state.release(); await flush();
  expect(state.calls[1]!.signal?.aborted).toBe(true);
  expect(state.app.messages().every(email => email.id.startsWith('archive-'))).toBe(true);
  expect(state.calls.map(call => [call.box,call.position])).toEqual([['inbox',0],['inbox',20],['archive',0]]);
});
it('cancels queued measurement and disconnects the observer when unmounted', async () => {
  const { result, calls } = await mount(); const before = disconnects;
  height = 2400; resize(); result.unmount(); await flush(); expect(calls).toHaveLength(1); expect(disconnects).toBeGreaterThan(before);
});
it('does not schedule another automatic page when an in-flight continuation settles after unmount', async () => {
  height = 2400; const state = await mount(200, { held: true }); await waitFor(() => expect(state.calls).toHaveLength(2));
  state.result.unmount(); state.release(); await flush(); expect(state.calls).toHaveLength(2);
});
it('does not continue the old account after logout while a page is in flight', async () => {
  height = 2400; const state = await mount(200, { held: true }); await waitFor(() => expect(state.calls).toHaveLength(2));
  await state.app.logout(); state.release(); await flush();
  expect(state.calls[1]!.signal?.aborted).toBe(true); expect(state.app.accountId()).toBeNull(); expect(state.app.messages()).toHaveLength(0); expect(state.calls).toHaveLength(2);
});
it('ignores a queued initial measurement after observer disposal', async () => {
  const element = document.createElement('div'); element.className = 'list__scroll'; const changed = vi.fn();
  const stop = observeListViewport(element, changed); stop(); await flush(); window.dispatchEvent(new Event('resize')); expect(changed).not.toHaveBeenCalled();
});
