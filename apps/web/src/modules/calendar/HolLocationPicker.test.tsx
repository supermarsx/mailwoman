// The import flow of `CalendarApp` for `.hol` files: the file is decoded by its
// byte order mark, and a pack with more than one location is not sent until the
// user has chosen.
//
// The oracle is what reaches the server: every `blob` of a
// `CalendarEvent/import` call is recorded and compared with text written out
// here. The backend answers with server-shaped bodies (the shapes of
// `contract.test.tsx`: `Calendar/get` from `calendar_row_to_json`, the import
// result from `event_import`); its `count` is canned and is not what is tested.

import { describe, it, expect } from 'vitest';
import { render, fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import type { JmapRequest, JmapResponse } from '../../api/jmap-types.ts';
import { createCalendarController } from './controller.ts';
import { CalendarApp } from './index.tsx';
import { ToastRegion } from '../../components/toast/index.ts';

const SERVER_BODIES: Record<string, unknown> = {
  'Calendar/get': {
    accountId: 'acct1',
    state: '2',
    list: [
      {
        id: 'cal-1', name: 'Calendar', color: '#3366ff', order: 0, isVisible: true, isSubscribed: true,
        role: 'default', shareWith: [], caldavUrl: null, syncToken: null, isReadOnlyOverlay: false, component: 'VEVENT',
      },
    ],
    notFound: [],
  },
  'CalendarEvent/get': { accountId: 'acct1', state: '2', list: [], notFound: [] },
  'CalendarEvent/expand': { accountId: 'acct1', list: [] },
  'Calendar/detectConflicts': { accountId: 'acct1', list: [] },
  'CalendarEvent/import': { accountId: 'acct1', imported: ['ev-1'], count: 1 },
};

/** Mount the app over a backend that records every imported `blob`. */
async function mountApp(): Promise<{ imports: string[]; pick: (bytes: Uint8Array<ArrayBuffer> | string, name: string) => void }> {
  const imports: string[] = [];
  const jmap = (body: JmapRequest): Promise<JmapResponse> => {
    const methodResponses = body.methodCalls.map(([name, args, callId]) => {
      if (name === 'CalendarEvent/import') imports.push(String((args as Record<string, unknown>)['blob']));
      return [name, SERVER_BODIES[name] ?? { type: 'unknownMethod', description: name }, callId];
    });
    return Promise.resolve({ methodResponses } as unknown as JmapResponse);
  };
  const controller = createCalendarController({ jmap, resolveAccount: () => Promise.resolve('acct1') });
  render(() => <><ToastRegion /><CalendarApp controller={controller} /></>);
  const input = (await screen.findByLabelText('Import calendar file')) as HTMLInputElement;
  await waitFor(() => expect(controller.calendars().length).toBe(1));
  const pick = (bytes: Uint8Array<ArrayBuffer> | string, name: string): void => {
    Object.defineProperty(input, 'files', { value: [new File([bytes], name)], configurable: true });
    fireEvent.change(input);
  };
  return { imports, pick };
}

/** `text` as Outlook stores a `.hol`: UTF-16LE, byte order mark first. */
function utf16le(text: string): Uint8Array<ArrayBuffer> {
  const out = [0xff, 0xfe];
  for (let i = 0; i < text.length; i++) out.push(text.charCodeAt(i) & 0xff, text.charCodeAt(i) >> 8);
  return Uint8Array.from(out);
}

const ONE = '[Portugal] 2\r\nAno Novo,2026/1/1\r\nDia da Região,2026/7/1\r\n';
const THREE =
  '[Argentina] 2\r\nSan Martín Day,2026/8/17\r\nEid al Adha,1447/12/10,6\r\n' +
  '[Portugal] 1\r\nDia de Portugal,2026/6/10\r\n' +
  '[Japan] 1\r\nShōwa Day,2026/4/29\r\n';

const picker = (): Promise<HTMLElement> => screen.findByRole('dialog', { name: 'Choose locations to import' });

describe('importing a .hol file', () => {
  it('a UTF-16 pack with one location is sent as the text it holds, with no question', async () => {
    const { imports, pick } = await mountApp();
    pick(utf16le(ONE), 'pt.hol');
    await waitFor(() => expect(imports).toEqual([ONE]));
    expect(screen.queryByRole('dialog', { name: 'Choose locations to import' })).toBeNull();
    await waitFor(() => expect(screen.getByTestId('calendar-feedback')).toHaveTextContent('Imported 1 event.'));
  });

  it('a pack with several locations sends nothing until some are chosen, then only those', async () => {
    const { imports, pick } = await mountApp();
    pick(utf16le(THREE), 'world.hol');

    const dialog = await picker();
    expect(within(dialog).getByText('This file holds holidays for 3 locations.', { exact: false })).toBeInTheDocument();
    const boxes = within(dialog).getAllByRole('checkbox') as HTMLInputElement[];
    expect(boxes).toHaveLength(3);
    expect(boxes.some((b) => b.checked)).toBe(false);
    const go = within(dialog).getByRole('button', { name: 'Import selected' });
    expect(go).toBeDisabled();
    expect(imports).toEqual([]);

    // Ticks survive the filter: Argentina, then Japan found by filtering.
    fireEvent.click(within(dialog).getByLabelText('Argentina'));
    fireEvent.input(within(dialog).getByLabelText('Filter locations'), { target: { value: 'JAP' } });
    expect(within(dialog).getAllByRole('checkbox')).toHaveLength(1);
    fireEvent.click(within(dialog).getByLabelText('Japan'));
    fireEvent.input(within(dialog).getByLabelText('Filter locations'), { target: { value: 'nowhere' } });
    expect(within(dialog).queryAllByRole('checkbox')).toHaveLength(0);
    expect(within(dialog).getByText('No location matches the filter.')).toBeInTheDocument();
    expect(within(dialog).getByRole('status')).toHaveTextContent('2 locations selected.');
    expect(go).toBeEnabled();
    expect(imports).toEqual([]);

    fireEvent.click(go);
    // File order, whole sections, and nothing of Portugal.
    await waitFor(() =>
      expect(imports).toEqual([
        '[Argentina] 2\r\nSan Martín Day,2026/8/17\r\nEid al Adha,1447/12/10,6\r\n[Japan] 1\r\nShōwa Day,2026/4/29\r\n\r\n',
      ]),
    );
    expect(screen.queryByRole('dialog', { name: 'Choose locations to import' })).toBeNull();
  });

  it('cancelling the choice, by the button or by Escape, sends nothing', async () => {
    const { imports, pick } = await mountApp();
    pick(utf16le(THREE), 'world.hol');
    fireEvent.click(within(await picker()).getByRole('button', { name: 'Cancel' }));
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Choose locations to import' })).toBeNull());

    // The same file can be picked again, and a tick alone imports nothing.
    pick(utf16le(THREE), 'world.hol');
    const dialog = await picker();
    fireEvent.click(within(dialog).getByLabelText('Portugal'));
    fireEvent.keyDown(dialog, { key: 'Escape' });
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Choose locations to import' })).toBeNull());
    expect(imports).toEqual([]);
    expect(screen.queryByTestId('calendar-feedback')).toBeNull();
  });

  it('a UTF-8 pack and an iCalendar document are sent as they were', async () => {
    const { imports, pick } = await mountApp();
    pick(ONE, 'pt-utf8.hol');
    await waitFor(() => expect(imports).toEqual([ONE]));
    // Bracketed lines inside an iCalendar document are not locations.
    const ics = 'BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:x\r\nDESCRIPTION:[a] 1\r\n[b] 2\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n';
    pick(ics, 'x.ics');
    await waitFor(() => expect(imports).toEqual([ONE, ics]));
    expect(screen.queryByRole('dialog', { name: 'Choose locations to import' })).toBeNull();
  });
});
