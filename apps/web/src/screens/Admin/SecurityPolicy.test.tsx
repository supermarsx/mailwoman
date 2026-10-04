import { describe, it, expect } from 'vitest';
import { screen } from '@solidjs/testing-library';
import { SecurityPolicy } from './SecurityPolicy.tsx';
import { mockAdminApi, renderWithAdmin } from './testkit.tsx';

// Every control this screen used to carry stored a value nothing applied; all of
// them were removed (26.20, t28-e8). These tests fail if one comes back.
describe('Admin › Security policy', () => {
  it('has no form control of any kind', () => {
    const { container } = renderWithAdmin(() => <SecurityPolicy />);
    expect(container.querySelectorAll('input, textarea, select, button, form')).toHaveLength(0);
  });

  it('does not offer the removed settings by name', () => {
    renderWithAdmin(() => <SecurityPolicy />);
    for (const label of [
      'Minimum TLS',
      'Capture policy',
      'Argon2 memory cost (KiB)',
      'Argon2 time cost',
      'Argon2 parallelism',
      'DLP rules (JSON)',
      'Require 2FA',
      'Enforce maximum-security floor',
      'Save policy',
    ]) {
      expect(screen.queryByText(label)).toBeNull();
      expect(screen.queryByLabelText(label)).toBeNull();
    }
    expect(screen.queryByLabelText('Require two-factor authentication')).toBeNull();
  });

  it('neither reads nor writes the stored policy record', () => {
    const { api } = renderWithAdmin(() => <SecurityPolicy />, mockAdminApi());
    expect(api.getSecurityPolicy).not.toHaveBeenCalled();
    expect(api.setSecurityPolicy).not.toHaveBeenCalled();
  });

  it('says where two-factor and DLP are actually decided', () => {
    renderWithAdmin(() => <SecurityPolicy />);
    const region = screen.getByRole('region', { name: 'Security policy' });
    expect(region).toHaveTextContent('Require two-factor screen');
    expect(region).toHaveTextContent('MW_DLP_RULES');
    expect(region).toHaveTextContent('never applied');
  });
});
