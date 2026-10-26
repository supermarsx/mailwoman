import { createResource, For, Show, onCleanup, type JSX } from 'solid-js';
import { apiFetch } from '../api/http.ts';
import { Icon } from '../components/Icon.tsx';
import { AttachmentViewer } from './AttachmentViewer.tsx';
import { inspectFile } from './fileInfo.ts';
import type { StripItem } from './ThumbnailStrip.tsx';
import { formatSize } from './attachments.ts';
import { t } from '../i18n';
import './attachment-tab.css';

export interface AttachmentTabItem extends StripItem { downloadUrl: string }
export function AttachmentTab(props: { item: AttachmentTabItem }): JSX.Element {
  const abort = new AbortController();
  let objectUrl: string | undefined;
  onCleanup(() => { abort.abort(); if (objectUrl) URL.revokeObjectURL(objectUrl); });
  const [file, { refetch }] = createResource(async () => {
    const response = await apiFetch(props.item.downloadUrl, { signal: abort.signal });
    if (!response.ok) throw new Error(t('mail-attachment-failed'));
    const blob = await response.blob();
    const header = new Uint8Array(await blob.slice(0, 64).arrayBuffer());
    const info = inspectFile(props.item.name, props.item.mime, header);
    // WebCrypto hashes in memory; retain a fixed cap rather than doubling an
    // arbitrarily large media allocation just to populate the details panel.
    const digest = blob.size <= 64 * 1024 * 1024 && globalThis.crypto?.subtle
      ? [...new Uint8Array(await crypto.subtle.digest('SHA-256', await blob.arrayBuffer()))].map(byte => byte.toString(16).padStart(2, '0')).join('') : null;
    abort.signal.throwIfAborted();
    if (objectUrl) URL.revokeObjectURL(objectUrl);
    objectUrl = URL.createObjectURL(blob);
    return { url: objectUrl, size: blob.size, digest, info };
  });
  const ready = () => file.state === 'ready' ? file.latest : undefined;
  const declared = () => inspectFile(props.item.name, props.item.mime);
  return <section class="attachment-tab" aria-label={props.item.name}>
    <header class="attachment-tab__header"><Icon name={declared().icon}/><h2>{props.item.name}</h2></header>
    <Show when={file.loading}><p role="status">{t('mail-attachment-loading')}</p></Show>
    <Show when={file.state === 'errored'}><p role="alert">{t('mail-attachment-failed')}</p><button class="btn" onClick={() => void refetch()}>{t('common-retry')}</button></Show>
    <Show when={ready()}>{loaded => <>
      <details class="attachment-tab__details"><summary>{t('mail-file-details')}</summary><dl>
        <dt>{t('mail-file-type')}</dt><dd>{loaded().info.description}</dd>
        <dt>{t('mail-file-extension')}</dt><dd>{loaded().info.extension || '—'}</dd>
        <dt>MIME</dt><dd>{props.item.mime || 'application/octet-stream'}</dd>
        <dt>{t('mail-file-detected')}</dt><dd>{loaded().info.detected || t('mail-file-unknown')}</dd>
        <dt>{t('mail-file-size')}</dt><dd>{formatSize(loaded().size)} ({loaded().size.toLocaleString()} bytes)</dd>
        <dt>SHA-256</dt><dd><code>{loaded().digest || t('mail-file-hash-unavailable')}</code></dd>
      </dl><p>{t('mail-file-fingerprint-help')}</p></details>
      <For each={loaded().info.warnings}>{warning => <p class="attachment-tab__warning" role="status"><Icon name="alert"/>{t(`mail-file-warning-${warning}`)}</p>}</For>
      <a class="btn btn--ghost attachment-tab__download" href={loaded().url} download={props.item.name.replace(/[\\/]/g, '_')} rel="noopener">{t('common-message-preview-download')}</a>
      <Show when={!loaded().info.blockPreview} fallback={<p>{t('mail-file-preview-blocked')}</p>}>
        <div class="attachment-tab__viewer"><AttachmentViewer part={{ partId: null, blobId: props.item.blobId, size: loaded().size, type: props.item.mime }} blobUrl={loaded().url} mime={props.item.mime} name={props.item.name}/></div>
      </Show>
    </>}</Show>
  </section>;
}
