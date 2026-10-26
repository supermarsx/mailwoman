import { afterEach, expect, it, vi } from 'vitest';
import { createSessionActivity } from './sessionActivity.ts';
import { activateAccountPreferences, suspendAccountPreferences } from './accountPreferenceLifecycle.ts';
const settle = async (): Promise<void> => { await Promise.resolve(); await Promise.resolve(); await Promise.resolve(); };
afterEach(() => suspendAccountPreferences());
it('renews only genuine foreground interaction and never schedules background renewal', async () => {
  let now = 0; let foreground = true;
  const send = vi.fn(async () => new Response(null, { status: 200 }));
  const tracker = createSessionActivity({ now: () => now, foreground: () => foreground, send });
  try {
    tracker.activity(true); expect(send).not.toHaveBeenCalled();
    activateAccountPreferences('alice');
    tracker.activity(false); foreground = false; tracker.activity(true); expect(send).not.toHaveBeenCalled();
    foreground = true; tracker.activity(true); await settle(); expect(send).toHaveBeenCalledTimes(1);
    now = 29_999; tracker.activity(true); expect(send).toHaveBeenCalledTimes(1);
    now = 30_000; tracker.activity(true); await settle(); expect(send).toHaveBeenCalledTimes(2);
    now = 900_000; await settle(); expect(send).toHaveBeenCalledTimes(2);
    foreground = false; tracker.activity(true); expect(send).toHaveBeenCalledTimes(2);
  } finally { tracker.dispose(); }
});
it('aborts the previous account and ignores its delayed unauthorized response', async () => {
  let now = 0;
  const requests: { signal: AbortSignal; resolve: (response: Response) => void }[] = [];
  const tracker = createSessionActivity({ now: () => now, foreground: () => true, send: signal => new Promise(resolve => requests.push({ signal, resolve })) });
  try {
    activateAccountPreferences('alice'); tracker.activity(true);
    suspendAccountPreferences(); expect(requests[0]!.signal.aborted).toBe(true);
    activateAccountPreferences('bob'); tracker.activity(true); expect(requests).toHaveLength(2);
    requests[0]!.resolve(new Response(null, { status: 401 })); requests[1]!.resolve(new Response(null, { status: 200 })); await settle();
    now = 30_000; tracker.activity(true); expect(requests).toHaveLength(3);
    requests[2]!.resolve(new Response(null, { status: 401 })); await settle();
    now = 60_000; tracker.activity(true); expect(requests).toHaveLength(3);
    suspendAccountPreferences(); activateAccountPreferences('bob'); tracker.activity(true); expect(requests).toHaveLength(4);
    requests[3]!.resolve(new Response(null, { status: 200 })); await settle();
  } finally { tracker.dispose(); }
});

it('never renews a mailbox from the administrator console even if an account is bound', async () => {
  const focus = vi.spyOn(document, 'hasFocus').mockReturnValue(true);
  const send = vi.fn(async () => new Response(null, { status: 200 }));
  const tracker = createSessionActivity({ send });
  try {
    activateAccountPreferences('alice'); window.history.replaceState({}, '', '/admin');
    tracker.activity(true); expect(send).not.toHaveBeenCalled();
    window.history.replaceState({}, '', '/'); tracker.activity(true); await settle();
    expect(send).toHaveBeenCalledTimes(1);
  } finally { tracker.dispose(); focus.mockRestore(); window.history.replaceState({}, '', '/'); }
});
