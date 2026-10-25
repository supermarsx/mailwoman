// Admin → UI plugins (rebuilt on the templates by f33 for t32-cfg §4.12):
// register, approve, enable, disable, grant and delete the sandboxed web-client
// plugins, over the `/admin/ui-plugins` routes (crates/mw-server/src/ui_plugins.rs).
//
// A list with the detail at `#/ui-plugins/<id>` (`ListDetailPage`). Registration
// (`#/ui-plugins/+register`) reads the manifest from a file, an address on this
// server or pasted text and previews it before anything is sent (`ManifestInput`).
// Disable and Delete ask first, naming the plugin.
//
// The list the server returns names each plugin's declared capabilities but not
// which of them are granted, so the detail offers the grant action and says that
// plainly instead of showing a grant state it does not have.

import { For, Show, createMemo, createResource, createSignal, createUniqueId, onMount, useContext, type JSX } from 'solid-js';
import { isolate, t, loadCatalog } from '../../../i18n';
import { ShellContext } from '../shell/context.ts';
import { AdminProblemError } from '../shell/adminApi.ts';
import { ListDetailPage } from '../shell/templates/ListDetailPage.tsx';
import { SettingsFormPage, type SettingsGroup, type SettingsLoad } from '../shell/templates/SettingsFormPage.tsx';
import { ConfirmDialog } from '../shell/ConfirmDialog.tsx';
import { Feedback } from '../shell/Feedback.tsx';
import { HelpNote } from '../shell/HelpNote.tsx';
import { ListEditor } from '../shell/ListEditor.tsx';
import { StatusBadge } from '../shell/StatusBadge.tsx';
import { adminDocsHref } from '../shell/docsLink.ts';
import { sectionPath, type AdminRoute } from '../shell/router.ts';
import type { Column } from '../shell/DataTable.tsx';
import * as shellCss from '../shell/shell.css.ts';
import * as ux from '../shell/ux.css.ts';
import * as css from '../shell/extras.css.ts';
import { EMPTY_MANIFEST, ManifestInput, type ManifestValue } from './ManifestInput.tsx';
import { createHttpUiPluginsApi, fileToBase64, NET_HOST_ALLOWLIST, UiPluginsApiError, type UiPluginInfo, type UiPluginsApi } from './service.ts';

export interface AdminUiPluginsProps {
  /** Tests inject a client; production defaults to the HTTP client. */
  api?: UiPluginsApi;
  route?: AdminRoute;
  navigate?: (path: string) => void;
}

const SECTION = 'ui-plugins';
const AREA = 'ui-plugins';
export const REGISTER = '+register';
const HOST = /^(?=.{1,253}$)[a-z0-9](?:[a-z0-9.-]{0,251}[a-z0-9])?(?::\d{1,5})?$/;

type Pending = { kind: 'disable' | 'delete'; plugin: UiPluginInfo };

