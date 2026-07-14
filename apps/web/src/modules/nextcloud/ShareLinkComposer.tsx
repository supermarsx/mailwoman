// Large-attachment share-link composer (SPEC §18.4, plan §3 e7): create a public
// Nextcloud share link for a file, with the optional password + expiry controls.
// EXPORTED for e14 to wire into the composer's "share instead of attach" path (large
// attachments). The created link is inserted into the draft body by the caller.

import { createSignal, onMount, Show, type JSX } from 'solid-js';
import { NextcloudService, type Fetcher, type ShareLink } from './service.ts';
import { t, loadCatalog, isolate } from '../../i18n';
import * as css from './styles.css.ts';

export interface ShareLinkComposerProps {
  /** The Nextcloud file to share (server-relative path). */
  path: string;
  /** The created link (caller inserts it into the draft). */
  onCreated: (link: ShareLink) => void;
  fetcher?: Fetcher;
  service?: NextcloudService;
}

export function ShareLinkComposer(props: ShareLinkComposerProps): JSX.Element {
  onMount(() => void loadCatalog('nextcloud'));
  const service = props.service ?? new NextcloudService(props.fetcher);
  const [withPassword, setWithPassword] = createSignal(false);
  const [password, setPassword] = createSignal('');
  const [withExpiry, setWithExpiry] = createSignal(false);
  const [expiresAt, setExpiresAt] = createSignal('');
  const [link, setLink] = createSignal<ShareLink | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');

  async function create(): Promise<void> {
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
      const created = await service.createShareLink({
        path: props.path,
        ...(withPassword() ? { password: password() } : {}),
        ...(withExpiry() ? { expiresAt: expiresAt() } : {}),
      });
      setLink(created);
      props.onCreated(created);
    } catch (e) {
      setError(e instanceof Error ? e.message : t('nextcloud-error-share-failed'));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div class={css.panel} data-module="nextcloud" aria-label={t('nextcloud-share-panel-label')}>
      <h2 class={css.heading}>{t('nextcloud-share-title')}</h2>
      <p class={css.meta}>{t('nextcloud-share-intro', { path: isolate(props.path) })}</p>

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
          <input
            class={css.input}
            type="password"
            aria-label={t('nextcloud-share-password')}
            value={password()}
            onInput={(e) => setPassword(e.currentTarget.value)}
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
            value={expiresAt()}
            onInput={(e) => setExpiresAt(e.currentTarget.value)}
          />
        </label>
      </Show>

      <button type="button" class={css.button} disabled={busy()} onClick={() => void create()}>
        {t('nextcloud-create-link')}
      </button>

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
