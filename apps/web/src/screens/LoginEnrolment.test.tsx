import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen } from '@solidjs/testing-library';
import { ApiError, NetworkError } from '../api/errors.ts';
import type { LoginEnrolmentApi } from '../api/loginEnrolment.ts';
import type { LoginChallenge } from '../api/client.ts';

const webauthn = vi.hoisted(() => ({ supported: false, registerPasskey: vi.fn() }));
vi.mock('./Settings/webauthn.ts', () => ({
  passkeySupported: () => webauthn.supported,
  registerPasskey: webauthn.registerPasskey,
}));

const { LoginEnrolment, enrolMethods, PENDING_TTL_SECS } = await import('./LoginEnrolment.tsx');

const SETUP = { secret: 'JBSWY3DPEHPK3PXP', otpauthUri: 'otpauth://totp/Mailwoman:u%40example.org?secret=JBSWY3DPEHPK3PXP&issuer=Mailwoman' };
const CODES = ['aaaa-bbbb', 'cccc-dddd'];

function fakeApi(overrides: Partial<LoginEnrolmentApi> = {}): LoginEnrolmentApi {
  return {
    totpBegin: vi.fn(async () => SETUP),
    totpConfirm: vi.fn(async () => ({ recoveryCodes: CODES })),
    passkeyBegin: vi.fn(async () => ({ challenge: 'Y2g', rpId: 'example.org', userHandle: 'dQ', userName: 'u@example.org', userVerification: 'preferred' })),
    passkeyFinish: vi.fn(async () => ({ recoveryCodes: CODES })),
    ...overrides,
  };
}

function setup(opts: { api?: LoginEnrolmentApi; challenge?: Partial<LoginChallenge>; allowPasskey?: boolean; now?: () => number } = {}) {
  const api = opts.api ?? fakeApi();
  const onComplete = vi.fn(async () => undefined);
  const onExpired = vi.fn();
  const onBack = vi.fn();
  const challenge: LoginChallenge = { pendingToken: 'tok', factors: ['totp'], enrollmentRequired: true, ...opts.challenge };
  render(() => (
    <LoginEnrolment
      challenge={challenge}
      allowPasskey={opts.allowPasskey ?? true}
      onComplete={onComplete}
      onExpired={onExpired}
      onBack={onBack}
      api={api}
      {...(opts.now ? { now: opts.now } : {})}
    />
  ));
  return { api, onComplete, onExpired, onBack };
}

async function reachSetup(): Promise<void> {
  fireEvent.click(await screen.findByTestId('enrol-totp'));
  await screen.findByTestId('enrol-totp-setup');
}

function enterCode(value: string): void {
  fireEvent.input(screen.getByLabelText('Authenticator code'), { target: { value } });
  fireEvent.click(screen.getByTestId('enrol-totp-confirm'));
}

beforeEach(() => {
  webauthn.supported = false;
  webauthn.registerPasskey.mockReset();
});

