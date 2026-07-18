// Admin › Server metadata (t14 26.14, plan §Workstream-2 E4, SPEC §24 / §19).
//
// Mounts the write-capable RFC 5464 METADATA editor (`MetadataView`, shipped in
// 26.13 fully write-capable behind `canEdit`, mounted READ-ONLY in user Settings)
// under `/admin` with `canEdit=true`. It follows the SSO local-flag mount pattern:
// the frozen `AdminSection` union is untouched — index.tsx layers this in with a
// `metaActive` signal, exactly like `ssoActive`.
//
// ── Session sourcing (HUMAN flag 3, resolved: admin-gated account passthrough) ──
// `ServerMetadata/*` is per-account JMAP-scoped (`createAclClient(accountId, jmap)`
// puts `accountId` in every method call), but the admin panel runs under a SEPARATE
// `mw_admin_session` that carries NO accountId and NO JMAP session. So the admin
// SELECTS a provisioned account (from `AdminApi.listUsers()`), and the client is
// built against the same-origin JMAP transport (`createConfiguredClient().jmap` →
// `/jmap/api`).
//
// What E-mount (Wave B) MUST provide for this to round-trip: the `/jmap/api`
// endpoint is cookie-authed on the MAILBOX session, not the admin one. E-mount
// adds the admin-gated passthrough so that an authenticated admin session may issue
// `ServerMetadata/get` / `ServerMetadata/set` (and `MailboxRights/*`) against the
// selected account's backend. Until that lands, the transport call 401s / throws
// and `MetadataView` surfaces its own honest `servermeta-load-failed` / op-failed
// state — this wrapper never fakes a session or silently swallows the failure.

import { createMemo, createResource, createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { MetadataView } from '../../modules/servermeta/MetadataView.tsx';
import { createAclClient, type AclClient, type JmapFn } from '../../api/acl-types.ts';
import { createConfiguredClient } from '../../api/transport.ts';
import { createHttpAdminApi, type AdminApi } from '../../state/slices/admin.ts';
import { t, loadCatalog } from '../../i18n';
import * as css from './admin.css.ts';

export interface ServerMetadataProps {
  /**
   * Lists provisioned accounts for the account picker. Defaults to the same-origin
   * admin HTTP client; index.tsx threads the panel's shared client, tests a mock.
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

  onMount(() => void loadCatalog('admin'));

  const [users] = createResource(() => api.listUsers());
  const [accountId, setAccountId] = createSignal<string | null>(null);

  // The per-account client is rebuilt whenever the admin picks a different account;
  // null until one is selected (nothing to scope `ServerMetadata/*` to yet).
  const client = createMemo<AclClient | null>(() => {
    const id = accountId();
    return id === null ? null : createAclClient(id, jmap);
  });

  return (
    <section class={css.section} aria-label={t('admin-servermeta-title')} data-testid="admin-servermeta">
      <h2 class={css.heading}>{t('admin-servermeta-title')}</h2>
      <p class={css.note}>{t('admin-servermeta-intro')}</p>

      <Show when={users.error as unknown}>
        <p class={css.error} role="alert">
          {t('admin-servermeta-load-error')}
        </p>
      </Show>

      <label class="field">
        <span>{t('admin-servermeta-account')}</span>
        <select
          value={accountId() ?? ''}
          data-testid="admin-servermeta-account"
          onChange={(e) => setAccountId(e.currentTarget.value === '' ? null : e.currentTarget.value)}
        >
          <option value="">{t('admin-servermeta-select-option')}</option>
          <For each={users() ?? []}>
            {(u) => <option value={u.accountId}>{`${u.username}@${u.domain}`}</option>}
          </For>
        </select>
      </label>

      <Show when={(users()?.length ?? 0) === 0 && !users.loading}>
        <p class={css.note}>{t('admin-servermeta-no-accounts')}</p>
      </Show>

      <Show when={client()} fallback={<p class={css.note}>{t('admin-servermeta-select-prompt')}</p>}>
        {(c) => <MetadataView client={c()} canEdit />}
      </Show>
    </section>
  );
}

export default ServerMetadata;
