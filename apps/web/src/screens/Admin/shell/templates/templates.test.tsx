// ListDetailPage, WizardPage, DashboardPage and StreamPage (UX3, UX11, UX12, UX13).
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { ListDetailPage } from './ListDetailPage.tsx';
import { WizardPage } from './WizardPage.tsx';
import { DashboardPage } from './DashboardPage.tsx';
import { StreamPage, type StreamHandlers } from './StreamPage.tsx';
import { AdminPage, PageState, errorCode } from './PageFrame.tsx';
import type { AdminRoute } from '../router.ts';
import { renderInShell } from '../testkit.tsx';
import { expectNoAxeViolations, expectNoInlineStyle, problem } from '../uxTestkit.ts';

beforeEach(() => history.replaceState(null, '', '/admin'));

interface Row {
  name: string;
  users: number;
}
const ROWS: Row[] = [
  { name: 'example.org', users: 3 },
  { name: 'example.net', users: 1 },
];

function ListHarness(props: { route: AdminRoute; navigate?: (p: string) => void }) {
  return (
    <ListDetailPage<Row>
      title="Domains"
      description="Mail domains this server accepts."
      section="domains"
      route={props.route}
      {...(props.navigate ? { navigate: props.navigate } : {})}
      table={{
        caption: 'Domains',
        columns: [
          { id: 'name', header: 'Name', cell: (r) => r.name },
          { id: 'users', header: 'Users', cell: (r) => String(r.users) },
        ],
        rows: ROWS,
        rowKey: (r) => r.name,
        state: 'ready',
        emptyText: 'No domains yet.',
        search: { label: 'Search domains', value: '', onSearch: () => undefined },
        filters: [{ id: 'listed', label: 'Listed', active: false }],
        paging: { hasPrev: false, hasNext: true, onPrev: () => undefined, onNext: () => undefined },
      }}
      detailTitle={(id) => id}
      renderDetail={(id) => <p>Detail of {id}</p>}
    />
  );
}

