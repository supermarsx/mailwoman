// "Renew access" for a linked mailbox whose access ended (t48). A themed modal
// over the mailbox picker or Settings › Linked mailboxes: sign the mailbox in
// again and the link stays as it is (id, nickname, order, primary choice). No
// unlink. A second factor, when the mailbox has one, is completed in the same
// dialog. The password lives only in this component and is cleared after use.
// t70: the server knows every renewable link's sign-in name, so the dialog
// asks for the password only; "Sign in with a different name" keeps the
// name-and-password form for the rare link whose name changed.
import { Show, createResource, createSignal, createUniqueId, onCleanup, type JSX } from 'solid-js';
import { Portal } from 'solid-js/web';
import { ApiError } from '../../api/client.ts';
import type { LinkedMailbox, LinkedMailboxChallenge, LinkMailboxProof, createLinkedAccountsApi } from '../../api/linkedAccounts.ts';
import { publicConfig } from '../../api/publicConfig.ts';
import { isolate, t } from '../../i18n/index.ts';
import { createFocusTrap } from '../a11y';
import { SigninChallenge } from '../SigninChallenge.tsx';
import { LinkedMailboxFactor } from '../../screens/Settings/LinkedMailboxFactor.tsx';
import { linkedAccountError } from './linkedAccountError.ts';
import * as shell from '../../screens/Admin/shell/shell.css.ts';
import * as ux from '../../screens/Admin/shell/ux.css.ts';
import * as css from '../../screens/Settings/styles.css.ts';

type Api = Pick<ReturnType<typeof createLinkedAccountsApi>, 'renew' | 'confirmRenew'>;

/** Why this link does not renew by itself, as a sentence (or nothing). */
export function renewalReasonText(reason: string | null | undefined): string | undefined {
  switch (reason) {
    case 'credentialRejected': return t('linked-accounts-renew-credential-rejected');
    case 'factorRequired': return t('linked-accounts-renew-factor');
    case 'zeroAccess': return t('linked-accounts-renew-zero-access');
    case 'policyOff':
    case 'linkingDisabled': return t('linked-accounts-renew-policy');
    case 'unsupported': return t('linked-accounts-renew-unsupported');
    case 'sessionRevoked': return t('linked-accounts-renew-revoked');
    default: return undefined;
  }
}

export function renewError(error: unknown): string {
  if (error instanceof ApiError && error.status === 401) return t('linked-accounts-renew-password-rejected');
  return linkedAccountError(error);
}

