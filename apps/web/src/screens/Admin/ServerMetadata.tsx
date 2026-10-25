// Admin › Server metadata (t14 26.14, SPEC §24 / §19; on the page frame by f33
// for t32-cfg §4.12).
//
// Mounts the write-capable RFC 5464 METADATA editor (`MetadataView`, shipped
// write-capable behind `canEdit`, mounted read-only in user Settings) with
// `canEdit=true` for one provisioned account the admin picks.
//
// ── Session sourcing (admin-gated account passthrough) ──
// `ServerMetadata/*` is per-account JMAP-scoped (`createAclClient(accountId, jmap)`
// puts `accountId` in every method call), but the admin console runs under its own
// session that carries no accountId and no JMAP session. So the admin selects a
// provisioned account (`AdminApi.listUsers()`), and the client is built against the
// same-origin JMAP transport (`createConfiguredClient().jmap` → `/jmap/api`). When
// that transport refuses the admin session the editor shows its own "could not
// load" state; this wrapper never fakes a session or hides the failure.
//
// States (UX11): the account list loading, failing (with Try again) or empty (with
// "Add a user", which opens the Users section), then the picker; the editor
// mounts only once an account is chosen.

import { createMemo, createResource, createSignal, createUniqueId, For, Show, onMount, useContext, type JSX } from 'solid-js';
import { MetadataView } from '../../modules/servermeta/MetadataView.tsx';
import { createAclClient, type AclClient, type JmapFn } from '../../api/acl-types.ts';
import { createConfiguredClient } from '../../api/transport.ts';
import { createHttpAdminApi, type AdminApi, type UserSummary } from '../../state/slices/admin.ts';
import { t, loadCatalog } from '../../i18n';
import { ShellContext } from './shell/context.ts';
import { AdminPage, PageState, type PageStateKind } from './shell/templates/PageFrame.tsx';
import { HelpNote } from './shell/HelpNote.tsx';
import { adminDocsHref } from './shell/docsLink.ts';
import { sectionPath } from './shell/router.ts';
import * as shell from './shell/shell.css.ts';
import * as ux from './shell/ux.css.ts';
import * as css from './shell/extras.css.ts';

export interface ServerMetadataProps {
  /**
   * Lists provisioned accounts for the account picker. Defaults to the same-origin
   * admin HTTP client; the console threads its shared client, tests a mock.
   */
  api?: AdminApi;
  /**
   * The JMAP transport the ACL/metadata client rides. Defaults to the configured
   * same-origin client (`/jmap/api`); tests inject a fake. See the session note above.
   */
  jmap?: JmapFn;
}

export function ServerMetadata(props: ServerMetadataProps): JSX.Element {
  const api = props.api ?? createHttpAdminApi();
  const jmap = props.jmap ?? createConfiguredClient().jmap;
  const shellCtx = useContext(ShellContext);
  const id = createUniqueId();

  onMount(() => void loadCatalog('admin'));

  const [users, { refetch }] = createResource(() => api.listUsers());
  const [accountId, setAccountId] = createSignal<string | null>(null);
  // `latest` throws once the read failed, so it is read only through these (and a
  // JSX ternary would read each condition on its own, so the state is built here).
  const loaded = (): UserSummary[] | undefined => (users.error === undefined ? users.latest : undefined);
  const list = (): UserSummary[] => loaded() ?? [];
  const pageState = (): PageStateKind => {
    if (users.error !== undefined) return { kind: 'error', error: users.error, onRetry: () => void refetch() };
    if (loaded() === undefined) return { kind: 'loading' };
    return {
      kind: 'empty',
      text: t('admin-servermeta-no-accounts'),
      ...(shellCtx ? { action: { label: t('admin-servermeta-add-user'), onClick: () => void shellCtx.router.navigate(sectionPath('users')) } } : {}),
    };
  };

  // The per-account client is rebuilt whenever the admin picks a different account;
  // null until one is selected (nothing to scope `ServerMetadata/*` to yet).
  const client = createMemo<AclClient | null>(() => {
    const a = accountId();
    return a === null ? null : createAclClient(a, jmap);
  });

  return (
    <AdminPage title={t('admin-servermeta-title')} description={t('admin-servermeta-intro')} titleId={`${id}-title`} testId="admin-servermeta">
      <Show when={list().length > 0} fallback={<PageState state={pageState()} />}>
        <div class={shell.field}>
          <label class={shell.fieldLabel} for={`${id}-account`}>
            {t('admin-servermeta-account')}
          </label>
          <select
            id={`${id}-account`}
            class={ux.control}
            value={accountId() ?? ''}
            aria-describedby={`${id}-account-help`}
            data-testid="admin-servermeta-account"
            onChange={(e) => setAccountId(e.currentTarget.value === '' ? null : e.currentTarget.value)}
          >
            <option value="">{t('admin-servermeta-select-option')}</option>
            <For each={list()}>
              {(u) => (
                <option value={u.accountId} selected={u.accountId === accountId()}>
                  {`${u.username}@${u.domain}`}
                </option>
              )}
            </For>
          </select>
          <HelpNote id={`${id}-account-help`}>
            <>
              {t('admin-servermeta-account-help')}{' '}
              <a class={ux.learnMore} href={adminDocsHref('server-metadata', 'account')} target="_blank" rel="noopener noreferrer">
                {t('admin-ux-learn-more')}
                <span class={shell.srOnly}> {t('admin-ux-learn-more-field', { label: t('admin-servermeta-account') })}</span>
              </a>
            </>
          </HelpNote>
        </div>
        <Show when={client()} fallback={<p class={css.panelText}>{t('admin-servermeta-select-prompt')}</p>}>
          {(c) => (
            <div class={css.panel}>
              <MetadataView client={c()} canEdit />
            </div>
          )}
        </Show>
      </Show>
    </AdminPage>
  );
}

export default ServerMetadata;
