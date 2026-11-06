import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { createSignal, Show } from 'solid-js';
import { Observability } from './Observability.tsx';
import { groupsToCsv, issueMarkdown, splitStack } from './ClientErrors.tsx';
import { mockAdminApi } from './testkit.tsx';
import { createAdminSlice } from '../../state/slices/admin.ts';
import { AdminContext } from './context.ts';
import { holding, renderInShell } from './shell/testkit.tsx';
import { useShell } from './shell/context.ts';
import type { Capability } from './shell/can.ts';
import * as api from '../../api/admin/clientErrors.ts';
import type { ClientErrorDetail, ClientErrorGroup } from '../../api/admin/clientErrors.ts';

vi.mock('../../api/admin/clientErrors.ts', () => ({
  listClientErrors: vi.fn(),
  loadClientError: vi.fn(),
  setClientErrorStatus: vi.fn(async () => undefined),
  deleteClientError: vi.fn(async () => undefined),
}));

const NOW = Math.floor(Date.now() / 1000);
const group = (over: Partial<ClientErrorGroup> = {}): ClientErrorGroup => ({
  id: 'g1', fingerprint: 'abcdef0123456789', kind: 'boundary', name: 'TypeError', message: 'x is undefined', boundary: 'Reader/AttachmentViewer/text',
  routeTemplate: '/mail/INBOX/:id', firstSeen: NOW - 7200, lastSeen: NOW - 60, count: 12, lastBuild: '26.18+abc', status: 'open', resolvedBuild: null,
  reopenedCount: 0, sessions: 4, series: [[NOW - 3600, 5], [NOW - 120, 7]], ...over,
});
const STACK = 'TypeError: x is undefined\n    at TextViewer (src/viewers/TextViewer.tsx:42:7)\n    at run (node_modules/solid-js/dist/solid.js:10:2)';
const detail = (over: Partial<ClientErrorDetail> = {}): ClientErrorDetail => ({
  group: group(),
  events: [{ id: '0123456789abcdef0123456789abcdef', time: NOW - 60, build: '26.18+abc', occurrences: 3, stackRaw: 'TypeError: x is undefined\n    at f (/assets/Reader-AbC.js:1:2)', stackSymbolicated: STACK,
    uaFamily: 'Chrome', osFamily: 'Windows', viewport: 'wide', locale: 'en', accountHash: null, extra: { route: '/mail/INBOX/:id', uaVersion: '126' } }],
  series: [[NOW - 3600, 5], [NOW - 120, 7]],
  breakdown: { browser: [['Chrome', 10], ['Firefox', 2]], os: [['Windows', 12]], build: [['26.18+abc', 12]], viewport: [['wide', 12]], route: [['/mail/INBOX/:id', 12]] },
  ...over,
});

function open(rest: string[], capabilities: Capability[] = ['observability.read', 'settings.write']) {
  const admin = createAdminSlice(mockAdminApi({}));
  const [route, setRoute] = createSignal({ section: 'observability', rest });
  const Ready = () => {
    const shell = useShell();
    return <Show when={shell.can.grants().kind === 'listed'}><AdminContext.Provider value={admin}><Observability /></AdminContext.Provider></Show>;
  };
  return renderInShell(() => <Ready />, {
    loadMe: holding(capabilities),
    route,
    navigate: (path) => setRoute({ section: 'observability', rest: path.split('#/observability')[1]?.split('/').filter(Boolean) ?? [] }),
  });
}

beforeEach(() => {
  vi.mocked(api.listClientErrors).mockReset().mockResolvedValue({ groups: [group(), group({ id: 'g2', name: 'RangeError', message: 'bad', count: 2, sessions: 1, boundary: '', series: [] })], since: NOW - 7 * 86400, policy: { enabled: true, sourceMaps: true, forward: false, includeAccountHash: false } });
  vi.mocked(api.loadClientError).mockReset().mockResolvedValue(detail());
  vi.mocked(api.setClientErrorStatus).mockClear();
  vi.mocked(api.deleteClientError).mockClear();
});

