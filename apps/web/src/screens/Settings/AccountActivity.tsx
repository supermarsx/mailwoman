import { createResource, For, onMount, Show, type JSX } from 'solid-js';
import { apiFetch } from '../../api/http.ts';
import { basePath } from '../../api/basePath.ts';
import { formatDateTime, loadCatalog, t } from '../../i18n';
import { messageTimeZone } from '../../components/TimeHelpers.tsx';
import { Icon, type IconName } from '../../components/Icon.tsx';
import { InlineLoading } from '../../components/InlineLoading.tsx';
import { AsyncError } from '../../components/AsyncState.tsx';
import { SettingsCard } from './layout/SettingsCard.tsx';
import * as css from './accountActivity.css.ts';

export interface AccountActivityEvent { id: string; timestamp: string; action: string; actor?: string; target?: string | null; detail?: unknown }
export const ACTIVITY_ACTIONS = ['linked-credential-stored','linked-credential-deleted','linked-credential-refresh-failed','linked-credential-renewed','linked-access-renew-failed','linked-access-renewed','domain-created','domain-updated','domain-deleted','user-provisioned','quota-changed','sessions-revoked','feature-flag-changed','zero-access-toggled','force-password-change','remote-cache-wipe','security-policy-changed','integration-changed','api-key-revoked','observability-changed','appearance-changed','cache-scope-changed','login-succeeded','login-failed','ip-banned','ip-unbanned','config-reloaded','plugin-load-admitted','plugin-load-refused','plugin-allowlist-approved','plugin-allowlist-revoked','plugin-uninstalled'] as const;
const known = new Set<string>(ACTIVITY_ACTIONS);
function eventIcon(action: string): IconName {
  if (action.includes('failed') || action.includes('refused')) return 'shield';
  if (action.includes('credential') || action.includes('password') || action.includes('key')) return 'key';
  if (action.includes('login') || action.includes('session')) return 'sessions';
  return 'audit';
}
async function fetchActivity(): Promise<AccountActivityEvent[]> {
  const response = await apiFetch(`${basePath()}/api/account/security/activity`, { credentials: 'same-origin' });
  if (!response.ok) throw new Error('activity');
  return (await response.json() as { events: AccountActivityEvent[] }).events;
}
export function AccountActivity(): JSX.Element {
  onMount(() => void loadCatalog('account-activity'));
  // Keep failures local to this card; a failed activity request must not replace Settings.
  const [activity, { refetch }] = createResource(async () => { try { return { events: await fetchActivity(), failed: false }; } catch { return { events: [] as AccountActivityEvent[], failed: true }; } });
  return <SettingsCard id="security-activity-card" title={t('security-center-activity')} description={t('account-activity-help')}>
    <div class={css.root} data-testid="account-activity" aria-busy={activity.loading}>
      <Show when={!activity.loading} fallback={<InlineLoading message={t('account-activity-loading')} onRetry={() => void refetch()} />}>
        <Show when={!activity()?.failed} fallback={<AsyncError message={t('account-activity-error')} onRetry={() => void refetch()} />}>
          <Show when={activity()?.events.length} fallback={<p class={css.empty}><Icon name="audit" />{t('account-activity-empty')}</p>}>
            <ol class={css.list} aria-label={t('security-center-activity')}><For each={activity()?.events}>{event => <li class={css.row}>
              <span class={css.icon}><Icon name={eventIcon(event.action)} /></span>
              <div class={css.body}>
                <div class={css.heading}>{known.has(event.action) ? t(`account-activity-${event.action}`) : t('account-activity-other')}</div>
                <time class={css.time} dateTime={event.timestamp}>{formatDateTime(event.timestamp, { dateStyle: 'medium', timeStyle: 'medium', timeZone: messageTimeZone() }) || t('account-activity-time-unknown')}</time>
                <details class={css.details}><summary>{t('account-activity-details')}</summary>
                  <dl class={css.facts}>
                    <dt>{t('account-activity-event')}</dt><dd>{event.action}</dd>
                    <dt>{t('account-activity-id')}</dt><dd>{event.id}</dd>
                    <dt>{t('account-activity-timestamp')}</dt><dd>{event.timestamp}</dd>
                    <Show when={event.actor}><dt>{t('account-activity-actor')}</dt><dd>{event.actor === 'system' ? t('account-activity-system') : event.actor}</dd></Show>
                    <Show when={event.target}><dt>{t('account-activity-target')}</dt><dd>{event.target}</dd></Show>
                  </dl>
                  <Show when={event.detail != null}><pre class={css.payload}>{JSON.stringify(event.detail, null, 2)}</pre></Show>
                </details>
              </div>
            </li>}</For></ol>
          </Show>
        </Show>
      </Show>
    </div>
  </SettingsCard>;
}
