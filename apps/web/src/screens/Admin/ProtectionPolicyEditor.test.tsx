import { beforeAll, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor, within } from '@solidjs/testing-library';
import { loadCatalog } from '../../i18n';
import { ProtectionPolicyEditor } from './ProtectionPolicyEditor.tsx';
import { presetDiff, ruleSlug, validateRule } from './protectionRules.ts';
import type { PolicyChange, ProtectionPolicy, ProtectionPolicyApi, SimResult } from '../../api/admin/requestProtection.ts';
import type { AdminSetting } from '../../api/admin/settings.ts';

beforeAll(async () => { await loadCatalog('admin'); await loadCatalog('admin-security'); });

function setting(short: string, kind: AdminSetting['kind'], value: unknown, def: unknown): AdminSetting {
  return {
    key: `security.protection.${short}`, family: false, kind, default: def, nullable: false, scopes: 'global', userOverride: false, category: 'security', mapsTo: '',
    env: null, dedicatedRoute: null, scopeId: 'account', hot: true, ceiling: false, userVisible: false, globalValue: value === def ? null : value, allowUserOverride: false,
    globalLocked: false, updatedBy: null, updatedAt: null, effective: { value, source: value === def ? 'default' : 'global' }, envState: null, overrides: {}, etag: `"${short}-1"`,
  };
}
const modes = { type: 'enum', values: ['off', 'observe', 'enforce'] } as const;
function policy(): ProtectionPolicy {
  return {
    rules: [
      setting('rateMode', { ...modes, values: [...modes.values] }, 'off', 'off'),
      setting('adaptiveMode', { ...modes, values: [...modes.values] }, 'observe', 'off'),
      setting('clientRequests', { type: 'int', min: 0, max: 1_000_000 }, 600, 600),
      setting('rateWindowSecs', { type: 'int', min: 1, max: 3600 }, 60, 60),
      setting('throttleSecs', { type: 'int', min: 1, max: 300 }, 3, 3),
      setting('signals', { type: 'stringList', maxItems: 6 }, ['malformed', 'oversized', 'credential-rejected'], ['malformed', 'oversized', 'credential-rejected']),
    ],
    presets: {
      relaxed: { 'security.protection.rateMode': 'observe' },
      balanced: { 'security.protection.rateMode': 'enforce', 'security.protection.adaptiveMode': 'enforce', 'security.protection.clientRequests': 600 },
      strict: { 'security.protection.rateMode': 'enforce', 'security.protection.clientRequests': 300 },
    },
    canEdit: true,
    domainScopes: false,
  };
}
function api(overrides: Partial<ProtectionPolicyApi> = {}): ProtectionPolicyApi {
  return {
    policy: vi.fn(async () => policy()),
    save: vi.fn(async (changes: PolicyChange[]) => changes.map((c) => {
      const base = policy().rules.find((r) => r.key === c.key)!;
      const value = c.value ?? base.default;
      return { ...base, globalValue: c.value, effective: { value, source: 'global' }, etag: '"saved"' };
    })),
    simulate: vi.fn(async (): Promise<SimResult> => ({ action: 'throttle', reason: 'short-absolute', enforced: true, durationSecs: 3, signalCounted: true, off: false, exempt: false, rules: [{ rule: 'short-absolute', enabled: true, value: 20, threshold: 20, fired: true }] })),
    ...overrides,
  };
}

describe('protection rule helpers', () => {
  it('slugs, validates against the registry bounds and diffs presets', () => {
    expect(ruleSlug('security.protection.shortWindowSecs')).toBe('short-window-secs');
    const rules = policy().rules;
    const client = rules.find((r) => r.key.endsWith('clientRequests'))!;
    expect(validateRule(client, 2_000_000)).toBe('max');
    expect(validateRule(client, 1.5)).toBe('whole');
    expect(validateRule(client, 10)).toBeNull();
    expect(presetDiff(rules, policy().presets.balanced).map((d) => d.rule.key)).toEqual(['security.protection.rateMode', 'security.protection.adaptiveMode']);
  });
});

