import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { FluentBundle, FluentResource } from '@fluent/bundle';
import arSettings from '../../../locales/ar/settings.ftl?raw';

import { TwoFactor } from './TwoFactor.tsx';
import { TwoFactorChallenge } from './TwoFactorChallenge.tsx';
import { Sessions } from './Sessions.tsx';
import { Signatures } from './Signatures.tsx';
import { Identities } from './Identities.tsx';
import { Notifications } from './Notifications.tsx';
import { SavedSearches } from './SavedSearches.tsx';
import { Preferences } from './Preferences.tsx';
import { AccountSettings } from './AccountSettings.tsx';
import { SettingsService, SettingsError, type Fetcher } from './service.ts';
import { DEFAULT_PREFS, loadPrefs, savePrefs, type SettingsPrefs } from './prefs.ts';
import { registerPasskey, assertPasskey } from './webauthn.ts';
import type { SettingsService as SS } from './service.ts';
import type { Identity, NotificationConfig, SavedSearch, Signature } from './types.ts';
import { createSignal } from 'solid-js';

/** Build a partial `SettingsService` for component injection. */
function fakeService(overrides: Partial<Record<keyof SS, unknown>>): SettingsService {
  return overrides as unknown as SettingsService;
}

function okJson(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
}

// ── 2FA management (S1) ────────────────────────────────────────────────────────

describe('TwoFactor — enrolment + recovery codes shown once', () => {
  it('enrols TOTP and shows recovery codes exactly once', async () => {
    const confirm = vi.fn(async () => ({ recoveryCodes: ['code-aaa', 'code-bbb'] }));
    const service = fakeService({
      twofaStatus: vi.fn(async () => ({ totp: false, passkeys: [], recoveryRemaining: 0, policyRequired: false })),
      totpBegin: vi.fn(async () => ({ secret: 'JBSWY3DP', otpauthUri: 'otpauth://totp/x' })),
      totpConfirm: confirm,
    });
    render(() => <TwoFactor service={service} />);

    fireEvent.click(await screen.findByText('Set up an authenticator app'));
    await waitFor(() => expect(screen.getByTestId('totp-secret')).toHaveTextContent('JBSWY3DP'));

    fireEvent.input(screen.getByLabelText('Authenticator code'), { target: { value: '123456' } });
    fireEvent.click(screen.getByText('Confirm'));

    const panel = await screen.findByTestId('recovery-codes');
    expect(panel).toHaveTextContent('code-aaa');
    expect(panel).toHaveTextContent('code-bbb');
    expect(confirm).toHaveBeenCalledWith('123456');

    // Acknowledging dismisses the one-time panel; it is not re-fetchable.
    fireEvent.click(screen.getByTestId('recovery-ack'));
    await waitFor(() => expect(screen.queryByTestId('recovery-codes')).toBeNull());
  });

  it('surfaces the policy-required banner and lists enrolled passkeys', async () => {
    const service = fakeService({
      twofaStatus: vi.fn(async () => ({
        totp: true,
        passkeys: [{ handle: 'h1', label: 'YubiKey', createdAt: '2026-01-02' }],
        recoveryRemaining: 7,
        policyRequired: true,
      })),
      passkeyRemove: vi.fn(async () => undefined),
    });
    render(() => <TwoFactor service={service} />);

    expect(await screen.findByTestId('policy-required')).toBeInTheDocument();
    expect(await screen.findByTestId('passkey-list')).toHaveTextContent('YubiKey');
    expect(screen.getByTestId('recovery-remaining')).toHaveTextContent('7');
    expect(screen.getByTestId('totp-on')).toBeInTheDocument();
  });
});

// ── Login-time challenge (S1 web half) ──────────────────────────────────────────

describe('TwoFactorChallenge — no-downgrade second factor at login', () => {
  const challenge = { pendingToken: 'tok', factors: ['totp', 'recovery'] };

  it('verifies a TOTP code and calls onSuccess', async () => {
    const verify = vi.fn(async () => undefined);
    const onSuccess = vi.fn();
    render(() => (
      <TwoFactorChallenge challenge={challenge} onSuccess={onSuccess} service={fakeService({ verifyLoginFactor: verify })} />
    ));
    fireEvent.input(screen.getByLabelText('Authenticator code'), { target: { value: '000111' } });
    fireEvent.click(screen.getByTestId('challenge-verify'));
    await waitFor(() => expect(onSuccess).toHaveBeenCalled());
    expect(verify).toHaveBeenCalledWith({ pendingToken: 'tok', method: 'totp', code: '000111' });
  });

  it('shows a uniform error and does not call onSuccess on a wrong factor', async () => {
    const verify = vi.fn(async () => {
      throw new SettingsError(401, 'second factor required');
    });
    const onSuccess = vi.fn();
    render(() => (
      <TwoFactorChallenge challenge={challenge} onSuccess={onSuccess} service={fakeService({ verifyLoginFactor: verify })} />
    ));
    fireEvent.input(screen.getByLabelText('Authenticator code'), { target: { value: 'bad' } });
    fireEvent.click(screen.getByTestId('challenge-verify'));
    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument());
    expect(onSuccess).not.toHaveBeenCalled();
  });
});

