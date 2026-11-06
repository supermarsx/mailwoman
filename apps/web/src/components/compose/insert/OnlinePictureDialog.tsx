// Insert a picture from a web address. The browser never contacts the remote
// site: the image is fetched by Mailwoman's anonymizing image proxy, and only if
// the proxy's policy allows it; the picture is then stored in the draft like any
// other inline picture. The recipient receives the picture itself, not a link.

import { createMemo, createSignal, type JSX } from 'solid-js';
import { apiFetch } from '../../../api/http.ts';
import { imageProxyUrl } from '../../../api/remote-images.ts';
import { t } from '../../../i18n/index.ts';
import { InsertDialog } from './InsertDialog.tsx';
import { INLINE_IMAGE_TYPES } from '../inlineImages.ts';

/** The address, or null when it is not a plain http(s) URL. */
export function onlinePictureUrl(raw: string): URL | null {
  try {
    const url = new URL(raw.trim());
    return (url.protocol === 'https:' || url.protocol === 'http:') && url.username === '' && url.password === '' ? url : null;
  } catch { return null; }
}

export function pictureNameFromUrl(url: URL, type: string): string {
  const last = decodeURIComponent(url.pathname.split('/').filter(Boolean).pop() ?? '').replace(/[^\w.-]+/g, '_');
  const ext = type.split('/')[1]?.replace('jpeg', 'jpg') ?? 'img';
  return last && /\.[a-z0-9]{2,4}$/i.test(last) ? last : `${last || 'picture'}.${ext}`;
}

export function OnlinePictureDialog(props: { returnTo: HTMLElement | null; maxBytes: number; fetcher?: typeof apiFetch; onClose: () => void; onPicked: (file: File) => void }): JSX.Element {
  const [raw, setRaw] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const url = createMemo(() => onlinePictureUrl(raw()));
  const go = (): void => {
    const u = url();
    if (!u) return;
    setBusy(true);
    setError(null);
    const get = props.fetcher ?? apiFetch;
    void get(imageProxyUrl(u.href), { credentials: 'same-origin' }).then(async (res) => {
      if (res.status === 403 || res.status === 451) { setError(t('ins-online-refused')); return; }
      if (!res.ok) { setError(t('ins-online-failed')); return; }
      const blob = await res.blob();
      const type = blob.type.split(';')[0]?.trim().toLowerCase() ?? '';
      if (!INLINE_IMAGE_TYPES.has(type)) { setError(t('ins-online-type')); return; }
      if (blob.size > props.maxBytes) { setError(t('ins-picture-too-big', { name: u.hostname })); return; }
      props.onPicked(new File([blob], pictureNameFromUrl(u, type), { type }));
    }).catch(() => setError(t('ins-online-failed'))).finally(() => setBusy(false));
  };
  return (
    <InsertDialog title={t('ins-online')} testId="insert-online-dialog" returnTo={props.returnTo} busy={busy()} onClose={props.onClose} onSubmit={go} submitLabel={t('ins-online-fetch')} submitDisabled={url() === null}>
      <label>{t('ins-online-url')}<input type="url" inputmode="url" data-testid="insert-online-url" placeholder="https://" value={raw()} onInput={(e) => setRaw(e.currentTarget.value)} /></label>
      <p class="mw-insert-warn" data-testid="insert-online-warning">{t('ins-online-warning')}</p>
      {error() && <p class="mw-insert-warn" role="alert" data-testid="insert-online-error">{error()}</p>}
    </InsertDialog>
  );
}
