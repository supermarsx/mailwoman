import { afterEach, describe, expect, it, vi } from 'vitest';
import { render, screen, fireEvent } from '@solidjs/testing-library';
import { SignatureGroup } from './SignatureGroup.tsx';
import type { SignatureController } from './signatureControl.ts';
import { _resetRegistry } from '../../i18n/registry.ts';
import { setupI18nForTests } from '../../test/i18n.ts';

vi.mock('./composeFeatures.ts', () => ({ useComposeFeature: () => () => true }));

const controller = {
  applied: () => null,
  busy: () => false,
  offer: () => null,
  position: () => 'below-quote',
  toggle: async () => true,
  remove: () => undefined,
  reposition: async () => undefined,
} as unknown as SignatureController;

afterEach(() => setupI18nForTests());

describe('SignatureGroup position labels', () => {
  it('shows translated labels, also when the catalog arrives after the menu rendered', () => {
    _resetRegistry();
    render(() => <SignatureGroup controller={controller} groups={() => []} disabled={() => false} onEdit={() => undefined} />);
    fireEvent.click(screen.getByTestId('signature-menu'));
    expect(screen.getByTestId('signature-position-end').textContent).toBe('sig-position-end');
    setupI18nForTests();
    expect(screen.getByTestId('signature-position-above-quote').textContent).toBe('Above the quoted message');
    expect(screen.getByTestId('signature-position-below-quote').textContent).toBe('Below the quoted message');
    expect(screen.getByTestId('signature-position-end').textContent).toBe('End of message');
  });
});
