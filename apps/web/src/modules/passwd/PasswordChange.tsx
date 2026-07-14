// In-app password change + policy display + zero-access re-wrap (SPEC §18.3, plan §3 e7).
//
// For a plain account this is a simple change form gated by the backend policy. For a
// ZERO-ACCESS account, changing the password must re-wrap the key hierarchy under the
// new password — and the HARD ORDERING constraint (plan) is that the recovery-phrase
// PRE-PROMPT is surfaced and acknowledged BEFORE the change is applied. This component
// enforces that with an explicit two-phase flow: the change POST for a zero-access
// account is unreachable until the recovery phrase (derived via the crypto worker from
// the CURRENT key hierarchy) has been shown and the user has confirmed they saved it.
//
// EXPORTED for e14 to mount (e.g. inside Settings' security section); this file does
// not touch the router or Settings.tsx (ownership boundary — coordinate with e6/e14).

import { createSignal, createResource, Show, onMount, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import {
  PasswordService,
  policyViolations,
  type Fetcher,
  type PasswordPolicy,
  type PasswordChangeOutcome,
  type PasswordChangeRequest,
  type RewrapPayload,
} from './service.ts';
import { recoveryPhraseBefore, rewrapUnderNewPassword } from './rewrap.ts';
import { utf8ToB64, type ZeroAccessCrypto } from '../zeroaccess/crypto.ts';
import type { ZeroAccessAccount } from '../zeroaccess/service.ts';
import * as css from './styles.css.ts';

export interface PasswordChangeProps {
  accountId: string;
  /** Zero-access re-wrap context. When present (account.enabled), the recovery-phrase
   *  pre-prompt runs before the change; `za` is the crypto worker facade. */
  zeroAccess?: { account: ZeroAccessAccount; za: ZeroAccessCrypto };
  fetcher?: Fetcher;
  service?: PasswordService;
  /** Tests may inject the policy directly to skip the fetch. */
  initialPolicy?: PasswordPolicy;
  /** Called when the change succeeds (e14 may refresh session state). */
  onChanged?: (outcome: PasswordChangeOutcome) => void;
}

type Phase = 'form' | 'recovery' | 'done';

export function PasswordChange(props: PasswordChangeProps): JSX.Element {
  const service = props.service ?? new PasswordService(props.fetcher);
  const [policy] = createResource<PasswordPolicy>(() => props.initialPolicy ?? service.policy());

  const [oldPw, setOldPw] = createSignal('');
  const [newPw, setNewPw] = createSignal('');
  const [confirm, setConfirm] = createSignal('');
  const [phase, setPhase] = createSignal<Phase>('form');
  const [phrase, setPhrase] = createSignal('');
  const [ack, setAck] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');
  const [outcome, setOutcome] = createSignal<PasswordChangeOutcome | null>(null);

  const isZeroAccess = (): boolean => props.zeroAccess?.account.enabled === true;

  onMount(() => void loadCatalog('passwd'));

  /** Validate inputs against policy + confirmation. Returns an error string or ''. */
  function validate(): string {
    const p = policy();
    if (oldPw() === '') return t('passwd-error-enter-current');
    if (newPw() !== confirm()) return t('passwd-error-mismatch');
    if (p !== undefined) {
      const missing = policyViolations(p, newPw());
      if (missing.length > 0) return t('passwd-error-policy', { rules: missing.join(', ') });
    }
    return '';
  }

  /** Step 1: submit the form. Plain account → change directly; zero-access → recovery
   *  pre-prompt FIRST (derive + show the recovery phrase; DO NOT change yet). */
  async function onSubmit(): Promise<void> {
    setError('');
    const problem = validate();
    if (problem !== '') {
      setError(problem);
      return;
    }
    if (!isZeroAccess()) {
      await applyChange();
      return;
    }
    // Zero-access: derive the recovery phrase from the CURRENT hierarchy BEFORE any
    // change is applied. This is the ordering guarantee — the change POST cannot run
    // until the user passes through this phase and acknowledges.
    setBusy(true);
    try {
      const { account, za } = props.zeroAccess!;
      const derived = await recoveryPhraseBefore(za, account, utf8ToB64(oldPw()));
      setPhrase(derived);
      setPhase('recovery');
    } catch (e) {
      setError(e instanceof Error ? e.message : t('passwd-error-prepare-recovery'));
    } finally {
      setBusy(false);
    }
  }

  /** Step 2 (zero-access only): after the recovery phrase is acknowledged, re-wrap the
   *  key hierarchy under the new password and apply the change. */
  async function onConfirmRewrap(): Promise<void> {
    if (!ack()) {
      setError(t('passwd-error-ack-first'));
      return;
    }
    await applyChange();
  }

  /** Perform the actual change POST (with re-wrap material for zero-access accounts). */
  async function applyChange(): Promise<void> {
    setError('');
    setBusy(true);
    try {
      let rewrap: RewrapPayload | undefined;
      if (isZeroAccess()) {
        const { account, za } = props.zeroAccess!;
        rewrap = await rewrapUnderNewPassword({
          za,
          account,
          oldSecretB64: utf8ToB64(oldPw()),
          newSecretB64: utf8ToB64(newPw()),
        });
      }
      const req: PasswordChangeRequest =
        rewrap !== undefined
          ? { oldPassword: oldPw(), newPassword: newPw(), rewrap }
          : { oldPassword: oldPw(), newPassword: newPw() };
      const result = await service.change(req);
      setOutcome(result);
      setPhase('done');
      // Wipe the in-memory secrets once applied.
      setOldPw('');
      setNewPw('');
      setConfirm('');
      setPhrase('');
      props.onChanged?.(result);
    } catch (e) {
      setError(e instanceof Error ? e.message : t('passwd-error-change'));
    } finally {
      setBusy(false);
    }
  }

  const errorId = 'passwd-error';
  const describedBy = (): string | undefined => (error() !== '' ? errorId : undefined);

  return (
    <div class={css.panel} role="form" aria-label={t('passwd-region-label')}>
      <Show when={policy()?.forceChange}>
        <p class={css.banner} role="alert" data-testid="force-change-banner">
          {t('passwd-force-change')}
        </p>
      </Show>

      <Show when={phase() === 'form'}>
        <section class={css.section}>
          <h2 class={css.heading}>{t('passwd-heading')}</h2>

          <label class={css.field}>
            <span class={css.label}>{t('passwd-current-label')}</span>
            <input
              class={css.input}
              type="password"
              autocomplete="current-password"
              aria-label={t('passwd-current-label')}
              aria-describedby={describedBy()}
              value={oldPw()}
              onInput={(e) => setOldPw(e.currentTarget.value)}
            />
          </label>
          <label class={css.field}>
            <span class={css.label}>{t('passwd-new-label')}</span>
            <input
              class={css.input}
              type="password"
              autocomplete="new-password"
              aria-label={t('passwd-new-label')}
              aria-describedby={describedBy()}
              value={newPw()}
              onInput={(e) => setNewPw(e.currentTarget.value)}
            />
          </label>
          <label class={css.field}>
            <span class={css.label}>{t('passwd-confirm-label')}</span>
            <input
              class={css.input}
              type="password"
              autocomplete="new-password"
              aria-label={t('passwd-confirm-label')}
              aria-describedby={describedBy()}
              value={confirm()}
              onInput={(e) => setConfirm(e.currentTarget.value)}
            />
          </label>

          <Show when={confirm() !== ''}>
            <p
              class={newPw() === confirm() ? css.success : css.error}
              role="status"
              aria-live="polite"
              data-testid="pw-match"
            >
              {newPw() === confirm() ? t('passwd-match-ok') : t('passwd-match-no')}
            </p>
          </Show>

          <Show when={policy()}>
            {(p) => (
              <div data-testid="policy">
                <p class={css.meta}>{p().description}</p>
                <ul class={css.policyList}>
                  <li>{t('passwd-rule-min-length', { count: p().minLength })}</li>
                  <Show when={p().requireUppercase}><li>{t('passwd-rule-uppercase')}</li></Show>
                  <Show when={p().requireLowercase}><li>{t('passwd-rule-lowercase')}</li></Show>
                  <Show when={p().requireDigit}><li>{t('passwd-rule-digit')}</li></Show>
                  <Show when={p().requireSymbol}><li>{t('passwd-rule-symbol')}</li></Show>
                </ul>
              </div>
            )}
          </Show>

          <Show when={isZeroAccess()}>
            <p class={css.warn} data-testid="rewrap-notice">
              {t('passwd-rewrap-notice')}
            </p>
          </Show>

          <button type="button" class={css.button} disabled={busy()} onClick={() => void onSubmit()}>
            {isZeroAccess() ? t('passwd-continue') : t('passwd-submit')}
          </button>
        </section>
      </Show>

      <Show when={phase() === 'recovery'}>
        <section class={css.section} data-testid="recovery-prompt">
          <h2 class={css.heading}>{t('passwd-recovery-heading')}</h2>
          <p class={css.prose}>
            {t('passwd-recovery-prose')}
          </p>
          <code class={css.phrase} data-testid="recovery-phrase">
            {phrase()}
          </code>
          <label class={css.check}>
            <input
              class={css.checkbox}
              type="checkbox"
              aria-label={t('passwd-recovery-ack-label')}
              checked={ack()}
              onChange={(e) => setAck(e.currentTarget.checked)}
            />
            <span>{t('passwd-recovery-ack-text')}</span>
          </label>
          <button
            type="button"
            class={css.button}
            disabled={busy() || !ack()}
            data-testid="confirm-change"
            onClick={() => void onConfirmRewrap()}
          >
            {t('passwd-submit')}
          </button>
        </section>
      </Show>

      <Show when={phase() === 'done'}>
        <section class={css.section} data-testid="change-done">
          <p class={css.success}>{t('passwd-done')}</p>
          <Show when={outcome()?.reencryptCredentials}>
            <p class={css.meta}>{t('passwd-done-reencrypt')}</p>
          </Show>
          <Show when={outcome()?.zeroaccessRewrapRequired}>
            <p class={css.meta}>{t('passwd-done-rewrap')}</p>
          </Show>
        </section>
      </Show>

      <Show when={error() !== ''}>
        <p class={css.error} id={errorId} role="alert">
          {error()}
        </p>
      </Show>
    </div>
  );
}

export default PasswordChange;
