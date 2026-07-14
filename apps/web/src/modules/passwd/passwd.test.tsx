import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { PasswordChange } from './PasswordChange.tsx';
import { PasswordService, policyViolations, type PasswordPolicy } from './service.ts';
import { recoveryPhraseBefore, rewrapUnderNewPassword } from './rewrap.ts';
import type { ZeroAccessCrypto } from '../zeroaccess/crypto.ts';
import type { ZeroAccessAccount } from '../zeroaccess/service.ts';

function okJson(body: unknown): Response {
  return new Response(JSON.stringify(body), { status: 200, headers: { 'content-type': 'application/json' } });
}

const POLICY: PasswordPolicy = {
  minLength: 8,
  requireUppercase: true,
  requireLowercase: true,
  requireDigit: true,
  requireSymbol: false,
  description: 'At least 8 characters with mixed case and a digit.',
  forceChange: false,
};

const ZA_ACCOUNT: ZeroAccessAccount = {
  enabled: true,
  saltB64: 'c2FsdA==',
  kdfParams: { mCost: 19456, tCost: 2, pCost: 1 },
  wrappedDataKeyB64: 'd3JhcHBlZA==',
  pairedDevices: [],
};

/** A mock crypto worker that records the operation order into `order`. */
function mockZa(order: string[]): ZeroAccessCrypto {
  let seq = 0;
  const ref = (): { keyRef: string } => ({ keyRef: `ref-${(seq += 1)}` });
  const za: Partial<ZeroAccessCrypto> = {
    deriveRootKey: vi.fn(async () => ref()),
    deriveKek: vi.fn(async () => ref()),
    unwrapKey: vi.fn(async () => ref()),
    wrapKey: vi.fn(async () => {
      order.push('wrap');
      return { blobB64: 'bmV3d3JhcA==' };
    }),
    recoveryPhrase: vi.fn(async () => {
      order.push('recovery');
      return { phrase: 'ocean marble tiger velvet ...' };
    }),
  };
  return za as ZeroAccessCrypto;
}

describe('password policy display + validation', () => {
  it('reports which policy rules a candidate fails', () => {
    expect(policyViolations(POLICY, 'short')).toContain('at least 8 characters');
    expect(policyViolations(POLICY, 'alllowercase1')).toContain('an uppercase letter');
    expect(policyViolations(POLICY, 'GoodPass1')).toEqual([]);
  });

  it('renders the policy summary from the backend', async () => {
    render(() => <PasswordChange accountId="a" initialPolicy={POLICY} />);
    await waitFor(() => expect(screen.getByTestId('policy')).toBeInTheDocument());
    expect(screen.getByTestId('policy')).toHaveTextContent('At least 8 characters');
  });

  it('shows the forced-change banner when the policy demands it', async () => {
    render(() => <PasswordChange accountId="a" initialPolicy={{ ...POLICY, forceChange: true }} />);
    await waitFor(() => expect(screen.getByTestId('force-change-banner')).toBeInTheDocument());
  });
});

describe('plain (non zero-access) change', () => {
  it('POSTs the change directly with no re-wrap material', async () => {
    const fetcher = vi.fn(async (_input: string, _init?: RequestInit) =>
      okJson({ changed: true, reencryptCredentials: false, zeroaccessRewrapRequired: false }),
    );
    render(() => <PasswordChange accountId="a" initialPolicy={POLICY} service={new PasswordService(fetcher)} />);

    fireEvent.input(screen.getByLabelText('Current password'), { target: { value: 'OldPass1' } });
    fireEvent.input(screen.getByLabelText('New password'), { target: { value: 'NewPass1' } });
    fireEvent.input(screen.getByLabelText('Confirm new password'), { target: { value: 'NewPass1' } });
    fireEvent.click(screen.getByRole('button', { name: 'Change password' }));

    await waitFor(() => expect(screen.getByTestId('change-done')).toBeInTheDocument());
    const body = JSON.parse((fetcher.mock.calls[0]?.[1]?.body as string) ?? '{}') as Record<string, unknown>;
    expect(body).not.toHaveProperty('rewrap');
  });
});

