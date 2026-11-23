import { expect, it, vi } from 'vitest';
import { render, screen } from '@solidjs/testing-library';
import { SaveContactsSettings } from './SaveContactsSettings.tsx';

vi.mock('../../state/context.ts', () => ({ useApp: () => ({}) }));
vi.mock('./useSetting.ts', () => ({ useSetting: (_key: string, fallback: unknown) => ({
  value: () => fallback, entry: () => ({ value: fallback }), locked: () => false,
  saving: () => false, error: () => '', reset: async () => true, set: async () => true,
}) }));

it('labels every "ticked first" option, including Sender and Reply-To', () => {
  render(() => <SaveContactsSettings />);
  const select = screen.getByLabelText('Ticked first in the dialog') as HTMLSelectElement;
  expect([...select.options].map((o) => o.textContent)).toEqual(['Sender only', 'Sender and Reply-To', 'Everyone on the message']);
});
