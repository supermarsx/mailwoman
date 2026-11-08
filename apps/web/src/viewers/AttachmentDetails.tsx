// The "File details" button of an attachment tab and the dialog it opens: file
// name, type, size, the SHA-256 fingerprint (in groups, with copy buttons) and
// what the open viewer learned about the document (a PDF's pages, producer,
// creation date, whether a password was asked for). One component for every
// attachment viewer, so images, text, PDFs, audio and the rest show the same
// facts the same way instead of an inline block under the title.
import { createSignal, createUniqueId, For, Show, type JSX } from 'solid-js';
import { Icon } from '../components/Icon.tsx';
import { SecurityPopup } from '../components/SecurityPanel.tsx';
import { formatNumber, t } from '../i18n';
import { formatSize } from './attachments.ts';
import { FINGERPRINT_ALGORITHM, groupHex, type AttachmentDetails } from './attachmentDetails.ts';

export type { AttachmentDetails } from './attachmentDetails.ts';

export function AttachmentDetailsButton(props: { details: AttachmentDetails }): JSX.Element {
  const [open, setOpen] = createSignal(false);
  const id = createUniqueId();
  let button: HTMLButtonElement | undefined;
  const close = (): void => { setOpen(false); button?.focus(); };
  return <>
    <button type="button" ref={button} class="btn btn--ghost attachment-details__button" aria-haspopup="dialog" aria-expanded={open()} aria-controls={open() ? id : undefined}
      title={t('mail-file-details-tooltip')} data-testid="attachment-details-button" onClick={() => setOpen(true)}>
      <Icon name="info"/><span>{t('mail-file-details-button')}</span>
    </button>
    <Show when={open()}><DetailsDialog id={id} details={props.details} onClose={close}/></Show>
  </>;
}

function DetailsDialog(props: { id: string; details: AttachmentDetails; onClose: () => void }): JSX.Element {
  const titleId = `${props.id}-title`;
  const [status, setStatus] = createSignal('');
  const d = (): AttachmentDetails => props.details;
  async function copy(text: string): Promise<void> {
    try { await navigator.clipboard.writeText(text); setStatus(t('mail-file-copied')); } catch { setStatus(t('mail-file-copy-failed')); }
  }
  const row = (label: string, value: JSX.Element, copyText?: string, copyLabel?: string): JSX.Element => <>
    <dt>{label}</dt>
    <dd>{value}<Show when={copyText !== undefined}><button type="button" class="attachment-details__copy" aria-label={copyLabel} title={copyLabel} onClick={() => void copy(copyText!)}><Icon name="copy"/></button></Show></dd>
  </>;
  const extra = (): NonNullable<AttachmentDetails['extra']> => d().extra ?? {};
  const created = (): string | undefined => { const iso = extra().created; return iso === undefined ? undefined : new Date(iso).toLocaleString(); };
  return <SecurityPopup id={props.id} labelledBy={`${props.id}-title`} title={t('mail-file-details')} onClose={props.onClose}>
    <div class="attachment-details" data-testid="attachment-details" aria-describedby={`${titleId}-note`}>
      <dl class="attachment-details__list">
        {row(t('mail-file-name'), <span class="attachment-details__name">{d().name}</span>, d().name, t('mail-file-copy-name'))}
        {row(t('mail-file-type'), d().type)}
        {row(t('mail-file-extension'), d().extension || '—')}
        {row('MIME', d().mime || 'application/octet-stream')}
        {row(t('mail-file-detected'), d().detected || t('mail-file-unknown'))}
        {row(t('mail-file-size'), `${formatSize(d().size)} (${t('mail-file-size-bytes', { count: formatNumber(d().size) })})`)}
        <Show when={extra().pages !== undefined}>{row(t('mail-file-pages'), formatNumber(extra().pages!))}</Show>
        <Show when={extra().pdfVersion}>{row(t('mail-file-pdf-version'), extra().pdfVersion!)}</Show>
        <Show when={extra().producer}>{row(t('mail-file-producer'), extra().producer!)}</Show>
        <Show when={extra().creator}>{row(t('mail-file-creator'), extra().creator!)}</Show>
        <Show when={created()}>{row(t('mail-file-created'), created()!)}</Show>
        <Show when={extra().passwordRequired !== undefined}>{row(t('mail-file-encryption'), extra().passwordRequired ? t('mail-file-encryption-password') : t('mail-file-encryption-none'))}</Show>
        {row(t('mail-file-fingerprint'), <Show when={d().digest} fallback={t('mail-file-hash-unavailable')}>
          <span class="attachment-details__algorithm">{FINGERPRINT_ALGORITHM}</span>
          <code class="attachment-details__fingerprint" data-testid="attachment-fingerprint"><For each={groupHex(d().digest!)}>{(group, index) => <>{index() > 0 ? ' ' : ''}<span>{group}</span></>}</For></code>
        </Show>, d().digest ?? undefined, t('mail-file-copy-fingerprint'))}
      </dl>
      <section class="attachment-details__verify" aria-labelledby={`${titleId}-verify`}>
        <h4 id={`${titleId}-verify`}>{t('mail-file-verify-title')}</h4>
        <p>{t('mail-file-verify-help')}</p>
        <p id={`${titleId}-note`}>{t('mail-file-fingerprint-help')}</p>
      </section>
      <p class="attachment-details__status" role="status" aria-live="polite">{status()}</p>
    </div>
  </SecurityPopup>;
}
