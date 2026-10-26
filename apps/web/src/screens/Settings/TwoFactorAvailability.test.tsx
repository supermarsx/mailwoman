import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@solidjs/testing-library';
import { TwoFactor } from './TwoFactor.tsx';
import type { SettingsService } from './service.ts';
vi.mock('./webauthn.ts', () => ({ passkeySupported: () => true, registerPasskey: vi.fn() }));
function service(passkeysEnabled?: boolean): SettingsService {
  return { twofaStatus: async () => ({ totp: false, passkeys: [{handle:'owned',label:'Existing key',createdAt:'2026-10-26'}], recoveryRemaining: 1, policyRequired: false, ...(passkeysEnabled === undefined ? {} : {passkeysEnabled}) }) } as unknown as SettingsService;
}
describe('effective passkey availability', () => {
  it('offers self-service enrollment when the effective policy enables it', async () => {
    render(() => <TwoFactor service={service(true)}/>);
    expect(await screen.findByRole('button',{name:'Add a passkey'})).toBeEnabled();
  });
  it('explains explicit policy denial while retaining management of enrolled credentials', async () => {
    render(() => <TwoFactor service={service(false)}/>);
    expect(await screen.findByText(/administrator has disabled passkey/)).toBeVisible();
    expect(screen.queryByRole('button',{name:'Add a passkey'})).toBeNull();
    expect(screen.getByText('Existing key')).toBeVisible();
    expect(screen.getByRole('button',{name:'Remove'})).toBeEnabled();
  });
  it('preserves enrollment compatibility with older servers using the enabled default', async () => {
    render(() => <TwoFactor service={service()}/>);
    expect(await screen.findByRole('button',{name:'Add a passkey'})).toBeEnabled();
  });
});
