import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import type { Email, JmapRequest, JmapResponse } from '../api/jmap-types.ts';
import { InviteCard, hasCalendarPart, isCalendarPart, type Invitation } from './InviteCard.tsx';

// The invitation is addressed to none of the account's addresses (`me: null`),
// so the card offers "Add to calendar" (no reply) rather than Accept/Decline;
// the answer flow is covered in event-card/EventCard.test.tsx.
const invitation: Invitation = {
  accountId: 'a', emailId: 'e', method: 'REQUEST', sender: 'organizer@example.com', organizer: 'organizer@example.com',
  me: null, answer: null, event: { title: 'Planning' }, items: [], todos: 0, eventId: null, stored: null,
  outcome: 'new', applied: false,
};
function response(request: JmapRequest, value: Record<string, unknown>): JmapResponse {
  const [name, , id] = request.methodCalls[0]!;
  if (name !== 'CalendarEvent/itip') return { sessionState: 's', methodResponses: [[name, { accountId: 'a', list: [] }, id]] };
  return { sessionState: 's', methodResponses: [['CalendarEvent/itip', value, id]] };
}
const itipCalls = (jmap: ReturnType<typeof vi.fn>) =>
  jmap.mock.calls.map((c) => (c[0] as JmapRequest).methodCalls[0]!).filter(([n]) => n === 'CalendarEvent/itip').map(([, a]) => a);

describe('received calendar invitations', () => {
  it('previews without changing the calendar and applies only after a click', async () => {
    let added = false;
    const jmap = vi.fn(async (request: JmapRequest) => {
      const apply = request.methodCalls[0]?.[1]['apply'] === true;
      const before = added;
      if (apply) added = true;
      return response(request, { ...invitation, applied: apply, outcome: apply || before ? 'current' : 'new', eventId: apply || before ? 'ev1' : null });
    });
    render(() => <InviteCard accountId="a" emailId="e" client={{ jmap }} />);
    expect(await screen.findByText('Planning')).toBeInTheDocument();
    expect(itipCalls(jmap)).toEqual([{ accountId: 'a', emailId: 'e', apply: false }]);
    fireEvent.click(screen.getByRole('button', { name: 'Add to calendar' }));
    await waitFor(() => expect(itipCalls(jmap).filter((a) => a['apply'] === true)).toHaveLength(1));
    await waitFor(() => expect(screen.queryByRole('button', { name: 'Add to calendar' })).not.toBeInTheDocument());
    expect(screen.getByText('Your calendar was updated.')).toBeInTheDocument();
  });

  it.each(['notOrganizer', 'notAttendee', 'stale', 'current', 'unknownEvent', 'unsupported'] as const)(
    'offers no mutation for %s', async (outcome) => {
      const jmap = vi.fn(async (request: JmapRequest) => response(request, { ...invitation, outcome }));
      render(() => <InviteCard accountId="a" emailId="e" client={{ jmap }} />);
      await screen.findByText('Planning');
      expect(screen.queryByRole('button')).not.toBeInTheDocument();
      expect(itipCalls(jmap)).toHaveLength(1);
    },
  );

  it('shows the refusal when the server finds the invitation stale at apply time', async () => {
    let stale = false;
    const jmap = vi.fn(async (request: JmapRequest) => {
      if (request.methodCalls[0]?.[1]['apply']) stale = true;
      return response(request, { ...invitation, outcome: stale ? 'stale' : 'new' });
    });
    render(() => <InviteCard accountId="a" emailId="e" client={{ jmap }} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Add to calendar' }));
    await screen.findByText('This invitation is out of date. A newer version is already in your calendar.');
    expect(screen.queryByRole('button', { name: 'Add to calendar' })).not.toBeInTheDocument();
  });

  it('shows nothing for a part the server does not read as a calendar', async () => {
    const jmap = vi.fn(async (request: JmapRequest) => response(request, { type: 'invalidArguments', description: 'not a calendar' }));
    render(() => <InviteCard accountId="a" emailId="e" client={{ jmap }} />);
    await waitFor(() => expect(jmap).toHaveBeenCalledTimes(1));
    await Promise.resolve();
    expect(screen.queryByRole('region')).not.toBeInTheDocument();
  });

  it('recognizes calendar MIME parts and .ics files sent as generic types', () => {
    expect(hasCalendarPart({ attachments: [{ type: 'text/calendar; method=REQUEST' }] } as unknown as Email)).toBe(true);
    expect(hasCalendarPart({ textBody: [{ type: 'text/calendar' }] } as Email)).toBe(true);
    expect(hasCalendarPart({ attachments: [{ type: 'application/ics', name: 'a.ics' }] } as unknown as Email)).toBe(true);
    // The server still reads the part; a file that only claims the name shows no card.
    expect(hasCalendarPart({ attachments: [{ type: 'application/octet-stream', name: 'invite.ics' }] } as unknown as Email)).toBe(true);
    expect(isCalendarPart({ type: 'image/png', name: 'x.ics' })).toBe(false);
    expect(isCalendarPart({ type: 'application/pdf', name: 'agenda.pdf' })).toBe(false);
  });
});