describe('zero-access re-wrap — recovery-phrase pre-prompt BEFORE the change (ordering)', () => {
  it('shows the recovery phrase and does NOT change until it is acknowledged', async () => {
    const order: string[] = [];
    const za = mockZa(order);
    const change = vi.fn(async (_input: string, _init?: RequestInit) => {
      order.push('change');
      return okJson({ changed: true, reencryptCredentials: true, zeroaccessRewrapRequired: true });
    });
    render(() => (
      <PasswordChange
        accountId="a"
        initialPolicy={POLICY}
        service={new PasswordService(change)}
        zeroAccess={{ account: ZA_ACCOUNT, za }}
      />
    ));

    fireEvent.input(screen.getByLabelText('Current password'), { target: { value: 'OldPass1' } });
    fireEvent.input(screen.getByLabelText('New password'), { target: { value: 'NewPass1' } });
    fireEvent.input(screen.getByLabelText('Confirm new password'), { target: { value: 'NewPass1' } });
    fireEvent.click(screen.getByRole('button', { name: 'Continue' }));

    // The recovery phrase is surfaced; the change has NOT been sent.
    await waitFor(() => expect(screen.getByTestId('recovery-prompt')).toBeInTheDocument());
    expect(screen.getByTestId('recovery-phrase')).toHaveTextContent('ocean marble tiger');
    expect(change).not.toHaveBeenCalled();
    expect(order).toEqual(['recovery']);

    // The change button is disabled until the phrase is acknowledged.
    const confirmBtn = screen.getByTestId('confirm-change') as HTMLButtonElement;
    expect(confirmBtn.disabled).toBe(true);

    fireEvent.click(screen.getByLabelText('I have saved my recovery phrase'));
    fireEvent.click(screen.getByTestId('confirm-change'));

    await waitFor(() => expect(screen.getByTestId('change-done')).toBeInTheDocument());

    // HARD ORDERING: the recovery phrase was derived BEFORE the change was applied,
    // and the re-wrap happened between them.
    expect(order.indexOf('recovery')).toBeGreaterThanOrEqual(0);
    expect(order.indexOf('change')).toBeGreaterThan(order.indexOf('recovery'));
    expect(order.indexOf('wrap')).toBeGreaterThan(order.indexOf('recovery'));
    expect(order.indexOf('change')).toBeGreaterThan(order.indexOf('wrap'));

    // The change carried re-wrapped material (never a plaintext key).
    const body = JSON.parse((change.mock.calls[0]?.[1]?.body as string) ?? '{}') as {
      rewrap?: { wrappedDataKeyB64: string; saltB64: string };
    };
    expect(body.rewrap?.wrappedDataKeyB64).toBe('bmV3d3JhcA==');
    expect(body.rewrap?.saltB64).toBeTruthy();
  });
});

describe('re-wrap helpers reuse the crypto worker (no JS crypto)', () => {
  it('derives the pre-change recovery phrase from the current root', async () => {
    const order: string[] = [];
    const za = mockZa(order);
    const phrase = await recoveryPhraseBefore(za, ZA_ACCOUNT, 'b2xk');
    expect(phrase).toContain('ocean');
    expect(za.deriveRootKey).toHaveBeenCalled();
    expect(za.recoveryPhrase).toHaveBeenCalled();
  });

  it('re-wraps the SAME data key under a fresh salt derived from the new secret', async () => {
    const order: string[] = [];
    const za = mockZa(order);
    const result = await rewrapUnderNewPassword({ za, account: ZA_ACCOUNT, oldSecretB64: 'b2xk', newSecretB64: 'bmV3' });
    expect(result.wrappedDataKeyB64).toBe('bmV3d3JhcA==');
    expect(result.saltB64).not.toBe(ZA_ACCOUNT.saltB64); // fresh salt
    expect(za.unwrapKey).toHaveBeenCalledTimes(1); // unwrap old
    expect(za.wrapKey).toHaveBeenCalledTimes(1); // wrap under new
  });
});
