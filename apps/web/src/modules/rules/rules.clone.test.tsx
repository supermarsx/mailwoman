// Regression (t43-e1): the builder edits a Solid store, and saving passed the
// store's proxies to the slice, which hands them to the network worker through
// postMessage ("could not be cloned"). The other rules tests mock the slice with
// a vi.fn, which never structured-clones its argument, so they could not see it.
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { RuleBuilder } from './RuleBuilder.tsx';
import { emptyRuleDraft, type RuleDraft } from '../../state/slices/rules.ts';
import { t } from '../../test/i18n.ts';

describe('RuleBuilder save payload', () => {
  it('is plain, structured-cloneable data after the draft was edited in the store', async () => {
    const onSave = vi.fn(async (_draft: RuleDraft) => undefined);
    render(() => <RuleBuilder initial={emptyRuleDraft()} onSave={onSave} onCancel={() => undefined} />);
    fireEvent.input(await screen.findByLabelText(t('rules-name-label')), { target: { value: 'My rule' } });
    fireEvent.input(screen.getByLabelText(t('rules-cond-value-label')), { target: { value: 'a@b' } });
    fireEvent.click(screen.getByText(t('rules-add-condition')));
    fireEvent.click(screen.getByText(t('rules-add-action')));
    fireEvent.click(screen.getByText(t('rules-save')));
    await waitFor(() => expect(onSave).toHaveBeenCalledTimes(1));
    const saved = onSave.mock.calls[0]![0];
    const cloned = structuredClone(saved); // throws DataCloneError on a Proxy
    expect(cloned).toEqual(saved);
    expect(cloned.conditions).toHaveLength(2);
    expect(cloned.conditions[0]).toEqual({ type: 'from', op: 'contains', value: 'a@b' });
    expect(cloned.actions).toHaveLength(2);
  });

  it('does not mutate the rule it was opened on', async () => {
    const initial: RuleDraft = { ...emptyRuleDraft(), id: 'r1', name: 'Orig' };
    render(() => <RuleBuilder initial={initial} onSave={async () => undefined} onCancel={() => undefined} />);
    fireEvent.input(await screen.findByLabelText(t('rules-cond-value-label')), { target: { value: 'changed' } });
    expect(initial.conditions[0]!.value).toBe('');
  });
});
