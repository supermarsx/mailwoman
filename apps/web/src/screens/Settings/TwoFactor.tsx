// Two-factor enrolment + management (t16 e15, SPEC §7.4/§19 — S1 web half).
//
// Reuses the WebAuthn ceremony plumbing (`webauthn.ts`, itself the create/get
// sibling of the zero-access PRF path) for passkey enrolment, and calls the
// `crates/mw-server/src/twofa_routes.rs` account routes for TOTP + recovery. The
// invariant the UI enforces: recovery codes are shown EXACTLY ONCE (server never
// re-serves them) — the user must acknowledge before the panel is dismissed.

import { createResource, createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { SettingsService } from './service.ts';
import { passkeySupported, registerPasskey } from './webauthn.ts';
import type { PasskeyPolicySummary, TwofaStatus } from './types.ts';
import * as css from './styles.css.ts';

export interface TwoFactorProps {
  /** Injected in tests to avoid a live server. */
  service?: SettingsService;
}

type TotpPhase = { kind: 'idle' } | { kind: 'enrolling'; secret: string; otpauthUri: string };

/** t39-e6: one line per administrator rule that narrows which passkeys count. */
export function passkeyPolicyLines(policy: PasskeyPolicySummary | undefined): string[] {
  if (policy === undefined) return [];
  const lines: string[] = [];
  if (policy.attachment === 'platform') lines.push(t('settings-2fa-passkey-rule-platform'));
  if (policy.attachment === 'cross-platform') lines.push(t('settings-2fa-passkey-rule-cross-platform'));
  if (policy.userVerification === 'required') lines.push(t('settings-2fa-passkey-rule-user-verification'));
  if (policy.residentKey === 'required') lines.push(t('settings-2fa-passkey-rule-resident-key'));
  return lines;
}

/** The browser refused the ceremony (no qualifying authenticator, cancelled). */
function browserRefused(e: unknown): boolean {
  return e instanceof Error && (e.name === 'NotAllowedError' || e.name === 'NotSupportedError' || e.name === 'ConstraintError');
}

export function TwoFactor(props: TwoFactorProps): JSX.Element {
  const service = props.service ?? new SettingsService();
  onMount(() => void loadCatalog('settings'));

  const [status, { refetch }] = createResource<TwofaStatus>(() => service.twofaStatus());

  const [totp, setTotp] = createSignal<TotpPhase>({ kind: 'idle' });
  const [code, setCode] = createSignal('');
  const [label, setLabel] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');
  // The one-time recovery codes to display; cleared only on explicit acknowledge.
  const [recovery, setRecovery] = createSignal<readonly string[] | null>(null);

  function fail(e: unknown, fallback: string): void {
    setError(e instanceof Error ? e.message : fallback);
  }

  async function beginTotp(): Promise<void> {
    setError('');
    setBusy(true);
    try {
      const begun = await service.totpBegin();
      setTotp({ kind: 'enrolling', secret: begun.secret, otpauthUri: begun.otpauthUri });
      setCode('');
    } catch (e) {
      fail(e, t('settings-2fa-error-generic'));
    } finally {
      setBusy(false);
    }
  }

  async function confirmTotp(): Promise<void> {
    setError('');
    setBusy(true);
    try {
      const out = await service.totpConfirm(code().trim());
      setTotp({ kind: 'idle' });
      setCode('');
      if (out.recoveryCodes.length > 0) setRecovery(out.recoveryCodes);
      await refetch();
    } catch (e) {
      fail(e, t('settings-2fa-error-code'));
    } finally {
      setBusy(false);
    }
  }

  async function disableTotp(): Promise<void> {
    setError('');
    setBusy(true);
    try {
      await service.totpDisable();
      await refetch();
    } catch (e) {
      fail(e, t('settings-2fa-error-generic'));
    } finally {
      setBusy(false);
    }
  }

  async function enrolPasskey(): Promise<void> {
    setError('');
    setBusy(true);
    try {
      const challenge = await service.passkeyBegin();
      let ceremony;
      try {
        ceremony = await registerPasskey(challenge);
      } catch (e) {
        // The browser found no authenticator that meets the requested options
        // (or the user cancelled): say what the policy asks for.
        if (browserRefused(e) && passkeyPolicyLines(status()?.passkeyPolicy).length > 0) {
          throw new Error(t('settings-2fa-passkey-browser-refused-policy'));
        }
        throw e;
      }
      const out = await service.passkeyFinish({
        clientDataJson: ceremony.clientDataJson,
        attestationObject: ceremony.attestationObject,
        transports: ceremony.transports,
        label: label().trim(),
        ...(ceremony.authenticatorAttachment !== undefined ? { authenticatorAttachment: ceremony.authenticatorAttachment } : {}),
        ...(ceremony.residentKey !== undefined ? { residentKey: ceremony.residentKey } : {}),
      });
      setLabel('');
      if (out.recoveryCodes.length > 0) setRecovery(out.recoveryCodes);
      await refetch();
    } catch (e) {
      fail(e, t('settings-2fa-error-passkey'));
    } finally {
      setBusy(false);
    }
  }

  async function removePasskey(handle: string): Promise<void> {
    setError('');
    setBusy(true);
    try {
      await service.passkeyRemove(handle);
      await refetch();
    } catch (e) {
      fail(e, t('settings-2fa-error-generic'));
    } finally {
      setBusy(false);
    }
  }

  async function regenerateRecovery(): Promise<void> {
    setError('');
    setBusy(true);
    try {
      const out = await service.recoveryRegenerate();
      setRecovery(out.recoveryCodes);
      await refetch();
    } catch (e) {
      fail(e, t('settings-2fa-error-generic'));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section class={css.section} aria-label={t('settings-2fa-title')}>
      <h2 class={css.heading}>{t('settings-2fa-title')}</h2>
      <p class={css.prose}>{t('settings-2fa-intro')}</p>

      <Show when={status()?.policyRequired}>
        <p class={css.warn} data-testid="policy-required">
          {t('settings-2fa-policy-required')}
        </p>
      </Show>

      {/* One-time recovery codes — shown once; acknowledge to dismiss. */}
      <Show when={recovery()}>
        {(codes) => (
          <div class={css.codesPanel} role="region" aria-label={t('settings-2fa-recovery-title')} data-testid="recovery-codes">
            <p class={css.subheading}>{t('settings-2fa-recovery-title')}</p>
            <p class={css.meta}>{t('settings-2fa-recovery-once')}</p>
            <div class={css.codesGrid}>
              <For each={codes()}>{(c) => <span>{c}</span>}</For>
            </div>
            <button type="button" class={css.button} onClick={() => setRecovery(null)} data-testid="recovery-ack">
              {t('settings-2fa-recovery-ack')}
            </button>
          </div>
        )}
      </Show>

      {/* Authenticator (TOTP) */}
      <div class={css.field}>
        <span class={css.subheading}>{t('settings-2fa-totp-title')}</span>
        <Show
          when={status()?.totp}
          fallback={
            <Show
              when={totp().kind === 'enrolling'}
              fallback={
                <div class={css.actions}>
                  <button type="button" class={css.button} disabled={busy()} onClick={() => void beginTotp()}>
                    {t('settings-2fa-totp-enrol')}
                  </button>
                </div>
              }
            >
              {(() => {
                const phase = totp() as { kind: 'enrolling'; secret: string; otpauthUri: string };
                return (
                  <div class={css.field} data-testid="totp-enrol">
                    <p class={css.meta}>{t('settings-2fa-totp-scan')}</p>
                    <code class={css.secret} data-testid="totp-secret">
                      {phase.secret}
                    </code>
                    <a class={css.meta} href={phase.otpauthUri} data-testid="totp-uri">
                      {t('settings-2fa-totp-uri-link')}
                    </a>
                    <label class={css.field}>
                      <span class={css.label}>{t('settings-2fa-totp-code-label')}</span>
                      <input
                        class={css.input}
                        inputmode="numeric"
                        autocomplete="one-time-code"
                        aria-label={t('settings-2fa-totp-code-label')}
                        value={code()}
                        onInput={(e) => setCode(e.currentTarget.value)}
                      />
                    </label>
                    <div class={css.actions}>
                      <button type="button" class={css.button} disabled={busy() || code().trim() === ''} onClick={() => void confirmTotp()}>
                        {t('settings-2fa-totp-confirm')}
                      </button>
                      <button type="button" class={css.ghost} disabled={busy()} onClick={() => setTotp({ kind: 'idle' })}>
                        {t('settings-cancel')}
                      </button>
                    </div>
                  </div>
                );
              })()}
            </Show>
          }
        >
          <div class={css.row}>
            <span class={css.badge} data-testid="totp-on">
              {t('settings-2fa-enabled')}
            </span>
            <button type="button" class={css.danger} disabled={busy()} onClick={() => void disableTotp()}>
              {t('settings-2fa-totp-disable')}
            </button>
          </div>
        </Show>
      </div>

      {/* Passkeys */}
      <div class={css.field}>
        <span class={css.subheading}>{t('settings-2fa-passkey-title')}</span>
        <Show when={status() && status()!.passkeys.length > 0}>
          <ul class={css.list} data-testid="passkey-list">
            <For each={status()!.passkeys}>
              {(pk) => (
                <li class={css.item}>
                  <div class={css.itemMain}>
                    <span class={css.itemName}>{pk.label}</span>
                    <span class={css.meta}>{pk.createdAt}</span>
                    <Show when={pk.meetsPolicy === false}>
                      <span class={css.warn} data-testid="passkey-policy-gap">
                        {t(status()?.passkeyPolicy?.legacy === 'refuse' ? 'settings-2fa-passkey-gap-refused' : 'settings-2fa-passkey-gap-allowed')}
                      </span>
                    </Show>
                  </div>
                  <button type="button" class={css.danger} disabled={busy()} onClick={() => void removePasskey(pk.handle)}>
                    {t('settings-2fa-passkey-remove')}
                  </button>
                </li>
              )}
            </For>
          </ul>
        </Show>
        <Show when={status()?.passkeysEnabled === false}>
          <p class={css.meta}>{t('settings-2fa-passkey-policy-disabled')}</p>
        </Show>
        <Show when={status()?.passkeysEnabled !== false && passkeyPolicyLines(status()?.passkeyPolicy).length > 0}>
          <div class={css.meta} data-testid="passkey-policy">
            <span>{t('settings-2fa-passkey-rules')}</span>
            <ul class={css.list}>
              <For each={passkeyPolicyLines(status()?.passkeyPolicy)}>{(line) => <li>{line}</li>}</For>
            </ul>
          </div>
        </Show>
        <Show
          when={status() && status()?.passkeysEnabled !== false && passkeySupported()}
          fallback={<Show when={status() && status()?.passkeysEnabled !== false}><p class={css.meta}>{t('settings-2fa-passkey-unsupported')}</p></Show>}
        >
          <div class={css.row}>
            <input
              class={`${css.input} ${css.grow}`}
              placeholder={t('settings-2fa-passkey-label-placeholder')}
              aria-label={t('settings-2fa-passkey-label-placeholder')}
              value={label()}
              onInput={(e) => setLabel(e.currentTarget.value)}
            />
            <button type="button" class={css.button} disabled={busy()} onClick={() => void enrolPasskey()}>
              {t('settings-2fa-passkey-add')}
            </button>
          </div>
        </Show>
      </div>

      {/* Recovery codes */}
      <div class={css.row}>
        <span class={css.meta} data-testid="recovery-remaining">
          {t('settings-2fa-recovery-remaining', { count: status()?.recoveryRemaining ?? 0 })}
        </span>
        <button type="button" class={css.ghost} disabled={busy()} onClick={() => void regenerateRecovery()}>
          {t('settings-2fa-recovery-regenerate')}
        </button>
      </div>

      <Show when={error() !== ''}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>
    </section>
  );
}

export default TwoFactor;
