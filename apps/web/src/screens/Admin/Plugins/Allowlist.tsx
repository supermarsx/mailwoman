// Admin → Plugins → Third-party allowlist (§7.2 / t15 26.15; rebuilt on the shell
// table and confirm dialog by f33).
//
// The trust surface for the only security-core loosening of 26.15: a component
// that is not first-party loads only after an administrator approves its exact
// SHA-256 digest. The first table lists the components present in the
// third-party plugin directory with the digest the SERVER computed over their
// bytes (the value being approved); the second lists the stored pins, approved
// and revoked, for oversight.
//
// Every change is a security action and asks first (`ConfirmDialog`), naming the
// component: approving shows the exact digest being trusted and asks for the id
// to be typed; revoking and uninstalling say what they change or delete. Copy is
// factual (memory: no-hype-wording).
//
// Two rules are stated rather than discovered by a refused request: high-power
// capabilities (account-backend) are never granted to a third-party plugin, and a
// component admitted by digest without a signature is the expected case.

import { Show, createSignal, onMount, type JSX } from 'solid-js';
import {
  createPluginsSlice,
  createHttpPluginsApi,
  HIGH_POWER_CAPABILITIES,
  type PluginsApi,
  type PluginsSlice,
  type AllowlistPresent,
  type AllowlistPin,
} from '../../../state/slices/plugins.ts';
import { formatDateTime, isolate, t, loadCatalog } from '../../../i18n';
import { DataTable, type Column } from '../shell/DataTable.tsx';
import { ConfirmDialog } from '../shell/ConfirmDialog.tsx';
import { Feedback } from '../shell/Feedback.tsx';
import { StatusBadge } from '../shell/StatusBadge.tsx';
import * as shellCss from '../shell/shell.css.ts';
import * as css from '../shell/extras.css.ts';

export interface AllowlistPanelProps {
  /** Tests / the admin shell inject a slice or a client; production defaults to HTTP. */
  slice?: PluginsSlice;
  api?: PluginsApi;
}

/** A pending confirmation: which action, and the row it targets. */
type Pending =
  | { readonly kind: 'approve'; readonly pluginId: string; readonly digestHex: string }
  | { readonly kind: 'revoke'; readonly pluginId: string; readonly digestHex: string }
  | { readonly kind: 'uninstall'; readonly pluginId: string };

/** The comma-joined high-power capability list shown in the not-grantable note. */
const HIGH_POWER_LABEL = HIGH_POWER_CAPABILITIES.join(', ');

function presentState(r: AllowlistPresent): JSX.Element {
  if (r.firstParty) {
    return (
      <StatusBadge tone="neutral" testId="allowlist-status">
        {t('admin-allowlist-status-firstparty')}
      </StatusBadge>
    );
  }
  return (
    <StatusBadge tone={r.approved ? 'ok' : 'off'} testId="allowlist-status">
      {r.approved ? t('admin-allowlist-status-approved') : t('admin-allowlist-status-pending')}
    </StatusBadge>
  );
}