export function AdminUiPlugins(props: AdminUiPluginsProps): JSX.Element {
  const api = props.api ?? createHttpUiPluginsApi();
  const shell = useContext(ShellContext);
  onMount(() => void loadCatalog('admin'));
  onMount(() => void loadCatalog('plugins'));

  const route = (): AdminRoute => props.route ?? shell?.router.route() ?? { section: SECTION, rest: [] };
  const navigate = (path: string): void => {
    if (props.navigate) props.navigate(path);
    else void shell?.router.navigate(path);
  };

  const [plugins, { refetch }] = createResource(() => api.list());
  // `latest` throws once the read failed, so it is read only through `loaded`.
  const loaded = (): UiPluginInfo[] | undefined => (plugins.error === undefined ? plugins.latest : undefined);
  const rows = (): UiPluginInfo[] => loaded() ?? [];
  const tableState = (): 'loading' | 'error' | 'ready' => (plugins.error !== undefined ? 'error' : loaded() === undefined ? 'loading' : 'ready');
  const find = (id: string): UiPluginInfo | undefined => rows().find((p) => p.id === id);
  const [query, setQuery] = createSignal('');
  const shown = createMemo(() => {
    const q = query().trim().toLowerCase();
    return rows().filter((p) => q === '' || p.id.toLowerCase().includes(q) || p.name.toLowerCase().includes(q));
  });
  const [refused, setRefused] = createSignal<string | null>(null);
  const [notice, setNotice] = createSignal<string | null>(null);
  const [pending, setPending] = createSignal<Pending | null>(null);

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

  const columns: Column<UiPluginInfo>[] = [
    {
      id: 'plugin',
      header: t('plugins-admin-col-plugin'),
      cell: (p) => (
        <>
          <span dir="auto">{p.name}</span> <span class={css.mono}>{t('admin-plugins-version', { version: p.version })}</span>
        </>
      ),
    },
    {
      id: 'signature',
      header: t('plugins-admin-ui-col-signature'),
      cell: (p) => (
        <StatusBadge tone={p.signed ? 'ok' : 'warn'} testId="ui-sig-chip">
          {p.signed ? t('admin-plugins-signed') : t('admin-plugins-unsigned')}
        </StatusBadge>
      ),
    },
    {
      id: 'state',
      header: t('plugins-admin-col-setup'),
      cell: (p) => (
        <span class={css.badgeRow}>
          <StatusBadge tone={p.approved ? 'ok' : 'off'}>{p.approved ? t('admin-plugins-approved') : t('plugins-admin-not-approved')}</StatusBadge>
          <StatusBadge tone={p.enabled ? 'ok' : 'off'} testId="ui-enabled-chip">
            {p.enabled ? t('admin-plugins-enabled') : t('plugins-admin-disabled')}
          </StatusBadge>
        </span>
      ),
    },
    { id: 'points', header: t('plugins-admin-ui-preview-points'), cell: (p) => <span class={css.mono}>{p.extensionPoints.join(', ')}</span> },
  ];

  const rowActions = (p: UiPluginInfo): JSX.Element => (
    <>
      <Show when={!p.approved}>
        <button type="button" class="btn" onClick={() => void change(() => api.approve(p.id), t('plugins-admin-ui-approved-done', { name: isolate(p.name) }))}>
          {t('plugins-admin-ui-approve')}
          <span class={shellCss.srOnly}> {p.name}</span>
        </button>
      </Show>
      <Show when={p.approved && !p.enabled}>
        <button type="button" class="btn" onClick={() => void change(() => api.enable(p.id), t('plugins-admin-enabled-done', { name: isolate(p.name) }))}>
          {t('admin-plugins-enable')}
          <span class={shellCss.srOnly}> {p.name}</span>
        </button>
      </Show>
      <Show when={p.enabled}>
        <button type="button" class="btn btn--ghost" onClick={() => setPending({ kind: 'disable', plugin: p })}>
          {t('admin-plugins-disable')}
          <span class={shellCss.srOnly}> {p.name}</span>
        </button>
      </Show>
    </>
  );

  const detail = (id: string): JSX.Element => {
    if (id === REGISTER) {
      return (
        <RegisterUiPlugin
          api={api}
          labelledBy={`${SECTION}-detail-title`}
          onRegistered={async (pid, unsigned) => {
            await refetch();
            setNotice(unsigned ? t('plugins-admin-ui-registered-unsigned', { id: isolate(pid) }) : t('plugins-admin-ui-registered', { id: isolate(pid) }));
            navigate(sectionPath(SECTION, [pid]));
          }}
        />
      );
    }
    return (
      <Show
        when={find(id)}
        fallback={plugins.loading ? <p role="status">{t('admin-shell-loading')}</p> : <Feedback kind="error" message={t('plugins-admin-not-found', { id: isolate(id) })} />}
      >
        {(p) => <UiPluginDetail plugin={p()} api={api} change={change} onDelete={() => setPending({ kind: 'delete', plugin: p() })} />}
      </Show>
    );
  };

  return (
    <div class={css.listEditor} data-screen="admin-ui-plugins">
      <Feedback kind="error" message={refused()} testId="ui-plugins-error" />
      <Feedback kind="success" message={notice()} testId="ui-plugins-notice" />
      <ListDetailPage<UiPluginInfo>
        title={t('plugins-admin-ui-title')}
        description={t('plugins-admin-ui-intro')}
        section={SECTION}
        route={route()}
        navigate={navigate}
        actions={
          <button type="button" class="btn btn--primary" onClick={() => navigate(sectionPath(SECTION, [REGISTER]))}>
            {t('plugins-admin-ui-register-heading')}
          </button>
        }
        table={{
          caption: t('plugins-admin-ui-title'),
          columns,
          rows: shown(),
          rowKey: (p) => p.id,
          state: tableState(),
          errorText: t('plugins-admin-ui-load-error'),
          onRetry: () => void refetch(),
          emptyText: rows().length === 0 ? t('plugins-admin-ui-empty') : t('plugins-admin-no-match'),
          ...(rows().length === 0 ? { emptyAction: { label: t('plugins-admin-ui-register-heading'), onClick: () => navigate(sectionPath(SECTION, [REGISTER])) } } : {}),
          search: { label: t('plugins-admin-ui-search'), value: query(), onSearch: setQuery, debounceMs: 0 },
          rowActions,
        }}
        detailTitle={(id) => (id === REGISTER ? t('plugins-admin-ui-register-heading') : (find(id)?.name ?? id))}
        renderDetail={detail}
        testId="admin-ui-plugins"
      />
      <ConfirmDialog
        open={pending() !== null}
        title={
          pending()?.kind === 'delete'
            ? t('plugins-admin-ui-delete-title', { name: isolate(pending()?.plugin.name ?? '') })
            : t('plugins-admin-ui-disable-title', { name: isolate(pending()?.plugin.name ?? '') })
        }
        body={pending()?.kind === 'delete' ? t('plugins-admin-ui-delete-body') : t('plugins-admin-ui-disable-body')}
        confirmLabel={pending()?.kind === 'delete' ? t('plugins-admin-ui-delete') : t('admin-plugins-disable')}
        onConfirm={async () => {
          const p = pending();
          if (p === null) return;
          if (p.kind === 'delete') {
            const ok = await change(() => api.remove(p.plugin.id), t('plugins-admin-ui-deleted-done', { name: isolate(p.plugin.name) }));
            if (ok && route().rest[0] === p.plugin.id) navigate(sectionPath(SECTION));
          } else {
            await change(() => api.disable(p.plugin.id), t('plugins-admin-disabled-done', { name: isolate(p.plugin.name) }));
          }
          setPending(null);
        }}
        onCancel={() => setPending(null)}
      />
    </div>
  );
}

