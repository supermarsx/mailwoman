import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { RulesSettings } from './RulesSettings.tsx';
import { RuleBuilder } from './RuleBuilder.tsx';
import { RawEditor } from './RawEditor.tsx';
import { emptyRuleDraft, type RulesSlice } from '../../state/slices/rules.ts';
import type { MailRule } from '../../api/crypto-types.ts';
import { t } from '../../test/i18n.ts';

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

  it('surfaces lint diagnostics in the raw editor', () => {
    const { slice } = fakeSlice([ruleFixture('1', 'r')]);
    render(() => <RulesSettings accountId="acct1" slice={slice} />);
    fireEvent.click(screen.getByRole('tab', { name: t('rules-tab-raw') }));
    const editor = screen.getByLabelText(t('rules-raw-label')) as HTMLTextAreaElement;
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

describe('RawEditor', () => {
  it('reports a clean script with no diagnostics', () => {
    render(() => <RawEditor source={'require ["fileinto"];\nfileinto "A";'} readOnly />);
    expect(screen.getByText(t('rules-lint-clean'))).toBeInTheDocument();
  });
});
