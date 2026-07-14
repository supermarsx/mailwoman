import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen } from '@solidjs/testing-library';
import { SecurityPolicy } from './SecurityPolicy.tsx';
import { mockAdminApi, renderWithAdmin } from './testkit.tsx';

describe('Admin › Security policy', () => {
  it('loads the policy into the form', async () => {
    renderWithAdmin(
      () => <SecurityPolicy />,
      mockAdminApi({
        getSecurityPolicy: vi.fn(async () => ({
          minTls: '1.3',
          require2fa: true,
          argon2MCost: 65_536,
          argon2TCost: 3,
          argon2PCost: 2,
          dlpRulesJson: '[]',
          maxSecurityFloor: true,
          capturePolicy: 'metadata',
        })),
      }),
    );
    const tls = (await screen.findByText('Minimum TLS')).parentElement!.querySelector('input')!;
    expect((tls as HTMLInputElement).value).toBe('1.3');
    expect((screen.getByLabelText('Require two-factor authentication') as HTMLInputElement).checked).toBe(true);
  });

  it('saves an edited policy', async () => {
    const setSecurityPolicy = vi.fn(async () => undefined);
    renderWithAdmin(() => <SecurityPolicy />, mockAdminApi({ setSecurityPolicy }));
    fireEvent.click(await screen.findByLabelText('Enforce maximum-security floor'));
    fireEvent.click(screen.getByRole('button', { name: 'Save policy' }));
    await Promise.resolve();
    expect(setSecurityPolicy).toHaveBeenCalledWith(expect.objectContaining({ maxSecurityFloor: true }));
    expect(await screen.findByRole('status')).toHaveTextContent('Saved.');
  });
});