type Change = (fn: () => Promise<unknown>, done?: string) => Promise<boolean>;

function UiPluginDetail(props: { plugin: UiPluginInfo; api: UiPluginsApi; change: Change; onDelete: () => void }): JSX.Element {
  const id = createUniqueId();
  const p = (): UiPluginInfo => props.plugin;
  const [hosts, setHosts] = createSignal<string[]>([]);

  return (
    <div class={css.listEditor} data-testid="ui-plugin-card" data-plugin-id={p().id}>
      <dl class={css.summary}>
        <dt class={css.summaryTerm}>{t('plugins-admin-ui-preview-id')}</dt>
        <dd class={`${css.summaryValue} ${css.mono}`}>{p().id}</dd>
        <dt class={css.summaryTerm}>{t('plugins-admin-register-version')}</dt>
        <dd class={css.summaryValue}>{p().version}</dd>
        <dt class={css.summaryTerm}>{t('plugins-admin-ui-preview-points')}</dt>
        <dd class={`${css.summaryValue} ${css.mono}`}>{p().extensionPoints.length > 0 ? p().extensionPoints.join(', ') : t('plugins-admin-hosts-empty')}</dd>
      </dl>

      <section class={css.panel} aria-labelledby={`${id}-grants`}>
        <h4 id={`${id}-grants`} class={css.panelTitle}>
          {t('plugins-admin-ui-grants-heading')}
        </h4>
        <HelpNote>
          <>
            {t('plugins-admin-ui-grants-note')}{' '}
            <a class={ux.learnMore} href={adminDocsHref(AREA, 'grants')} target="_blank" rel="noopener noreferrer">
              {t('admin-ux-learn-more')}
              <span class={shellCss.srOnly}> {t('admin-ux-learn-more-field', { label: t('plugins-admin-ui-grants-heading') })}</span>
            </a>
          </>
        </HelpNote>
        <Show when={p().capabilities.length > 0} fallback={<p class={css.panelText}>{t('plugins-admin-ui-no-capabilities')}</p>}>
          <ul class={css.listItems}>
            <For each={p().capabilities}>
              {(capability) => (
                <li class={css.listEditor}>
                  <Show when={capability === NET_HOST_ALLOWLIST}>
                    <label class={shellCss.fieldLabel} for={`${id}-hosts`}>
                      {t('plugins-admin-ui-grant-hosts', { capability, name: p().name })}
                    </label>
                    <ListEditor
                      id={`${id}-hosts`}
                      label={t('plugins-admin-ui-grant-hosts', { capability, name: p().name })}
                      items={hosts()}
                      onChange={setHosts}
                      lowerCase
                      mono
                      emptyText={t('plugins-admin-ui-hosts-none')}
                      validate={(h) => (HOST.test(h) ? null : t('plugins-admin-register-host-invalid', { host: isolate(h) }))}
                    />
                  </Show>
                  <div class={css.actions}>
                    <button
                      type="button"
                      class="btn"
                      aria-label={t('plugins-admin-ui-grant-for', { capability, name: p().name })}
                      onClick={() =>
                        void props.change(
                          () => props.api.grant(p().id, capability, capability === NET_HOST_ALLOWLIST ? { hosts: hosts() } : {}),
                          t('plugins-admin-ui-granted', { capability }),
                        )
                      }
                    >
                      {t('plugins-admin-ui-grant', { capability })}
                    </button>
                  </div>
                </li>
              )}
            </For>
          </ul>
        </Show>
      </section>

      <section class={css.panel} aria-labelledby={`${id}-remove`}>
        <h4 id={`${id}-remove`} class={css.panelTitle}>
          {t('plugins-admin-panel-remove')}
        </h4>
        <div class={css.actions}>
          <button type="button" class="btn btn--danger" onClick={() => props.onDelete()}>
            {t('plugins-admin-ui-delete')}
            <span class={shellCss.srOnly}> {p().name}</span>
          </button>
        </div>
      </section>
    </div>
  );
}

