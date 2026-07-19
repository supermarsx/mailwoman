// Active-session listing + revocation (t16 e15, SPEC §19 — S11).
//
// Lists the account's live sessions (metadata only; raw ids never leave the
// server) and revokes them: one by handle, or "everywhere else" (all but the
// current). Rides the `crates/mw-server/src/twofa_routes.rs` session routes.

import { createResource, createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { t, loadCatalog, isolate } from '../../i18n';
import { SettingsService } from './service.ts';
import type { SessionMeta } from './types.ts';
import * as css from './styles.css.ts';

export interface SessionsProps {
  service?: SettingsService;
}

export function Sessions(props: SessionsProps): JSX.Element {
  const service = props.service ?? new SettingsService();
  onMount(() => void loadCatalog('settings'));

  const [sessions, { refetch }] = createResource<SessionMeta[]>(() => service.sessions());
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');

  function fail(e: unknown): void {
    setError(e instanceof Error ? e.message : t('settings-sessions-error'));
  }

  async function revoke(handle: string): Promise<void> {
    setError('');
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
    setError('');
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

  const others = (): number => (sessions() ?? []).filter((s) => !s.current).length;

  return (
    <section class={css.section} aria-label={t('settings-sessions-title')}>
      <h2 class={css.heading}>{t('settings-sessions-title')}</h2>
      <p class={css.prose}>{t('settings-sessions-intro')}</p>

      <ul class={css.list} data-testid="session-list">
        <For each={sessions() ?? []}>
          {(s) => (
            <li class={css.item}>
              <div class={css.itemMain}>
                <span class={css.itemName}>{isolate(s.username)}</span>
                <span class={css.meta}>{t('settings-sessions-last-seen', { when: s.lastSeen })}</span>
              </div>
              <Show
                when={s.current}
                fallback={
                  <button type="button" class={css.danger} disabled={busy()} onClick={() => void revoke(s.handle)}>
                    {t('settings-sessions-revoke')}
                  </button>
                }
              >
                <span class={css.badge}>{t('settings-sessions-current')}</span>
              </Show>
            </li>
          )}
        </For>
      </ul>

      <Show when={others() > 0}>
        <div class={css.actions}>
          <button type="button" class={css.danger} disabled={busy()} onClick={() => void revokeOthers()} data-testid="revoke-others">
            {t('settings-sessions-revoke-others', { count: others() })}
          </button>
        </div>
      </Show>

      <Show when={error() !== ''}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>
    </section>
  );
}

export default Sessions;
