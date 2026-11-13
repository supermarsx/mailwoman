import { createResource, createSignal, For, onMount, Show, type JSX } from 'solid-js';
import { fetchSettings, putSetting, deleteSetting, type SettingEntry } from '../../api/settings.ts';
import { formatDateTime, loadCatalog, t } from '../../i18n';
import { LockNote } from './LockNote.tsx';
import { SettingsService } from './service.ts';
import {
  acknowledgeSessionAlerts,
  fetchSessionAlerts,
  isLocationKind,
  knownAction,
  knownKind,
  noticeDate,
} from './sessionAlertsModel.ts';
import { ControlRow } from './layout/SettingRow.tsx';
import { SettingsCard } from './layout/SettingsCard.tsx';
import * as css from './styles.css.ts';
import * as noticeCss from './sessionNotice.css.ts';
import {SessionNoticeDetails} from './SessionNoticeDetails.tsx';
import {PersonalAccountBoundary} from './PersonalAccountBoundary.tsx';


const COUNTRY = 'security.alerts.countryChange';
const CHANNELS = 'security.alerts.channels';
const SENSITIVITY = 'security.alerts.sensitivity';
const CHANNEL_VALUES = ['app', 'email', 'both'] as const;
const SENSITIVITY_VALUES = ['low', 'normal', 'high'] as const;

export interface SessionAlertsProps {
  service?: SettingsService;
}

/**
 * Recent session security notices and the user's three alert preferences.
 * Preferences use the ordinary user settings API, so an administrator lock shows
 * as a LockNote with the control disabled.
 */
