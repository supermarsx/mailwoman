// Admin → UI plugins screen: register, approve, enable, disable, grant and delete
// the sandboxed web-client plugins, over the `/admin/ui-plugins` routes
// (crates/mw-server/src/ui_plugins.rs).
//
// The list the server returns names each plugin's declared capabilities but not
// which of them are granted, so this screen offers the grant action and says that
// plainly instead of showing a grant state it does not have.

import { createMemo, createResource, createSignal, For, onMount, Show, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../../i18n';
import { parseHosts } from '../Plugins/index.tsx';
import * as css from '../Plugins/styles.css.ts';
import {
  createHttpUiPluginsApi,
  fileToBase64,
  NET_HOST_ALLOWLIST,
  UiPluginsApiError,
  type UiPluginInfo,
  type UiPluginsApi,
} from './service.ts';

export interface AdminUiPluginsProps {
  /** Tests inject a client; production defaults to the HTTP client. */
  api?: UiPluginsApi;
}

export function AdminUiPlugins(props: AdminUiPluginsProps): JSX.Element {
  const api = props.api ?? createHttpUiPluginsApi();
  onMount(() => void loadCatalog('admin'));
  onMount(() => void loadCatalog('plugins'));

  const [plugins, { refetch }] = createResource(() => api.list());
  const rows = (): UiPluginInfo[] => (plugins.error === undefined ? (plugins.latest ?? []) : []);
  // Cards are keyed by plugin id, so a card survives the re-read after each change.
  const ids = createMemo(() => rows().map((p) => p.id), [] as string[], {
    equals: (a, b) => a.length === b.length && a.every((id, i) => id === b[i]),
  });
  // The last refusal (the server's own words) or the last grant that was stored.
  const [refused, setRefused] = createSignal<string | null>(null);
  const [notice, setNotice] = createSignal<string | null>(null);

  /** Run one change; show a refusal; re-read the list either way. */
  async function change(fn: () => Promise<unknown>, done?: string): Promise<boolean> {
    setRefused(null);
    setNotice(null);
    try {
      await fn();
      if (done !== undefined) setNotice(done);
      return true;
    } catch (e) {
      if (!(e instanceof UiPluginsApiError)) throw e;
      setRefused(t('plugins-admin-ui-refused', { detail: e.message }));
      return false;
    } finally {
      await refetch();
    }
  }

  return (
    <section class={css.screen} data-screen="admin-ui-plugins" aria-label={t('plugins-admin-ui-title')}>
      <div>
        <h2 class={css.heading}>{t('plugins-admin-ui-title')}</h2>
        <p class={css.prose}>{t('plugins-admin-ui-intro')}</p>
      </div>

      <Show when={refused()}>
        {(message) => (
          <p class={css.error} role="alert" data-testid="ui-plugins-error">
            {message()}
          </p>
        )}
      </Show>
      <Show when={notice()}>
        {(message) => (
          <p class={css.statusLine} role="status" data-testid="ui-plugins-notice">
            {message()}
          </p>
        )}
      </Show>

      <RegisterForm api={api} change={change} onInvalid={() => setRefused(t('plugins-admin-ui-manifest-invalid'))} />

      <Show when={plugins.error !== undefined}>
        <p class={css.error} role="alert">
          {t('plugins-admin-ui-load-error')}
        </p>
      </Show>
      <Show when={plugins.error === undefined && !plugins.loading && rows().length === 0}>
        <p class={css.meta} data-testid="ui-plugins-empty">
          {t('plugins-admin-ui-empty')}
        </p>
      </Show>

      <ul class={css.list}>
        <For each={ids()}>
          {(id) => (
            <Show when={rows().find((p) => p.id === id)}>
              {(plugin) => <UiPluginCard plugin={plugin()} api={api} change={change} />}
            </Show>
          )}
        </For>
      </ul>
    </section>
  );
}

type Change = (fn: () => Promise<unknown>, done?: string) => Promise<boolean>;

function RegisterForm(props: { api: UiPluginsApi; change: Change; onInvalid: () => void }): JSX.Element {
  const [manifest, setManifest] = createSignal('');
  const [bundle, setBundle] = createSignal<File | null>(null);
  const [allowUnsigned, setAllowUnsigned] = createSignal(false);

  async function submit(e: Event): Promise<void> {
    e.preventDefault();
    let parsed: unknown;
    try {
      parsed = JSON.parse(manifest());
    } catch {
      props.onInvalid();
      return;
    }
    const file = bundle();
    const encoded = file === null ? null : await fileToBase64(file);
    if (await props.change(() => props.api.register(parsed, encoded, allowUnsigned()))) {
      setManifest('');
      setBundle(null);
      setAllowUnsigned(false);
    }
  }

  return (
    <form class={css.form} aria-label={t('plugins-admin-ui-register-heading')} onSubmit={(e) => void submit(e)}>
      <h3 class={css.subheading}>{t('plugins-admin-ui-register-heading')}</h3>
      <label class={css.field}>
        <span>{t('plugins-admin-ui-manifest')}</span>
        <textarea
          class={css.input}
          rows={6}
          required
          value={manifest()}
          onInput={(e) => setManifest(e.currentTarget.value)}
        />
      </label>
      <label class={css.field}>
        <span>{t('plugins-admin-ui-bundle')}</span>
        <input type="file" onChange={(e) => setBundle(e.currentTarget.files?.[0] ?? null)} />
        <span class={css.meta}>{t('plugins-admin-ui-bundle-hint')}</span>
      </label>
      <label class={css.check}>
        <input type="checkbox" checked={allowUnsigned()} onChange={(e) => setAllowUnsigned(e.currentTarget.checked)} />
        <span>{t('plugins-admin-ui-allow-unsigned')}</span>
      </label>
      <div class={css.row}>
        <button type="submit" class={css.button}>
          {t('plugins-admin-ui-register')}
        </button>
      </div>
    </form>
  );
}

function UiPluginCard(props: { plugin: UiPluginInfo; api: UiPluginsApi; change: Change }): JSX.Element {
  const p = (): UiPluginInfo => props.plugin;
  const api = props.api;
  const [hosts, setHosts] = createSignal('');
  const [confirmDelete, setConfirmDelete] = createSignal(false);

  const grant = (capability: string): Promise<boolean> =>
    props.change(
      () => api.grant(p().id, capability, capability === NET_HOST_ALLOWLIST ? { hosts: parseHosts(hosts()) } : {}),
      t('plugins-admin-ui-granted', { capability }),
    );

  return (
    <li class={css.card} data-plugin-id={p().id} data-testid="ui-plugin-card">
      <div class={css.cardHead}>
        <div>
          <p class={css.title}>
            <span dir="auto">{p().name}</span>{' '}
            <span class={css.meta}>{t('admin-plugins-version', { version: p().version })}</span>
          </p>
          <div class={css.row}>
            <span class={`${css.chip} ${p().signed ? css.signedChip : css.unsignedChip}`} data-testid="ui-sig-chip">
              {p().signed ? t('admin-plugins-signed') : t('admin-plugins-unsigned')}
            </span>
            <Show when={p().approved}>
              <span class={css.chip}>{t('admin-plugins-approved')}</span>
            </Show>
            <Show when={p().enabled}>
              <span class={css.chip} data-testid="ui-enabled-chip">
                {t('admin-plugins-enabled')}
              </span>
            </Show>
          </div>
        </div>
        <div class={css.row}>
          <Show when={!p().approved}>
            <button type="button" class={css.button} onClick={() => void props.change(() => api.approve(p().id))}>
              {t('plugins-admin-ui-approve')}
            </button>
          </Show>
          <Show when={p().approved && !p().enabled}>
            <button type="button" class={css.button} onClick={() => void props.change(() => api.enable(p().id))}>
              {t('admin-plugins-enable')}
            </button>
          </Show>
          <Show when={p().enabled}>
            <button type="button" class={css.danger} onClick={() => void props.change(() => api.disable(p().id))}>
              {t('admin-plugins-disable')}
            </button>
          </Show>
        </div>
      </div>

      <Show when={p().extensionPoints.length > 0}>
        <p class={css.limits}>{t('plugins-admin-ui-extension-points', { points: p().extensionPoints.join(', ') })}</p>
      </Show>

      <Show when={p().capabilities.length > 0}>
        <div class={css.field}>
          <p class={css.subheading}>{t('plugins-admin-ui-grants-heading')}</p>
          <span class={css.meta}>{t('plugins-admin-ui-grants-note')}</span>
          <For each={p().capabilities}>
            {(capability) => (
              <div class={css.row}>
                <Show when={capability === NET_HOST_ALLOWLIST}>
                  <input
                    class={css.input}
                    aria-label={t('plugins-admin-ui-grant-hosts', { capability, name: p().name })}
                    title={t('plugins-admin-ui-grant-hosts-hint')}
                    value={hosts()}
                    onInput={(e) => setHosts(e.currentTarget.value)}
                  />
                </Show>
                <button
                  type="button"
                  class={css.ghost}
                  aria-label={t('plugins-admin-ui-grant-for', { capability, name: p().name })}
                  onClick={() => void grant(capability)}
                >
                  {t('plugins-admin-ui-grant', { capability })}
                </button>
              </div>
            )}
          </For>
        </div>
      </Show>

      <div class={css.row}>
        <Show
          when={confirmDelete()}
          fallback={
            <button type="button" class={css.ghost} onClick={() => setConfirmDelete(true)}>
              {t('plugins-admin-ui-delete')}
            </button>
          }
        >
          <button type="button" class={css.danger} onClick={() => void props.change(() => api.remove(p().id))}>
            {t('plugins-admin-ui-delete-confirm')}
          </button>
          <button type="button" class={css.ghost} onClick={() => setConfirmDelete(false)}>
            {t('common-cancel')}
          </button>
        </Show>
      </div>
    </li>
  );
}

export default AdminUiPlugins;
