// Admin → Plugins (SPEC §22; rebuilt on the templates by f33 for t32-cfg §4.12).
//
// A list of the engine plugins with their detail at `#/plugins/<id>`
// (`ListDetailPage`). The table shows each plugin's trust (first-party digest or
// approved third-party digest, signed or not), whether it runs (from the server's
// `loaded`, `restartRequired` and `notLoadedReason`: a plugin that is enabled but
// not running is shown as not running) and whether it is approved and enabled.
// Approve and Enable act at once; Disable asks first. "Register a plugin" opens
// the registration form at `#/plugins/+register`.
//
// The detail (`PluginDetail`) holds the grants, network and limits, the spam
// classifier address and test, the allow-unsigned switch and Uninstall. Below the
// list is the third-party allowlist (`AllowlistPanel`), sharing this screen's slice.
//
// A refused change is shown with the server's refusal code (`refusalText`).

import { Show, createEffect, createMemo, createSignal, onMount, useContext, type JSX } from 'solid-js';
import { createPluginsSlice, createHttpPluginsApi, type PluginsApi, type PluginsSlice, type PluginInfo } from '../../../state/slices/plugins.ts';
import { isolate, t, loadCatalog } from '../../../i18n';
import { ShellContext } from '../shell/context.ts';
import { ListDetailPage } from '../shell/templates/ListDetailPage.tsx';
import { ConfirmDialog } from '../shell/ConfirmDialog.tsx';
import { Feedback } from '../shell/Feedback.tsx';
import { StatusBadge } from '../shell/StatusBadge.tsx';
import { sectionPath, type AdminRoute } from '../shell/router.ts';
import type { Column, FilterChip } from '../shell/DataTable.tsx';
import * as shellCss from '../shell/shell.css.ts';
import * as css from '../shell/extras.css.ts';
import { AllowlistPanel } from './Allowlist.tsx';
import { PluginDetail } from './PluginDetail.tsx';
import { RegisterPlugin } from './RegisterPlugin.tsx';
import { StatusCell, TrustCell, refusalText } from './shared.tsx';

export { parseHosts, reasonText, refusalText } from './shared.tsx';

export interface AdminPluginsProps {
  /** Tests inject a slice or a client; production defaults to the HTTP client. */
  slice?: PluginsSlice;
  api?: PluginsApi;
  /** The route and navigation. Default: the shell router. */
  route?: AdminRoute;
  navigate?: (path: string) => void;
}

const SECTION = 'plugins';
/** The registration form's place in the list (`+` is not a plugin id character). */
export const REGISTER = '+register';