function RegisterUiPlugin(props: { api: UiPluginsApi; labelledBy: string; onRegistered: (id: string, unsigned: boolean) => void | Promise<void> }): JSX.Element {
  const docs = (fid: string): string => adminDocsHref(AREA, fid);
  const groups = (): SettingsGroup[] => [
    {
      id: 'register',
      heading: t('plugins-admin-ui-register-heading'),
      fields: [
        {
          id: 'manifest',
          label: t('plugins-admin-ui-manifest'),
          help: t('plugins-admin-ui-manifest-help'),
          docsHref: docs('manifest'),
          kind: 'text',
          validate: (v) => {
            const m = v as ManifestValue;
            if (m.parsed === null) return m.problem ?? t('plugins-admin-ui-manifest-needed');
            return null;
          },
          render: (ctx) => (
            <ManifestInput id={ctx.id} describedBy={ctx.describedBy} invalid={ctx.invalid} value={(ctx.value as ManifestValue | undefined) ?? EMPTY_MANIFEST} onChange={(v) => ctx.setValue(v)} />
          ),
        },
        {
          id: 'bundle',
          label: t('plugins-admin-ui-bundle'),
          help: t('plugins-admin-ui-bundle-hint'),
          docsHref: docs('bundle'),
          kind: 'text',
          validate: (v, all) => ((all.manifest as ManifestValue | undefined)?.parsed?.signed && !(v instanceof Blob) ? t('plugins-admin-ui-bundle-needed') : null),
          render: (ctx) => (
            <input
              id={ctx.id}
              class={css.fileInput}
              type="file"
              aria-describedby={ctx.describedBy}
              aria-invalid={ctx.invalid ? true : undefined}
              onChange={(e) => ctx.setValue(e.currentTarget.files?.[0] ?? null)}
            />
          ),
        },
        {
          id: 'allowUnsigned',
          label: t('plugins-admin-ui-allow-unsigned'),
          help: t('plugins-admin-ui-allow-unsigned-help'),
          docsHref: docs('unsigned'),
          kind: 'boolean',
          when: (v) => (v.manifest as ManifestValue | undefined)?.parsed?.signed === false,
        },
      ],
    },
  ];

  async function load(): Promise<SettingsLoad> {
    return { values: { manifest: EMPTY_MANIFEST, bundle: null, allowUnsigned: false } };
  }

  async function save(_changed: Record<string, unknown>, ctx: { values: Readonly<Record<string, unknown>> }): Promise<void> {
    const m = (ctx.values.manifest as ManifestValue).parsed;
    if (m === null) return;
    const file = ctx.values.bundle instanceof Blob ? ctx.values.bundle : null;
    const unsignedOk = !m.signed && ctx.values.allowUnsigned === true;
    try {
      const out = await props.api.register(m.raw, file === null ? null : await fileToBase64(file), unsignedOk);
      await props.onRegistered(out.id, !out.signed);
    } catch (e) {
      if (!(e instanceof UiPluginsApiError)) throw e;
      throw new AdminProblemError(422, { detail: t('plugins-admin-ui-refused', { detail: e.message }), field: '/manifest' }, null);
    }
  }

  return (
    <SettingsFormPage
      title={t('plugins-admin-ui-register-heading')}
      frame="none"
      labelledBy={props.labelledBy}
      groups={groups()}
      load={load}
      save={save}
      saveLabel={t('plugins-admin-ui-register')}
      testId="ui-plugin-register"
    />
  );
}

export default AdminUiPlugins;
