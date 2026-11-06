// Client error reporting: normalisation and scrubbing, de-duplication, the
// circuit breaker, batching and the beacon fallback, opt-out and admin-off, and
// the failure panel's reference line. The transport and clock are injected; no
// request is made.
import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import {
  LIMITS,
  Reporter,
  fingerprintOf,
  normalise,
  reportText,
  routeTemplate,
  scrubMessage,
  scrubStack,
  setReporterForTests,
  stripQueries,
  uaClass,
  type ClientErrorPolicy,
  type ClientErrorReport,
  type Context,
  type Transport,
} from './clientErrors.ts';
import { AsyncBoundary } from '../components/ErrorBoundary.tsx';

const ctx: Context = {
  route: '#/mail/INBOX/m-0a1b2c3d4e5f6a7b?q=secret',
  build: '26.18+abc123',
  userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36',
  width: 1280,
  locale: 'en',
  online: true,
  reducedMotion: false,
  theme: 'paper',
};

const policy = (over: Partial<ClientErrorPolicy> = {}): ClientErrorPolicy => ({ enabled: true, sampleRate: 1, maxPerSession: 20, captureConsole: false, captureCsp: true, ...over });

interface Rig {
  reporter: Reporter;
  sent: ClientErrorReport[][];
  unloading: boolean[];
  timers: { fn: () => void; ms: number }[];
  clock: { now: number };
  transport: { send: ReturnType<typeof vi.fn> };
}

function rig(result: Awaited<ReturnType<Transport['send']>> | (() => Awaited<ReturnType<Transport['send']>>) = { ok: true, status: 204 }, p: ClientErrorPolicy | null = policy()): Rig {
  const sent: ClientErrorReport[][] = [];
  const unloading: boolean[] = [];
  const timers: { fn: () => void; ms: number }[] = [];
  const clock = { now: 1_000_000 };
  const send = vi.fn(async (reports: ClientErrorReport[], opts: { unloading: boolean }) => {
    sent.push(reports);
    unloading.push(opts.unloading);
    return typeof result === 'function' ? result() : result;
  });
  const reporter = new Reporter({
    transport: { send, loadPolicy: async () => p },
    context: () => ctx,
    now: () => clock.now,
    random: () => 0,
    setTimer: (fn, ms) => {
      timers.push({ fn, ms });
      return timers.length;
    },
    clearTimer: () => undefined,
  });
  return { reporter, sent, unloading, timers, clock, transport: { send } };
}

afterEach(() => {
  cleanup();
  setReporterForTests(null);
  vi.restoreAllMocks();
});

