import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import PolicyTab from './PolicyTab.tsx';
import RecordsTab from './RecordsTab.tsx';
import DpiaTab, { riskLevel, riskScore } from './DpiaTab.tsx';
import type { ComplianceApi, Dpia, PolicyView } from '../../../api/admin/compliance.ts';
import { renderInShell } from '../shell/testkit.tsx';

afterEach(cleanup);

const view = (over: Partial<PolicyView> = {}): PolicyView => ({
  domain: '', lang: 'en', document: 'privacy', langs: ['en', 'pt-PT'],
  sections: [
    { id: 'controller', title: 'Who is responsible', body: 'Controller: (not yet filled in by the operator)', overridden: false, missing: ['policy.controller.name'] },
    { id: 'contact', title: 'Contact and complaints', body: 'Write to us.', overridden: true, missing: [] },
  ],
  published: null, warnings: ['policy.controller.name'], version: 2, publishedAt: 1_800_000_000,
  pending: [{ key: 'cookies.mode', old: 'off', new: 'on' }], autoVersion: true,
  versions: [{ version: 2, kind: 'auto', createdBy: 'system', createdAt: 1_800_000_000, changes: [{ key: 'cookies.mode', old: 'off', new: 'on' }] }, { version: 1, kind: 'published', createdBy: 'admin', createdAt: 1_790_000_000, changes: [] }],
  overrides: [], ...over,
});

const dpia = (over: Partial<Dpia> = {}): Dpia => ({
  id: 'd1', title: 'Assistant', status: 'draft', template: 'assistant', version: 1, reviewAt: null, due: false, overdue: false,
  createdBy: 'a', updatedBy: 'a', createdAt: 1, updatedAt: 1,
  body: { feature: 'assistant', activityId: '', description: 'd', purposes: '', necessity: '', proportionality: '', mitigations: '', dpoOpinion: '', consultation: '', notes: '', links: [], risks: [{ title: 'Leak', description: '', mitigations: '', likelihood: 3, severity: 3 }] },
  ...over,
});

describe('Policy tab', () => {
  it('shows the version, pending changes, warnings, history and publishes', async () => {
    const api = { policy: vi.fn(async () => view()), publish: vi.fn(async () => ({ version: 3, changed: true })), saveOverride: vi.fn(async () => undefined) } as unknown as ComplianceApi;
    renderInShell(() => <PolicyTab api={api} />);
    expect(await screen.findByTestId('policy-version')).toHaveTextContent('Version 2');
    expect(screen.getByTestId('policy-pending')).toBeInTheDocument();
    expect(screen.getByTestId('policy-warnings')).toBeInTheDocument();
    expect(screen.getByTestId('policy-history-1')).toBeInTheDocument();
    fireEvent.click(screen.getByTestId('policy-publish'));
    await waitFor(() => expect(api.publish).toHaveBeenCalled());
  });

  it('saves a section override for the chosen language', async () => {
    const api = { policy: vi.fn(async () => view()), saveOverride: vi.fn(async () => undefined), publish: vi.fn() } as unknown as ComplianceApi;
    renderInShell(() => <PolicyTab api={api} />);
    fireEvent.click(await screen.findByTestId('policy-override-controller'));
    fireEvent.input(screen.getByTestId('policy-draft-controller'), { target: { value: 'Example Ltd' } });
    fireEvent.click(screen.getByTestId('policy-save-controller'));
    await waitFor(() => expect(api.saveOverride).toHaveBeenCalledWith('en', 'controller', 'Example Ltd'));
  });

  it('hides write controls without write access', async () => {
    const api = { policy: vi.fn(async () => view()) } as unknown as ComplianceApi;
    renderInShell(() => <PolicyTab api={api} canWrite={false} />);
    await screen.findByTestId('policy-version');
    expect(screen.queryByTestId('policy-publish')).toBeNull();
    expect(screen.queryByTestId('policy-override-controller')).toBeNull();
  });
});

