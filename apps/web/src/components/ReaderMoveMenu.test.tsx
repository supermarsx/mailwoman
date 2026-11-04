import { describe, expect, it, vi } from 'vitest';
import { moveActions } from './ReaderMoveMenu.tsx';
import type { Email } from '../api/jmap-types.ts';

describe('reader Move to folder', () => {
  const app = {
    mailboxes: () => [
      { id: 'inbox', name: 'Inbox', role: 'inbox' },
      { id: 'work', name: 'Work' },
      { id: 'saved', name: 'Saved search', mailwomanSearchQuery: 'x' },
    ],
    moveMessage: vi.fn(),
  } as never;

  it('offers every real folder except the ones the message is already in and saved searches', () => {
    const actions = moveActions({ id: 'm1', mailboxIds: { inbox: true } } as unknown as Email, app);
    expect(actions.map((a) => a.label)).toEqual(['Work']);
  });

  it('moves the open message to the chosen folder', () => {
    const [work] = moveActions({ id: 'm1', mailboxIds: { inbox: true } } as unknown as Email, app);
    work!.run?.();
    expect((app as { moveMessage: ReturnType<typeof vi.fn> }).moveMessage).toHaveBeenCalledWith('m1', 'work');
  });
});
