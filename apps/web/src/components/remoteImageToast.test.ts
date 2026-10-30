// Loading-images toast and the single stats read (t55): start, done, failed +
// Retry, message switch, fast load suppressed, one in-place toast, and at most
// one /api/rc/stats request per displayed body (none once the message is gone).
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { createRoot, createSignal } from 'solid-js';
import {
  createImageLoadToast,
  createRemoteImageLoad,
  AUTO_RETRY_DELAY_MS,
  MAX_AUTO_RETRIES,
  LOAD_CAP_MS,
  POLL_MS,
  STATS_SETTLE_MS,
  TOAST_DELAY_MS,
  TOAST_DONE_MS,
  type RemoteImageLoadSource,
  type ToastSink,
} from './remoteImageToast.ts';
import type { RemoteContentStats } from '../api/remote-content.ts';

interface Shown { tone: string; message: string; id: string | undefined; duration: number | undefined; action: string | undefined }
function sink(): ToastSink & { shown: Shown[]; dismissed: string[]; run: () => void } {
  const shown: Shown[] = []; const dismissed: string[] = []; let last: (() => void) | undefined;
  return {
    shown, dismissed,
    show: (tone, message, opts) => { shown.push({ tone, message, id: opts?.id, duration: opts?.duration, action: opts?.action?.label }); last = opts?.action ? () => void opts.action?.run() : undefined; return opts?.id ?? ''; },
    dismiss: (id) => { dismissed.push(id); },
    run: () => last?.(),
  };
}

beforeEach(() => { vi.useFakeTimers(); });
afterEach(() => { vi.useRealTimers(); });

describe('createImageLoadToast', () => {
  it('a body displayed within 300 ms shows nothing', () => {
    const s = sink(); const tt = createImageLoadToast({ id: 'r', retry: () => undefined, sink: s });
    tt.start('L1', 12);
    vi.advanceTimersByTime(TOAST_DELAY_MS - 50);
    tt.displayed('L1');
    tt.finish('L1', 0);
    vi.advanceTimersByTime(5_000);
    expect(s.shown).toEqual([]);
  });

  it('a slow load shows one toast, replaced in place by "Images loaded" for 2 s', () => {
    const s = sink(); const tt = createImageLoadToast({ id: 'r', retry: () => undefined, sink: s });
    tt.start('L1', 12);
    tt.start('L1', 12); // re-render of the same body: no restart
    vi.advanceTimersByTime(TOAST_DELAY_MS);
    tt.finish('L1', 0);
    expect(s.shown.map((x) => [x.tone, x.message, x.id])).toEqual([['info', 'Loading images…', 'r'], ['success', 'Images loaded', 'r']]);
    expect(s.shown[1]?.duration).toBe(TOAST_DONE_MS);
  });

  it('failures replace the toast with a count and Retry, even after a fast load', () => {
    const retry = vi.fn(); const s = sink(); const tt = createImageLoadToast({ id: 'r', retry, sink: s });
    tt.start('L1', 3); tt.displayed('L1'); tt.finish('L1', 2);
    expect(s.shown).toHaveLength(1);
    expect(s.shown[0]).toMatchObject({ tone: 'warning', message: "2 images couldn't load", id: 'r', action: 'Retry' });
    s.run();
    expect(retry).toHaveBeenCalledOnce();
  });

  it('switching message closes the old toast; late answers for it are ignored', () => {
    const s = sink(); const tt = createImageLoadToast({ id: 'r', retry: () => undefined, sink: s });
    tt.start('A', 4); vi.advanceTimersByTime(TOAST_DELAY_MS);
    tt.start('B', 0);
    expect(s.dismissed).toEqual(['r']);
    tt.finish('A', 1);
    expect(s.shown).toHaveLength(1);
  });

  it('no remote images: no toast', () => {
    const s = sink(); const tt = createImageLoadToast({ id: 'r', retry: () => undefined, sink: s });
    tt.start('A', 0); vi.advanceTimersByTime(LOAD_CAP_MS);
    expect(s.shown).toEqual([]);
  });
});

