import { createResource, createSignal, For, Show, onCleanup, type JSX } from 'solid-js';
import type { ViewerProps } from '../contracts/viewer.ts';
import { apiFetch } from '../api/http.ts';
import { transportBase } from '../api/transport.ts';
import { bodyFrameDoc } from './sandbox.ts';
import { formatNumber, t } from '../i18n';
import { displayText, leetspeakEnabled } from '../i18n/presentation.ts';
import { leetspeakHtml } from './leetspeak.ts';

interface ExtractedFile { name: string; mime: string; size: number; dataBase64: string; kind: 'file' | 'message' }
interface Preview { format: 'eml' | 'tnef'; subject?: string; from?: string; to?: string; date?: string; text?: string; html?: string; attachments: ExtractedFile[]; warnings: string[] }
const MAX_BYTES = 4 * 1024 * 1024;

export function MessageAttachmentViewer(props: ViewerProps): JSX.Element {
  const urls = new Set<string>();
  onCleanup(() => { for (const url of urls) URL.revokeObjectURL(url); });
  const [failure, setFailure] = createSignal('');
  const [preview, { refetch }] = createResource(() => props.blobUrl, async url => {
    setFailure('');
    try {
      if (props.part.size > MAX_BYTES) throw new Error(t('common-message-preview-too-large'));
      const source = await apiFetch(url);
      if (!source.ok) throw new Error(t('common-message-preview-failed'));
      const bytes = new Uint8Array(await source.arrayBuffer());
      if (bytes.length > MAX_BYTES) throw new Error(t('common-message-preview-too-large'));
      let binary = '';
      for (let index = 0; index < bytes.length; index += 8192) binary += String.fromCharCode(...bytes.subarray(index, index + 8192));
      const format = /tnef/i.test(props.mime) || /winmail\.dat$/i.test(props.name) ? 'tnef' : 'eml';
      const response = await apiFetch(`${transportBase()}/api/attachments/preview`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ format, bytesBase64: btoa(binary) }) });
      if (!response.ok) throw new Error(t('common-message-preview-failed'));
      const body = await response.json() as { preview: Preview };
      if (!body.preview || !Array.isArray(body.preview.attachments) || body.preview.attachments.length > 64 || !Array.isArray(body.preview.warnings)) throw new Error(t('common-message-preview-failed'));
      return body.preview;
    } catch (cause) { setFailure(cause instanceof Error ? cause.message : t('common-message-preview-failed')); return null; }
  });
  function download(file: ExtractedFile): void {
    try {
      if (file.size > MAX_BYTES || file.dataBase64.length > 6 * 1024 * 1024) throw new Error();
      const binary = atob(file.dataBase64);
      if (binary.length > MAX_BYTES || binary.length !== file.size) throw new Error();
      const blob = new Blob([Uint8Array.from(binary, char => char.charCodeAt(0))], { type: 'application/octet-stream' });
      const url = URL.createObjectURL(blob); urls.add(url);
      const link = document.createElement('a'); link.href = url; link.download = [...file.name].map(char => char === '/' || char === '\\' || char.charCodeAt(0) < 32 ? '_' : char).join('') || 'attachment'; link.rel = 'noopener'; document.body.append(link); link.click(); link.remove();
    } catch { setFailure(t('common-message-preview-failed')); }
  }
  return <section class="mw-viewer__message" aria-label={props.name}>
    <Show when={preview.loading}><p role="status">{t('common-viewer-loading')}</p></Show>
    <Show when={failure()}><p role="alert">{failure()}</p><button class="btn" onClick={() => void refetch()}>{t('common-retry')}</button></Show>
    <Show when={preview()} keyed>{message => <>
      <h2>{displayText(message.subject || props.name)}</h2>
      <Show when={message.html?.startsWith('<pre data-mw-body-blocked="true">')}><p role="status">{t('privacy-body-blocked')}</p></Show>
      <p>{message.from}</p><p>{message.to}</p><p>{message.date}</p>
      <For each={message.warnings}>{warning => <p role="status">{warning}</p>}</For>
      <iframe class="mw-viewer__frame mw-viewer__frame--text" title={message.subject || props.name} sandbox="" srcdoc={message.html ? bodyFrameDoc('sanitized-no-media', { html: leetspeakEnabled() ? leetspeakHtml(message.html) : message.html }) : bodyFrameDoc('plain-text', { text: displayText(message.text ?? '') })} />
      <Show when={message.attachments.length}><h3>{t('common-message-preview-attachments')}</h3><ul><For each={message.attachments}>{file => <li><span>{file.name} ({t('common-message-preview-size-bytes', { count: formatNumber(file.size) })})</span> <button class="btn btn--ghost" onClick={() => download(file)}>{t('common-message-preview-download')}</button></li>}</For></ul></Show>
    </>}</Show>
  </section>;
}
export default MessageAttachmentViewer;
