import { afterEach, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { BreachNoticeBanner } from './BreachNoticeBanner.tsx';

afterEach(cleanup);

it('shows unread breach notices and acknowledges one', async () => {
  const ack = vi.fn(async () => {});
  render(() => <BreachNoticeBanner load={async () => [
    { id: 'n1', reference: 'BR-1', subject: 'Notice A', body: 'What happened.', acknowledged: false },
    { id: 'n2', reference: 'BR-2', subject: 'Old', body: 'Seen.', acknowledged: true },
  ]} ack={ack} />);
  expect(await screen.findByTestId('breach-notice-n1')).toHaveTextContent('Notice A');
  expect(screen.queryByTestId('breach-notice-n2')).toBeNull();
  fireEvent.click(screen.getByRole('button'));
  await waitFor(() => expect(ack).toHaveBeenCalledWith('n1'));
});

it('shows nothing when the load fails or there is nothing unread', async () => {
  render(() => <BreachNoticeBanner load={async () => { throw new Error('down'); }} />);
  await Promise.resolve();
  expect(screen.queryByTestId('breach-notice-banner')).toBeNull();
});
