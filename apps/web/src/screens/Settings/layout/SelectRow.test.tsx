import { expect, it } from 'vitest';
import { createSignal } from 'solid-js';
import { render, screen } from '@solidjs/testing-library';
import { SelectRow } from './rows.tsx';
import { isCheckPolicy, CHECK_POLICIES, type CheckPolicy } from '../../../components/compose/sendChecks.ts';

const handle = (initial: CheckPolicy) => {
  const [value, setValue] = createSignal<CheckPolicy>(initial);
  return { value, entry: () => ({ value: initial }), locked: () => false, saving: () => false, error: () => '', reset: async () => true,
    set: async (next: CheckPolicy) => { setValue(next); return true; } } as never;
};

// The labels come from a catalog that can finish loading after the first render; the
// option list is then rebuilt, and the select must keep showing the stored value.
it.each(CHECK_POLICIES)('shows the stored option %s, also after the labels are rebuilt', async (policy) => {
  const [suffix, setSuffix] = createSignal('');
  render(() => <SelectRow id="x" label="Check" handle={handle(policy)} accept={isCheckPolicy}
    options={CHECK_POLICIES.map((value) => ({ value, label: `label-${value}${suffix()}` }))} />);
  const select = screen.getByLabelText('Check') as HTMLSelectElement;
  expect(select.value).toBe(policy);
  setSuffix('-loaded');
  await Promise.resolve();
  expect(select.value).toBe(policy);
  expect(select.selectedOptions[0]?.textContent).toBe(`label-${policy}-loaded`);
});