describe('ListDetailPage', () => {
  it('lists rows with an Open link to #/<section>/<id>; search, filter chips and paging come from the table', () => {
    const navigate = vi.fn();
    render(() => <ListHarness route={{ section: 'domains', rest: [] }} navigate={navigate} />);
    expect(screen.getByRole('heading', { level: 2, name: 'Domains' })).toBeInTheDocument();
    expect(screen.getByRole('searchbox', { name: 'Search domains' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Listed' })).toHaveAttribute('aria-pressed', 'false');
    expect(screen.getByRole('navigation', { name: /Domains/ })).toBeInTheDocument();
    const open = screen.getByRole('link', { name: 'Open example.org' });
    expect(open).toHaveAttribute('href', '/admin#/domains/example.org');
    fireEvent.click(open);
    expect(navigate).toHaveBeenCalledWith('/admin#/domains/example.org');
    expect(screen.queryByTestId('list-detail')).toBeNull();
  });

  it('the open item shows its detail, focus moves to its heading, and Back returns to the list', async () => {
    const navigate = vi.fn();
    const [route, setRoute] = createSignal<AdminRoute>({ section: 'domains', rest: [] });
    render(() => <ListHarness route={route()} navigate={navigate} />);
    setRoute({ section: 'domains', rest: ['example.net'] });
    const detail = await screen.findByTestId('list-detail');
    expect(within(detail).getByText('Detail of example.net')).toBeInTheDocument();
    const heading = within(detail).getByRole('heading', { level: 3, name: 'example.net' });
    await waitFor(() => expect(document.activeElement).toBe(heading));
    expect(screen.queryByRole('link', { name: 'Open example.net' })).not.toBeInTheDocument();
    expect(screen.queryByRole('table')).not.toBeInTheDocument();
    expect(detail.parentElement).toHaveAttribute('data-detail-open');
    fireEvent.click(within(detail).getByRole('link', { name: 'Back to the list' }));
    expect(navigate).toHaveBeenCalledWith('/admin#/domains');
  });

  it('without a navigate prop it uses the shell router', () => {
    const { navigate } = renderInShell(() => <ListHarness route={{ section: 'domains', rest: [] }} />);
    fireEvent.click(screen.getByRole('link', { name: 'Open example.net' }));
    expect(navigate).toHaveBeenCalledWith('/admin#/domains/example.net');
  });

  it('has no axe violations with a detail open, and no inline styles', async () => {
    const { container } = render(() => <ListHarness route={{ section: 'domains', rest: ['example.org'] }} />);
    await screen.findByTestId('list-detail');
    await expectNoAxeViolations(container);
    expectNoInlineStyle(container);
  });
});

describe('WizardPage', () => {
  it('steps with per-step validation, the description on the first step, a review step and Save', async () => {
    const onSave = vi.fn(async () => undefined);
    const [name, setName] = createSignal('');
    const { container } = render(() => (
      <WizardPage
        title="Add a domain"
        description="Adds a domain this server accepts mail for."
        steps={[
          {
            id: 'name',
            title: 'Name',
            content: () => (
              <label>
                Domain name
                <input value={name()} onInput={(e) => setName(e.currentTarget.value)} />
              </label>
            ),
            validate: () => (name() === '' ? 'Enter the domain name.' : null),
          },
        ]}
        review={() => [{ label: 'Domain name', value: name() }]}
        onSave={onSave}
      />
    ));
    expect(screen.getByRole('heading', { level: 2, name: 'Add a domain' })).toBeInTheDocument();
    expect(screen.getByText('Adds a domain this server accepts mail for.')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Next' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Enter the domain name.');
    fireEvent.input(screen.getByLabelText('Domain name'), { target: { value: 'example.org' } });
    fireEvent.click(screen.getByRole('button', { name: 'Next' }));
    expect(await screen.findByText('example.org')).toBeInTheDocument();
    await expectNoAxeViolations(container);
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(onSave).toHaveBeenCalledTimes(1));
  });
});

describe('DashboardPage', () => {
  it('a grid of tiles in a named list; other states replace the grid', () => {
    const [state, setState] = createSignal<Parameters<typeof DashboardPage>[0]['state']>({ kind: 'loading' });
    render(() => (
      <DashboardPage
        title="Live metrics"
        state={state()}
        tilesLabel="Figures"
        tiles={[
          { id: 'req', render: () => <p>Requests: 12</p> },
          { id: 'chart', wide: true, render: () => <p>Chart</p> },
        ]}
      />
    ));
    expect(screen.getByRole('status')).toHaveTextContent('Loading');
    setState({ kind: 'unavailable', reason: 'metrics are switched off.' });
    expect(screen.getByText('Not available — metrics are switched off.')).toBeInTheDocument();
    setState({ kind: 'ready' });
    const list = screen.getByRole('list', { name: 'Figures' });
    expect(within(list).getAllByRole('listitem')).toHaveLength(2);
    expect(list.querySelector('[data-tile="chart"]')!.className).toMatch(/\S+ \S+/);
  });
});

describe('PageState and AdminPage', () => {
  it('error: what failed, the code and Try again; empty: the next step', () => {
    const retry = vi.fn();
    const add = vi.fn();
    render(() => (
      <AdminPage title="Bans" actions={<button type="button">Add a ban</button>}>
        <PageState state={{ kind: 'error', error: problem(503, { code: 'maintenance' }), onRetry: retry }} />
        <PageState state={{ kind: 'empty', text: 'No bans.', action: { label: 'Add the first ban', onClick: add } }} />
      </AdminPage>
    ));
    expect(screen.getByRole('alert')).toBeInTheDocument();
    expect(screen.getByText('Error code: maintenance')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Try again' }));
    fireEvent.click(screen.getByRole('button', { name: 'Add the first ban' }));
    expect(retry).toHaveBeenCalled();
    expect(add).toHaveBeenCalled();
    expect(screen.getByRole('region', { name: 'Bans' })).toBeInTheDocument();
    expect(errorCode({ status: 500 })).toBe('HTTP 500');
    expect(errorCode(new Error('x'))).toBeNull();
  });
});

interface Line {
  id: string;
  text: string;
}

describe('StreamPage', () => {
  function setup() {
    let handlers!: StreamHandlers<Line>;
    const close = vi.fn();
    const connect = vi.fn((h: StreamHandlers<Line>) => {
      handlers = h;
      return close;
    });
    const utils = render(() => (
      <StreamPage<Line>
        title="Logs"
        connect={connect}
        renderEntry={(l) => l.text}
        entryKey={(l) => l.id}
        entryText={(l) => l.text}
        emptyText="No log lines yet."
        logLabel="Log lines"
      />
    ));
    return { ...utils, connect, close, push: (l: Line) => handlers.onEntry(l), state: (s: Parameters<StreamHandlers<Line>['onState']>[0]) => handlers.onState(s) };
  }

  it('shows entries as a focusable log; the status is said in words', () => {
    const s = setup();
    expect(screen.getByTestId('stream-status')).toHaveTextContent('Connecting…');
    expect(screen.getByText('No log lines yet.')).toBeInTheDocument();
    s.state('live');
    s.push({ id: '1', text: 'GET /api/x 200' });
    s.push({ id: '2', text: 'POST /api/y 500' });
    const log = screen.getByRole('log', { name: 'Log lines' });
    expect(log).toHaveAttribute('tabindex', '0');
    expect(log).toHaveAttribute('aria-live', 'off');
    expect(within(log).getAllByRole('listitem').map((li) => li.textContent)).toEqual(['GET /api/x 200', 'POST /api/y 500']);
    expect(screen.getByTestId('stream-status')).toHaveTextContent('Live');
  });

  it('Pause holds new entries and counts them; Resume shows them', () => {
    const s = setup();
    s.state('live');
    s.push({ id: '1', text: 'one' });
    fireEvent.click(screen.getByRole('button', { name: 'Pause' }));
    s.push({ id: '2', text: 'two' });
    s.push({ id: '3', text: 'three' });
    expect(screen.getByTestId('stream-status')).toHaveTextContent('Paused — 2 new entries held');
    expect(within(screen.getByRole('log')).getAllByRole('listitem')).toHaveLength(1);
    fireEvent.click(screen.getByRole('button', { name: 'Resume' }));
    expect(within(screen.getByRole('log')).getAllByRole('listitem')).toHaveLength(3);
  });

  it('filter, read-aloud switch, lost connection with Reconnect, and close on unmount', () => {
    const s = setup();
    s.push({ id: '1', text: 'alpha' });
    s.push({ id: '2', text: 'beta' });
    fireEvent.input(screen.getByRole('searchbox', { name: 'Filter' }), { target: { value: 'bet' } });
    expect(within(screen.getByRole('log')).getAllByRole('listitem').map((li) => li.textContent)).toEqual(['beta']);
    fireEvent.click(screen.getByRole('checkbox', { name: 'Read new entries aloud' }));
    expect(screen.getByRole('log')).toHaveAttribute('aria-live', 'polite');
    s.state('error');
    fireEvent.click(screen.getByRole('button', { name: 'Reconnect' }));
    expect(s.connect).toHaveBeenCalledTimes(2);
    s.unmount();
    expect(s.close).toHaveBeenCalled();
  });

  it('a stream the server does not offer says Not available', () => {
    const s = setup();
    s.state('unavailable');
    expect(screen.getByTestId('stream-status')).toHaveTextContent('Not available — this server does not offer it yet.');
    expect(screen.queryByRole('log')).toBeNull();
  });

  it('has no axe violations', async () => {
    const s = setup();
    s.state('live');
    s.push({ id: '1', text: 'one' });
    await expectNoAxeViolations(s.container);
    expectNoInlineStyle(s.container);
  });
});
