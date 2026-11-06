import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, within, waitFor } from '@solidjs/testing-library';
import { Outbox } from './Outbox.tsx';
import { OutboxChip } from './OutboxChip.tsx';
import { AppContext } from '../state/context.ts';
import { stripIsolates } from '../i18n/index.ts';
import { toLocalInput } from '../state/sendDelay.ts';
import type { AppState } from '../state/store.ts';
import type { OutboxBody, OutboxMessage, OutboxSubmission, RescheduleResult } from '../state/slices/outbox.ts';

// The Outbox reader (status header and its actions), the edit-send-time dialog
// and the queued-count chip. Slice calls are stubs: what the server does with
// a reschedule is crates/mw-engine/tests/t28_submission_hold.rs.

function sub(id: string, over: Partial<OutboxSubmission> = {}): OutboxSubmission {
  return { id, emailId: `e-${id}`, identityId: null, sendAt: null, undoStatus: 'pending', mailwomanHoldSeconds: 0, ...over };
}
const BODY: OutboxBody = { subject: 'Quarterly numbers', from: [], to: [{ name: null, email: 'boss@example.net' }], cc: [], text: 'See attached.' };
const inDays = (days: number): Date => new Date(Date.now() + days * 86_400_000);

function setup(subs: OutboxSubmission[], opts: { reschedule?: RescheduleResult; messages?: Record<string, OutboxMessage> } = {}) {
  const calls = {
    refreshOutbox: vi.fn(async () => undefined),
    sendOutboxNow: vi.fn(async (_id: string) => undefined),
    cancelOutbox: vi.fn(async (_id: string) => undefined),
    recallOutbox: vi.fn(async (_id: string) => undefined),
    deleteOutbox: vi.fn(async (_id: string) => true),
    sendOutboxAgain: vi.fn(async (_id: string) => true),
    rescheduleOutbox: vi.fn(async (_id: string, sendAt: string): Promise<RescheduleResult> => opts.reschedule ?? { ok: true, sendAt }),
    outboxBody: vi.fn(async (_emailId: string) => BODY),
    showToast: vi.fn(),
  };
  const app = { outbox: () => subs, outboxLoading: () => false, outboxMessages: () => opts.messages ?? {}, cancelableOutbox: () => subs, ...calls } as unknown as AppState;
  return { app, calls };
}
function renderOutbox(subs: OutboxSubmission[], opts?: Parameters<typeof setup>[1]) {
  const { app, calls } = setup(subs, opts);
  render(() => (
    <AppContext.Provider value={app}>
      <Outbox />
    </AppContext.Provider>
  ));
  return calls;
}
const text = (el: Element): string => stripIsolates(el.textContent ?? '');
const open = (state: string): HTMLElement => {
  const row = document.querySelector<HTMLElement>(`.outbox__row[data-state="${state}"]`)!;
  fireEvent.click(row);
  return row;
};