export function AllowlistPanel(props: AllowlistPanelProps): JSX.Element {
  const slice = props.slice ?? createPluginsSlice(props.api ?? createHttpPluginsApi());

  onMount(() => void loadCatalog('admin'));
  onMount(() => void loadCatalog('plugins'));
  onMount(() => void slice.loadAllowlist());

  const [pending, setPending] = createSignal<Pending | null>(null);
  const [done, setDone] = createSignal<string | null>(null);

  async function runPending(): Promise<void> {
    const p = pending();
    if (p === null) return;
    // A failure rejects: the dialog stays open and says what failed.
    if (p.kind === 'approve') await slice.approveDigest(p.pluginId, p.digestHex);
    else if (p.kind === 'revoke') await slice.revokeDigest(p.pluginId, p.digestHex);
    else await slice.uninstall(p.pluginId);
    setDone(t(`admin-allowlist-${p.kind}-done`, { id: isolate(p.pluginId) }));
    setPending(null);
  }

  const presentColumns: Column<AllowlistPresent>[] = [
    { id: 'id', header: t('admin-allowlist-col-component'), cell: (r) => <span dir="auto">{r.pluginId}</span> },
    {
      id: 'digest',
      header: t('admin-allowlist-digest-label'),
      cell: (r) => (
        <span class={css.mono} data-testid="allowlist-digest">
          {r.computedDigest}
        </span>
      ),
    },
    { id: 'state', header: t('admin-allowlist-col-state'), cell: presentState },
  ];

  const presentActions = (r: AllowlistPresent): JSX.Element => (
    <>
      <Show when={!r.firstParty && !r.approved}>
        <button type="button" class="btn" onClick={() => setPending({ kind: 'approve', pluginId: r.pluginId, digestHex: r.computedDigest })}>
          {t('admin-allowlist-approve')}
          <span class={shellCss.srOnly}> {r.pluginId}</span>
        </button>
      </Show>
      <Show when={!r.firstParty && r.approved}>
        <button type="button" class="btn btn--ghost" onClick={() => setPending({ kind: 'revoke', pluginId: r.pluginId, digestHex: r.computedDigest })}>
          {t('admin-allowlist-revoke')}
          <span class={shellCss.srOnly}> {r.pluginId}</span>
        </button>
        <button type="button" class="btn btn--ghost" onClick={() => setPending({ kind: 'uninstall', pluginId: r.pluginId })}>
          {t('admin-allowlist-uninstall')}
          <span class={shellCss.srOnly}> {r.pluginId}</span>
        </button>
      </Show>
    </>
  );

  const pinColumns: Column<AllowlistPin>[] = [
    {
      id: 'name',
      header: t('admin-allowlist-col-component'),
      cell: (pin) => (
        <>
          <span dir="auto">{pin.name ?? pin.pluginId}</span>
          <Show when={pin.version}> {t('admin-plugins-version', { version: pin.version ?? '' })}</Show>
        </>
      ),
    },
    { id: 'digest', header: t('admin-allowlist-digest-label'), cell: (pin) => <span class={css.mono}>{pin.digestHex}</span> },
    {
      id: 'approved',
      header: t('admin-allowlist-col-approved'),
      cell: (pin) => t('admin-allowlist-pin-approved-by', { by: isolate(pin.approvedBy), at: formatDateTime(pin.approvedAt) }),
    },
    {
      id: 'state',
      header: t('admin-allowlist-col-state'),
      cell: (pin) => (
        <StatusBadge tone={pin.revoked ? 'off' : 'ok'} testId="allowlist-pin-state">
          {pin.revoked ? t('admin-allowlist-pin-revoked') : t('admin-allowlist-status-approved')}
        </StatusBadge>
      ),
    },
  ];

  const p = (): Pending | null => pending();

  return (
    <section class={css.panel} data-screen="admin-allowlist" aria-labelledby="admin-allowlist-title">
      <h3 id="admin-allowlist-title" class={css.panelTitle}>
        {t('admin-allowlist-title')}
      </h3>
      <p class={css.panelText}>{t('admin-allowlist-intro')}</p>
      <p class={css.panelText} data-testid="allowlist-unsigned-note">
        {t('admin-allowlist-unsigned-note')}
      </p>
      <p class={css.panelText} data-testid="allowlist-highpower-note">
        {t('admin-allowlist-highpower-note', { caps: HIGH_POWER_LABEL })}
      </p>
      <Show when={slice.allowlist().present.some((r) => r.firstParty)}>
        <p class={css.panelText} data-testid="allowlist-firstparty-note">
          {t('admin-allowlist-firstparty-note')}
        </p>
      </Show>
      <Feedback kind="success" message={done()} testId="allowlist-notice" />

      <h4 class={css.panelTitle}>{t('admin-allowlist-present-heading')}</h4>
      <DataTable<AllowlistPresent>
        caption={t('admin-allowlist-present-heading')}
        columns={presentColumns}
        rows={slice.allowlist().present}
        rowKey={(r) => `${r.pluginId}:${r.computedDigest}`}
        state={slice.allowlistLoadFailed() ? 'error' : slice.allowlistLoading() && slice.allowlist().present.length === 0 ? 'loading' : 'ready'}
        errorText={t('admin-allowlist-load-error')}
        onRetry={() => void slice.loadAllowlist()}
        emptyText={t('admin-allowlist-present-empty')}
        rowActions={presentActions}
      />

      <h4 class={css.panelTitle}>{t('admin-allowlist-pins-heading')}</h4>
      <DataTable<AllowlistPin>
        caption={t('admin-allowlist-pins-heading')}
        columns={pinColumns}
        rows={slice.allowlist().pins}
        rowKey={(pin) => `${pin.pluginId}:${pin.digestHex}`}
        state={slice.allowlistLoadFailed() ? 'error' : 'ready'}
        errorText={t('admin-allowlist-load-error')}
        emptyText={t('admin-allowlist-pins-empty')}
      />

      <ConfirmDialog
        open={p() !== null}
        title={t(p()?.kind === 'approve' ? 'admin-allowlist-approve-title' : p()?.kind === 'revoke' ? 'admin-allowlist-revoke-title' : 'admin-allowlist-uninstall-title')}
        body={t(p()?.kind === 'approve' ? 'admin-allowlist-approve-detail' : p()?.kind === 'revoke' ? 'plugins-admin-revoke-detail' : 'plugins-admin-uninstall-detail')}
        confirmLabel={t(p()?.kind === 'approve' ? 'admin-allowlist-approve-confirm' : p()?.kind === 'revoke' ? 'admin-allowlist-revoke-confirm' : 'admin-allowlist-uninstall-confirm')}
        {...(p()?.kind === 'uninstall' || p()?.kind === 'approve' ? { typedConfirmation: p()?.pluginId ?? '' } : {})}
        onConfirm={runPending}
        onCancel={() => setPending(null)}
      >
        <dl class={css.summary} data-testid="allowlist-dialog">
          <dt class={css.summaryTerm}>{t('admin-allowlist-col-component')}</dt>
          <dd class={css.summaryValue} dir="auto">
            {p()?.pluginId}
          </dd>
          <Show when={p()?.kind !== 'uninstall'}>
            <dt class={css.summaryTerm}>{t('admin-allowlist-digest-label')}</dt>
            <dd class={`${css.summaryValue} ${css.mono}`} data-testid="allowlist-dialog-digest">
              {(p() as { digestHex?: string } | null)?.digestHex ?? ''}
            </dd>
          </Show>
        </dl>
      </ConfirmDialog>
    </section>
  );
}

export default AllowlistPanel;
