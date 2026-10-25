import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { RulesSettings } from './RulesSettings.tsx';
import { RuleBuilder } from './RuleBuilder.tsx';
import { RawEditor } from './RawEditor.tsx';
import { emptyRuleDraft, type RulesSlice } from '../../state/slices/rules.ts';
import type { MailRule } from '../../api/crypto-types.ts';
import { t } from '../../test/i18n.ts';
import type { SieveApi } from './sieve-api.ts';

function fakeSlice(initial: MailRule[]): { slice: RulesSlice; saveRule: ReturnType<typeof vi.fn>; deleteRule: ReturnType<typeof vi.fn> } {
  const [rules, setRules] = createSignal<MailRule[]>(initial);
  const saveRule = vi.fn(async () => undefined);
  const deleteRule = vi.fn(async (id: string): Promise<void> => {
    setRules(rules().filter((r) => r.id !== id));
  });
  const toggleRule = vi.fn(async () => undefined);
  const slice: RulesSlice = {
    rules,
    rulesLoading: () => false,
    loadRules: vi.fn(async () => undefined),
    saveRule,
    deleteRule,
    toggleRule,
  };
  return { slice, saveRule, deleteRule };
}

function ruleFixture(id: string, name: string, runsAt: MailRule['runsAt'] = 'engine'): MailRule {
  return {
    id,
    name,
    matchAll: true,
    conditions: [{ type: 'from', op: 'contains', value: 'x@y' }],
    actions: [{ type: 'move', value: 'A' }],
    enabled: true,
    runsAt,
  };
}

describe('RulesSettings section', () => {
  it('renders the rules heading and the rule list', () => {
    const { slice } = fakeSlice([ruleFixture('1', 'Newsletters', 'server-sieve')]);
    render(() => <RulesSettings accountId="acct1" slice={slice} />);
    expect(screen.getByText(t('rules-title'))).toBeInTheDocument();
    // The (untrusted) name is bidi-isolated, so match loosely.
    expect(screen.getByText((c) => c.includes('Newsletters'))).toBeInTheDocument();
    // Where-it-runs badge reflects runsAt.
    expect(screen.getAllByText(t('rules-runs-server')).length).toBeGreaterThan(0);
  });

  it('shows the empty state with no rules', () => {
    const { slice } = fakeSlice([]);
    render(() => <RulesSettings accountId="acct1" slice={slice} />);
    expect(screen.getByText(t('rules-empty'))).toBeInTheDocument();
  });

  it('opens the builder on "New rule" and saves', async () => {
    const { slice, saveRule } = fakeSlice([]);
    render(() => <RulesSettings accountId="acct1" slice={slice} />);
    fireEvent.click(screen.getByText(t('rules-new')));
    const name = await screen.findByLabelText(t('rules-name-label'));
    fireEvent.input(name, { target: { value: 'My rule' } });
    fireEvent.click(screen.getByText(t('rules-save')));
    await waitFor(() => expect(saveRule).toHaveBeenCalledTimes(1));
    expect(saveRule.mock.calls[0]![0].name).toBe('My rule');
  });

  it('surfaces lint diagnostics in the raw editor', async () => {
    const { slice } = fakeSlice([ruleFixture('1', 'r')]);
    const api: SieveApi = {
      connect: vi.fn(async () => ({ maxSizeScriptName: 512, maxSizeScript: 65536, maxNumberScripts: 5, maxNumberRedirects: 4, sieveExtensions: [], 'urn:mailwoman:policy': { rawEdit: true, forwardExternal: false, ownDomains: ['example.test'], runsAt: 'engine' as const } })),
      get: vi.fn(async () => ({ accountId: 'acct1', state: 's1', list: [], notFound: [] })),
      source: vi.fn(async () => 'keep;'), validate: vi.fn(async () => null), save: vi.fn(async () => undefined), activate: vi.fn(async () => undefined), remove: vi.fn(async () => undefined),
    };
    render(() => <RulesSettings accountId="acct1" slice={slice} scriptApi={api} />);
    fireEvent.click(screen.getByRole('tab', { name: t('rules-tab-raw') }));
    const editor = await screen.findByLabelText(t('rules-raw-label')) as HTMLTextAreaElement;
    // Break the Sieve: an unterminated string must surface a diagnostic.
    fireEvent.input(editor, { target: { value: 'fileinto "Spam;' } });
    expect(screen.getByText(/unterminated string/)).toBeInTheDocument();
  });

  it('deletes a rule', async () => {
    const { slice, deleteRule } = fakeSlice([ruleFixture('1', 'Doomed')]);
    const { getByLabelText } = render(() => <RulesSettings accountId="acct1" slice={slice} />);
    // The aria-label wraps the (untrusted) name in bidi isolates, so match loosely.
    fireEvent.click(getByLabelText((content) => content.includes('Doomed')));
    await waitFor(() => expect(deleteRule).toHaveBeenCalledWith('1'));
  });
});