describe('LoginEnrolment › authenticator app', () => {
  it('begins, shows the key and QR, confirms, and opens the app only after the codes are acknowledged', async () => {
    const { api, onComplete, onExpired } = setup();
    expect(screen.getByRole('heading', { name: 'Set up two-step verification to continue' })).toBeInTheDocument();
    await reachSetup();
    expect(api.totpBegin).toHaveBeenCalledWith('tok');
    expect(screen.getByTestId('enrol-totp-secret')).toHaveTextContent(SETUP.secret);
    expect(screen.getByTestId('enrol-totp-qr').getAttribute('src')).toMatch(/^data:image\/svg\+xml,/);
    expect(screen.getByRole('img', { name: 'QR code for adding this account to an authenticator app' })).toBeInTheDocument();

    enterCode(' 123 456 ');
    const panel = await screen.findByTestId('enrol-recovery-codes');
    expect(api.totpConfirm).toHaveBeenCalledWith('tok', '123456');
    for (const c of CODES) expect(panel).toHaveTextContent(c);
    // The session exists now; going back would strand it, so the way back is gone.
    expect(screen.queryByRole('button', { name: 'Back to sign in' })).toBeNull();

    const next = screen.getByTestId('enrol-continue');
    expect(next).toBeDisabled();
    fireEvent.click(next);
    expect(onComplete).not.toHaveBeenCalled();
    fireEvent.click(screen.getByTestId('enrol-recovery-ack'));
    expect(next).not.toBeDisabled();
    fireEvent.click(next);
    await vi.waitFor(() => expect(onComplete).toHaveBeenCalledTimes(1));
    expect(onExpired).not.toHaveBeenCalled();
  });

  it('opens the app at once when the account already had recovery codes (none returned)', async () => {
    const { onComplete } = setup({ api: fakeApi({ totpConfirm: vi.fn(async () => ({ recoveryCodes: [] })) }) });
    await reachSetup();
    enterCode('123456');
    await vi.waitFor(() => expect(onComplete).toHaveBeenCalledTimes(1));
    expect(screen.queryByTestId('enrol-recovery-codes')).toBeNull();
  });

  it('keeps the same key after a wrong code and lets the user try again', async () => {
    const totpConfirm = vi.fn()
      .mockRejectedValueOnce(new ApiError(401, 'second factor required'))
      .mockResolvedValueOnce({ recoveryCodes: CODES });
    const { api, onExpired } = setup({ api: fakeApi({ totpConfirm }) });
    await reachSetup();
    enterCode('000000');
    expect(await screen.findByRole('alert')).toHaveTextContent('That code was not accepted.');
    expect(onExpired).not.toHaveBeenCalled();
    expect(api.totpBegin).toHaveBeenCalledTimes(1);
    expect(screen.getByTestId('enrol-totp-setup')).toBeInTheDocument();
    enterCode('123456');
    await screen.findByTestId('enrol-recovery-codes');
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('returns to sign-in when the pending sign-in has timed out', async () => {
    let clock = 1_000_000;
    const { onExpired } = setup({
      api: fakeApi({ totpConfirm: vi.fn(async () => { throw new ApiError(401, 'second factor required'); }) }),
      now: () => clock,
    });
    await reachSetup();
    clock += PENDING_TTL_SECS * 1000;
    enterCode('123456');
    await vi.waitFor(() => expect(onExpired).toHaveBeenCalledTimes(1));
  });

  it('returns to sign-in when begin is refused (the pending sign-in is gone)', async () => {
    const { onExpired } = setup({ api: fakeApi({ totpBegin: vi.fn(async () => { throw new ApiError(401, 'second factor required'); }) }) });
    fireEvent.click(await screen.findByTestId('enrol-totp'));
    await vi.waitFor(() => expect(onExpired).toHaveBeenCalledTimes(1));
  });

  it('says so when the sign-in source is rate limited', async () => {
    const { onExpired } = setup({ api: fakeApi({ totpConfirm: vi.fn(async () => { throw new ApiError(429, 'locked', { code: 'signinLocked', retryAfterSecs: 60 }); }) }) });
    await reachSetup();
    enterCode('123456');
    expect(await screen.findByRole('alert')).toHaveTextContent('Too many sign-in attempts');
    expect(onExpired).not.toHaveBeenCalled();
  });

  it('reports a network failure and recovers on retry', async () => {
    const totpBegin = vi.fn()
      .mockRejectedValueOnce(new NetworkError('offline'))
      .mockResolvedValueOnce(SETUP);
    const { onExpired } = setup({ api: fakeApi({ totpBegin }) });
    fireEvent.click(await screen.findByTestId('enrol-totp'));
    expect(await screen.findByRole('alert')).toHaveTextContent('Could not reach the server');
    expect(onExpired).not.toHaveBeenCalled();
    fireEvent.click(screen.getByTestId('enrol-totp'));
    await screen.findByTestId('enrol-totp-setup');
  });

  it('shows a generic error for a server failure without leaving the step', async () => {
    setup({ api: fakeApi({ totpConfirm: vi.fn(async () => { throw new ApiError(500, 'server error'); }) }) });
    await reachSetup();
    enterCode('123456');
    expect(await screen.findByRole('alert')).toHaveTextContent('That did not work. Try again.');
    expect(screen.getByTestId('enrol-totp-setup')).toBeInTheDocument();
  });

  it('goes back to the credential form from the step', async () => {
    const { onBack } = setup();
    await reachSetup();
    fireEvent.click(screen.getByRole('button', { name: 'Back to sign in' }));
    expect(onBack).toHaveBeenCalledTimes(1);
  });
});

describe('LoginEnrolment › passkey', () => {
  it('offers a passkey only when the server, the client and the browser allow it', () => {
    const offer: LoginChallenge = { pendingToken: 't', factors: ['totp', 'webauthn'], enrollmentRequired: true };
    expect(enrolMethods(offer, true)).toEqual(['totp']);
    webauthn.supported = true;
    expect(enrolMethods(offer, true)).toEqual(['totp', 'webauthn']);
    expect(enrolMethods(offer, false)).toEqual(['totp']);
    expect(enrolMethods({ ...offer, factors: ['totp'] }, true)).toEqual(['totp']);
  });

  it('registers a passkey over the pending token and shows the codes', async () => {
    webauthn.supported = true;
    webauthn.registerPasskey.mockResolvedValue({ clientDataJson: 'cd', attestationObject: 'ao', transports: 'usb' });
    const { api } = setup({ challenge: { factors: ['totp', 'webauthn'] } });
    fireEvent.click(await screen.findByTestId('enrol-passkey'));
    fireEvent.click(await screen.findByTestId('enrol-passkey-create'));
    await screen.findByTestId('enrol-recovery-codes');
    expect(api.passkeyBegin).toHaveBeenCalledWith('tok');
    expect(api.passkeyFinish).toHaveBeenCalledWith('tok', { clientDataJson: 'cd', attestationObject: 'ao', transports: 'usb', label: '' });
  });

  it('names the passkey rule the server refused, and offers the other method', async () => {
    webauthn.supported = true;
    webauthn.registerPasskey.mockResolvedValue({ clientDataJson: 'cd', attestationObject: 'ao', transports: '' });
    setup({
      challenge: { factors: ['totp', 'webauthn'] },
      api: fakeApi({ passkeyFinish: vi.fn(async () => { throw new ApiError(403, 'refused', { code: 'passkeyPolicyPlatform' }); }) }),
    });
    fireEvent.click(await screen.findByTestId('enrol-passkey'));
    fireEvent.click(await screen.findByTestId('enrol-passkey-create'));
    const alert = await screen.findByRole('alert');
    expect(alert.textContent).not.toBe('');
    expect(alert).not.toHaveTextContent('That did not work');
    fireEvent.click(screen.getByTestId('enrol-other-method'));
    expect(await screen.findByTestId('enrol-totp')).toBeInTheDocument();
  });

  it('treats a cancelled browser ceremony as a passkey failure, not as a lost sign-in', async () => {
    webauthn.supported = true;
    webauthn.registerPasskey.mockRejectedValue(Object.assign(new Error('cancelled'), { name: 'NotAllowedError' }));
    const { onExpired } = setup({ challenge: { factors: ['totp', 'webauthn'] } });
    fireEvent.click(await screen.findByTestId('enrol-passkey'));
    fireEvent.click(await screen.findByTestId('enrol-passkey-create'));
    expect(await screen.findByRole('alert')).toHaveTextContent('The passkey could not be registered.');
    expect(onExpired).not.toHaveBeenCalled();
  });

  it('returns to sign-in when the passkey begin is refused with 401', async () => {
    webauthn.supported = true;
    const { onExpired } = setup({
      challenge: { factors: ['totp', 'webauthn'] },
      api: fakeApi({ passkeyBegin: vi.fn(async () => { throw new ApiError(401, 'second factor required'); }) }),
    });
    fireEvent.click(await screen.findByTestId('enrol-passkey'));
    fireEvent.click(await screen.findByTestId('enrol-passkey-create'));
    await vi.waitFor(() => expect(onExpired).toHaveBeenCalledTimes(1));
  });
});
