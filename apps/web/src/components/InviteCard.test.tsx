import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import type { Email, JmapRequest, JmapResponse } from '../api/jmap-types.ts';
import { InviteCard, hasCalendarPart, type Invitation } from './InviteCard.tsx';

const invitation: Invitation = {
  accountId: 'a', emailId: 'e', method: 'REQUEST', sender: 'organizer@example.com',
  event: { title: 'Planning' }, outcome: 'new', applied: false,
};
function response(value: Record<string, unknown>): JmapResponse {
  return { sessionState: 's', methodResponses: [['CalendarEvent/itip', value, 'invite']] };
}

describe('received calendar invitations', () => {
  it('previews without changing the calendar and applies only after a click', async () => {
    const jmap = vi.fn(async (request: JmapRequest) => response({
      ...invitation, applied: request.methodCalls[0]?.[1]['apply'] === true,
    }));
    render(() => <InviteCard accountId="a" emailId="e" client={{ jmap }} />);
    expect(await screen.findByText('Planning')).toBeInTheDocument();
    expect(jmap).toHaveBeenCalledTimes(1);
    expect(jmap.mock.calls[0]?.[0].methodCalls[0]?.[1]).toEqual({ accountId: 'a', emailId: 'e', apply: false });
    fireEvent.click(screen.getByRole('button', { name: 'Add to calendar' }));
    await waitFor(() => expect(jmap).toHaveBeenCalledTimes(2));
    expect(jmap.mock.calls[1]?.[0].methodCalls[0]?.[1]).toEqual({ accountId: 'a', emailId: 'e', apply: true });
    await waitFor(() => expect(screen.queryByRole('button', { name: 'Add to calendar' })).not.toBeInTheDocument());
    expect(screen.getByText('Your calendar was updated.')).toBeInTheDocument();
  });

  it.each(['notOrganizer', 'notAttendee', 'stale', 'current', 'unknownEvent', 'unsupported'] as const)(
    'offers no mutation for %s', async (outcome) => {
      const jmap = vi.fn(async () => response({ ...invitation, outcome }));
      render(() => <InviteCard accountId="a" emailId="e" client={{ jmap }} />);
      await screen.findByText('Planning');
      expect(screen.queryByRole('button')).not.toBeInTheDocument();
      expect(jmap).toHaveBeenCalledTimes(1);
    },
  );

  it('shows the new refusal when the server finds a stale invitation at apply time', async () => {
    const jmap = vi.fn(async (request: JmapRequest) => response({
      ...invitation, outcome: request.methodCalls[0]?.[1]['apply'] ? 'stale' : 'new',
    }));
    render(() => <InviteCard accountId="a" emailId="e" client={{ jmap }} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Add to calendar' }));
    await screen.findByText('This message is older than the event in your calendar.');
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });

  it('does not turn a plain calendar file into an invitation', async () => {
    const jmap = vi.fn(async () => response({ type: 'invalidArguments', description: 'not iTIP' }));
    render(() => <InviteCard accountId="a" emailId="e" client={{ jmap }} />);
    await waitFor(() => expect(jmap).toHaveBeenCalledTimes(1));
    await Promise.resolve();
    expect(screen.queryByRole('region')).not.toBeInTheDocument();
  });

  it('recognizes calendar MIME parts, not a filename alone', () => {
    expect(hasCalendarPart({ attachments: [{ type: 'text/calendar; method=REQUEST' }] } as unknown as Email)).toBe(true);
    expect(hasCalendarPart({ textBody: [{ type: 'text/calendar' }] } as Email)).toBe(true);
    expect(hasCalendarPart({ attachments: [{ type: 'application/octet-stream', name: 'fake.ics' }] } as unknown as Email)).toBe(false);
  });
});
