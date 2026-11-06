// Picture options after the files are chosen: inline or attachment, a size preset
// (canvas downscale), EXIF strip, alt text. The admin default preset and the inline
// size limit come from the host.

import { createSignal, For, Show, type JSX } from 'solid-js';
import { t } from '../../../i18n/index.ts';
import { INLINE_IMAGE_TYPES } from '../inlineImages.ts';
import { InsertDialog } from './InsertDialog.tsx';
import { prepareImage, type ResizePreset } from './imageResize.ts';

const PRESETS: readonly ResizePreset[] = ['small', 'medium', 'large', 'original'];

export function PictureFileDialog(props: {
  files: File[];
  defaultPreset: ResizePreset;
  maxBytes: number;
  returnTo: HTMLElement | null;
  onClose: () => void;
  onInsert: (files: File[], alt: string, asAttachment: boolean) => void;
}): JSX.Element {
  const [preset, setPreset] = createSignal<ResizePreset>(props.defaultPreset);
  const [strip, setStrip] = createSignal(true);
  const [asAttachment, setAsAttachment] = createSignal(false);
  const [alt, setAlt] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const unsupported = (): File[] => props.files.filter((f) => !INLINE_IMAGE_TYPES.has(f.type));
  const insert = (): void => {
    setBusy(true);
    setError(null);
    void Promise.all(props.files.map((f) => (INLINE_IMAGE_TYPES.has(f.type) ? prepareImage(f, { preset: preset(), stripExif: strip() }) : Promise.resolve(f))))
      .then((prepared) => {
        const tooBig = prepared.find((f) => f.size > props.maxBytes);
        if (!asAttachment() && tooBig) { setError(t('ins-picture-too-big', { name: tooBig.name })); return; }
        props.onInsert(prepared, alt().trim(), asAttachment());
      })
      .catch(() => setError(t('ins-picture-failed')))
      .finally(() => setBusy(false));
  };
  return (
    <InsertDialog title={t('ins-picture-file')} testId="insert-picture-dialog" returnTo={props.returnTo} busy={busy()} onClose={props.onClose} onSubmit={insert} submitDisabled={props.files.length === 0}>
      <p style={{ margin: '0' }} data-testid="insert-picture-count">{t('ins-picture-count', { count: props.files.length })}</p>
      <label>{t('ins-picture-placement')}
        <select data-testid="insert-picture-placement" value={asAttachment() ? 'attach' : 'inline'} onChange={(e) => setAsAttachment(e.currentTarget.value === 'attach')}>
          <option value="inline">{t('ins-picture-inline')}</option>
          <option value="attach">{t('ins-picture-attachment')}</option>
        </select>
      </label>
      <label>{t('ins-picture-size')}
        <select data-testid="insert-picture-preset" value={preset()} onChange={(e) => setPreset(e.currentTarget.value as ResizePreset)}>
          <For each={PRESETS}>{(p) => <option value={p}>{t(`ins-picture-preset-${p}`)}</option>}</For>
        </select>
      </label>
      <label class="mw-insert-check"><input type="checkbox" data-testid="insert-picture-exif" checked={strip()} onChange={(e) => setStrip(e.currentTarget.checked)} />{t('ins-picture-exif')}</label>
      <Show when={!asAttachment()}>
        <label>{t('ins-alt')}<input data-testid="insert-picture-alt" value={alt()} placeholder={props.files.length === 1 ? props.files[0]!.name : ''} onInput={(e) => setAlt(e.currentTarget.value)} /></label>
      </Show>
      <Show when={unsupported().length > 0}><p class="mw-insert-warn">{t('ins-picture-unsupported', { name: unsupported()[0]!.name })}</p></Show>
      <Show when={error()}>{(m) => <p class="mw-insert-warn" role="alert">{m()}</p>}</Show>
    </InsertDialog>
  );
}
