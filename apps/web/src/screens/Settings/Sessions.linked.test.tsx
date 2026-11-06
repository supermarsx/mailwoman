import { render, screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
import { expect, it, vi } from 'vitest';
import { Sessions } from './Sessions.tsx';
import { SettingsService } from './service.ts';
import { linkedOriginView, sessionSearchValues } from './sessionOrigin.ts';
import type { SessionMeta } from './types.ts';

const text = (el: Element | null): string => (el?.textContent ?? '').replace(/[⁦-⁩]/g, '');
const line = (block: HTMLElement): string => text(block.querySelector('p span'));
const base = { createdAt: 'yesterday', lastSeen: 'now', current: false } as const;
const linked: SessionMeta = {
  ...base, handle: 'link1', username: 'bob@other.test', deviceLabel: 'Linked mailbox access', clientType: 'linked',
  linkKind: 'linked', originEmail: 'alice@example.test', targetEmail: 'bob@other.test', linkCreatedAt: 1_700_000_000, lastActivity: 'just now',
};
function mount(rows: SessionMeta[]) {
  const service = new SettingsService();
  vi.spyOn(service, 'sessions').mockResolvedValue(rows);
  const revoke = vi.spyOn(service, 'revokeSession').mockResolvedValue(undefined);
  render(() => <Sessions service={service} />);
  return { revoke };
}

it('names the origin account on a linked mailbox row and keeps the mailbox address', async () => {
  mount([{ ...base, handle: 'cur', username: 'bob@other.test', current: true, deviceLabel: 'Laptop' }, linked]);
  const block = await screen.findByTestId('linked-origin');
  expect(line(block)).toBe('Linked mailbox access · from alice@example.test');
  expect(text(block.closest('td'))).toContain('bob@other.test');
  fireEvent.click(within(block).getByText('Link details'));
  expect(text(within(block).getByText('Comes from').nextElementSibling)).toBe('alice@example.test');
  expect(text(within(block).getByText('Mailbox opened').nextElementSibling)).toBe('bob@other.test');
  expect(within(block).getByText('Linked mailbox')).toBeInTheDocument();
});

it('revokes a linked row through the existing session revoke with a named button', async () => {
  const { revoke } = mount([linked]);
  fireEvent.click(await screen.findByText('Link details'));
  fireEvent.click(screen.getByRole('button', { name: 'Revoke link access for bob@other.test' }));
  await waitFor(() => expect(revoke).toHaveBeenCalledWith('link1'));
});

it('shows the server-masked origin as given and says only "linked" when hidden', async () => {
  mount([{ ...linked, originEmail: 'a***@example.test' }, { ...base, handle: 'h2', username: 'bob@other.test', linkKind: 'linked' }]);
  const blocks = await screen.findAllByTestId('linked-origin');
  expect(line(blocks[0]!)).toBe('Linked mailbox access · from a***@example.test');
  expect(line(blocks[1]!)).toBe('Linked mailbox access');
});

it('says Unknown (removed account) when the origin was deleted', async () => {
  const { originEmail: _gone, ...removed } = linked;
  mount([{ ...removed, originRemoved: true }]);
  expect(line(await screen.findByTestId('linked-origin'))).toBe('Linked mailbox access · from Unknown (removed account)');
});

it('finds a linked row by origin or mailbox address in the search box', async () => {
  mount([linked, { ...base, handle: 'x', username: 'carol@else.test', deviceLabel: 'Phone' }]);
  await screen.findByText('Phone');
  fireEvent.input(screen.getByRole('searchbox'), { target: { value: 'alice@example' } });
  expect(screen.queryByText('Phone')).not.toBeInTheDocument();
  expect(screen.getByTestId('linked-origin')).toBeInTheDocument();
});

it('derives the view: target only when it differs, nothing for ordinary sessions', () => {
  expect(linkedOriginView({ ...base, handle: 'p', username: 'a@b.c' })).toBeUndefined();
  expect(linkedOriginView(linked)).toMatchObject({ origin: 'alice@example.test', target: 'bob@other.test', removed: false, brandHost: 'example.test' });
  expect(linkedOriginView({ ...linked, targetEmail: 'alice@example.test' })?.target).toBeUndefined();
  expect(sessionSearchValues(linked)).toContain('alice@example.test');
});