describe('Records tab', () => {
  it.each(['activity', 'processor'] as const)('lists generated and custom rows, exports and guides adding a %s', async (kind) => {
    const api = {
      records: vi.fn(async () => ({
        activities: [{ id: 'mail-service', builtin: true, enabled: true, name: 'Mail service', purpose: 'Providing mail' }, { id: 'c1', builtin: false, enabled: true, name: 'Payroll', purpose: 'Pay staff' }],
        processors: [{ id: 'push', builtin: true, enabled: true, name: 'Push services' }],
      })),
      exportRecords: vi.fn(async () => 'name\r\n'),
      addRecord: vi.fn(async () => ({})),
    } as unknown as ComplianceApi;
    URL.createObjectURL = vi.fn(() => 'blob:x');
    URL.revokeObjectURL = vi.fn();
    renderInShell(() => <RecordsTab api={api} />);
    expect(await screen.findByTestId('record-c1')).toHaveTextContent('Payroll');
    expect(screen.getByTestId('record-mail-service')).toHaveTextContent('Generated');
    fireEvent.click(screen.getByTestId('export-activities-csv'));
    await waitFor(() => expect(api.exportRecords).toHaveBeenCalledWith('activities', 'csv', 'en'));
    fireEvent.click(screen.getByTestId(`add-${kind}`));
    fireEvent.input(screen.getByTestId('record-name'), { target: { value: 'Newsletter' } });
    fireEvent.click(screen.getByTestId('record-next'));
    fireEvent.click(screen.getByTestId('record-next'));
    expect(screen.getByRole('dialog')).toHaveTextContent('Review and save');
    fireEvent.click(screen.getByTestId('record-save'));
    await waitFor(() => expect(api.addRecord).toHaveBeenCalledWith(kind, 'Newsletter', expect.objectContaining({ purpose: '' }), true));
  });
});

describe('DPIA tab', () => {
  it('scores risks by likelihood times severity', () => {
    expect(riskScore(3, 3)).toBe(9);
    expect(riskLevel(9)).toBe('high');
    expect(riskLevel(12)).toBe('veryHigh');
    expect(riskLevel(null)).toBeNull();
    expect(riskScore(null, 3)).toBeNull();
  });

  it('lists assessments with due badges, suggestions, and creates from a suggestion', async () => {
    const api = {
      dpias: vi.fn(async () => [dpia({ id: 'd2', title: 'Tracking', status: 'approved', reviewAt: 1_700_000_000, due: true, overdue: true })]),
      suggestions: vi.fn(async () => [{ feature: 'assistant', template: 'assistant', setting: 'assistant.enabled' }]),
      createDpia: vi.fn(async () => dpia()),
    } as unknown as ComplianceApi;
    renderInShell(() => <DpiaTab api={api} />);
    expect(await screen.findByTestId('dpia-row-d2')).toHaveTextContent('Overdue');
    fireEvent.click(await screen.findByText('Start from template'));
    await waitFor(() => expect(api.createDpia).toHaveBeenCalledWith('AI assistant', 'assistant', 'en'));
    expect(await screen.findByTestId('dpia-editor')).toBeInTheDocument();
    expect(screen.getByTestId('dpia-score-0')).toHaveTextContent('9 high');
  });

  it('saves with the loaded version, the chosen status and the edited risk scores', async () => {
    const api = {
      dpias: vi.fn(async () => [dpia()]),
      suggestions: vi.fn(async () => []),
      dpia: vi.fn(async () => dpia()),
      saveDpia: vi.fn(async () => dpia({ version: 2, status: 'review' })),
    } as unknown as ComplianceApi;
    renderInShell(() => <DpiaTab api={api} />);
    fireEvent.click(await screen.findByText('Edit'));
    await screen.findByTestId('dpia-editor');
    fireEvent.change(screen.getByTestId('dpia-status'), { target: { value: 'review' } });
    fireEvent.change(screen.getAllByLabelText('Residual likelihood')[0]!, { target: { value: '1' } });
    fireEvent.click(screen.getByTestId('dpia-save'));
    await waitFor(() => expect(api.saveDpia).toHaveBeenCalled());
    const [id, version, change] = (api.saveDpia as ReturnType<typeof vi.fn>).mock.calls[0]!;
    expect(id).toBe('d1');
    expect(version).toBe(1);
    expect(change.status).toBe('review');
    expect(change.body.risks[0].residualLikelihood).toBe(1);
  });
});