// ── Sessions (S11) ──────────────────────────────────────────────────────────────

describe('Sessions — list + revoke', () => {
  it('renders sessions, flags the current one, and revokes the others', async () => {
    const revokeOthers = vi.fn(async () => undefined);
    let call = 0;
    const service = fakeService({
      sessions: vi.fn(async () => {
        call += 1;
        return call === 1
          ? [
              { handle: 'cur', username: 'a@ex', createdAt: '', lastSeen: 'now', current: true },
              { handle: 'other', username: 'a@ex', createdAt: '', lastSeen: 'yesterday', current: false },
            ]
          : [{ handle: 'cur', username: 'a@ex', createdAt: '', lastSeen: 'now', current: true }];
      }),
      revokeOtherSessions: revokeOthers,
    });
    render(() => <Sessions service={service} />);

    expect(await screen.findByText('This session')).toBeInTheDocument();
    const revoke = await screen.findByTestId('revoke-others');
    fireEvent.click(revoke);
    await waitFor(() => expect(revokeOthers).toHaveBeenCalled());
  });
});

// ── Signatures (W12) ────────────────────────────────────────────────────────────

describe('Signatures — CRUD', () => {
  it('creates a signature via upsert', async () => {
    const upsert = vi.fn(async (_s: Signature) => undefined);
    const service = fakeService({ listSignatures: vi.fn(async () => []), upsertSignature: upsert });
    render(() => <Signatures service={service} />);

    fireEvent.click(await screen.findByText('New signature'));
    fireEvent.input(screen.getByLabelText('Name'), { target: { value: 'Work' } });
    fireEvent.input(screen.getByLabelText('Signature'), { target: { value: '— Alex' } });
    fireEvent.click(screen.getByTestId('signature-save'));

    await waitFor(() => expect(upsert).toHaveBeenCalled());
    expect(upsert.mock.calls[0]?.[0]).toMatchObject({ name: 'Work', body: '— Alex', isDefault: false });
  });

  it('refuses to save an unnamed signature', async () => {
    const upsert = vi.fn(async (_s: Signature) => undefined);
    const service = fakeService({ listSignatures: vi.fn(async () => []), upsertSignature: upsert });
    render(() => <Signatures service={service} />);
    fireEvent.click(await screen.findByText('New signature'));
    fireEvent.click(screen.getByTestId('signature-save'));
    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument());
    expect(upsert).not.toHaveBeenCalled();
  });
});

// ── Identities ──────────────────────────────────────────────────────────────────

describe('Identities — validation + create', () => {
  it('blocks an invalid email and saves a valid identity', async () => {
    const upsert = vi.fn(async (_i: Identity) => undefined);
    const service = fakeService({
      listIdentities: vi.fn(async () => []),
      listSignatures: vi.fn(async () => []),
      upsertIdentity: upsert,
    });
    render(() => <Identities service={service} />);

    fireEvent.click(await screen.findByText('New identity'));
    fireEvent.input(screen.getByLabelText('Display name'), { target: { value: 'Alex' } });
    fireEvent.input(screen.getByLabelText('Email address'), { target: { value: 'not-an-email' } });
    fireEvent.click(screen.getByTestId('identity-save'));
    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument());
    expect(upsert).not.toHaveBeenCalled();

    fireEvent.input(screen.getByLabelText('Email address'), { target: { value: 'alex@ex.com' } });
    fireEvent.click(screen.getByTestId('identity-save'));
    await waitFor(() => expect(upsert).toHaveBeenCalled());
    expect(upsert.mock.calls[0]?.[0]).toMatchObject({ name: 'Alex', email: 'alex@ex.com' });
  });
});

// ── Notifications (W15) ─────────────────────────────────────────────────────────

describe('Notifications — rules + quiet hours', () => {
  it('adds a rule and saves the config', async () => {
    const save = vi.fn(async (_c: NotificationConfig) => undefined);
    const service = fakeService({
      notifications: vi.fn(async () => ({
        enabled: true,
        rules: [],
        quietHours: { enabled: false, start: '22:00', end: '07:00' },
      })),
      saveNotifications: save,
    });
    render(() => <Notifications service={service} />);

    fireEvent.click(await screen.findByTestId('notif-add-rule'));
    const list = screen.getByTestId('notif-rule-list');
    fireEvent.input(list.querySelector('input')!, { target: { value: 'boss@ex.com' } });
    fireEvent.click(screen.getByTestId('notif-save'));

    await waitFor(() => expect(save).toHaveBeenCalled());
    const cfg = save.mock.calls[0]![0];
    expect(cfg.rules[0]?.match).toBe('boss@ex.com');
    expect(await screen.findByTestId('notif-saved')).toBeInTheDocument();
  });
});

