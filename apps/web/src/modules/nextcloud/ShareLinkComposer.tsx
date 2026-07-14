// Large-attachment share-link composer (SPEC §18.4, plan §3 e7): create a public
// Nextcloud share link for a file, with the optional password + expiry controls.
// EXPORTED for e14 to wire into the composer's "share instead of attach" path (large
// attachments). The created link is inserted into the draft body by the caller.

import { createSignal, Show, type JSX } from 'solid-js';
import { NextcloudService, type Fetcher, type ShareLink } from './service.ts';
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
      setError('enter a password or turn password protection off');
      return;
    }
    if (withExpiry() && expiresAt() === '') {
      setError('pick an expiry date or turn expiry off');
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
      setError(e instanceof Error ? e.message : 'could not create the share link');
    } finally {
      setBusy(false);
    }
  }

  return (
    <div class={css.panel} data-module="nextcloud" aria-label="Create share link">
      <h2 class={css.heading}>Share link</h2>
      <p class={css.meta}>Create a public link to {props.path} instead of attaching the file.</p>

      <label class={css.check}>
        <input
          type="checkbox"
          aria-label="Protect with a password"
          checked={withPassword()}
          onChange={(e) => setWithPassword(e.currentTarget.checked)}
        />
        <span>Protect with a password</span>
      </label>
      <Show when={withPassword()}>
        <label class={css.field}>
          <span class={css.label}>Password</span>
          <input
            class={css.input}
            type="password"
            aria-label="Share password"
            value={password()}
            onInput={(e) => setPassword(e.currentTarget.value)}
          />
        </label>
      </Show>

      <label class={css.check}>
        <input
          type="checkbox"
          aria-label="Set an expiry date"
          checked={withExpiry()}
          onChange={(e) => setWithExpiry(e.currentTarget.checked)}
        />
        <span>Set an expiry date</span>
      </label>
      <Show when={withExpiry()}>
        <label class={css.field}>
          <span class={css.label}>Expires on</span>
          <input
            class={css.input}
            type="date"
            aria-label="Expiry date"
            value={expiresAt()}
            onInput={(e) => setExpiresAt(e.currentTarget.value)}
          />
        </label>
      </Show>

      <button type="button" class={css.button} disabled={busy()} onClick={() => void create()}>
        Create link
      </button>

      <Show when={link()}>
        {(l) => (
          <div class={css.field} data-testid="nc-share-result">
            <code class={css.linkBox} data-testid="nc-share-url">
              {l().url}
            </code>
            <p class={css.meta}>
              {l().passwordProtected ? 'Password-protected' : 'No password'}
              {l().expiresAt ? ` · expires ${l().expiresAt}` : ' · no expiry'}
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