describe('Adaptive protection editor', () => {
  it('explains today\'s policy in words', async () => {
    render(() => <ProtectionPolicyEditor api={api()} canWrite />);
    const summary = await screen.findByTestId('protection-summary');
    expect(summary).toHaveTextContent('Request budget: Off.');
    expect(summary).toHaveTextContent('Unusual requests: Observe only.');
    expect(summary).toHaveTextContent(/Counted as unusual: Malformed request, Request too large, Wrong password or key/);
    expect(screen.getAllByText('Example:').length).toBeGreaterThan(3);
  });

  it('shows a preset\'s differences and applies only those, naming the preset', async () => {
    const a = api();
    render(() => <ProtectionPolicyEditor api={a} canWrite />);
    fireEvent.click(await screen.findByRole('radio', { name: /Balanced/ }));
    const diff = screen.getByTestId('preset-diff');
    expect(within(diff).getAllByRole('row')).toHaveLength(3);
    expect(diff).toHaveTextContent('Enforce');
    fireEvent.click(screen.getByRole('button', { name: 'Apply 2 changes' }));
    await waitFor(() => expect(a.save).toHaveBeenCalledWith([
      { key: 'security.protection.rateMode', value: 'enforce', etag: '"rateMode-1"' },
      { key: 'security.protection.adaptiveMode', value: 'enforce', etag: '"adaptiveMode-1"' },
    ], 'balanced'));
    expect(await screen.findByText('Balanced applied.')).toBeInTheDocument();
  });

  it('edits a rule in place with validation, its ETag, and resets it to the default', async () => {
    const a = api();
    render(() => <ProtectionPolicyEditor api={a} canWrite />);
    const row = await screen.findByTestId('protect-rule-client-requests');
    const input = within(row).getByLabelText('Requests per client');
    fireEvent.input(input, { target: { value: '2000000' } });
    fireEvent.click(within(row).getByRole('button', { name: 'Save Requests per client' }));
    expect(await within(row).findByRole('alert')).toHaveTextContent('The largest allowed value is 1,000,000.');
    expect(a.save).not.toHaveBeenCalled();
    fireEvent.input(input, { target: { value: '450' } });
    fireEvent.click(within(row).getByRole('button', { name: 'Save Requests per client' }));
    await waitFor(() => expect(a.save).toHaveBeenCalledWith([{ key: 'security.protection.clientRequests', value: 450, etag: '"clientRequests-1"' }], undefined));
    const reset = await within(await screen.findByTestId('protect-rule-client-requests')).findByRole('button', { name: 'Reset Requests per client to its default' });
    expect(screen.getByTestId('protect-rule-client-requests')).toHaveTextContent('Default: 600');
    await waitFor(() => expect(reset).not.toBeDisabled());
    fireEvent.click(reset);
    await waitFor(() => expect(a.save).toHaveBeenLastCalledWith([{ key: 'security.protection.clientRequests', value: null, etag: '"saved"' }], undefined));
  });

  it('a server refusal stays on the row', async () => {
    const a = api({ save: vi.fn(async () => { throw { status: 422, code: 'invalidPolicy', detail: 'These values together do not make a usable protection policy' }; }) });
    render(() => <ProtectionPolicyEditor api={a} canWrite />);
    const row = await screen.findByTestId('protect-rule-throttle-secs');
    fireEvent.input(within(row).getByLabelText('Slow-down'), { target: { value: '10' } });
    fireEvent.click(within(row).getByRole('button', { name: 'Save Slow-down' }));
    expect(await within(row).findByRole('alert')).toBeInTheDocument();
  });

  it('is read-only without settings.write', async () => {
    render(() => <ProtectionPolicyEditor api={api()} canWrite={false} />);
    expect(await screen.findByTestId('protection-readonly')).toBeInTheDocument();
    expect(within(screen.getByTestId('protect-rule-client-requests')).getByLabelText('Requests per client')).toBeDisabled();
  });

  it('simulates a sample request and says which rule decided', async () => {
    const a = api();
    render(() => <ProtectionPolicyEditor api={a} canWrite />);
    const sim = await screen.findByTestId('protection-simulator');
    fireEvent.input(within(sim).getByLabelText('Unusual requests already in the short window'), { target: { value: '19' } });
    fireEvent.click(within(sim).getByRole('button', { name: 'Check' }));
    const result = await screen.findByTestId('protection-sim-result');
    expect(result).toHaveAttribute('data-action', 'throttle');
    expect(result).toHaveTextContent(/Slowed down for 3 s, because of: Short-window anomaly count/);
    expect(a.simulate).toHaveBeenCalledWith({ subjectKind: 'client', sample: { requestsInWindow: 1, short: { normal: 0, anomalous: 19 }, long: { normal: 0, anomalous: 0 }, signal: 'malformed', previouslyThrottled: false } });
  });
});
