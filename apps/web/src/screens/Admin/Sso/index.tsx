// Admin › Single sign-on (t9 e4, SPEC §18.3 / §19; rebuilt on the templates by
// f33 for t32-cfg §4.12).
//
// A list of the OIDC and SAML sign-in providers with their detail at
// `#/sso/<id>` (`ListDetailPage`): the table shows each provider's name, protocol,
// where it applies, whether it is on and, for SAML, whether its signing
// certificates are valid, expiring or expired. Opening one shows its form
// (`SsoEditor`); "Add a sign-in provider" opens an empty one at `#/sso/new`.
// Turning a provider off and deleting it each ask first, naming the provider.
//
// The server lists one scope per request (`GET /admin/sso?scope=`), so the
// screen asks for the deployment scope and for every domain the console knows
// (`GET /admin/domains` through the shared admin client, when the console
// provides one).
//
// It talks to `/admin/sso` through its own `SsoAdminApi` (tests inject a mock).

import { Show, createMemo, createResource, createSignal, onMount, useContext, type JSX } from 'solid-js';
import { createHttpSsoAdminApi, ssoMetadataPath, type SsoAdminApi, type SsoBackendInput, type SsoBackendRow } from '../../../modules/sso';
import { isolate, loadCatalog, t } from '../../../i18n';
import { AdminContext } from '../context.ts';
import { ShellContext } from '../shell/context.ts';
import { ListDetailPage } from '../shell/templates/ListDetailPage.tsx';
import { ConfirmDialog } from '../shell/ConfirmDialog.tsx';
import { Feedback, describeAdminError } from '../shell/Feedback.tsx';
import { FieldHistory } from '../shell/FieldHistory.tsx';
import { StatusBadge } from '../shell/StatusBadge.tsx';
import { sectionPath, type AdminRoute } from '../shell/router.ts';
import type { Column, FilterChip } from '../shell/DataTable.tsx';
import * as shellCss from '../shell/shell.css.ts';
import * as css from '../shell/extras.css.ts';
import { CertBadge } from './CertList.tsx';
import { certInfo, certState, type CertInfo } from './saml.ts';
import { SsoEditor } from './SsoEditor.tsx';

export interface AdminSsoProps {
  /** Tests inject a mock; production defaults to the same-origin HTTP client. */
  api?: SsoAdminApi;
  /** Domains to list and offer. Default: the console's admin client. */
  listDomains?: () => Promise<string[]>;
  /** The route and navigation. Default: the shell router. */
  route?: AdminRoute;
  navigate?: (path: string) => void;
  /** "Now" for certificate states (tests). */
  now?: Date;
}

const SECTION = 'sso';
const NEW = 'new';

/** The certificate of a SAML backend that needs attention first (expired, then soonest). */
export function worstCert(row: SsoBackendRow, now: Date = new Date()): CertInfo | null {
  if (row.config?.kind !== 'saml') return null;
  const certs = row.config.idpSigningCertsPem.map((p) => certInfo(p));
  if (certs.length === 0) return null;
  const rank = (c: CertInfo): number => ({ expired: 0, unreadable: 1, expiring: 2, 'not-yet-valid': 3, valid: 4 })[certState(c, now)];
  return [...certs].sort((a, b) => rank(a) - rank(b) || (a.notAfter?.getTime() ?? 0) - (b.notAfter?.getTime() ?? 0))[0] ?? null;
}

/** The scope as words: the whole deployment or one domain. */
function scopeText(scope: string): string {
  return scope.startsWith('domain:') ? t('admin-sso-scope-for-domain', { domain: isolate(scope.slice('domain:'.length)) }) : t('admin-sso-scope-deployment');
}

/** The `POST` body that changes only `enabled` of a stored row. */
function toggled(row: SsoBackendRow, enabled: boolean): SsoBackendInput | null {
  if (row.config === null || row.claimMap === null) return null;
  return { id: row.id, displayName: row.displayName, scope: row.scope, enabled, config: row.config, claimMap: row.claimMap };
}

type Pending = { kind: 'disable' | 'delete'; row: SsoBackendRow };

