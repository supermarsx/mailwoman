// Active-session listing + revocation (t16 e15, SPEC §19 — S11).
//
// Lists the account's live sessions (metadata only; raw ids never leave the
// server) and revokes them: one by handle, or "everywhere else" (all but the
// current). Rides the `crates/mw-server/src/twofa_routes.rs` session routes.

import { createResource, createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { t, loadCatalog, isolate } from '../../i18n';
import { SettingsService } from './service.ts';
import type { SessionMeta } from './types.ts';
import { toast } from '../../components/toast/index.ts';
import { Icon } from '../../components/Icon.tsx';
import * as table from './sessions.css.ts';
import * as css from './styles.css.ts';

export interface SessionsProps {
  service?: SettingsService;
}

export function Sessions(props: SessionsProps): JSX.Element {
  const service = props.service ?? new SettingsService();
  onMount(() => void loadCatalog('settings'));

  const [sessions, { refetch }] = createResource<SessionMeta[]>(() => service.sessions());
  const [query, setQuery] = createSignal('');
  const [busy, setBusy] = createSignal(false);

  // A failed action is a toast (t32 f39).
  function fail(e: unknown): void {
    toast.error(e instanceof Error ? e.message : t('settings-sessions-error'));
  }

  async function revoke(handle: string): Promise<void> {
    setBusy(true);
    try {
      await service.revokeSession(handle);
      await refetch();
    } catch (e) {
      fail(e);
    } finally {
      setBusy(false);
    }
  }

  async function revokeOthers(): Promise<void> {
    setBusy(true);
    try {
      await service.revokeOtherSessions();
      await refetch();
    } catch (e) {
      fail(e);
    } finally {
      setBusy(false);
    }
  }

  const loaded = (): SessionMeta[] => sessions.error ? [] : sessions.latest ?? [];
  const others = (): number => loaded().filter((s) => !s.current).length;

  return (
    <section class={css.section} aria-label={t('settings-sessions-title')}>
      <h2 class={css.heading}>{t('settings-sessions-title')}</h2>
      <p class={css.prose}>{t('settings-sessions-intro')}</p>

      <label class={css.field}>{t('settings-sessions-search')}<input class={css.input} type="search" value={query()} onInput={(event) => setQuery(event.currentTarget.value)} /></label>
      <Show when={sessions.error}><p role="alert">{t('settings-sessions-error')}</p></Show>
      <div class={table.wrap} aria-busy={sessions.loading}>
        <table class={table.table} data-testid="session-list"><caption>{t('settings-sessions-title')}</caption>
          <thead><tr><th>{t('settings-sessions-account')}</th><th>{t('settings-sessions-device')}</th><th>{t('settings-sessions-location')}</th><th>{t('settings-sessions-activity')}</th><th>{t('settings-sessions-actions')}</th></tr></thead>
          <tbody><For each={loaded().filter((row) => [row.username, row.ip, row.deviceLabel, row.userAgent].some((value) => value?.toLowerCase().includes(query().trim().toLowerCase())))}>{(s) => <tr>
            <td>{isolate(s.username)}</td><td>{s.deviceLabel || s.clientType || '—'} {s.clientVersion}<p class={css.meta}>{s.userAgent || '—'}</p></td>
            <td>{s.ip || '—'}<p class={css.meta}>{s.country || t('settings-sessions-location-unavailable')}</p></td>
            <td>{t('settings-sessions-last-seen', { when: s.lastSeen })}<p class={css.meta}>{t('security-center-first-seen', { when: s.createdAt })}</p></td>
            <td><Show when={s.current} fallback={<button type="button" class={css.danger} disabled={busy()} aria-label={t('settings-sessions-revoke-device', { device: s.deviceLabel || s.clientType || s.username })} title={t('settings-sessions-revoke-device', { device: s.deviceLabel || s.clientType || s.username })} onClick={() => void revoke(s.handle)}><Icon name="close" /></button>}><span class={css.badge}>{t('settings-sessions-current')}</span></Show></td>
          </tr>}</For></tbody>
        </table>
      </div>

      <Show when={others() > 0}>
        <div class={css.actions}>
          <button type="button" class={css.danger} disabled={busy()} onClick={() => void revokeOthers()} data-testid="revoke-others">
            {t('settings-sessions-revoke-others', { count: others() })}
          </button>
        </div>
      </Show>
    </section>
  );
}

export default Sessions;
