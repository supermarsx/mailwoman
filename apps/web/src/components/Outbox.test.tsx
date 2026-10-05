import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, within } from '@solidjs/testing-library';
import { Outbox } from './Outbox.tsx';
import { AppContext } from '../state/context.ts';
import { stripIsolates } from '../i18n/index.ts';
import type { AppState } from '../state/store.ts';
import type { OutboxMessage, OutboxSubmission } from '../state/slices/outbox.ts';

// Rendering only: the rows come from a stub of the slice, so this says what the
// Outbox shows for a row and which slice call a button makes. What a release
// does on the server is `crates/mw-server/tests/t28_mcp_hold.rs`.

function sub(id: string, over: Partial<OutboxSubmission> = {}): OutboxSubmission {
  return { id, emailId: `e-${id}`, identityId: null, sendAt: null, undoStatus: 'pending', mailwomanHoldSeconds: 0, ...over };
}

function renderOutbox(subs: OutboxSubmission[], messages: Record<string, OutboxMessage> = {}) {
  const calls = {
    refreshOutbox: vi.fn(async () => undefined),
    sendOutboxNow: vi.fn(async (_id: string) => undefined),
    cancelOutbox: vi.fn(async (_id: string) => undefined),
  };
  const app = { outbox: () => subs, outboxMessages: () => messages, ...calls } as unknown as AppState;
  render(() => (
    <AppContext.Provider value={app}>
      <Outbox />
    </AppContext.Provider>
  ));
  return calls;
}

function row(state: string): HTMLElement {
  const el = document.querySelector<HTMLElement>(`.outbox__row[data-state="${state}"]`);
  if (el === null) throw new Error(`no ${state} row`);
  return el;
}

const text = (el: HTMLElement): string => stripIsolates(el.textContent ?? '');

describe('Outbox', () => {
  it('shows a held row with who created it and what it would send, and offers Release and Discard', () => {
    const calls = renderOutbox(
      [sub('h1', { mailwomanHold: 'manual', mailwomanOrigin: { kind: 'apiKey', name: 'abcd1234' } })],
      { 'e-h1': { subject: 'Quarterly numbers', to: [{ name: null, email: 'boss@example.net' }] } },
    );
    const held = row('held');
    expect(text(held)).toContain('Held — created by API key abcd1234. Not sent until released.');
    expect(text(held)).toContain('Quarterly numbers');
    expect(text(held)).toContain('To boss@example.net');
    // A held row is released or discarded; it has no "Send now" / "Cancel".
    expect(within(held).queryByRole('button', { name: 'Send now' })).toBeNull();
    expect(within(held).queryByRole('button', { name: 'Cancel' })).toBeNull();

    fireEvent.click(within(held).getByRole('button', { name: 'Release' }));
    expect(calls.sendOutboxNow).toHaveBeenCalledWith('h1');
    expect(calls.cancelOutbox).not.toHaveBeenCalled();
    fireEvent.click(within(held).getByRole('button', { name: 'Discard' }));
    expect(calls.cancelOutbox).toHaveBeenCalledWith('h1');
  });

  it('names a connected app, and says only "held" when the server gave no origin', () => {
    renderOutbox([
      sub('h1', { mailwomanHold: 'manual', mailwomanOrigin: { kind: 'oauthClient', name: 'agent-app' } }),
      sub('h2', { mailwomanHold: 'manual', mailwomanOrigin: null }),
    ]);
    const rows = [...document.querySelectorAll<HTMLElement>('.outbox__row[data-state="held"]')].map(text);
    expect(rows[0]).toContain('Held — created by the connected app agent-app. Not sent until released.');
    expect(rows[1]).toContain('Held. Not sent until released.');
    expect(rows[1]).not.toContain('created by');
  });

  it('keeps Send now and Cancel on scheduled and undo-window rows', () => {
    const calls = renderOutbox([
      sub('s1', { sendAt: new Date(Date.now() + 3_600_000).toISOString() }),
      sub('w1', { mailwomanHoldSeconds: 10 }),
    ]);
    for (const state of ['scheduled', 'holding']) {
      const r = row(state);
      expect(within(r).getByRole('button', { name: 'Send now' })).toBeTruthy();
      expect(within(r).getByRole('button', { name: 'Cancel' })).toBeTruthy();
      expect(within(r).queryByRole('button', { name: 'Release' })).toBeNull();
      expect(text(r)).not.toContain('Not sent until released');
    }
    fireEvent.click(within(row('scheduled')).getByRole('button', { name: 'Send now' }));
    expect(calls.sendOutboxNow).toHaveBeenCalledWith('s1');
  });

  it('has no buttons on sent, canceled or not-sent rows, and says which is which', () => {
    renderOutbox([
      sub('f1', { undoStatus: 'final', mailwomanOrigin: { kind: 'apiKey', name: 'abcd1234' } }),
      sub('c1', { undoStatus: 'canceled' }),
      sub('x1', {
        undoStatus: 'canceled',
        mailwomanFailed: true,
        mailwomanAttempts: 8,
        mailwomanLastError: 'transport error: connection refused',
      }),
    ]);
    expect(screen.queryAllByRole('button', { name: /Release|Discard|Send now|Cancel/ })).toHaveLength(0);
    // A released-and-sent row still says where it came from, without "held".
    expect(text(row('sent'))).toContain('Created by API key abcd1234');
    expect(text(row('sent'))).not.toContain('Held');
    expect(text(row('canceled'))).toContain('Canceled');
    const failed = text(row('failed'));
    expect(failed).toContain('Not sent');
    expect(failed).toContain('Tried 8 times. Last error: transport error: connection refused');
  });

  it('shows a retrying row its last error', () => {
    renderOutbox([sub('r1', { mailwomanAttempts: 1, mailwomanLastError: 'transport error: connection refused' })]);
    expect(text(row('holding'))).toContain('Error: transport error: connection refused');
  });
});
