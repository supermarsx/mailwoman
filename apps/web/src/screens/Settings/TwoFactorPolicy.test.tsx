// t39-e6: what a user sees of the administrator passkey policy — the rules,
// an older passkey marked (grandfathered or refused), a named refusal of a
// registration, a browser that found no qualifying authenticator, and the
// last-factor rule.
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@solidjs/testing-library';
import { TwoFactor } from './TwoFactor.tsx';
import { SettingsError, settingsErrorText, type SettingsService } from './service.ts';
import type { PasskeyPolicySummary, TwofaStatus } from './types.ts';
import { registerPasskey, registrationSelection } from './webauthn.ts';

vi.mock('./webauthn.ts', async (orig) => {
  const real = await orig<typeof import('./webauthn.ts')>();
  return { ...real, passkeySupported: () => true, registerPasskey: vi.fn() };
});

const STRICT: PasskeyPolicySummary = { attachment: 'platform', userVerification: 'required', residentKey: 'required', legacy: 'allow', lastFactorRemoval: 'allow' };

function status(over: Partial<TwofaStatus> = {}): TwofaStatus {
  return {
    totp: false,
    passkeys: [{ handle: 'old', label: 'Old USB key', createdAt: '2026-01-02', meetsPolicy: false, policyGaps: ['passkeyPolicyPlatform'] }],
    recoveryRemaining: 4,
    policyRequired: false,
    passkeysEnabled: true,
    passkeyPolicy: STRICT,
    ...over,
  };
}

function service(st: TwofaStatus, extra: Partial<SettingsService> = {}): SettingsService {
  return { twofaStatus: async () => st, ...extra } as unknown as SettingsService;
}

function refusal(status: number, code: string): SettingsError {
  return new SettingsError(status, settingsErrorText(status, code, null), { code });
}

describe('passkey policy in Settings', () => {
  beforeEach(() => { vi.mocked(registerPasskey).mockReset(); });

  it('lists the rules the administrator set', async () => {
    render(() => <TwoFactor service={service(status())} />);
    const rules = await screen.findByTestId('passkey-policy');
    expect(rules).toHaveTextContent('Your organization accepts only passkeys that:');
    expect(rules).toHaveTextContent('are stored on this device');
    expect(rules).toHaveTextContent('check your PIN, fingerprint or face');
    expect(rules).toHaveTextContent('are stored on the authenticator');
  });

  it('says nothing about rules under the default policy', async () => {
    render(() => <TwoFactor service={service(status({ passkeyPolicy: { ...STRICT, attachment: 'any', userVerification: 'preferred', residentKey: 'discouraged' }, passkeys: [] }))} />);
    await screen.findByRole('button', { name: 'Add a passkey' });
    expect(screen.queryByTestId('passkey-policy')).toBeNull();
  });

  it('marks a grandfathered passkey as still working', async () => {
    render(() => <TwoFactor service={service(status())} />);
    expect(await screen.findByTestId('passkey-policy-gap')).toHaveTextContent('It still works; add a new passkey and remove this one.');
  });

  it('marks a refused older passkey as no longer accepted', async () => {
    render(() => <TwoFactor service={service(status({ passkeyPolicy: { ...STRICT, legacy: 'refuse' } }))} />);
    expect(await screen.findByTestId('passkey-policy-gap')).toHaveTextContent('no longer accepted for sign-in');
  });

  it('names the rule when the server refuses a registration', async () => {
    vi.mocked(registerPasskey).mockResolvedValue({ clientDataJson: 'c', attestationObject: 'a', transports: 'usb', authenticatorAttachment: 'cross-platform' });
    const passkeyFinish = vi.fn(async () => { throw refusal(403, 'passkeyPolicyPlatform'); });
    render(() => <TwoFactor service={service(status(), { passkeyBegin: async () => ({ challenge: 'x', rpId: 'r', userHandle: 'u', userName: 'n', userVerification: 'required', authenticatorAttachment: 'platform' }), passkeyFinish } as Partial<SettingsService>)} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Add a passkey' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Your organization only accepts passkeys stored on this device');
    expect(passkeyFinish).toHaveBeenCalledWith(expect.objectContaining({ authenticatorAttachment: 'cross-platform', transports: 'usb' }));
  });

  it('a user-verification refusal says what is missing', async () => {
    vi.mocked(registerPasskey).mockResolvedValue({ clientDataJson: 'c', attestationObject: 'a', transports: 'internal', authenticatorAttachment: 'platform', residentKey: true });
    render(() => <TwoFactor service={service(status(), { passkeyBegin: async () => ({ challenge: 'x', rpId: 'r', userHandle: 'u', userName: 'n', userVerification: 'required' }), passkeyFinish: async () => { throw refusal(403, 'passkeyPolicyUserVerification'); } } as Partial<SettingsService>)} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Add a passkey' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('checks your PIN, fingerprint or face');
  });

  it('explains a browser that found no qualifying authenticator', async () => {
    vi.mocked(registerPasskey).mockImplementation(async () => { const e = new Error('not allowed'); e.name = 'NotAllowedError'; throw e; });
    render(() => <TwoFactor service={service(status(), { passkeyBegin: async () => ({ challenge: 'x', rpId: 'r', userHandle: 'u', userName: 'n', userVerification: 'required' }) } as Partial<SettingsService>)} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Add a passkey' }));
    expect(await screen.findByRole('alert')).toHaveTextContent("Your browser found no authenticator that meets your organization's passkey policy");
  });

  it('shows the last-factor refusals by name', async () => {
    render(() => <TwoFactor service={service(status(), { passkeyRemove: async () => { throw refusal(409, 'lastFactorNeedsRecoveryCodes'); } } as Partial<SettingsService>)} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Remove' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Generate recovery codes before you remove your last second factor.');
    expect(settingsErrorText(409, 'lastFactorBlocked', null)).toBe('Your organization does not allow removing your last second factor. Add another one first.');
  });

  it('asks the browser for what the policy requires', () => {
    expect(registrationSelection({ userVerification: 'required', residentKey: 'required', authenticatorAttachment: 'platform' }))
      .toEqual({ userVerification: 'required', residentKey: 'required', requireResidentKey: true, authenticatorAttachment: 'platform' });
    expect(registrationSelection({ userVerification: 'preferred' }))
      .toEqual({ userVerification: 'preferred', residentKey: 'discouraged', requireResidentKey: false });
    expect(registrationSelection({ userVerification: 'discouraged', authenticatorAttachment: 'bogus' }))
      .toEqual({ userVerification: 'discouraged', residentKey: 'discouraged', requireResidentKey: false });
  });
});
