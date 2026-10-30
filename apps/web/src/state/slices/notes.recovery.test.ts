// t91: a note create that fails in transit shows ONE toast with Retry, and
// Retry sends the same request with the same idempotency key (so a create
// whose answer was lost is not made twice). A refused create is not retried.
import { describe, expect, it, vi } from 'vitest';
import { createRoot } from 'solid-js';
import { NetworkError, type Client, type JmapCallOptions } from '../../api/client.ts';
import type { JmapRequest, JmapResponse, JmapSession } from '../../api/jmap-types.ts';
import { CAP_NOTES } from '../../api/pim-types.ts';
import { RecoveryError } from '../../api/recovery.ts';
import type { ToastExtra } from '../store.ts';
import { createNotesSlice } from './notes.ts';

function harness(answers: Array<'fail' | 'ok' | 'refuse'>) {
  const calls: Array<{ body: string; key: string | false | undefined }> = [];
  const jmap = vi.fn(async (body: JmapRequest, opts?: JmapCallOptions): Promise<JmapResponse> => {
    calls.push({ body: JSON.stringify(body), key: opts?.idempotencyKey });
    const answer = answers.shift() ?? 'ok';
    if (answer === 'fail') throw new NetworkError('lost', new RecoveryError('network').reason);
    const set = answer === 'ok' ? { created: { new: { id: 'n1' } } } : { notCreated: { new: { type: 'invalidProperties' } } };
    return { methodResponses: [['Note/set', { accountId: 'acct1', ...set }, 's']], sessionState: 's' } as JmapResponse;
  });
  const session = async (): Promise<JmapSession> => ({ primaryAccounts: { [CAP_NOTES]: 'acct1' }, accounts: { acct1: {} } }) as unknown as JmapSession;
  const toasts: Array<{ kind: string; message: string; extra: ToastExtra | undefined }> = [];
  const slice = createRoot(() => createNotesSlice({
    client: { jmap, session } as unknown as Client,
    showToast: (kind, message, _ttl, extra) => { toasts.push({ kind, message, extra }); },
  }));
  return { slice, calls, toasts };
}

describe('note create recovery (t91)', () => {
  it('shows one toast whose Retry sends the same request with the same key', async () => {
    const { slice, calls, toasts } = harness(['fail', 'ok']);
    await expect(slice.createNote({ title: 'Plan' })).rejects.toBeInstanceOf(NetworkError);
    expect(toasts).toHaveLength(1);
    expect(toasts[0]?.kind).toBe('error');
    expect(toasts[0]?.extra?.id).toMatch(/^recover:notes:create:/);
    expect(toasts[0]?.message).toContain('The server could not be reached.');
    expect(calls[0]?.key).toEqual(expect.any(String));
    await toasts[0]!.extra!.action!.run();
    await vi.waitFor(() => expect(slice.notes()).toHaveLength(1));
    expect(calls).toHaveLength(2);
    expect(calls[1]).toEqual(calls[0]);
    expect(slice.notes()[0]?.id).toBe('n1');
  });

  it('a refused create gets its own message and no Retry', async () => {
    const { slice, toasts } = harness(['refuse']);
    await expect(slice.createNote({ title: 'Plan' })).rejects.toThrow();
    expect(toasts).toHaveLength(1);
    expect(toasts[0]?.extra).toBeUndefined();
  });
});
