import { Show, createSignal, createUniqueId, type JSX } from 'solid-js';
import { render } from 'solid-js/web';
import { loadCatalog, t } from '../i18n';
import { createFocusTrap } from './a11y';
import { assertPasskey } from '../screens/Settings/webauthn.ts';
import * as css from '../screens/Admin/shell/shell.css.ts';

export interface StepUpPrompt {
  methods: readonly string[];
  send: (body: Record<string, unknown>, passkeyBegin?: boolean) => Promise<Response>;
}

function StepUpDialog(props: StepUpPrompt & { finish: (confirmed: boolean) => void }): JSX.Element {
  const id = createUniqueId();
  const [password, setPassword] = createSignal('');
  const [code, setCode] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');
  const [element, setElement] = createSignal<HTMLElement>();
  const cancel = (): void => { if (!busy()) props.finish(false); };
  createFocusTrap(element, { onEscape: cancel });
  async function confirm(passkey: boolean): Promise<void> {
    if (busy()) return;
    setBusy(true);
    setError('');
    try {
      const body: Record<string, unknown> = {};
      if (passkey) {
        const response = await props.send({}, true);
        if (!response.ok) throw new Error(t('security-stepup-failed'));
        const challenge = await response.json() as Parameters<typeof assertPasskey>[0];
        body.passkey = await assertPasskey({ ...challenge, userVerification: 'required' });
      } else {
        if (props.methods.includes('password')) body.password = password();
        if (props.methods.includes('totp')) body.code = code();
      }
      const response = await props.send(body);
      if (!response.ok) {
        const failure = await response.json().catch(() => null) as { detail?: unknown } | null;
        throw new Error(typeof failure?.detail === 'string' ? failure.detail : t('security-stepup-failed'));
      }
      setPassword('');
      setCode('');
      props.finish(true);
    } catch (failure) {
      setError(failure instanceof Error ? failure.message : t('security-stepup-failed'));
    } finally { setBusy(false); }
  }
  return <>
    <div class={css.dialogScrim} />
    <form ref={setElement} class={css.dialog} role="dialog" aria-modal="true" aria-labelledby={`${id}-title`} aria-describedby={`${id}-body`} tabindex="-1" onSubmit={(event) => { event.preventDefault(); void confirm(false); }}>
      <h2 class={css.dialogTitle} id={`${id}-title`}>{t('security-stepup-title')}</h2>
      <p class={css.help} id={`${id}-body`}>{t('security-stepup-body')}</p>
      <Show when={props.methods.includes('password')}><label class="field"><span>{t('security-stepup-password')}</span><input type="password" autocomplete="current-password" value={password()} onInput={(event) => setPassword(event.currentTarget.value)} disabled={busy()} /></label></Show>
      <Show when={props.methods.includes('totp')}><label class="field"><span>{t('security-stepup-code')}</span><input type="text" inputmode="numeric" autocomplete="one-time-code" value={code()} onInput={(event) => setCode(event.currentTarget.value)} disabled={busy()} /></label></Show>
      <Show when={props.methods.length === 0}><p role="alert">{t('security-stepup-unavailable')}</p></Show>
      <Show when={error()}><p role="alert">{error()}</p></Show>
      <div class={css.dialogActions}>
        <button type="button" onClick={cancel} disabled={busy()}>{t('security-stepup-cancel')}</button>
        <Show when={props.methods.includes('passkey')}><button type="button" onClick={() => void confirm(true)} disabled={busy()}>{t('security-stepup-passkey')}</button></Show>
        <Show when={props.methods.includes('password') || props.methods.includes('totp')}><button type="submit" disabled={busy()}>{t('security-stepup-confirm')}</button></Show>
      </div>
    </form>
  </>;
}

/** Mounted only after a server refusal; secrets never leave this transient dialog. */
export async function requestStepUp(prompt: StepUpPrompt): Promise<boolean> {
  await loadCatalog('security');
  return new Promise((resolve) => {
    const host = document.createElement('div');
    document.body.append(host);
    let dispose = (): void => {};
    const finish = (confirmed: boolean): void => {
      dispose();
      host.remove();
      resolve(confirmed);
    };
    dispose = render(() => <StepUpDialog {...prompt} finish={finish} />, host);
  });
}
