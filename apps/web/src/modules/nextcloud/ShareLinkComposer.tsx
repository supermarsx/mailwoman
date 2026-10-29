import {InstanceSelector} from './InstanceSelector.tsx';
// Large-attachment share-link composer (SPEC §18.4, plan §3 e7): create a public
// Nextcloud share link for a file, with the optional password + expiry controls.
// Mounted by the composer for a file over the attachment size limit (t39 L7): with
// `file`, the local file is first uploaded to the linked account's "Mail
// attachments" folder in chunks, then shared. The caller inserts the link.

import { createSignal, onMount, Show, type JSX } from 'solid-js';
import { NextcloudService, type Fetcher, type ShareLink } from './service.ts';
import { t, loadCatalog, isolate } from '../../i18n';
import { toast } from '../../components/toast/index.ts';
import * as css from './styles.css.ts';
import { PasswordInput } from '../../components/PasswordInput.tsx';

export interface ShareLinkComposerProps {
  /** The Nextcloud file to share (server-relative path). Ignored with `file`. */
  path?: string;
  /** A local file to upload first (the large-attachment path). */
  file?: File;
  /** The created link (caller inserts it into the draft). */
  onCreated: (link: ShareLink) => void;
  /** Shown as a "Not now" button when given. */
  onDismiss?: () => void;
  fetcher?: Fetcher;
  service?: NextcloudService;
}

export function ShareLinkComposer(props: ShareLinkComposerProps): JSX.Element {
  onMount(() => void loadCatalog('nextcloud'));
  const service = (props.service ?? new NextcloudService(props.fetcher)).forAccount(undefined);
  const [ready,setReady]=createSignal(false);
  const [withPassword, setWithPassword] = createSignal(false);
  const [password, setPassword] = createSignal('');
  const [withExpiry, setWithExpiry] = createSignal(false);
  const [expiresAt, setExpiresAt] = createSignal('');
  const [link, setLink] = createSignal<ShareLink | null>(null);
  const [busy, setBusy] = createSignal(false);
  // Bytes sent so far while a local file uploads (null when not uploading).
  const [sent, setSent] = createSignal<number | null>(null);
  // Once stored, a retry (e.g. after a refused share) shares the same copy.
  let stored: string | null = null;
  const subject = (): string => props.file?.name ?? props.path ?? '';
  const percent = (): number => {
    const total = props.file?.size ?? 0;
    return total > 0 ? Math.min(100, Math.floor(((sent() ?? 0) * 100) / total)) : 0;
  };
  // A link may not expire in the past; Nextcloud refuses today in some setups.
  const tomorrow = (): string => {
    const at = new Date(Date.now() + 86_400_000);
    return `${at.getFullYear()}-${String(at.getMonth() + 1).padStart(2, '0')}-${String(at.getDate()).padStart(2, '0')}`;
  };
  // A missing password or date stays on the form; a refused request is a toast (t32 f39).
  const [error, setError] = createSignal('');

  async function create(): Promise<void> {
    if(!ready())return;
    setError('');
    if (withPassword() && password().trim() === '') {
      setError(t('nextcloud-error-need-password'));
      return;
    }
    if (withExpiry() && expiresAt() === '') {
      setError(t('nextcloud-error-need-expiry'));
      return;
    }
    setBusy(true);
    try {
      let path = props.path ?? '';
      if (props.file) {
        if (stored === null) {
          setSent(0);
          stored = (await service.uploadFile(props.file, setSent)).path;
        }
        path = stored;
      }
      const created = await service.createShareLink({
        path,
        ...(withPassword() ? { password: password() } : {}),
        ...(withExpiry() ? { expiresAt: expiresAt() } : {}),
      });
      setLink(created);
      props.onCreated(created);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : t('nextcloud-error-share-failed'));
    } finally {
      setSent(null);
      setBusy(false);
    }
  }

  return (
    <div class={css.panel} data-module="nextcloud" aria-label={t('nextcloud-share-panel-label')}>
      <InstanceSelector disabled={busy()} service={service} onChange={()=>{setReady(true);setLink(null);}}/>
      <h2 class={css.heading}>{t('nextcloud-share-title')}</h2>
      <p class={css.meta}>{t(props.file ? 'nextcloud-share-upload-intro' : 'nextcloud-share-intro', { path: isolate(subject()) })}</p>

      <label class={css.check}>
        <input
          type="checkbox"
          aria-label={t('nextcloud-protect-password')}
          checked={withPassword()}
          onChange={(e) => setWithPassword(e.currentTarget.checked)}
        />
        <span>{t('nextcloud-protect-password')}</span>
      </label>
      <Show when={withPassword()}>
        <label class={css.field}>
          <span class={css.label}>{t('nextcloud-password-label')}</span>
          <PasswordInput
            class={css.input}
            label={t('nextcloud-share-password')}
            autocomplete="new-password"
            value={password()}
            onInput={(e) => setPassword(e.currentTarget.value)}
            hideWhen={busy}
          />
        </label>
      </Show>

      <label class={css.check}>
        <input
          type="checkbox"
          aria-label={t('nextcloud-set-expiry')}
          checked={withExpiry()}
          onChange={(e) => setWithExpiry(e.currentTarget.checked)}
        />
        <span>{t('nextcloud-set-expiry')}</span>
      </label>
      <Show when={withExpiry()}>
        <label class={css.field}>
          <span class={css.label}>{t('nextcloud-expires-on')}</span>
          <input
            class={css.input}
            type="date"
            aria-label={t('nextcloud-expiry-date')}
            min={tomorrow()}
            value={expiresAt()}
            onInput={(e) => setExpiresAt(e.currentTarget.value)}
          />
        </label>
      </Show>

      <div class={css.actions}>
        <button type="button" class={css.button} disabled={busy()||!ready()||link()!==null} onClick={() => void create()}>
          {t(props.file ? 'nextcloud-upload-and-link' : 'nextcloud-create-link')}
        </button>
        <Show when={props.onDismiss}>
          {(dismiss) => (
            <button type="button" class={css.ghost} disabled={busy()} onClick={() => dismiss()()}>
              {t('nextcloud-share-not-now')}
            </button>
          )}
        </Show>
      </div>
      <Show when={sent() !== null}>
        <p class={css.meta} role="status" data-testid="nc-upload-progress">
          <progress max={100} value={percent()} aria-label={t('nextcloud-upload-progress-label')} />{' '}
          {t('nextcloud-upload-progress', { percent: percent() })}
        </p>
      </Show>

      <Show when={link()}>
        {(l) => (
          <div class={css.field} data-testid="nc-share-result">
            <code class={css.linkBox} data-testid="nc-share-url">
              {l().url}
            </code>
            <p class={css.meta}>
              {l().passwordProtected ? t('nextcloud-password-protected') : t('nextcloud-no-password')}
              {' · '}
              {l().expiresAt ? t('nextcloud-expires', { date: l().expiresAt ?? '' }) : t('nextcloud-no-expiry')}
            </p>
          </div>
        )}
      </Show>

      <Show when={error() !== ''}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>
    </div>
  );
}

export default ShareLinkComposer;