describe('Outbox reader', () => {
  it('opens a scheduled row with its status, due time, body and actions', async () => {
    const calls = renderOutbox([sub('s1', { sendAt: inDays(2).toISOString(), mailwomanDueAt: inDays(2).toISOString() })]);
    open('scheduled');
    const reader = await screen.findByTestId('outbox-reader');
    expect(text(within(reader).getByTestId('outbox-status-label'))).toBe('Scheduled');
    expect(text(within(reader).getByTestId('outbox-status-when'))).toMatch(/^Sends /);
    await waitFor(() => expect(text(reader)).toContain('See attached.'));
    expect(text(reader)).toContain('Quarterly numbers');
    for (const action of ['send-now', 'edit-time', 'recall', 'edit-message', 'delete']) {
      expect(reader.querySelector(`[data-action="${action}"]`), action).not.toBeNull();
    }
    fireEvent.click(reader.querySelector('[data-action="send-now"]')!);
    expect(calls.sendOutboxNow).toHaveBeenCalledWith('s1');
    fireEvent.click(reader.querySelector('[data-action="recall"]')!);
    expect(calls.cancelOutbox).toHaveBeenCalledWith('s1');
    fireEvent.click(reader.querySelector('[data-action="edit-message"]')!);
    expect(calls.recallOutbox).toHaveBeenCalledWith('s1');
  });

  it('counts down an undo window and offers no send-time edit for it', async () => {
    renderOutbox([sub('w1', { mailwomanHoldSeconds: 30, mailwomanDueAt: new Date(Date.now() + 25_000).toISOString() })]);
    open('holding');
    const reader = await screen.findByTestId('outbox-reader');
    expect(text(within(reader).getByTestId('outbox-status-label'))).toBe('Queued');
    expect(text(within(reader).getByTestId('outbox-status-when'))).toMatch(/\(in 0:\d\d\)/);
    expect(reader.querySelector('[data-action="edit-time"]')).toBeNull();
  });

  it('shows retry count and last error for a message the server could not hand over', async () => {
    renderOutbox([sub('r1', { mailwomanDueAt: new Date(Date.now() - 5000).toISOString(), mailwomanAttempts: 3, mailwomanLastError: '421 try later', mailwomanNextAttemptAt: inDays(0.01).toISOString() })]);
    open('holding');
    const reader = await screen.findByTestId('outbox-reader');
    expect(text(within(reader).getByTestId('outbox-status-label'))).toBe('Retrying');
    expect(text(within(reader).getByTestId('outbox-status-attempts'))).toBe('3 attempts so far');
    expect(text(within(reader).getByTestId('outbox-status-error'))).toContain('421 try later');
    expect(text(within(reader).getByTestId('outbox-status-when'))).toMatch(/^Next attempt /);
  });

  it('a failed row says why and offers Send again and Delete (delete asks twice)', async () => {
    const calls = renderOutbox([sub('f1', { undoStatus: 'canceled', mailwomanFailed: true, mailwomanAttempts: 5, mailwomanLastError: '550 mailbox unavailable' })]);
    open('failed');
    const reader = await screen.findByTestId('outbox-reader');
    expect(text(within(reader).getByTestId('outbox-status-label'))).toBe('Not sent');
    expect(text(reader)).toContain('550 mailbox unavailable');
    fireEvent.click(reader.querySelector('[data-action="send-again"]')!);
    expect(calls.sendOutboxAgain).toHaveBeenCalledWith('f1');
    const del = reader.querySelector('[data-action="delete"]')!;
    fireEvent.click(del);
    expect(calls.deleteOutbox).not.toHaveBeenCalled();
    fireEvent.click(del);
    expect(calls.deleteOutbox).toHaveBeenCalledWith('f1');
  });

  it('edit send time: a quick choice saves its ISO time', async () => {
    const calls = renderOutbox([sub('s1', { sendAt: inDays(2).toISOString(), mailwomanDueAt: inDays(2).toISOString() })]);
    open('scheduled');
    const reader = await screen.findByTestId('outbox-reader');
    fireEvent.click(reader.querySelector('[data-action="edit-time"]')!);
    const dialog = await screen.findByTestId('outbox-edit-time');
    fireEvent.click(dialog.querySelector('[data-preset="tomorrow"]')!);
    await waitFor(() => expect(calls.rescheduleOutbox).toHaveBeenCalledTimes(1));
    const [id, iso] = calls.rescheduleOutbox.mock.calls[0]!;
    expect(id).toBe('s1');
    expect(new Date(iso).toISOString()).toBe(iso);
    await waitFor(() => expect(screen.queryByTestId('outbox-edit-time')).toBeNull());
  });

  it('edit send time: a past time and a time beyond the administrator maximum are refused before any request', async () => {
    const calls = renderOutbox([sub('s1', { sendAt: inDays(2).toISOString(), mailwomanDueAt: inDays(2).toISOString() })]);
    open('scheduled');
    const reader = await screen.findByTestId('outbox-reader');
    fireEvent.click(reader.querySelector('[data-action="edit-time"]')!);
    const dialog = await screen.findByTestId('outbox-edit-time');
    const input = dialog.querySelector<HTMLInputElement>('input[type="datetime-local"]')!;
    const save = within(dialog).getByTestId('outbox-edit-time-save');

    fireEvent.input(input, { target: { value: toLocalInput(inDays(-1)) } });
    fireEvent.click(save);
    expect(await within(dialog).findByRole('alert')).toHaveTextContent('Choose a time in the future.');

    fireEvent.input(input, { target: { value: toLocalInput(inDays(400)) } });
    fireEvent.click(save);
    await waitFor(() => expect(within(dialog).getByRole('alert')).toHaveTextContent(/too far ahead/));
    expect(calls.rescheduleOutbox).not.toHaveBeenCalled();
  });

  it('edit send time: a conflict from the server (already sent / moved) is shown and the dialog stays open', async () => {
    const calls = renderOutbox([sub('s1', { sendAt: inDays(2).toISOString(), mailwomanDueAt: inDays(2).toISOString() })], { reschedule: { ok: false, kind: 'conflict', reason: 'stateMismatch' } });
    open('scheduled');
    const reader = await screen.findByTestId('outbox-reader');
    fireEvent.click(reader.querySelector('[data-action="edit-time"]')!);
    const dialog = await screen.findByTestId('outbox-edit-time');
    fireEvent.input(dialog.querySelector('input[type="datetime-local"]')!, { target: { value: toLocalInput(inDays(3)) } });
    fireEvent.click(within(dialog).getByTestId('outbox-edit-time-save'));
    expect(await within(dialog).findByRole('alert')).toHaveTextContent(/already sent/);
    expect(calls.rescheduleOutbox).toHaveBeenCalledTimes(1);
    expect(screen.getByTestId('outbox-edit-time')).toBeInTheDocument();
  });

  it('the row context menu offers the actions that apply, including Edit send time', async () => {
    renderOutbox([sub('s1', { sendAt: inDays(2).toISOString(), mailwomanDueAt: inDays(2).toISOString() })]);
    const row = document.querySelector<HTMLElement>('.outbox__row[data-state="scheduled"]')!;
    fireEvent.contextMenu(row);
    const labels = (await screen.findAllByRole('menuitem')).map(text);
    expect(labels).toEqual(expect.arrayContaining(['Send now', 'Edit send time', 'Recall to drafts', 'Recall and edit', 'Delete']));
    fireEvent.click(screen.getByRole('menuitem', { name: 'Edit send time' }));
    expect(await screen.findByTestId('outbox-edit-time')).toBeInTheDocument();
  });
});