// ── Saved searches (W13) ────────────────────────────────────────────────────────

describe('SavedSearches — promote to a folder', () => {
  it('toggles as_folder via upsert', async () => {
    const upsert = vi.fn(async (_s: SavedSearch) => undefined);
    const service = fakeService({
      listSavedSearches: vi.fn(async () => [
        { id: 's1', name: 'From finance', queryJson: '{}', asFolder: false },
      ]),
      upsertSavedSearch: upsert,
    });
    render(() => <SavedSearches service={service} />);

    const checkbox = await screen.findByLabelText('Show as folder');
    fireEvent.click(checkbox);
    await waitFor(() => expect(upsert).toHaveBeenCalled());
    expect(upsert.mock.calls[0]?.[0]).toMatchObject({ id: 's1', asFolder: true });
  });

  it('shows an empty state with no saved searches', async () => {
    const service = fakeService({ listSavedSearches: vi.fn(async () => []) });
    render(() => <SavedSearches service={service} />);
    expect(await screen.findByText('You have no saved searches yet.')).toBeInTheDocument();
  });
});

// ── Preferences (W14 / W16 / W20) ───────────────────────────────────────────────

describe('Preferences — keyboard preset, offline purge, direction', () => {
  function harness(initial: SettingsPrefs = { ...DEFAULT_PREFS }) {
    const [prefs, setPrefs] = createSignal<SettingsPrefs>(initial);
    const onChange = vi.fn((next: SettingsPrefs) => setPrefs(next));
    return { prefs, onChange };
  }

  it('switches the keyboard preset and previews its bindings', async () => {
    const { prefs, onChange } = harness();
    render(() => <Preferences prefs={prefs} onChange={onChange} />);
    fireEvent.click(screen.getByText('Vim'));
    expect(onChange).toHaveBeenCalled();
    expect(onChange.mock.calls.at(-1)?.[0]).toMatchObject({ keyboardPreset: 'vim' });
  });

  it('dispatches a purge event and mirrors the preview when RTL is chosen', async () => {
    const purge = vi.fn();
    window.addEventListener('mw:offline-purge', purge);
    const { prefs, onChange } = harness({ ...DEFAULT_PREFS, direction: 'rtl' });
    render(() => <Preferences prefs={prefs} onChange={onChange} />);

    expect(screen.getByTestId('dir-preview')).toHaveAttribute('dir', 'rtl');
    fireEvent.click(screen.getByTestId('offline-purge'));
    expect(purge).toHaveBeenCalled();
    window.removeEventListener('mw:offline-purge', purge);
  });
});

// ── prefs persistence ───────────────────────────────────────────────────────────

describe('prefs — persistence + junk tolerance', () => {
  beforeEach(() => localStorage.clear());

  it('round-trips through localStorage', () => {
    savePrefs({ ...DEFAULT_PREFS, keyboardPreset: 'gmail', direction: 'rtl', offlineBudgetMb: 500 });
    const back = loadPrefs();
    expect(back.keyboardPreset).toBe('gmail');
    expect(back.direction).toBe('rtl');
    expect(back.offlineBudgetMb).toBe(500);
  });

  it('falls back to defaults per field on junk', () => {
    localStorage.setItem('mw.settings.prefs.v1', '{"keyboardPreset":"nope","offlineBudgetMb":-9}');
    const back = loadPrefs();
    expect(back.keyboardPreset).toBe(DEFAULT_PREFS.keyboardPreset);
    expect(back.offlineBudgetMb).toBe(DEFAULT_PREFS.offlineBudgetMb);
  });
});

// ── service transport ───────────────────────────────────────────────────────────