export function SessionAlerts(props: SessionAlertsProps): JSX.Element {
  return <PersonalAccountBoundary>{()=> <OwnedSessionAlerts {...props}/>}</PersonalAccountBoundary>;
}
function OwnedSessionAlerts(props:SessionAlertsProps):JSX.Element {
  const service = props.service ?? new SettingsService();
  onMount(() => void loadCatalog('security'));
  const [alerts, { refetch }] = createResource(() => fetchSessionAlerts().catch(() => null));
  const [prefs, { refetch: refetchPrefs }] = createResource(() => fetchSettings().catch(() => ({}) as Record<string, SettingEntry>));
  const [busy, setBusy] = createSignal(false);
  const [message, setMessage] = createSignal('');
  const [revoked, setRevoked] = createSignal(false);

  const entry = (key: string): SettingEntry | undefined => prefs()?.[key];
  const locked = (key: string): boolean => entry(key)?.locked === true;

  async function write(key: string, value: unknown): Promise<void> {
    setBusy(true);
    setMessage('');
    try {
      await putSetting(key, value);
      await refetchPrefs();
    } catch {
      setMessage(t('session-alerts-pref-error'));
    } finally {
      setBusy(false);
    }
  }
  async function reset(key: string): Promise<void> {
    setBusy(true);
    setMessage('');
    try {
      await deleteSetting(key);
      await refetchPrefs();
    } catch {
      setMessage(t('session-alerts-pref-error'));
    } finally {
      setBusy(false);
    }
  }
  async function dismiss(target: { id: string } | { all: true }): Promise<void> {
    setBusy(true);
    try {
      await acknowledgeSessionAlerts(target);
      await refetch();
    } catch {
      setMessage(t('session-alerts-error'));
    } finally {
      setBusy(false);
    }
  }
  async function notMe(id: string): Promise<void> {
    setBusy(true);
    setMessage('');
    try {
      await service.revokeOtherSessions();
      await acknowledgeSessionAlerts({ id }).catch(() => undefined);
      setRevoked(true);
      await refetch();
    } catch {
      setMessage(t('session-alerts-error'));
    } finally {
      setBusy(false);
    }
  }

  const notices = () => alerts()?.notices ?? [];
  const unread = () => notices().filter((n) => !n.acknowledged);
  const countries = (n: { fromCountry?: string | null; toCountry?: string | null }): string =>
    n.fromCountry || n.toCountry
      ? t('session-alerts-countries', { from: n.fromCountry || t('session-alerts-unknown-country'), to: n.toCountry || t('session-alerts-unknown-country') })
      : '';

  return (
    <>
    <SettingsCard id="session-alerts-card" title={t('session-alerts-title')} description={t('session-alerts-help')} testId="session-alerts">
      <Show when={revoked()}>
        <p class={css.prose} role="status" data-testid="session-alerts-revoked">
          {t('session-alerts-revoked')}
        </p>
      </Show>
      <Show when={notices().length > 0} fallback={<p class={css.meta}>{t('session-alerts-none')}</p>}>
        <ul class={css.list}>
          <For each={notices()}>
            {(notice) => {
              const kind = knownKind(notice.kind);
              return (
                <li class={noticeCss.notice} data-testid="session-alert-notice" data-unread={notice.acknowledged ? 'false' : 'true'}>
                  <div>
                    <p class={css.prose}>
                      <time>{formatDateTime(noticeDate(notice.ts), { dateStyle: 'medium', timeStyle: 'short' })}</time>
                      {' · '}
                      {t(`session-alerts-kind-${kind}`, { from: notice.fromCountry || t('session-alerts-unknown-country'), to: notice.toCountry || t('session-alerts-unknown-country') })}
                    </p>
                    <p class={css.meta}>
                      {t(`session-alerts-action-${knownAction(notice.action)}`)}
                      {countries(notice) ? ` · ${countries(notice)}` : ''}
                      {notice.session ? ` · ${t('session-alerts-session', { id: notice.session })}` : ''}
                    </p>
                    <Show when={isLocationKind(kind)}>
                      <p class={css.meta} role="note">
                        {t('session-alerts-caveat')}
                      </p>
                    </Show>
                  </div>
                  <SessionNoticeDetails context={notice.context}/>
                  <div class={css.actions}>
                    <button type="button" disabled={busy()} onClick={() => void notMe(notice.id)}>
                      {t('session-alerts-not-me')}
                    </button>
                    <Show when={!notice.acknowledged}>
                      <button type="button" disabled={busy()} onClick={() => void dismiss({ id: notice.id })}>
                        {t('session-alerts-dismiss')}
                      </button>
                    </Show>
                  </div>
                </li>
              );
            }}
          </For>
        </ul>
        <Show when={unread().length > 1}>
          <div class={css.actions}>
            <button type="button" disabled={busy()} onClick={() => void dismiss({ all: true })}>
              {t('session-alerts-dismiss-all')}
            </button>
          </div>
        </Show>
      </Show>

    </SettingsCard>

    <SettingsCard id="session-alerts-prefs-card" title={t('session-alerts-prefs')}>
      <Show when={prefs()}>
        <ControlRow controlId="session-alerts-country" label={t('session-alerts-country')} help={t('session-alerts-country-help')} note={<LockNote locked={locked(COUNTRY)} id="session-alerts-country-note" />}>{(describedBy) =>
          <input
            id="session-alerts-country"
            type="checkbox"
            disabled={busy() || locked(COUNTRY)}
            checked={entry(COUNTRY)?.value !== false}
            aria-describedby={`${describedBy()} session-alerts-country-note`}
            onChange={(e) => void write(COUNTRY, e.currentTarget.checked)}
          />
        }</ControlRow>
        <p class={css.meta}>{t('session-alerts-caveat')}</p>
        <ControlRow controlId="session-alerts-channels" label={t('session-alerts-channels')} note={<LockNote locked={locked(CHANNELS)} />}>{() =>
          <select id="session-alerts-channels" class={css.input} disabled={busy() || locked(CHANNELS)} value={String(entry(CHANNELS)?.value ?? 'both')} onChange={(e) => void write(CHANNELS, e.currentTarget.value)}>
            <For each={CHANNEL_VALUES}>{(v) => <option value={v}>{t(`session-alerts-channel-${v}`)}</option>}</For>
          </select>
        }</ControlRow>
        <ControlRow controlId="session-alerts-sensitivity" label={t('session-alerts-sensitivity')} note={<LockNote locked={locked(SENSITIVITY)} />}>{() =>
          <select id="session-alerts-sensitivity" class={css.input} disabled={busy() || locked(SENSITIVITY)} value={String(entry(SENSITIVITY)?.value ?? 'normal')} onChange={(e) => void write(SENSITIVITY, e.currentTarget.value)}>
            <For each={SENSITIVITY_VALUES}>{(v) => <option value={v}>{t(`session-alerts-sensitivity-${v}`)}</option>}</For>
          </select>
        }</ControlRow>
        <div class={css.actions}>
          <button
            type="button"
            disabled={busy() || [COUNTRY, CHANNELS, SENSITIVITY].every((k) => locked(k) || entry(k)?.userValue === undefined)}
            onClick={() => void Promise.all([COUNTRY, CHANNELS, SENSITIVITY].filter((k) => !locked(k) && entry(k)?.userValue !== undefined).map(reset))}
          >
            {t('session-alerts-reset')}
          </button>
        </div>
      </Show>
      <Show when={message()}>
        <p role="alert">{message()}</p>
      </Show>
    </SettingsCard>
    </>
  );
}
export default SessionAlerts;