export function AdminSso(props: AdminSsoProps): JSX.Element {
  const api = props.api ?? createHttpSsoAdminApi();
  const shell = useContext(ShellContext);
  const admin = useContext(AdminContext);
  onMount(() => void loadCatalog('admin'));

  const route = (): AdminRoute => props.route ?? shell?.router.route() ?? { section: SECTION, rest: [] };
  const navigate = (path: string): void => {
    if (props.navigate) props.navigate(path);
    else void shell?.router.navigate(path);
  };

  const listDomains = props.listDomains ?? (admin ? async () => (await admin.api.listDomains()).map((d) => d.name) : async () => []);
  const [domains] = createResource(async () => {
    try {
      return await listDomains();
    } catch {
      return [];
    }
  });

  const [rows, { refetch }] = createResource(
    () => domains(),
    async (ds) => {
      const scopes = ['deployment', ...ds.map((d) => `domain:${d}`)];
      const lists = await Promise.all(scopes.map((s) => api.list(s)));
      return lists.flat();
    },
  );

  const [query, setQuery] = createSignal('');
  const [kinds, setKinds] = createSignal<string[]>([]);
  const [pending, setPending] = createSignal<Pending | null>(null);
  const [notice, setNotice] = createSignal<string | null>(null);
  const [failure, setFailure] = createSignal<string | null>(null);

  const all = (): SsoBackendRow[] => (rows.error === undefined ? (rows.latest ?? []) : []);
  const shown = createMemo(() => {
    const q = query().trim().toLowerCase();
    return all().filter(
      (r) =>
        (kinds().length === 0 || kinds().includes(r.config?.kind ?? '')) &&
        (q === '' || r.id.toLowerCase().includes(q) || r.displayName.toLowerCase().includes(q) || r.scope.toLowerCase().includes(q)),
    );
  });
  const find = (id: string): SsoBackendRow | undefined => all().find((r) => r.id === id);

  const filters = (): FilterChip[] => [
    { id: 'oidc', label: t('admin-sso-kind-oidc'), active: kinds().includes('oidc') },
    { id: 'saml', label: t('admin-sso-kind-saml'), active: kinds().includes('saml') },
  ];

  const columns: Column<SsoBackendRow>[] = [
    {
      id: 'name',
      header: t('admin-sso-col-name'),
      cell: (r) => (
        <>
          <span dir="auto">{r.displayName}</span> <span class={css.mono}>{r.id}</span>
        </>
      ),
    },
    { id: 'kind', header: t('admin-sso-kind'), cell: (r) => (r.config ? t(`admin-sso-kind-${r.config.kind}`) : t('admin-sso-config-unreadable')) },
    { id: 'scope', header: t('admin-sso-scope'), cell: (r) => scopeText(r.scope) },
    {
      id: 'state',
      header: t('admin-sso-col-state'),
      cell: (r) => (
        <span class={css.badgeRow}>
          <StatusBadge tone={r.enabled ? 'ok' : 'off'} status={r.enabled ? 'enabled' : 'disabled'}>
            {r.enabled ? t('admin-sso-badge-enabled') : t('admin-sso-badge-disabled')}
          </StatusBadge>
          <Show when={worstCert(r, props.now)}>{(c) => <CertBadge cert={c()} now={props.now} />}</Show>
        </span>
      ),
    },
  ];

  async function change(fn: () => Promise<void>, done: string): Promise<void> {
    setNotice(null);
    setFailure(null);
    await fn();
    setNotice(done);
    await refetch();
  }

  async function enable(row: SsoBackendRow): Promise<void> {
    const body = toggled(row, true);
    if (body === null) return;
    try {
      await change(() => api.save(body), t('admin-sso-enabled-done', { name: isolate(row.displayName) }));
    } catch (err) {
      setFailure(describeAdminError(err));
    }
  }

  async function confirmPending(): Promise<void> {
    const p = pending();
    if (p === null) return;
    if (p.kind === 'disable') {
      const body = toggled(p.row, false);
      if (body !== null) await change(() => api.save(body), t('admin-sso-disabled-done', { name: isolate(p.row.displayName) }));
    } else {
      await change(() => api.remove(p.row.id), t('admin-sso-deleted-done', { name: isolate(p.row.displayName) }));
      if (route().rest[0] === p.row.id) navigate(sectionPath(SECTION));
    }
    setPending(null);
  }

  async function save(input: SsoBackendInput): Promise<SsoBackendRow | null> {
    await api.save(input);
    await refetch();
    const stored = find(input.id) ?? null;
    if (route().rest[0] === NEW) navigate(sectionPath(SECTION, [input.id]));
    return stored;
  }

  const rowActions = (r: SsoBackendRow): JSX.Element => (
    <>
      <Show
        when={r.enabled}
        fallback={
          <button type="button" class="btn btn--ghost" disabled={r.config === null} onClick={() => void enable(r)}>
            {t('admin-sso-enable')}
            <span class={shellCss.srOnly}> {r.displayName}</span>
          </button>
        }
      >
        <button type="button" class="btn btn--ghost" onClick={() => setPending({ kind: 'disable', row: r })}>
          {t('admin-sso-disable')}
          <span class={shellCss.srOnly}> {r.displayName}</span>
        </button>
      </Show>
      <button type="button" class="btn btn--ghost" onClick={() => setPending({ kind: 'delete', row: r })}>
        {t('admin-sso-delete')}
        <span class={shellCss.srOnly}> {r.displayName}</span>
      </button>
    </>
  );

  const detail = (id: string): JSX.Element => {
    if (id === NEW) {
      return (
        <SsoEditor
          row={null}
          takenIds={all().map((r) => r.id)}
          domains={domains() ?? []}
          save={save}
          labelledBy={`${SECTION}-detail-title`}
          now={props.now}
        />
      );
    }
    return (
      <Show
        when={rows.error === undefined && rows.latest !== undefined}
        fallback={rows.error !== undefined ? <Feedback kind="error" message={t('admin-sso-load-error')} /> : <p role="status">{t('admin-shell-loading')}</p>}
      >
        <Show when={find(id)} fallback={<Feedback kind="error" message={t('admin-sso-not-found', { id: isolate(id) })} />}>
          {(row) => (
            <>
              <div class={css.actions}>
                <FieldHistory target={`sso:${row().id}`} updatedAt={row().updatedAt ?? null} />
                <Show when={row().config?.kind === 'saml'}>
                  <a class="btn btn--ghost" href={ssoMetadataPath(row().id)} target="_blank" rel="noopener noreferrer">
                    {t('admin-sso-metadata')}
                  </a>
                </Show>
              </div>
              <Show when={row().config === null}>
                <Feedback kind="error" message={t('admin-sso-config-unreadable-note')} />
              </Show>
              {/* renderDetail runs again for another id, so each provider gets a fresh form. */}
              <SsoEditor row={row()} takenIds={[]} domains={domains() ?? []} save={save} labelledBy={`${SECTION}-detail-title`} now={props.now} />
            </>
          )}
        </Show>
      </Show>
    );
  };

  return (
    <>
      <Feedback kind="success" message={notice()} testId="sso-notice" />
      <Feedback kind="error" message={failure()} />
      <ListDetailPage<SsoBackendRow>
        title={t('admin-sso-title')}
        description={t('admin-sso-intro')}
        section={SECTION}
        route={route()}
        navigate={navigate}
        actions={
          <button type="button" class="btn btn--primary" onClick={() => navigate(sectionPath(SECTION, [NEW]))}>
            {t('admin-sso-add')}
          </button>
        }
        table={{
          caption: t('admin-sso-title'),
          columns,
          rows: shown(),
          rowKey: (r) => r.id,
          state: rows.error !== undefined ? 'error' : rows.latest === undefined ? 'loading' : 'ready',
          errorText: t('admin-sso-load-error'),
          onRetry: () => void refetch(),
          emptyText: all().length === 0 ? t('admin-sso-empty') : t('admin-sso-no-match'),
          ...(all().length === 0 ? { emptyAction: { label: t('admin-sso-add'), onClick: () => navigate(sectionPath(SECTION, [NEW])) } } : {}),
          search: { label: t('admin-sso-search'), value: query(), onSearch: setQuery, debounceMs: 0 },
          filters: filters(),
          onToggleFilter: (id) => setKinds((k) => (k.includes(id) ? k.filter((x) => x !== id) : [...k, id])),
          rowActions,
        }}
        detailTitle={(id) => (id === NEW ? t('admin-sso-add') : (find(id)?.displayName ?? id))}
        renderDetail={detail}
        testId="admin-sso"
      />
      <ConfirmDialog
        open={pending() !== null}
        title={
          pending()?.kind === 'delete'
            ? t('admin-sso-delete-title', { name: isolate(pending()?.row.displayName ?? '') })
            : t('admin-sso-disable-title', { name: isolate(pending()?.row.displayName ?? '') })
        }
        body={pending()?.kind === 'delete' ? t('admin-sso-delete-body') : t('admin-sso-disable-body')}
        confirmLabel={pending()?.kind === 'delete' ? t('admin-sso-delete') : t('admin-sso-disable')}
        onConfirm={confirmPending}
        onCancel={() => setPending(null)}
      />
    </>
  );
}


export default AdminSso;