describe('RuleBuilder', () => {
  it('adds a condition and reflects the where-it-runs indicator', () => {
    const onSave = vi.fn();
    const { getAllByLabelText, getByText } = render(() => (
      <RuleBuilder initial={emptyRuleDraft()} onSave={onSave} onCancel={() => undefined} />
    ));
    fireEvent.click(getByText(t('rules-add-condition')));
    // Two condition rows now.
    expect(getAllByLabelText(t('rules-cond-value-label')).length).toBe(2);
    // A move action keeps the rule server-expressible.
    expect(getByText(t('rules-runs-server'))).toBeInTheDocument();
  });

  it('requires a name before saving', () => {
    const onSave = vi.fn();
    const { getByText } = render(() => (
      <RuleBuilder initial={emptyRuleDraft()} onSave={onSave} onCancel={() => undefined} />
    ));
    fireEvent.click(getByText(t('rules-save')));
    expect(onSave).not.toHaveBeenCalled();
  });
});

describe('RuleBuilder — thread conditions (t32-h2)', () => {
  it('does not offer thread as a condition field', () => {
    const { getByLabelText } = render(() => (
      <RuleBuilder initial={emptyRuleDraft()} onSave={vi.fn()} onCancel={() => undefined} />
    ));
    const field = getByLabelText(t('rules-cond-type-label')) as HTMLSelectElement;
    const values = Array.from(field.options).map((o) => o.value);
    expect(values).toEqual(['from', 'to', 'subject']);
  });

  it('shows a stored thread condition read-only with a note and blocks saving', () => {
    const onSave = vi.fn();
    const initial = {
      ...emptyRuleDraft(),
      name: 'old thread rule',
      conditions: [
        { type: 'thread' as const, op: 'is' as const, value: 'T1' },
        { type: 'from' as const, op: 'contains' as const, value: 'alice' },
      ],
    };
    const { getAllByLabelText, getByText, getAllByRole } = render(() => (
      <RuleBuilder initial={initial} onSave={onSave} onCancel={() => undefined} />
    ));
    // Only the from row is editable.
    expect(getAllByLabelText(t('rules-cond-type-label')).length).toBe(1);
    expect(getByText(/thread conditions are not supported/)).toBeInTheDocument();
    const save = getByText(t('rules-save')) as HTMLButtonElement;
    expect(save.disabled).toBe(true);
    fireEvent.click(save);
    expect(onSave).not.toHaveBeenCalled();

    // Removing the thread row makes the rule savable again.
    fireEvent.click(getAllByRole('button', { name: t('rules-remove-condition') })[0]!);
    expect(save.disabled).toBe(false);
  });

  it('re-enables saving after the save is refused', async () => {
    const onSave = vi.fn(async () => {
      throw new Error('refused');
    });
    const { getByText } = render(() => (
      <RuleBuilder initial={{ ...emptyRuleDraft(), name: 'n' }} onSave={onSave} onCancel={() => undefined} />
    ));
    const save = getByText(t('rules-save')) as HTMLButtonElement;
    fireEvent.click(save);
    await waitFor(() => expect(onSave).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(save.disabled).toBe(false));
  });
});

describe('RawEditor', () => {
  it('reports a clean script with no diagnostics', () => {
    render(() => <RawEditor source={'require ["fileinto"];\nfileinto "A";'} readOnly />);
    expect(screen.getByText(t('rules-lint-clean'))).toBeInTheDocument();
  });
});

// ── t32-h6: an upstream without MailRule/get ────────────────────────────────────
//
// In proxy mode `MailRule/get` goes to the upstream JMAP server, and one that does
// not implement this Mailwoman extension answers `unknownMethod`. The section used
// to drop that rejection on the floor (an unhandled rejection, then "No rules yet"
// and a builder whose saves could never succeed).

describe('RulesSettings — contained load failures (t32-h6)', () => {
  function failingSlice(first: Error): { slice: RulesSlice; loadRules: ReturnType<typeof vi.fn> } {
    const { slice } = fakeSlice([]);
    const loadRules = vi.fn().mockRejectedValueOnce(first).mockResolvedValue(undefined);
    return { slice: { ...slice, loadRules }, loadRules };
  }

  it('unknownMethod reads as "not available" and hides the builder', async () => {
    const { slice } = failingSlice(new Error('JMAP method error: unknownMethod mock does not implement MailRule/get'));
    render(() => <RulesSettings accountId="acct1" slice={slice} />);

    expect(await screen.findByText(t('rules-unavailable'))).toBeInTheDocument();
    expect(screen.getByText(t('rules-title'))).toBeInTheDocument();
    expect(screen.queryByText(t('rules-new'))).not.toBeInTheDocument();
    expect(screen.queryByText(t('rules-empty'))).not.toBeInTheDocument();
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });

  it('any other failure is shown in the section with a retry that reloads it', async () => {
    const { slice, loadRules } = failingSlice(new Error('JMAP method error: serverFail'));
    render(() => <RulesSettings accountId="acct1" slice={slice} />);

    expect(await screen.findByRole('alert')).toHaveTextContent(t('rules-load-failed'));
    expect(screen.queryByText(t('rules-new'))).not.toBeInTheDocument();

    fireEvent.click(screen.getByTestId('rules-retry'));
    await waitFor(() => expect(screen.getByText(t('rules-new'))).toBeInTheDocument());
    expect(loadRules).toHaveBeenCalledTimes(2);
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });
});