describe('SettingsService — endpoints + error surfacing', () => {
  it('POSTs a TOTP confirm and revokes all other sessions', async () => {
    const fetcher = vi.fn(async (input: string) => {
      if (input === '/api/account/2fa/totp/confirm') return okJson({ ok: true, recoveryCodes: ['x'] });
      if (input === '/api/account/sessions/revoke') return okJson({ ok: true, revoked: 3 });
      return okJson({});
    }) as unknown as Fetcher;
    const service = new SettingsService(fetcher);

    const codes = await service.totpConfirm('123456');
    expect(codes.recoveryCodes).toEqual(['x']);
    await service.revokeOtherSessions();

    const revokeCall = (fetcher as unknown as ReturnType<typeof vi.fn>).mock.calls.find(
      (c) => c[0] === '/api/account/sessions/revoke',
    );
    expect(JSON.parse(revokeCall?.[1]?.body as string)).toEqual({ all: true });
  });

  it('surfaces the server error string with its status (e.g. last-factor 409)', async () => {
    const fetcher = vi.fn(async () =>
      okJson({ error: 'your organization requires two-factor authentication' }, 409),
    ) as unknown as Fetcher;
    const service = new SettingsService(fetcher);
    await expect(service.totpDisable()).rejects.toMatchObject({
      status: 409,
      message: 'your organization requires two-factor authentication',
    });
  });
});

// ── WebAuthn ceremony encoding (W stubs) ────────────────────────────────────────

describe('webauthn — ceremony plumbing encodes base64url', () => {
  const buf = (bytes: number[]): ArrayBuffer => new Uint8Array(bytes).buffer;

  beforeEach(() => {
    vi.stubGlobal('PublicKeyCredential', class {});
  });
  afterEach(() => {
    vi.unstubAllGlobals();
    delete (navigator as { credentials?: unknown }).credentials;
  });

  it('registerPasskey returns base64url-encoded ceremony outputs', async () => {
    Object.defineProperty(navigator, 'credentials', {
      configurable: true,
      value: {
        create: vi.fn(async () => ({
          rawId: buf([1, 2, 3]),
          response: {
            clientDataJSON: buf([251, 255, 0]),
            attestationObject: buf([10, 20, 30]),
            getTransports: () => ['internal', 'hybrid'],
          },
        })),
      },
    });
    const out = await registerPasskey({
      challenge: 'AQID', // base64url of [1,2,3]
      rpId: 'mail.ex',
      userHandle: 'AQID',
      userName: 'a@ex',
      userVerification: 'preferred',
    });
    // 0xFB 0xFF 0x00 → base64url "-_8A" (no padding, url-safe alphabet).
    expect(out.clientDataJson).toBe('-_8A');
    expect(out.transports).toBe('internal,hybrid');
    expect(out.attestationObject).not.toContain('=');
  });

  it('assertPasskey encodes the assertion fields', async () => {
    Object.defineProperty(navigator, 'credentials', {
      configurable: true,
      value: {
        get: vi.fn(async () => ({
          rawId: buf([1, 2, 3]),
          response: {
            clientDataJSON: buf([1, 2, 3]),
            authenticatorData: buf([4, 5, 6]),
            signature: buf([7, 8, 9]),
          },
        })),
      },
    });
    const out = await assertPasskey({ challenge: 'AQID', credentialIds: ['AQID'], rpId: 'mail.ex', userVerification: 'preferred' });
    expect(out.credentialId).toBe('AQID');
    expect(out.clientDataJson).toBe('AQID');
    expect(out.authenticatorData).toBe('BAUG');
    expect(out.signature).toBe('BwgJ');
  });
});

// ── Aggregator + W20 RTL locale ─────────────────────────────────────────────────

describe('AccountSettings — composition + direction', () => {
  beforeEach(() => localStorage.clear());

  it('renders all sections and mirrors when the direction pref is RTL', async () => {
    const service = fakeService({
      twofaStatus: vi.fn(async () => ({ totp: false, passkeys: [], recoveryRemaining: 0, policyRequired: false })),
      sessions: vi.fn(async () => []),
      listSignatures: vi.fn(async () => []),
      listIdentities: vi.fn(async () => []),
      notifications: vi.fn(async () => ({ enabled: true, rules: [], quietHours: { enabled: false, start: '22:00', end: '07:00' } })),
      listSavedSearches: vi.fn(async () => []),
    });
    savePrefs({ ...DEFAULT_PREFS, direction: 'rtl' });
    render(() => <AccountSettings service={service} />);
    const root = await screen.findByTestId('account-settings');
    expect(root).toHaveAttribute('dir', 'rtl');
    expect(screen.getByLabelText('Two-factor authentication')).toBeInTheDocument();
    expect(screen.getByLabelText('Active sessions')).toBeInTheDocument();
  });
});

describe('W20 — the shipped Arabic (RTL) settings catalog is valid and renderable', () => {
  it('parses with no Fluent errors and formats Arabic strings', () => {
    const bundle = new FluentBundle('ar', { useIsolating: false });
    const errors = bundle.addResource(new FluentResource(arSettings));
    expect(errors).toHaveLength(0);
    const msg = bundle.getMessage('settings-2fa-title');
    expect(msg?.value).toBeTruthy();
    const out = bundle.formatPattern(msg!.value!, {});
    expect(out).toBe('المصادقة الثنائية');
  });
});