export function AdminPlugins(props: AdminPluginsProps): JSX.Element {
  const slice = props.slice ?? createPluginsSlice(props.api ?? createHttpPluginsApi());
  const shell = useContext(ShellContext);
  onMount(() => void loadCatalog('admin'));
  onMount(() => void loadCatalog('plugins'));
  onMount(() => void slice.load());

  const route = (): AdminRoute => props.route ?? shell?.router.route() ?? { section: SECTION, rest: [] };
  const navigate = (path: string): void => {
    if (props.navigate) props.navigate(path);
    else void shell?.router.navigate(path);
  };

  const [query, setQuery] = createSignal('');
  const [chips, setChips] = createSignal<string[]>([]);
  const [disabling, setDisabling] = createSignal<PluginInfo | null>(null);
  const [notice, setNotice] = createSignal<string | null>(null);
  // A notice belongs to the change that made it; a later refusal replaces it.
  createEffect(() => {
    if (slice.lastError() !== null) setNotice(null);
  });

  const shown = createMemo(() => {
    const q = query().trim().toLowerCase();
    return slice.plugins().filter(
      (p) =>
        (!chips().includes('third-party') || !p.firstParty) &&
        (!chips().includes('not-running') || !p.loaded) &&
        (q === '' || p.id.toLowerCase().includes(q) || p.name.toLowerCase().includes(q)),
    );
  });
  const find = (id: string): PluginInfo | undefined => slice.plugins().find((p) => p.id === id);

  const filters = (): FilterChip[] => [
    { id: 'third-party', label: t('plugins-admin-filter-third-party'), active: chips().includes('third-party') },
    { id: 'not-running', label: t('plugins-admin-filter-not-running'), active: chips().includes('not-running') },
  ];

  async function act(fn: () => Promise<boolean>, done: string): Promise<boolean> {
    setNotice(null);
    const ok = await fn();
    if (ok) setNotice(done);
    return ok;
  }

  const columns: Column<PluginInfo>[] = [
    {
      id: 'plugin',
      header: t('plugins-admin-col-plugin'),
      cell: (p) => (
        <>
          <span dir="auto">{p.name}</span> <span class={css.mono}>{t('admin-plugins-version', { version: p.version })}</span>
        </>
      ),
    },
    { id: 'trust', header: t('plugins-admin-col-trust'), cell: (p) => <TrustCell plugin={p} /> },
    { id: 'status', header: t('plugins-admin-col-status'), cell: (p) => <StatusCell plugin={p} /> },
    {
      id: 'setup',
      header: t('plugins-admin-col-setup'),
      cell: (p) => (
        <span class={css.badgeRow}>
          <StatusBadge tone={p.approved ? 'ok' : 'off'}>{p.approved ? t('admin-plugins-approved') : t('plugins-admin-not-approved')}</StatusBadge>
          <StatusBadge tone={p.enabled ? 'ok' : 'off'}>{p.enabled ? t('admin-plugins-enabled') : t('plugins-admin-disabled')}</StatusBadge>
        </span>
      ),
    },
  ];

  const rowActions = (p: PluginInfo): JSX.Element => (
    <>
      <Show when={!p.approved}>
        <button type="button" class="btn" onClick={() => void act(() => slice.approve(p.id), t('plugins-admin-approved-done', { name: isolate(p.name) }))}>
          {t('admin-plugins-approve')}
          <span class={shellCss.srOnly}> {p.name}</span>
        </button>
      </Show>
      <Show when={p.approved && !p.enabled}>
        <button type="button" class="btn" onClick={() => void act(() => slice.enable(p.id), t('plugins-admin-enabled-done', { name: isolate(p.name) }))}>
          {t('admin-plugins-enable')}
          <span class={shellCss.srOnly}> {p.name}</span>
        </button>
      </Show>
      <Show when={p.enabled}>
        <button type="button" class="btn btn--ghost" onClick={() => setDisabling(p)}>
          {t('admin-plugins-disable')}
          <span class={shellCss.srOnly}> {p.name}</span>
        </button>
      </Show>
    </>
  );

  const detail = (id: string): JSX.Element => {
    if (id === REGISTER) {
      return (
        <RegisterPlugin
          slice={slice}
          labelledBy={`${SECTION}-detail-title`}
          onRegistered={(pid) => {
            setNotice(t('plugins-admin-registered-done', { id: isolate(pid) }));
            // After the form has settled (no longer dirty), so leaving it does not ask.
            setTimeout(() => navigate(sectionPath(SECTION, [pid])), 0);
          }}
        />
      );
    }
    return (
      <Show when={find(id)} fallback={slice.loading() ? <p role="status">{t('admin-shell-loading')}</p> : <Feedback kind="error" message={t('plugins-admin-not-found', { id: isolate(id) })} />}>
        {(p) => (
          <PluginDetail
            plugin={p()}
            slice={slice}
            onNotice={setNotice}
            onUninstalled={() => {
              setNotice(t('plugins-admin-uninstalled-done', { id: isolate(id) }));
              navigate(sectionPath(SECTION));
            }}
          />
        )}
      </Show>
    );
  };

  return (
    <div class={css.listEditor} data-screen="admin-plugins">
      <Show when={slice.hasUnsignedLoaded()}>
        <Feedback kind="error" message={t('plugins-admin-unsigned-banner')} testId="unsigned-banner" />
      </Show>
      <Show when={slice.lastError()}>{(error) => <Feedback kind="error" message={refusalText(error())} testId="plugins-error" />}</Show>
      <Feedback kind="success" message={notice()} testId="plugins-notice" />
      <ListDetailPage<PluginInfo>
        title={t('admin-plugins-title')}
        description={t('plugins-admin-intro')}
        section={SECTION}
        route={route()}
        navigate={navigate}
        actions={
          <button type="button" class="btn btn--primary" onClick={() => navigate(sectionPath(SECTION, [REGISTER]))}>
            {t('plugins-admin-register-heading')}
          </button>
        }
        table={{
          caption: t('admin-plugins-title'),
          columns,
          rows: shown(),
          rowKey: (p) => p.id,
          state: slice.loadFailed() && slice.plugins().length === 0 ? 'error' : slice.loading() && slice.plugins().length === 0 ? 'loading' : 'ready',
          errorText: t('plugins-admin-load-error'),
          onRetry: () => void slice.load(),
          emptyText: slice.plugins().length === 0 ? t('admin-plugins-empty') : t('plugins-admin-no-match'),
          ...(slice.plugins().length === 0 ? { emptyAction: { label: t('plugins-admin-register-heading'), onClick: () => navigate(sectionPath(SECTION, [REGISTER])) } } : {}),
          search: { label: t('plugins-admin-search'), value: query(), onSearch: setQuery, debounceMs: 0 },
          filters: filters(),
          onToggleFilter: (fid) => setChips((c) => (c.includes(fid) ? c.filter((x) => x !== fid) : [...c, fid])),
          rowActions,
        }}
        detailTitle={(id) => (id === REGISTER ? t('plugins-admin-register-heading') : (find(id)?.name ?? id))}
        renderDetail={detail}
        testId="admin-plugins"
      />
      <Show when={slice.loadFailed() && slice.plugins().length > 0}>
        <Feedback kind="error" message={t('plugins-admin-load-error')} testId="plugins-load-error" />
      </Show>
      <AllowlistPanel slice={slice} />
      <ConfirmDialog
        open={disabling() !== null}
        title={t('plugins-admin-disable-title', { name: isolate(disabling()?.name ?? '') })}
        body={t('plugins-admin-disable-body')}
        confirmLabel={t('admin-plugins-disable')}
        onConfirm={async () => {
          const p = disabling();
          if (p) await act(() => slice.disable(p.id), t('plugins-admin-disabled-done', { name: isolate(p.name) }));
          setDisabling(null);
        }}
        onCancel={() => setDisabling(null)}
      />
    </div>
  );
}

export default AdminPlugins;
