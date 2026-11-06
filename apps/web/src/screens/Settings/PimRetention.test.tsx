import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { PimRetention, type PimApi } from './PimRetention.tsx';
import type { PimOverview } from '../../api/retention.ts';

const overview = (over: Partial<PimOverview> = {}): PimOverview => ({
  policy: { calendar: { allowed: true, minDays: 30, maxDays: 365 }, notes: { allowed: true, minDays: 30, maxDays: 3650 }, undoDays: 7 },
  calendar: { enabled: false, days: 0, calendars: [] },
  notes: { enabled: false, days: 0 },
  calendars: [{ id: 'c1', name: 'Work' }, { id: 'c2', name: 'Home' }],
  held: [],
  ...over,
});

const summary = { skipped: null, eventsMatched: 3, eventsDeleted: 0, notesMatched: 2, notesRemoved: 0, disabledSkipped: 0, errors: 0 };

const api = (data: PimOverview, over: Partial<PimApi> = {}): PimApi => ({
  load: vi.fn(async () => data),
  save: vi.fn(async () => data),
  preview: vi.fn(async () => summary),
  run: vi.fn(async () => ({ ...summary, eventsDeleted: 3, notesRemoved: 2 })),
  restore: vi.fn(async () => ({ restored: true })),
  ...over,
});

afterEach(() => cleanup());

describe('Settings > Retention: calendar and notes', () => {
  it('shows only a lock note while the administrator has them off', async () => {
    const off = overview({ policy: { calendar: { allowed: false, minDays: 30, maxDays: 365 }, notes: { allowed: false, minDays: 30, maxDays: 3650 }, undoDays: 7 } });
    render(() => <PimRetention api={api(off)} />);
    expect(await screen.findByTestId('pim-calendar-off')).toHaveTextContent('Turned off by your administrator');
    expect(screen.getByTestId('pim-notes-off')).toBeInTheDocument();
    expect(document.querySelector('#pim-cal-enabled')).toBeNull();
    expect(document.querySelector('#pim-notes-enabled')).toBeNull();
    expect(screen.queryByTestId('pim-run')).toBeNull();
  });

  it('saves a calendar choice with the limits enforced locally', async () => {
    const a = api(overview());
    render(() => <PimRetention api={a} />);
    fireEvent.click(await screen.findByLabelText('Delete ended events'));
    fireEvent.click(screen.getByLabelText('Work'));
    fireEvent.input(screen.getByLabelText('Days after an event ended'), { target: { value: '5' } });
    expect(screen.getByTestId('pim-calendar-save')).toBeDisabled();
    expect(screen.getByRole('alert')).toHaveTextContent('between 30 and 365');
    fireEvent.input(screen.getByLabelText('Days after an event ended'), { target: { value: '120' } });
    fireEvent.click(screen.getByTestId('pim-calendar-save'));
    await waitFor(() => expect(a.save).toHaveBeenCalledWith({ calendar: { enabled: true, days: 120, calendars: ['c1'] } }));
  });

  it('saves a notes choice and previews counts without running', async () => {
    const a = api(overview());
    render(() => <PimRetention api={a} />);
    fireEvent.click(await screen.findByLabelText('Remove untouched notes'));
    fireEvent.input(screen.getByLabelText('Days since a note was last changed'), { target: { value: '200' } });
    fireEvent.click(screen.getByTestId('pim-notes-save'));
    await waitFor(() => expect(a.save).toHaveBeenCalledWith({ notes: { enabled: true, days: 200 } }));
    await waitFor(() => expect(screen.getByTestId('pim-preview')).toBeEnabled());
    fireEvent.click(screen.getByTestId('pim-preview'));
    expect(await screen.findByTestId('pim-result')).toHaveTextContent('3 event(s) and 2 note(s)');
    expect(a.run).not.toHaveBeenCalled();
  });

  it('lists removed notes and restores one', async () => {
    const a = api(overview({ held: [{ noteId: 'n1', title: 'Old plan', heldAt: 1_800_000_000, dueAt: 1_800_600_000 }] }));
    render(() => <PimRetention api={a} />);
    const held = await screen.findByTestId('pim-held');
    expect(held).toHaveTextContent('Old plan');
    fireEvent.click(screen.getByTestId('pim-restore-n1'));
    await waitFor(() => expect(a.restore).toHaveBeenCalledWith('n1'));
  });
});