export function RenewAccessDialog(props: {
  account: LinkedMailbox;
  /** Nickname or address, as the row shows it. */
  label: string;
  api: Api;
  onRenewed: (account: LinkedMailbox) => void;
  onCancel: () => void;
  mount?: Node | undefined;
}): JSX.Element {
  const id = createUniqueId();
  const [config] = createResource(() => publicConfig().catch(() => undefined));
  const [username, setUsername] = createSignal('');
  const [otherName, setOtherName] = createSignal(false);
  const [password, setPassword] = createSignal('');
  const [proof, setProof] = createSignal<string>();
  const [reset, setReset] = createSignal(0);
  const [challenge, setChallenge] = createSignal<LinkedMailboxChallenge>();
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');
  const [el, setEl] = createSignal<HTMLElement>();
  let passwordInput: HTMLInputElement | undefined;
  let disposed = false;
  onCleanup(() => { disposed = true; setPassword(''); });
  const knownAddress = (): string | undefined => props.account.address;
  /** The password alone renews the link (the server knows the name). */
  const passwordOnly = (): boolean => !!knownAddress() && !otherName();
  const needsProof = (): boolean => config()?.antiBot?.mailbox === true;
  const cancel = (): void => { if (!busy()) props.onCancel(); };
  createFocusTrap(el, { onEscape: cancel, initialFocus: () => passwordInput });

  async function submit(event: Event): Promise<void> {
    event.preventDefault();
    if (busy()) return;
    const typed = username().trim();
    if (!password() || (!passwordOnly() && !typed)) { setError(t('linked-accounts-fields-required')); return; }
    if (needsProof() && !proof()) { setError(t('linked-accounts-verification-required')); return; }
    setBusy(true); setError('');
    try {
      const result = await props.api.renew(props.account.id, {
        ...(passwordOnly() ? {} : { username: typed }),
        password: password(),
        ...(proof() ? { altcha: proof()! } : {}),
      });
      if (disposed) return;
      if ('twofaRequired' in result) setChallenge(result);
      else props.onRenewed(result.account);
    } catch (cause) {
      if (!disposed) setError(renewError(cause));
    } finally {
      setPassword('');
      setProof(undefined);
      setReset((n) => n + 1);
      if (!disposed) setBusy(false);
    }
  }
  async function confirm(value: LinkMailboxProof): Promise<void> {
    const pending = challenge();
    if (!pending) return;
    const result = await props.api.confirmRenew(props.account.id, pending.challengeId, value);
    if (!disposed) props.onRenewed(result.account);
  }

  return (
    <Portal {...(props.mount ? { mount: props.mount } : {})}>
      <div class={shell.dialogScrim} onClick={cancel} />
      <div ref={setEl} class={shell.dialog} role="dialog" aria-modal="true" aria-labelledby={`${id}-title`} aria-describedby={`${id}-body`} tabindex="-1" data-testid="linked-renew-dialog">
        <h2 id={`${id}-title`} class={shell.dialogTitle}>{t('linked-accounts-renew-title', { name: isolate(props.label) })}</h2>
        <Show
          when={challenge()}
          fallback={
            <form onSubmit={(event) => void submit(event)}>
              <p id={`${id}-body`} class={shell.help}>
                <Show when={passwordOnly() && knownAddress()} fallback={t('linked-accounts-renew-help')}>
                  {(address) => t('linked-accounts-renew-password-only', { address: isolate(address()) })}
                </Show>
              </p>
              <Show when={renewalReasonText(props.account.renewalReason)}>{(text) => <p class={css.meta} data-renew-reason>{text()}</p>}</Show>
              <fieldset class={`${css.stack} ${css.bareFieldset}`} disabled={busy()}>
                <Show
                  when={passwordOnly() ? knownAddress() : undefined}
                  fallback={
                    <label class={css.field}>
                      <span class={css.label}>{t('linked-accounts-address')}</span>
                      <input class={css.input} autocomplete="username" autocapitalize="none" spellcheck={false} required maxlength={320} value={username()} placeholder={knownAddress() ?? ''} onInput={(event) => setUsername(event.currentTarget.value)} data-renew-username />
                    </label>
                  }
                >
                  {(address) => <input type="text" hidden autocomplete="username" value={address()} readOnly />}
                </Show>
                <label class={css.field}>
                  <span class={css.label}>{t('linked-accounts-password')}</span>
                  <input ref={passwordInput} class={css.input} type="password" autocomplete="current-password" required value={password()} onInput={(event) => setPassword(event.currentTarget.value)} data-renew-password />
                </label>
                <Show when={knownAddress() && !otherName()}>
                  <div><button type="button" class={ux.linkButton} data-renew-other-name
                    onClick={() => { setOtherName(true); setError(''); }}>{t('linked-accounts-renew-other-name')}</button></div>
                </Show>
                <Show when={needsProof()}><SigninChallenge surface="mailbox" reset={reset()} onProof={setProof} /></Show>
              </fieldset>
              <Show when={error()}><p class={css.error} role="alert">{error()}</p></Show>
              <div class={shell.dialogActions}>
                <button type="button" class="btn btn--ghost" disabled={busy()} onClick={cancel}>{t('linked-accounts-cancel')}</button>
                <button type="submit" class="btn btn--primary" disabled={busy()} data-renew-submit>{t('linked-accounts-renew-submit')}</button>
              </div>
            </form>
          }
        >
          {(pending) => <LinkedMailboxFactor challenge={pending()} onConfirm={confirm} onCancel={cancel} errorText={renewError} />}
        </Show>
      </div>
    </Portal>
  );
}
