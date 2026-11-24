// The small "Error reference" block a failure panel shows under its message:
// a short reference to quote to support, the state of the automatic report
// ("Reported" / "Reporting is off") and a button that copies the scrubbed
// details. It only reads a report handle; building the handle never throws
// (`observability/clientErrors.ts`), and everything here is guarded so a
// failure panel can never fail while it is being drawn.

import { createSignal, createEffect, onCleanup, Show, type JSX } from 'solid-js';
import { t } from '../i18n/index.ts';
import type { ReportHandle, ReportStatus } from '../observability/clientErrors.ts';
import * as css from './asyncState.css.ts';
import * as a11y from './mailA11y.css.ts';

export function ErrorReference(props: { report: ReportHandle | null | undefined }): JSX.Element {
  const [status, setStatus] = createSignal<ReportStatus>('pending');
  const [copied, setCopied] = createSignal(false);
  const [sending,setSending]=createSignal(false);
  const send=async():Promise<void>=>{if(sending())return;setSending(true);try{await props.report?.send();}catch{setStatus('failed');}finally{setSending(false);}};
  createEffect(()=>{
    const handle = props.report;
    if (handle == null) return;
    setStatus(handle.status());
    onCleanup(handle.subscribe(setStatus));
  });
  const copy = (): void => {
    try {
      const text = props.report?.details() ?? '';
      void navigator.clipboard.writeText(text).then(() => setCopied(true), () => setCopied(false));
    } catch {
      setCopied(false);
    }
  };
  return (
    <Show when={props.report}>
      {(handle) => (
        <div class={css.errorReference} data-testid="error-reference">
          <p class={css.errorReferenceLine}>
            <span>{t('ce-ref')}</span> <code data-testid="error-reference-id">{handle().ref}</code>
            <span class={css.errorReferenceState} role="status" data-report-status={status()}>{t(`ce-status-${status()}`)}</span>
          </p>
          <Show when={status()!=='reported'&&status()!=='off'}><button type="button" class={`${css.errorReferenceCopy} ${a11y.focusable}`} data-testid="error-send" disabled={sending()} onClick={()=>void send()}>{sending()?t('ce-status-pending'):t(status()==='failed'?'ce-send-retry':'ce-send')}</button></Show>
          <button type="button" class={`${css.errorReferenceCopy} ${a11y.focusable}`} data-testid="error-copy" onClick={copy}>
            {copied() ? t('ce-copied') : t('ce-copy')}
          </button>
          <p class={css.errorReferenceHelp}>{t('ce-report-help')}</p>
        </div>
      )}
    </Show>
  );
}