describe('Outbox chip', () => {
  function renderChip(subs: OutboxSubmission[], messages: Record<string, OutboxMessage> = {}) {
    const { app, calls } = setup(subs, { messages });
    const onOpenOutbox = vi.fn();
    render(() => (
      <AppContext.Provider value={app}>
        <OutboxChip onOpenOutbox={onOpenOutbox} />
      </AppContext.Provider>
    ));
    return { calls, onOpenOutbox };
  }

  it('is not drawn when nothing waits', () => {
    renderChip([sub('done', { undoStatus: 'final' }), sub('x', { undoStatus: 'canceled' })]);
    expect(screen.queryByTestId('outbox-chip')).toBeNull();
  });

  it('shows the live count and lists queued messages soonest first with Send now and Recall', async () => {
    const { calls, onOpenOutbox } = renderChip(
      [
        sub('later', { sendAt: inDays(3).toISOString(), mailwomanDueAt: inDays(3).toISOString() }),
        sub('soon', { sendAt: inDays(1).toISOString(), mailwomanDueAt: inDays(1).toISOString() }),
        sub('sent', { undoStatus: 'final' }),
      ],
      { 'e-soon': { subject: 'Soon one', to: [] }, 'e-later': { subject: 'Later one', to: [] } },
    );
    expect(text(screen.getByTestId('outbox-chip-count'))).toBe('2');
    fireEvent.click(screen.getByRole('button', { name: /2 messages waiting to send/ }));
    const items = document.querySelectorAll<HTMLElement>('.outbox-chip__item');
    expect([...items].map((item) => item.dataset['subId'])).toEqual(['soon', 'later']);
    expect(text(screen.getByTestId('outbox-chip-next'))).toMatch(/^Next send:/);
    fireEvent.click(items[0]!.querySelector('[data-action="send-now"]')!);
    expect(calls.sendOutboxNow).toHaveBeenCalledWith('soon');
    fireEvent.click(items[1]!.querySelector('[data-action="recall"]')!);
    expect(calls.cancelOutbox).toHaveBeenCalledWith('later');
    fireEvent.click(document.querySelector('[data-action="open-outbox"]')!);
    expect(onOpenOutbox).toHaveBeenCalled();
  });

  it('a held message offers Release but no Recall', () => {
    renderChip([sub('h', { mailwomanHold: 'manual' })]);
    fireEvent.click(screen.getByRole('button', { name: /1 message waiting to send/ }));
    expect(document.querySelector('.outbox-chip__item [data-action="send-now"]')).toHaveTextContent('Release');
    expect(document.querySelector('.outbox-chip__item [data-action="recall"]')).toBeNull();
  });
});

describe('Outbox reader: where the copy will be saved', () => {
  const folders = [{ id: 'proj', name: 'Project X' }];
  const queued = (copy: NonNullable<OutboxSubmission['mailwomanSentCopy']> | null): OutboxSubmission => sub('s1', { sendAt: inDays(2).toISOString(), mailwomanSentCopy: copy });
  const readerFor = (s: OutboxSubmission, boxes = folders): string | null => {
    const { app } = setup([s]);
    (app as unknown as { mailboxes: () => typeof boxes }).mailboxes = () => boxes;
    render(() => (<AppContext.Provider value={app}><Outbox /></AppContext.Provider>));
    fireEvent.click(document.querySelector<HTMLElement>('.outbox__row')!);
    const line = screen.queryByTestId('outbox-sent-copy');
    return line ? text(line) : null;
  };
  it('names the chosen folder for a scheduled send', () => {
    expect(readerFor(queued({ mailboxId: 'proj' }))).toBe('Will be saved in Project X');
  });
  it('says no copy is kept when the send skips it', () => {
    expect(readerFor(queued({ skip: true }))).toBe('No copy will be saved');
  });
  it('falls back to Sent in the text when the chosen folder is gone', () => {
    expect(readerFor(queued({ mailboxId: 'deleted' }))).toBe('Will be saved in Sent (the chosen folder is not available)');
  });
  it('shows nothing when the default Sent folder applies', () => {
    expect(readerFor(queued(null))).toBeNull();
  });
});
