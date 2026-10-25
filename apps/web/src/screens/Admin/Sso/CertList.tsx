// The IdP signing certificates of a SAML backend, as a list instead of a PEM
// textarea (t32-cfg §4.12): each certificate shows its subject, its end date and
// whether it is valid, expiring within 30 days, expired or unreadable (text and
// icon, not colour alone). Certificates are added from a file (.pem, .crt, .cer)
// or pasted PEM, and removed one by one; focus moves on as in ListEditor.

import { For, Show, createSignal, createUniqueId, type JSX } from 'solid-js';
import { formatDate, isolate, t } from '../../../i18n';
import { readFileText } from '../shell/files.ts';
import { StatusBadge, type StatusTone } from '../shell/StatusBadge.tsx';
import { Icon } from '../shell/icons.tsx';
import * as shell from '../shell/shell.css.ts';
import * as ux from '../shell/ux.css.ts';
import * as css from '../shell/extras.css.ts';
import { certInfo, certState, daysLeft, splitPem, type CertInfo, type CertState } from './saml.ts';

const TONE: Record<CertState, StatusTone> = {
  valid: 'ok',
  expiring: 'warn',
  expired: 'danger',
  'not-yet-valid': 'warn',
  unreadable: 'danger',
};

/** The badge text for a certificate's state. */
export function certStateText(c: CertInfo, now: Date = new Date()): string {
  const state = certState(c, now);
  const days = daysLeft(c, now) ?? 0;
  switch (state) {
    case 'valid':
      return t('admin-sso-cert-valid', { date: c.notAfter ? formatDate(c.notAfter) : '' });
    case 'expiring':
      return t('admin-sso-cert-expiring', { days });
    case 'expired':
      return t('admin-sso-cert-expired', { date: c.notAfter ? formatDate(c.notAfter) : '' });
    case 'not-yet-valid':
      return t('admin-sso-cert-not-yet', { date: c.notBefore ? formatDate(c.notBefore) : '' });
    case 'unreadable':
      return t('admin-sso-cert-unreadable');
  }
}

export function CertBadge(props: { cert: CertInfo; now?: Date | undefined }): JSX.Element {
  return (
    <StatusBadge tone={TONE[certState(props.cert, props.now)]} status={certState(props.cert, props.now)}>
      {certStateText(props.cert, props.now)}
    </StatusBadge>
  );
}

export interface CertListProps {
  id: string;
  describedBy: string | undefined;
  invalid: boolean;
  disabled: boolean;
  certs: readonly string[];
  onChange: (certs: string[]) => void;
  now?: Date | undefined;
}

export function CertList(props: CertListProps): JSX.Element {
  const own = createUniqueId();
  const [paste, setPaste] = createSignal('');
  const [error, setError] = createSignal<string | null>(null);
  let list: HTMLUListElement | undefined;
  let file: HTMLInputElement | undefined;

  function add(text: string): void {
    const found = splitPem(text);
    if (found.length === 0) {
      setError(t('admin-sso-cert-none-found'));
      return;
    }
    const next = [...props.certs];
    for (const pem of found) if (!next.includes(pem)) next.push(pem);
    setError(null);
    setPaste('');
    props.onChange(next);
  }

  async function fromFile(f: File | undefined): Promise<void> {
    if (!f) return;
    add(await readFileText(f));
    if (file) file.value = '';
  }

  function remove(i: number): void {
    props.onChange(props.certs.filter((_, j) => j !== i));
    queueMicrotask(() => {
      const buttons = list?.isConnected ? list.querySelectorAll<HTMLButtonElement>('button') : undefined;
      const target = buttons?.[Math.min(i, (buttons?.length ?? 1) - 1)];
      (target ?? file)?.focus();
    });
  }

  return (
    <div class={css.listEditor} id={props.id} role="group" aria-label={t('admin-sso-idp-certs')} aria-describedby={props.describedBy} data-testid="sso-certs">
      <Show when={props.certs.length > 0} fallback={<p class={css.listEmpty}>{t('admin-sso-cert-empty')}</p>}>
        <ul class={css.listItems} ref={list} aria-label={t('admin-sso-idp-certs')}>
          <For each={props.certs.map((pem) => certInfo(pem))}>
            {(c, i) => (
              <li class={css.listItem} data-testid="sso-cert">
                <span class={css.listText}>
                  <span dir="auto">{c.subject ?? t('admin-sso-cert-no-subject')}</span> <CertBadge cert={c} now={props.now} />
                </span>
                <button
                  type="button"
                  class="btn btn--ghost"
                  disabled={props.disabled}
                  aria-label={t('admin-sso-cert-remove', { subject: isolate(c.subject ?? t('admin-sso-cert-no-subject')) })}
                  onClick={() => remove(i())}
                >
                  <Icon name="close" />
                </button>
              </li>
            )}
          </For>
        </ul>
      </Show>
      <div class={shell.field}>
        <label class={shell.fieldLabel} for={`${own}-file`}>
          {t('admin-sso-cert-file')}
        </label>
        <input
          ref={file}
          id={`${own}-file`}
          class={css.fileInput}
          type="file"
          accept=".pem,.crt,.cer,application/x-pem-file,application/x-x509-ca-cert"
          disabled={props.disabled}
          onChange={(e) => void fromFile(e.currentTarget.files?.[0])}
        />
      </div>
      <div class={shell.field}>
        <label class={shell.fieldLabel} for={`${own}-paste`}>
          {t('admin-sso-cert-paste')}
        </label>
        <textarea
          id={`${own}-paste`}
          class={`${ux.control} ${css.mono}`}
          rows={3}
          spellcheck={false}
          disabled={props.disabled}
          value={paste()}
          aria-describedby={error() ? `${own}-error` : undefined}
          aria-invalid={error() || props.invalid ? true : undefined}
          onInput={(e) => setPaste(e.currentTarget.value)}
        />
        <div class={css.actions}>
          <button type="button" class="btn" disabled={props.disabled || paste().trim() === ''} onClick={() => add(paste())}>
            {t('admin-sso-cert-add')}
          </button>
        </div>
      </div>
      <Show when={error()}>
        {(msg) => (
          <p id={`${own}-error`} class={shell.fieldError} role="alert">
            <Icon name="alert" />
            <span>{msg()}</span>
          </p>
        )}
      </Show>
    </div>
  );
}