describe('createRemoteImageLoad', () => {
  const stats = (over: Partial<RemoteContentStats> = {}): RemoteContentStats => ({ count: 2, limited: false, windowSecs: 3600, ...over });
  function setup(fetch = vi.fn(async () => stats())) {
    const s = sink();
    const [source, setSource] = createSignal<RemoteImageLoadSource | null>(null);
    const [displayed, setDisplayed] = createSignal(0);
    const [enabled, setEnabled] = createSignal(true);
    const retry = vi.fn();
    let dispose = (): void => undefined;
    const hook = createRoot((d) => { dispose = d; return createRemoteImageLoad({ source, displayed, toastEnabled: enabled, retry, fetchStats: fetch, sink: s }); });
    return { s, fetch, setSource, setDisplayed, setEnabled, retry, hook, dispose: () => dispose() };
  }
  const body = (load: string, over: Partial<RemoteImageLoadSource> = {}): RemoteImageLoadSource => ({ scope: 'acc:M1', emailId: 'M1', load, total: 12, stats: true, ...over });

  it('opening one message with images reads the stats once, after display', async () => {
    const h = setup();
    h.setSource(body('L1'));
    vi.advanceTimersByTime(TOAST_DELAY_MS);
    expect(h.s.shown.map((x) => x.message)).toEqual(['Loading images…']);
    h.setDisplayed(1); h.setDisplayed(2); // second display debounces, no second read
    await vi.advanceTimersByTimeAsync(STATS_SETTLE_MS);
    expect(h.fetch).toHaveBeenCalledTimes(1);
    expect(h.hook.trackers()).toEqual({ count: 2, limited: false });
    expect(h.s.shown.map((x) => x.message)).toEqual(['Loading images…', 'Images loaded']);
    await vi.advanceTimersByTimeAsync(60_000);
    expect(h.fetch).toHaveBeenCalledTimes(1);
    h.dispose();
  });

  it('Show images (a new load) reads once more; leaving the message makes no more requests', async () => {
    const h = setup();
    h.setSource(body('L1', { total: 0, stats: false }));
    h.setDisplayed(1);
    await vi.advanceTimersByTimeAsync(10_000);
    expect(h.fetch).toHaveBeenCalledTimes(0);
    h.setSource(body('L2'));
    h.setDisplayed(2);
    await vi.advanceTimersByTimeAsync(STATS_SETTLE_MS);
    expect(h.fetch).toHaveBeenCalledTimes(1);
    h.setSource(body('L3'));
    h.setDisplayed(3);
    h.setSource(null); // closed before the settle delay ran out
    await vi.advanceTimersByTimeAsync(60_000);
    expect(h.fetch).toHaveBeenCalledTimes(1);
    expect(h.s.dismissed.length).toBeGreaterThan(0);
    h.dispose();
  });

  it('never exposes a zero count and never goes backwards', async () => {
    const counts = [0, 3, 1];
    const h = setup(vi.fn(async () => stats({ count: counts.shift() ?? 0 })));
    for (const [i, load] of ['L1', 'L2', 'L3'].entries()) {
      h.setSource(body(load)); h.setDisplayed(i + 1);
      await vi.advanceTimersByTimeAsync(STATS_SETTLE_MS);
      if (i === 0) expect(h.hook.trackers()).toBeNull();
    }
    expect(h.hook.trackers()).toEqual({ count: 3, limited: false });
    h.dispose();
  });

  it('server-reported failures become the Retry toast', async () => {
    const h = setup(vi.fn(async () => stats({ imagesFailed: 2 })));
    h.setSource(body('L1')); h.setDisplayed(1);
    await vi.advanceTimersByTimeAsync(STATS_SETTLE_MS);
    expect(h.s.shown.at(-1)).toMatchObject({ tone: 'warning', message: "2 images couldn't load", action: 'Retry' });
    h.s.run();
    expect(h.retry).toHaveBeenCalledOnce();
    h.dispose();
  });

  // t67: the toast stayed on "Loading images…" after the images had failed.
  it('the frame never reports the body displayed: polling settles on the failure count (t67)', async () => {
    const answers = [stats({ imagesLoaded: 0, imagesFailed: 1 }), stats({ imagesLoaded: 0, imagesFailed: 5 })];
    const h = setup(vi.fn(async () => answers.shift() ?? stats({ imagesLoaded: 0, imagesFailed: 5 })));
    h.setSource(body('L1', { total: 5 }));
    await vi.advanceTimersByTimeAsync(POLL_MS);
    expect(h.s.shown.map((x) => x.message)).toEqual(['Loading images…', 'Loading images… 0/5']);
    await vi.advanceTimersByTimeAsync(POLL_MS);
    expect(h.s.shown.at(-1)).toMatchObject({ tone: 'warning', message: "5 images couldn't load", action: 'Retry' });
    const polls = h.fetch.mock.calls.length;
    await vi.advanceTimersByTimeAsync(LOAD_CAP_MS);
    expect(h.fetch).toHaveBeenCalledTimes(polls);
    h.dispose();
  });

  it('a late display reuses final counters from polling without another stats request',async()=>{
    const h=setup(vi.fn(async()=>stats({imagesLoaded:10,imagesFailed:2})));
    h.setSource(body('L1'));
    await vi.advanceTimersByTimeAsync(POLL_MS);
    expect(h.fetch).toHaveBeenCalledTimes(1);
    expect(h.hook.trackers()).toEqual({count:2,limited:false});
    h.setDisplayed(1);h.setDisplayed(2);
    await vi.advanceTimersByTimeAsync(LOAD_CAP_MS);
    expect(h.fetch).toHaveBeenCalledTimes(1);
    expect(h.s.shown.at(-1)).toMatchObject({tone:'warning',message:"2 images couldn't load"});
    h.dispose();
  });

  it('a re-render of the same body while it loads does not drop its answer (t67)', async () => {
    const h = setup(vi.fn(async () => stats({ imagesFailed: 2 })));
    h.setSource(body('L1'));
    await vi.advanceTimersByTimeAsync(TOAST_DELAY_MS);
    h.setDisplayed(1);
    h.setSource(body('L1', { stats: false })); // the grant refetches: same body, other flags
    h.setSource(body('L1'));
    await vi.advanceTimersByTimeAsync(STATS_SETTLE_MS);
    expect(h.s.shown.at(-1)).toMatchObject({ tone: 'warning', message: "2 images couldn't load" });
    h.dispose();
  });

  it('no answer by the cap closes the toast instead of claiming the images loaded (t67)', async () => {
    const h = setup(vi.fn(async () => stats({ imagesLoaded: 1, imagesFailed: 0 })));
    h.setSource(body('L1', { total: 5 }));
    await vi.advanceTimersByTimeAsync(LOAD_CAP_MS + POLL_MS);
    expect(h.s.shown.map((x) => x.message)).toEqual(['Loading images…', 'Loading images… 1/5']);
    expect(h.s.dismissed).toContain('remote-images');
    h.dispose();
  });

  it('Retry asks for a retry signing of this message, then reloads (t67)', async () => {
    const h = setup(vi.fn(async () => stats({ imagesLoaded: 0, imagesFailed: 3 })));
    h.setSource(body('L1', { total: 3 }));
    await vi.advanceTimersByTimeAsync(POLL_MS);
    const mod = await import('../api/remote-content.ts');
    const fetchMock = vi.fn(async () => new Response(JSON.stringify({ images: [], parts: [], files: [], refused: [] }), { status: 200, headers: { 'content-type': 'application/json' } }));
    vi.stubGlobal('fetch', fetchMock);
    h.s.run();
    expect(h.retry).toHaveBeenCalledOnce();
    await mod.signContent('M1', { urls: ['https://img.example/a.png'] });
    expect(JSON.parse(String((fetchMock.mock.calls[0] as unknown as [string, RequestInit])[1].body))).toMatchObject({ retry: true });
    vi.unstubAllGlobals();
    h.dispose();
  });

  it('shows actual progress and retries only transient failed references, then settles on success',async()=>{
    const reference='a'.repeat(64);let recovered=false;
    const h=setup(vi.fn(async()=>stats(recovered?{imagesLoaded:2,imagesFailed:0,failedImageRefs:[],retryableImageRefs:[]}:{imagesLoaded:1,imagesFailed:1,failedImageRefs:[reference],retryableImageRefs:[reference]})));
    h.setSource(body('L1',{total:2}));await vi.advanceTimersByTimeAsync(POLL_MS);
    expect(h.s.shown.some(x=>x.message==='Loading images… 1/2')).toBe(true);
    await vi.advanceTimersByTimeAsync(AUTO_RETRY_DELAY_MS);
    expect(h.retry).toHaveBeenCalledExactlyOnceWith([reference]);
    recovered=true;h.setSource(body('L2',{total:2}));await vi.advanceTimersByTimeAsync(POLL_MS);
    expect(h.s.shown.some(x=>x.message.includes('Retry 1/2'))).toBe(true);
    expect(h.s.shown.at(-1)?.message).toBe('Images loaded');expect(h.hook.failures()).toBeNull();
    await vi.advanceTimersByTimeAsync(LOAD_CAP_MS);expect(h.retry).toHaveBeenCalledTimes(1);h.dispose();
  });
  it('permanent failures never auto-retry but remain available for explicit Retry',async()=>{
    const reference='b'.repeat(64);const h=setup(vi.fn(async()=>stats({imagesLoaded:0,imagesFailed:1,failedImageRefs:[reference],retryableImageRefs:[]})));
    h.setSource(body('L1',{total:1}));await vi.advanceTimersByTimeAsync(POLL_MS+AUTO_RETRY_DELAY_MS*3);
    expect(h.retry).not.toHaveBeenCalled();h.hook.retry();expect(h.retry).toHaveBeenCalledExactlyOnceWith([reference]);h.dispose();
  });
  it('automatic recovery is capped and a message/account switch cancels pending retries',async()=>{
    const reference='c'.repeat(64);const h=setup(vi.fn(async()=>stats({imagesLoaded:0,imagesFailed:1,failedImageRefs:[reference],retryableImageRefs:[reference]})));
    h.setSource(body('L0',{total:1}));
    for(let attempt=1;attempt<=MAX_AUTO_RETRIES;attempt++){
      await vi.advanceTimersByTimeAsync(POLL_MS+AUTO_RETRY_DELAY_MS);expect(h.retry).toHaveBeenCalledTimes(attempt);h.setSource(body(`L${attempt}`,{total:1}));
    }
    await vi.advanceTimersByTimeAsync(LOAD_CAP_MS);expect(h.retry).toHaveBeenCalledTimes(MAX_AUTO_RETRIES);
    h.setSource(body('new',{scope:'other-account:other-message',total:1}));await vi.advanceTimersByTimeAsync(POLL_MS);
    h.setSource(null);await vi.advanceTimersByTimeAsync(AUTO_RETRY_DELAY_MS);expect(h.retry).toHaveBeenCalledTimes(MAX_AUTO_RETRIES);h.dispose();
  });

  it('toast off (admin/user setting): no toast, the stats read still happens once', async () => {
    const h = setup();
    h.setEnabled(false);
    h.setSource(body('L1'));
    await vi.advanceTimersByTimeAsync(TOAST_DELAY_MS);
    h.setDisplayed(1);
    await vi.advanceTimersByTimeAsync(STATS_SETTLE_MS);
    expect(h.s.shown).toEqual([]);
    expect(h.fetch).toHaveBeenCalledTimes(1);
    h.dispose();
  });
});