describe('scrubbing', () => {
  it('removes query strings and hash params from urls and stack frames', () => {
    expect(stripQueries('GET /api/x?token=abc#frag failed')).toBe('GET /api/x failed');
    expect(scrubStack('Error: x\n    at f (https://mail.example.test/assets/index-AbC123.js?v=9&k=SECRET:10:20)')).toBe('Error: x\n    at f (/assets/index-AbC123.js:10:20)');
    expect(scrubMessage('failed at https://h.test/a/b?q=secret#x ok')).not.toMatch(/secret|#x/);
    expect(stripQueries('really? yes')).toBe('really? yes');
  });

  it('removes addresses, bearer credentials, long tokens and file names from messages', () => {
    const m = scrubMessage('cannot send to ana@example.test with Bearer abcdefghijklmnop12345 key eyJhbGciOiJIUzI1NiJ9abcdef0123456789 file Q3-invoice.pdf');
    expect(m).not.toContain('ana@example.test');
    expect(m).not.toContain('abcdefghijklmnop12345');
    expect(m).not.toContain('eyJhbGci');
    expect(m).not.toContain('Q3-invoice');
    expect(m).toContain('[address]');
    expect(m).toContain('[token]');
    expect(m).toContain('[file]');
  });

  it('keeps hashed chunk names in stacks and bounds long messages', () => {
    expect(scrubStack('    at x (/assets/AdminSettingsSection-Dk3jF9aS1xQ2.js:1:2)')).toContain('AdminSettingsSection-Dk3jF9aS1xQ2.js:1:2');
    expect(scrubMessage('x'.repeat(5000)).length).toBeLessThanOrEqual(LIMITS.message + 1);
  });

  it('templates routes: ids become :id, query and hash vanish', () => {
    expect(routeTemplate('#/mail/INBOX/m-0a1b2c3d4e5f6a7b?q=secret')).toBe('/mail/INBOX/:id');
    expect(routeTemplate('#/message/12345')).toBe('/message/:id');
    expect(routeTemplate('#/settings/privacy')).toBe('/settings/privacy');
    expect(routeTemplate('')).toBe('/');
  });

  it('classifies the browser coarsely', () => {
    expect(uaClass(ctx.userAgent)).toEqual({ family: 'Chrome', version: '126', os: 'Windows' });
    expect(uaClass('Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15')).toEqual({ family: 'Safari', version: '17', os: 'macOS' });
    expect(uaClass('Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0')).toMatchObject({ family: 'Firefox', os: 'Linux' });
  });
});

describe('normalise', () => {
  it('builds a report with the build, route template, coarse client and a stable fingerprint', () => {
    const error = Object.assign(new TypeError('x is undefined for ana@example.test'), { stack: 'TypeError: x is undefined\n    at f (https://h.test/assets/Reader-AbC123De.js?v=1:10:20)' });
    const a = normalise(error, 'boundary', 'Reader/AttachmentViewer/text', ctx);
    expect(a).toMatchObject({ kind: 'boundary', name: 'TypeError', boundary: 'Reader/AttachmentViewer/text', route: '/mail/INBOX/:id', build: '26.18+abc123', uaFamily: 'Chrome', osFamily: 'Windows', viewport: 'wide', locale: 'en' });
    expect(a.message).not.toContain('ana@example.test');
    expect(a.stack).toBe('TypeError: x is undefined\n    at f (/assets/Reader-AbC123De.js:10:20)');
    expect(a.id).toMatch(/^[0-9a-f-]{36}$/);
    // The same defect in another build (new content hash, other positions) groups together.
    const other = Object.assign(new TypeError('x is undefined for bob@example.test'), { stack: 'TypeError: x is undefined\n    at g (https://h.test/assets/Reader-ZzZ999Yy.js:77:8)' });
    expect(normalise(other, 'boundary', 'Reader/AttachmentViewer/text', ctx).fingerprint).toBe(a.fingerprint);
    expect(a.fingerprint).toBe(fingerprintOf('boundary', 'TypeError', a.message, a.stack, 'Reader/AttachmentViewer/text'));
    // A different defect does not.
    expect(normalise(new RangeError('bad'), 'boundary', 'Reader', ctx).fingerprint).not.toBe(a.fingerprint);
  });

  it('copes with non-Error values', () => {
    expect(normalise('plain string', 'unhandledrejection', '', ctx).name).toBe('NonError');
    expect(normalise(null, 'window.onerror', '', ctx).message).toBe('');
    expect(normalise({ name: 'Weird', message: 'm' }, 'worker', 'Worker/x', ctx).name).toBe('Weird');
  });

  it('reportText carries the reference and no raw url', () => {
    const r = normalise(new Error('boom'), 'boundary', 'Reader', ctx);
    const text = reportText(r);
    expect(text).toContain(`Error reference: ${r.id.slice(0, 8)}`);
    expect(text).toContain('Chrome 126 on Windows');
    expect(text).not.toContain('q=secret');
  });
});

describe('Reporter', () => {
  it('sends a report batched after the flush delay, with the visit id and a handle', async () => {
    const r = rig();
    const handle = r.reporter.report(new Error('boom'), 'boundary', 'Reader');
    expect(handle?.ref).toMatch(/^[0-9a-f]{8}$/);
    expect(handle?.status()).toBe('pending');
    expect(r.timers.length).toBe(1);
    expect(r.timers[0]!.ms).toBe(LIMITS.flushMs);
    await r.reporter.flush(false);
    expect(r.sent).toHaveLength(1);
    expect(r.sent[0]![0]).toMatchObject({ kind: 'boundary', boundary: 'Reader', visit: r.reporter.visit });
    expect(handle?.status()).toBe('reported');
  });

  it('sends the same fingerprint once per 60 s and folds repeats into one counted follow-up', async () => {
    const r = rig();
    for (let i = 0; i < 4; i++) r.reporter.report(new Error('boom'), 'boundary', 'Reader');
    await r.reporter.flush(false);
    expect(r.sent.flat()).toHaveLength(1);
    // The tail timer fires at the end of the window.
    const tail = r.timers.find((t) => t.ms > LIMITS.flushMs)!;
    expect(tail.ms).toBeLessThanOrEqual(LIMITS.dedupeMs);
    r.clock.now += LIMITS.dedupeMs;
    tail.fn();
    await r.reporter.flush(false);
    expect(r.sent.flat()).toHaveLength(2);
    expect(r.sent.flat()[1]).toMatchObject({ count: 3 });
    // After the window a new occurrence is a new report.
    r.reporter.report(new Error('boom'), 'boundary', 'Reader');
    await r.reporter.flush(false);
    expect(r.sent.flat()).toHaveLength(3);
  });

  it('flushes pending counts when the page is unloading', async () => {
    const r = rig();
    for (let i = 0; i < 3; i++) r.reporter.report(new Error('boom'), 'boundary', 'Reader');
    await r.reporter.flush(true);
    expect(r.sent.flat().map((x) => x.count)).toEqual([1, 2]);
    expect(r.unloading.every(Boolean)).toBe(true);
  });

  it('batches different errors into one request of at most 20', async () => {
    const r = rig({ ok: true, status: 204 }, policy({ maxPerSession: 100 }));
    for (let i = 0; i < 25; i++) r.reporter.report(new Error(`distinct ${String.fromCharCode(97 + (i % 26))}${'x'.repeat(i)}`), 'boundary', `B${i}`);
    await r.reporter.flush(false);
    expect(r.sent.map((b) => b.length)).toEqual([20, 5]);
  });

  it('stops at the per-session cap', async () => {
    const r = rig({ ok: true, status: 204 }, policy({ maxPerSession: 2 }));
    const handles = [1, 2, 3].map((i) => r.reporter.report(new Error(`e${i}`), 'boundary', `B${i}`));
    await r.reporter.flush(false);
    expect(r.sent.flat()).toHaveLength(2);
    expect(handles[2]?.status()).toBe('failed');
  });

  it('reports nothing when the administrator switched reporting off', async () => {
    const r = rig({ ok: true, status: 204 }, policy({ enabled: false }));
    const handle = r.reporter.report(new Error('boom'), 'boundary', 'Reader');
    await r.reporter.flush(false);
    expect(r.transport.send).not.toHaveBeenCalled();
    expect(handle?.status()).toBe('off');
  });

  it('honours the person opt-out the server reports, and stays quiet afterwards', async () => {
    const r = rig({ ok: false, status: 200, off: true });
    const first = r.reporter.report(new Error('one'), 'boundary', 'A');
    await r.reporter.flush(false);
    expect(first?.status()).toBe('off');
    const second = r.reporter.report(new Error('two'), 'boundary', 'B');
    await r.reporter.flush(false);
    expect(second?.status()).toBe('off');
    expect(r.transport.send).toHaveBeenCalledTimes(1);
  });

  it('respects the sample rate', async () => {
    const r = rig({ ok: true, status: 204 }, policy({ sampleRate: 0.5 }));
    // random() is 0 in the rig, so 0 < 0.5 sends; patch to 0.9 to drop.
    const dropping = new Reporter({ transport: { send: r.transport.send, loadPolicy: async () => policy({ sampleRate: 0.5 }) }, context: () => ctx, random: () => 0.9, setTimer: () => 1, clearTimer: () => undefined });
    const handle = dropping.report(new Error('x'), 'boundary', 'A');
    await dropping.flush(false);
    expect(handle?.status()).toBe('off');
    expect(r.transport.send).not.toHaveBeenCalled();
  });

  it('opens a circuit breaker after three consecutive transport failures', async () => {
    const r = rig({ ok: false, status: 500 });
    for (let i = 0; i < 3; i++) {
      r.reporter.report(new Error(`fail ${i}`), 'boundary', `B${i}`);
      await r.reporter.flush(false);
    }
    expect(r.transport.send).toHaveBeenCalledTimes(3);
    expect(r.reporter.suspended).toBe(true);
    const handle = r.reporter.report(new Error('after'), 'boundary', 'C');
    await r.reporter.flush(false);
    expect(r.transport.send).toHaveBeenCalledTimes(3);
    expect(handle?.status()).toBe('failed');
    r.clock.now += LIMITS.breakerMs + 1;
    expect(r.reporter.suspended).toBe(false);
  });

  it('pauses after a 429 for the Retry-After time', async () => {
    const r = rig({ ok: false, status: 429, retryAfter: 40 });
    r.reporter.report(new Error('a'), 'boundary', 'A');
    await r.reporter.flush(false);
    expect(r.reporter.suspended).toBe(true);
    r.clock.now += 41_000;
    expect(r.reporter.suspended).toBe(false);
  });

  it('never reports while it is reporting, and never throws on a broken transport', async () => {
    const broken = new Reporter({
      transport: { send: () => { throw new Error('transport exploded'); }, loadPolicy: async () => policy() },
      context: () => ctx,
      setTimer: () => 1,
      clearTimer: () => undefined,
    });
    expect(() => broken.report(new Error('x'), 'boundary', 'A')).not.toThrow();
    await expect(broken.flush(false)).resolves.toBeUndefined();
    const throwingContext = new Reporter({ transport: { send: async () => ({ ok: true, status: 204 }), loadPolicy: async () => null }, context: () => { throw new Error('no document'); } });
    expect(throwingContext.report(new Error('x'), 'boundary', 'A')).toBeNull();
  });

  it('throttles console errors to five a minute', () => {
    const r = rig();
    const results = Array.from({ length: 7 }, () => r.reporter.allowConsole());
    expect(results.filter(Boolean)).toHaveLength(LIMITS.consolePerMinute);
    r.clock.now += 61_000;
    expect(r.reporter.allowConsole()).toBe(true);
  });

  it('keeps a report within the 16 KB cap', async () => {
    const r = rig();
    const huge = Object.assign(new Error('boom'), { stack: Array.from({ length: 60 }, (_, i) => `    at fn${i} (/assets/chunk-AbC${i}.js:${i}:${i})${'x'.repeat(400)}`).join('\n') });
    r.reporter.report(huge, 'boundary', 'A');
    await r.reporter.flush(false);
    expect(JSON.stringify(r.sent[0]![0]).length).toBeLessThanOrEqual(LIMITS.report);
  });
});

describe('failure panel', () => {
  function Thrower(): never {
    throw Object.assign(new TypeError('section exploded for ana@example.test'), { stack: 'TypeError: section exploded\n    at Section (/assets/Section-AbC123De.js?x=1:3:4)' });
  }

  it('shows the error reference, the report state and a copy button; Try again stays', async () => {
    const r = rig();
    setReporterForTests(r.reporter);
    const writeText = vi.fn(async () => undefined);
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true });
    render(() => <AsyncBoundary name="Reader/AttachmentViewer/TextViewer"><Thrower /></AsyncBoundary>);
    expect(screen.getByRole('alert').textContent).toContain('Something went wrong');
    const ref = await screen.findByTestId('error-reference-id');
    expect(ref.textContent).toMatch(/^[0-9a-f]{8}$/);
    expect(screen.getByTestId('async-retry')).toBeTruthy();
    await r.reporter.flush(false);
    expect(r.sent[0]![0]).toMatchObject({ boundary: 'Reader/AttachmentViewer/TextViewer', kind: 'boundary', name: 'TypeError' });
    expect(r.sent[0]![0]!.message).not.toContain('ana@example.test');
    await waitFor(() => expect(document.querySelector('[data-report-status]')?.getAttribute('data-report-status')).toBe('reported'));
    fireEvent.click(screen.getByTestId('error-copy'));
    await waitFor(() => expect(writeText).toHaveBeenCalledTimes(1));
    const copied = (writeText.mock.calls[0] as unknown as [string])[0];
    expect(copied).toContain(ref.textContent!);
    expect(copied).not.toContain('ana@example.test');
    expect(copied).not.toContain('x=1');
  });

  it('draws the panel even when reporting is switched off or the reporter breaks', () => {
    const r = rig({ ok: true, status: 204 }, policy({ enabled: false }));
    setReporterForTests(r.reporter);
    render(() => <AsyncBoundary><Thrower /></AsyncBoundary>);
    expect(screen.getByRole('alert').textContent).toContain('Something went wrong');
    cleanup();
    setReporterForTests({ report: () => { throw new Error('reporter broke'); } } as unknown as Reporter);
    render(() => <AsyncBoundary><Thrower /></AsyncBoundary>);
    expect(screen.getByRole('alert').textContent).toContain('Something went wrong');
    expect(screen.queryByTestId('error-reference')).toBeNull();
  });
});
