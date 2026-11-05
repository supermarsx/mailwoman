import { beforeAll, describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { loadCatalog } from '../../i18n';
import { AdaptiveInsights, AggregateInsights, InsightsSummary, RangeSelect, createProtectionHistory, classLabel, ruleLabel } from './ProtectionInsights.tsx';
import { RequestProtection } from './RequestProtection.tsx';
import { rememberSimulation } from './simulationMemory.ts';
import { renderInShell } from './shell/testkit.tsx';
import type { ProtectionApi, ProtectionHistory, ProtectionRange, ProtectionStatus, SimResult } from '../../api/admin/requestProtection.ts';

beforeAll(async () => { for (const c of ['admin', 'admin-security', 'admin-settings', 'admin-stats']) await loadCatalog(c); });

const zero = { throttleObserved: 0, throttleEnforced: 0, blockObserved: 0, blockEnforced: 0 };
function history(over: Partial<ProtectionHistory> = {}): ProtectionHistory {
  const t0 = 1_800_000_000;
  return {
    rangeSecs: 3600, stepSecs: 60, start: t0, scope: 'x',
    points: [
      { t: t0, allowed: 10, throttled: 1, blocked: 0, actions: zero },
      { t: t0 + 60, allowed: 20, throttled: 2, blocked: 3, actions: { ...zero, throttleEnforced: 2, blockEnforced: 3 } },
    ],
    totals: { evaluated: 36, allowed: 30, throttled: 3, blocked: 3 },
    budgets: [
      { budget: 'client', allowed: 20, throttled: 3, blocked: 0 },
      { budget: 'account', allowed: 10, throttled: 0, blocked: 0 },
      { budget: 'admin', allowed: 0, throttled: 0, blocked: 0 },
      { budget: 'userAgent', allowed: 0, throttled: 0, blocked: 3 },
    ],
    userAgents: [{ class: 'ai-training', allowed: 0, throttled: 0, blocked: 3 }, { class: 'search', allowed: 4, throttled: 0, blocked: 0 }],
    tiers: { unknown: 20, known: 6, authenticated: 10 },
    actions: { throttleObserved: 1, throttleEnforced: 2, blockObserved: 0, blockEnforced: 3 },
    rules: [{ rule: 'shortRatio', count: 4 }, { rule: 'probePath', count: 1 }],
    ...over,
  };
}
const status = { policy: { rateMode: 'enforce', adaptiveMode: 'enforce' }, subjectCapacity: 20000 } as unknown as ProtectionStatus;
function api(h: ProtectionHistory | Error | null = history()): ProtectionApi & { history: ReturnType<typeof vi.fn> } {
  return {
    status: async () => status,
    subjects: async () => ({ subjects: [], cursor: null, total: 7 }),
    events: async () => ({ events: [], cursor: null, total: 0 }),
    clear: async () => undefined,
    history: vi.fn(async (_r: ProtectionRange) => { if (h instanceof Error) throw h; return h as ProtectionHistory; }),
  } as never;
}
function Harness(props: { a: ProtectionApi; kind: 'aggregate' | 'adaptive' | 'summary' }) {
  const h = createProtectionHistory(() => props.a);
  return <>
    <RangeSelect history={h} />
    {props.kind === 'aggregate' ? <AggregateInsights history={h} /> : props.kind === 'adaptive' ? <AdaptiveInsights history={h} status={status} objects={7} /> : <InsightsSummary history={h} />}
  </>;
}

describe('Aggregate insights', () => {
  it('shows tiles, the three charts and the top limited budget and class', async () => {
    renderInShell(() => <Harness a={api()} kind="aggregate" />);
    expect(await screen.findByTestId('agg-evaluated')).toHaveTextContent('36');
    expect(screen.getByTestId('agg-throttled')).toHaveTextContent('3');
    expect(screen.getByTestId('agg-share')).toHaveTextContent('16.7%');
    expect(screen.getByTestId('agg-top-budget')).toHaveTextContent('Per address');
    expect(screen.getByTestId('agg-top-class')).toHaveTextContent('AI training crawler');
    for (const id of ['aggregate-chart-time', 'aggregate-chart-budgets', 'aggregate-chart-classes']) expect(screen.getByTestId(id)).toBeInTheDocument();
    expect(screen.getByTestId('insights-scope')).toHaveTextContent('Counts only');
  });
  it('gives each chart a data table behind a View as table switch and never relies on colour alone', async () => {
    renderInShell(() => <Harness a={api()} kind="aggregate" />);
    const card = await screen.findByTestId('aggregate-chart-time');
    const toggle = within(card).getByRole('button', { name: 'View as table' });
    expect(toggle).toHaveAttribute('aria-pressed', 'false');
    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-pressed', 'true');
    const table = within(card).getByRole('table');
    expect(within(table).getAllByRole('row')).toHaveLength(3);
    expect(within(table).getByRole('columnheader', { name: 'Throttled' })).toBeInTheDocument();
    expect(within(card).getByRole('list', { name: /legend/i })).toHaveTextContent('Throttled or blocked');
    expect(card).toHaveAccessibleDescription(/30 requests were allowed/);
  });
  it('shows empty states when nothing was counted', async () => {
    const empty = history({ points: [], totals: { evaluated: 0, allowed: 0, throttled: 0, blocked: 0 }, userAgents: [], budgets: history().budgets.map((b) => ({ ...b, allowed: 0, throttled: 0, blocked: 0 })) });
    renderInShell(() => <Harness a={api(empty)} kind="aggregate" />);
    expect(await screen.findAllByText('No requests were counted in this range.')).not.toHaveLength(0);
    expect(screen.getByText('No user-agent class was counted in this range.')).toBeInTheDocument();
  });
  it('asks the server again when the shared range changes', async () => {
    const a = api();
    renderInShell(() => <Harness a={a} kind="aggregate" />);
    await screen.findByTestId('agg-evaluated');
    expect(a.history).toHaveBeenCalledWith('1h');
    fireEvent.change(screen.getByRole('combobox', { name: 'Range' }), { target: { value: '6h' } });
    await waitFor(() => expect(a.history).toHaveBeenCalledWith('6h'));
  });
  it('says so when the history cannot be loaded or the server has none', async () => {
    renderInShell(() => <Harness a={api(new Error('x'))} kind="aggregate" />);
    expect(await screen.findByTestId('insights-error')).toHaveTextContent('could not be loaded');
  });
  it('says so when the API has no history', async () => {
    const a = api();
    delete (a as { history?: unknown }).history;
    renderInShell(() => <Harness a={a} kind="aggregate" />);
    expect(await screen.findByText('This server does not report request protection history yet.')).toBeInTheDocument();
  });
});

describe('Adaptive insights', () => {
  it('shows tiles with the mode, objects, restrictions, tier requests and top rule', async () => {
    renderInShell(() => <Harness a={api()} kind="adaptive" />);
    expect(await screen.findByTestId('ada-mode')).toHaveTextContent('Enforce');
    expect(screen.getByTestId('ada-objects')).toHaveTextContent('7');
    expect(screen.getByTestId('ada-restrictions')).toHaveTextContent('5');
    expect(screen.getByTestId('ada-observed')).toHaveTextContent('1');
    expect(screen.getByTestId('ada-top-rule')).toHaveTextContent('Short-window unusual-request proportion');
    expect(screen.getByTestId('ada-tier-unknown')).toHaveTextContent('20');
  });
  it('charts actions over time, the tier distribution with percentages and ranked rules', async () => {
    renderInShell(() => <Harness a={api()} kind="adaptive" />);
    const tiers = await screen.findByTestId('adaptive-chart-tiers');
    fireEvent.click(within(tiers).getByRole('button', { name: 'View as table' }));
    expect(within(tiers).getByRole('table')).toHaveTextContent('55.6%');
    expect(within(screen.getByTestId('adaptive-chart-rules')).getAllByRole('listitem', { hidden: true })[0]).toHaveTextContent('Short-window unusual-request proportion');
    expect(screen.getByTestId('adaptive-chart-actions')).toBeInTheDocument();
  });
  it('shows the simulator last result, or that it was not run', async () => {
    rememberSimulation(undefined);
    renderInShell(() => <Harness a={api()} kind="adaptive" />);
    expect(await screen.findByText('The simulator has not been run in this session.')).toBeInTheDocument();
    rememberSimulation({ action: 'throttle', reason: 'short-ratio', enforced: true, durationSecs: 60, off: false, exempt: false, signalCounted: true, rules: [] } as SimResult);
    const box = await screen.findByTestId('insights-simulation');
    await waitFor(() => expect(box).toHaveTextContent('Slowed down'));
    expect(box.querySelector('time[title]')).not.toBeNull();
    rememberSimulation(undefined);
  });
  it('says plainly that de-escalations are not counted', async () => {
    renderInShell(() => <Harness a={api()} kind="adaptive" />);
    expect(await screen.findByText(/does not count de-escalations/)).toBeInTheDocument();
  });
});

describe('Request protection screen', () => {
  const route = (tab: string) => () => ({ section: 'security', rest: ['protection', tab] });
  it('puts the charts on the Aggregate and Adaptive subtabs and a summary on the Overview', async () => {
    const a = api();
    const view = renderInShell(() => <RequestProtection api={a} />, { route: route('aggregate') });
    expect(await screen.findByTestId('aggregate-tiles')).toBeInTheDocument();
    view.unmount();
    renderInShell(() => <RequestProtection api={a} />, { route: route('adaptive') });
    expect(await screen.findByTestId('adaptive-tiles')).toBeInTheDocument();
    expect(await screen.findByTestId('ada-objects')).toHaveTextContent('7');
  });
  it('shows the summary on the Overview', async () => {
    renderInShell(() => <RequestProtection api={api()} />, { route: route('overview') });
    expect(await screen.findByTestId('insights-summary')).toBeInTheDocument();
  });
});

describe('labels', () => {
  it('names known classes and rules and passes unknown classes through', () => {
    expect(classLabel('ai-training')).toBe('AI training crawler');
    expect(classLabel('other')).toBe('Other classes');
    expect(classLabel('weird')).toBe('weird');
    expect(ruleLabel('probePath')).toMatch(/scan path/);
  });
});