describe('Admin > Observability > Client errors', () => {
  it('lists groups with counts, sessions, build, boundary and status, filtered by status', async () => {
    open(['client-errors']);
    const rows = await screen.findAllByTestId('ce-row');
    expect(rows).toHaveLength(2);
    expect(within(rows[0]!).getByText('TypeError')).toBeTruthy();
    expect(within(rows[0]!).getByText('12')).toBeTruthy();
    expect(within(rows[0]!).getByText('Reader/AttachmentViewer/text')).toBeTruthy();
    expect(within(rows[0]!).getByText('Open')).toBeTruthy();
    expect(vi.mocked(api.listClientErrors).mock.calls[0]![0]).toMatchObject({ status: 'open', rangeHours: 168 });
    fireEvent.change(screen.getByLabelText('Status'), { target: { value: 'resolved' } });
    await waitFor(() => expect(vi.mocked(api.listClientErrors).mock.calls.at(-1)![0]).toMatchObject({ status: 'resolved' }));
  });

  it('says so when reporting is off and when nothing was reported', async () => {
    vi.mocked(api.listClientErrors).mockResolvedValue({ groups: [], since: NOW, policy: { enabled: false, sourceMaps: true, forward: false, includeAccountHash: false } });
    open(['client-errors']);
    expect(await screen.findByTestId('ce-disabled')).toBeTruthy();
    expect(await screen.findByTestId('ce-empty')).toBeTruthy();
  });

  it('opens a group by deep link: app frames highlighted, vendor frames folded, raw stack on request', async () => {
    open(['client-errors', 'g1']);
    const view = await screen.findByTestId('client-error-detail');
    const frames = await within(view).findByTestId('ce-frames');
    const shown = within(frames).getAllByRole('listitem');
    expect(shown).toHaveLength(1);
    expect(shown[0]!.getAttribute('data-frame')).toBe('app');
    expect(shown[0]!.textContent).toContain('src/viewers/TextViewer.tsx:42:7');
    fireEvent.click(within(view).getByLabelText(/Show 1 library/));
    expect(within(frames).getAllByRole('listitem')).toHaveLength(2);
    fireEvent.click(within(view).getByLabelText('Show the raw (minified) stack'));
    expect((await within(view).findByTestId('ce-raw')).textContent).toContain('/assets/Reader-AbC.js:1:2');
    expect(within(view).getByTestId('ce-chart-browser')).toBeTruthy();
    expect(within(view).getByTestId('ce-chart-route')).toBeTruthy();
    expect(within(view).getByTestId('ce-events')).toBeTruthy();
  });

  it('says when no source map matched', async () => {
    vi.mocked(api.loadClientError).mockResolvedValue(detail({ events: [{ ...detail().events[0]!, stackSymbolicated: null }] }));
    open(['client-errors', 'g1']);
    expect(await screen.findByTestId('ce-not-symbolicated')).toBeTruthy();
    expect((await screen.findByTestId('ce-raw')).textContent).toContain('Reader-AbC.js');
  });

  it('resolves, ignores and deletes a group; a reader without settings.write gets no actions', async () => {
    const { unmount } = open(['client-errors', 'g1']);
    fireEvent.click(await screen.findByRole('button', { name: 'Mark resolved' }));
    await waitFor(() => expect(api.setClientErrorStatus).toHaveBeenCalledWith('g1', 'resolved'));
    fireEvent.click(screen.getByRole('button', { name: 'Ignore' }));
    await waitFor(() => expect(api.setClientErrorStatus).toHaveBeenCalledWith('g1', 'ignored'));
    fireEvent.click(screen.getByRole('button', { name: 'Delete group' }));
    await waitFor(() => expect(screen.getAllByRole('button', { name: 'Delete group' }).length).toBe(2));
    fireEvent.click(screen.getAllByRole('button', { name: 'Delete group' }).at(-1)!);
    await waitFor(() => expect(api.deleteClientError).toHaveBeenCalledWith('g1'));
    unmount();
    open(['client-errors', 'g1'], ['observability.read']);
    await screen.findByTestId('client-error-detail');
    expect(screen.queryByRole('button', { name: 'Mark resolved' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Delete group' })).toBeNull();
    expect(screen.getByRole('button', { name: 'Copy as issue' })).toBeTruthy();
  });
});

describe('helpers', () => {
  it('splits chrome and firefox stacks into message lines and frames', () => {
    const s = splitStack('Error: a\nmessage line\n    at f (src/a.ts:1:2)\nrun@src/b.ts:3:4\n    at g (node_modules/x/y.js:1:1)');
    expect(s.head).toEqual(['Error: a', 'message line']);
    expect(s.frames.map((f) => f.app)).toEqual([true, true, false]);
  });

  it('exports CSV with formula characters defused and an issue template', () => {
    const csv = groupsToCsv([group({ message: '=HYPERLINK("x")' })]);
    expect(csv.split('\r\n')).toHaveLength(2);
    expect(csv).toContain(`"'=HYPERLINK(""x"")"`);
    const md = issueMarkdown(detail(), STACK);
    expect(md).toContain('### TypeError: x is undefined');
    expect(md).toContain('```\nTypeError: x is undefined');
  });
});
