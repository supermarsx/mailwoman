// t39-e6: what a user sees of the administrator passkey policy — the rules,
// an older passkey marked (grandfathered or refused), a named refusal of a
// registration, a browser that found no qualifying authenticator, and the
// last-factor rule (t77: the last passkey or authenticator app is kept, and its
// remove button says why).
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@solidjs/testing-library';
import { keepsLastFactor, TwoFactor } from './TwoFactor.tsx';
import { TwoFactorChallenge } from './TwoFactorChallenge.tsx';
import { SettingsError, settingsErrorText, type SettingsService } from './service.ts';
import type { PasskeyPolicySummary, TwofaStatus } from './types.ts';
import { assertPasskey, registerPasskey, registrationSelection } from './webauthn.ts';

vi.mock('./webauthn.ts', async (orig) => {
  const real = await orig<typeof import('./webauthn.ts')>();
  return { ...real, passkeySupported: () => true, registerPasskey: vi.fn(), assertPasskey: vi.fn() };
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
    // Under `allow` the button is live; a server that still refuses (the
    // policy changed meanwhile) is named.
    render(() => <TwoFactor service={service(status(), { passkeyRemove: async () => { throw refusal(409, 'lastFactorBlocked'); } } as Partial<SettingsService>)} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Remove' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Your organization does not allow removing your last second factor. Add another sign-in method first.');
    expect(settingsErrorText(409, 'lastFactorRequired', null)).toBe('Your organization requires two-factor authentication. Add another sign-in method before removing this one.');
  });

  it('keeps the last passkey under block and says why (t77)', async () => {
    const passkeyRemove = vi.fn(async () => undefined);
    render(() => <TwoFactor service={service(status({ passkeyPolicy: { ...STRICT, lastFactorRemoval: 'block' } }), { passkeyRemove } as Partial<SettingsService>)} />);
    const button = await screen.findByRole('button', { name: 'Remove' });
    expect(button).toHaveAttribute('aria-disabled', 'true');
    expect(button).not.toBeDisabled(); // still focusable and hoverable for the tooltip
    expect(button).toHaveAttribute('data-tooltip', 'Add another sign-in method first. Your organization requires at least one passkey or authenticator app on your account.');
    const hint = screen.getByTestId('last-factor-hint');
    expect(hint).toHaveTextContent('Add another sign-in method first.');
    expect(button.getAttribute('aria-describedby')).toContain(hint.id);
    fireEvent.click(button);
    expect(passkeyRemove).not.toHaveBeenCalled();
  });

  it('keeps the only authenticator app when two-factor sign-in is required (t77)', async () => {
    const totpDisable = vi.fn(async () => undefined);
    render(() => <TwoFactor service={service(status({ totp: true, passkeys: [], policyRequired: true }), { totpDisable } as Partial<SettingsService>)} />);
    const button = await screen.findByRole('button', { name: 'Remove authenticator' });
    expect(button).toHaveAttribute('aria-disabled', 'true');
    fireEvent.click(button);
    expect(totpDisable).not.toHaveBeenCalled();
    expect(screen.getByTestId('last-factor-hint')).toHaveTextContent('Add another sign-in method first.');
  });

  it('lets a factor go under block while another one remains (t77)', async () => {
    const passkeyRemove = vi.fn(async () => undefined);
    render(() => <TwoFactor service={service(status({ totp: true, passkeyPolicy: { ...STRICT, lastFactorRemoval: 'block' } }), { passkeyRemove } as Partial<SettingsService>)} />);
    const button = await screen.findByRole('button', { name: 'Remove' });
    expect(button).not.toHaveAttribute('aria-disabled');
    expect(button).not.toHaveAttribute('data-tooltip');
    expect(screen.queryByTestId('last-factor-hint')).toBeNull();
    fireEvent.click(button);
    await vi.waitFor(() => expect(passkeyRemove).toHaveBeenCalledWith('old'));
  });

  it('keepsLastFactor counts passkeys and the authenticator app, not recovery codes', () => {
    const block = { ...STRICT, lastFactorRemoval: 'block' as const };
    expect(keepsLastFactor(undefined)).toBe(false);
    expect(keepsLastFactor(status())).toBe(false); // allow
    expect(keepsLastFactor(status({ passkeyPolicy: block }))).toBe(true);
    expect(keepsLastFactor(status({ passkeyPolicy: block, recoveryRemaining: 10 }))).toBe(true);
    expect(keepsLastFactor(status({ passkeyPolicy: block, totp: true }))).toBe(false);
    expect(keepsLastFactor(status({ passkeyPolicy: block, passkeys: [] }))).toBe(false); // nothing to remove
    expect(keepsLastFactor(status({ policyRequired: true }))).toBe(true);
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

describe('passkey policy at sign-in', () => {
  const challenge = { pendingToken: 'p', factors: ['webauthn', 'recovery'], webauthn: { challenge: 'c', credentialIds: ['id'], rpId: 'r', userVerification: 'required' } };
  beforeEach(() => { vi.mocked(assertPasskey).mockReset(); });

  it('names the policy when the server refuses a verified assertion', async () => {
    vi.mocked(assertPasskey).mockResolvedValue({ credentialId: 'id', clientDataJson: 'c', authenticatorData: 'a', signature: 's', userHandle: null });
    const verifyLoginFactor = vi.fn(async () => { throw refusal(403, 'passkeyPolicyLegacy'); });
    render(() => <TwoFactorChallenge challenge={challenge} onSuccess={() => {}} service={{ verifyLoginFactor } as unknown as SettingsService} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Use a passkey' }));
    expect(await screen.findByRole('alert')).toHaveTextContent("added before your organization's current passkey policy");
  });

  it('explains a browser refusal instead of showing the browser text', async () => {
    vi.mocked(assertPasskey).mockImplementation(async () => { const e = new Error('The operation either timed out or was not allowed.'); e.name = 'NotAllowedError'; throw e; });
    render(() => <TwoFactorChallenge challenge={challenge} onSuccess={() => {}} service={{} as unknown as SettingsService} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Use a passkey' }));
    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent('such as a PIN, fingerprint or face check');
    expect(alert).not.toHaveTextContent('timed out or was not allowed.');
  });
});
