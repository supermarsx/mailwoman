import { preferenceAccount } from '../../api/accountPreferenceLifecycle.ts';
import { PersonalAccountBoundary } from './PersonalAccountBoundary.tsx';
// Notification rules + quiet hours use verified-owner personal_notification_rules.
// Rules match a
// substring on sender/mailbox/subject and either surface or mute a notification;
// quiet hours suppress all notifications within a local time window.

import { createEffect, createResource, createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { SettingsService } from './service.ts';
import { useSetting } from './useSetting.ts';
import { LockNote } from './LockNote.tsx';
import { BrowserPermissions } from './BrowserPermissions.tsx';
import type { NotificationConfig, NotificationRule } from './types.ts';
import * as css from './styles.css.ts';

export interface NotificationsProps {
  service?: SettingsService;
}

const DEFAULT_CONFIG: NotificationConfig = {
  enabled: false,
  rules: [],
  quietHours: { enabled: false, start: '22:00', end: '07:00' },
};

function newRuleId(): string {
  return `r-${Date.now().toString(36)}-${Math.floor(Math.random() * 1e6).toString(36)}`;
}

export function Notifications(props: NotificationsProps): JSX.Element {
 return <PersonalAccountBoundary>{()=> <OwnedNotifications {...props}/>}</PersonalAccountBoundary>;
}
function OwnedNotifications(props: NotificationsProps): JSX.Element {
  const owner=preferenceAccount();
  const service = props.service ?? new SettingsService();
  onMount(() => void loadCatalog('settings'));
  onMount(() => void loadCatalog('settings-policy'));
  // An administrator can hold notifications on or off (`notifications.enabled`,
  // t32 settings model C); the server refuses a held change, so offer none.
  const enabledHold = useSetting('notifications.enabled', false);

  const [loaded] = createResource<NotificationConfig>(() => service.notifications());
  // Working copy; hydrated from the resource once it settles (exactly once).
  const [config, setConfig] = createSignal<NotificationConfig>(DEFAULT_CONFIG);
  const [hydrated, setHydrated] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');
  const [saved, setSaved] = createSignal(false);
  const [permission, setPermission] = createSignal<NotificationPermission | 'unsupported'>(typeof Notification === 'undefined' ? 'unsupported' : Notification.permission);
  async function requestNotifications(): Promise<void> {
    if (!hydrated() || !view().enabled || typeof Notification === 'undefined') return;
    try { setPermission(await Notification.requestPermission()); }
    catch { setError(t('settings-notif-permission-error')); }
  }


  createEffect(() => {
    if (loaded.error) { setError(t('settings-notif-error')); return; }
    const fetched = loaded();
    if (fetched !== undefined && !hydrated()) {
      setConfig(fetched);
      setHydrated(true);
    }
  });

  const locked = (): boolean => !hydrated() || busy();

  const view = (): NotificationConfig =>
    enabledHold.locked() ? { ...config(), enabled: enabledHold.value() } : config();

  function patch(next: Partial<NotificationConfig>): void {
    setConfig({ ...config(), ...next });
    setSaved(false);
  }

  function patchQuiet(next: Partial<NotificationConfig['quietHours']>): void {
    patch({ quietHours: { ...config().quietHours, ...next } });
  }

  function addRule(): void {
    const rule: NotificationRule = { id: newRuleId(), label: '', match: '', action: 'notify' };
    patch({ rules: [...config().rules, rule] });
  }

  function updateRule(id: string, next: Partial<NotificationRule>): void {
    patch({ rules: config().rules.map((r) => (r.id === id ? { ...r, ...next } : r)) });
  }

  function removeRule(id: string): void {
    patch({ rules: config().rules.filter((r) => r.id !== id) });
  }

  async function save(): Promise<void> {
    if(owner!==preferenceAccount())return;
    setError('');
    setBusy(true);
    try {
      await service.saveNotifications(view());
      if(owner!==preferenceAccount())return;
      window.dispatchEvent(new Event('mw:notifications-changed'));
      setSaved(true);
    } catch (e) {
      setError(e instanceof Error ? e.message : t('settings-notif-error'));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section class={css.section} aria-label={t('settings-notif-title')}>
      <h2 class={css.heading}>{t('settings-notif-title')}</h2>
      <p class={css.prose}>{t('settings-notif-intro')}</p>

      {/* Each subgroup is its own fieldset so the card rules them off; every one
          stays disabled until the config loads and while a save runs. */}
      <fieldset class={css.group} disabled={locked()}>
        <label class={css.check}>
          <input
            class={css.checkbox}
            type="checkbox"
            aria-label={t('settings-notif-enabled-label')}
            aria-describedby={enabledHold.locked() ? 'settings-notif-enabled-lock' : undefined}
            disabled={enabledHold.locked()}
            checked={view().enabled}
            onChange={(e) => patch({ enabled: e.currentTarget.checked })}
          />
          <span>{t('settings-notif-enabled-label')}</span>
        </label>
        <LockNote id="settings-notif-enabled-lock" locked={enabledHold.locked()} />
      </fieldset>

      <fieldset class={css.group} disabled={locked()}>
        <h3 class={css.subheading}>{t('settings-notif-device-title')}</h3>
        <p class={css.prose}>{t('settings-notif-device-help')}</p>
        <p role="status">{t(`settings-notif-permission-${permission()}`)}</p>
        <Show when={permission() === 'default'}>
          <div class={css.actions}>
            <button type="button" class={css.button} disabled={!view().enabled || !hydrated()} onClick={() => void requestNotifications()}>{t('settings-notif-permission-request')}</button>
          </div>
        </Show>
      </fieldset>

      <BrowserPermissions disabled={locked()} enabled={config().enabled} pushAllowed={config().pushAllowed === true} persistentStorageAllowed={config().persistentStorageAllowed === true} />

      {/* Quiet hours */}
      <fieldset class={css.group} disabled={locked()}>
        <h3 class={css.subheading}>{t('settings-notif-quiet-title')}</h3>
        <label class={css.check}>
          <input
            class={css.checkbox}
            type="checkbox"
            aria-label={t('settings-notif-quiet-enabled-label')}
            checked={view().quietHours.enabled}
            onChange={(e) => patchQuiet({ enabled: e.currentTarget.checked })}
          />
          <span>{t('settings-notif-quiet-enabled-label')}</span>
        </label>
        <Show when={view().quietHours.enabled}>
          <div class={css.row}>
            <label class={css.field}>
              <span class={css.label}>{t('settings-notif-quiet-start')}</span>
              <input
                class={css.input}
                type="time"
                aria-label={t('settings-notif-quiet-start')}
                value={view().quietHours.start}
                onInput={(e) => patchQuiet({ start: e.currentTarget.value })}
              />
            </label>
            <label class={css.field}>
              <span class={css.label}>{t('settings-notif-quiet-end')}</span>
              <input
                class={css.input}
                type="time"
                aria-label={t('settings-notif-quiet-end')}
                value={view().quietHours.end}
                onInput={(e) => patchQuiet({ end: e.currentTarget.value })}
              />
            </label>
            <label class={css.field}>
              <span class={css.label}>{t('settings-notif-timezone')}</span>
              <input class={css.input} value={view().quietHours.timezone ?? 'UTC'} onInput={event => patchQuiet({ timezone: event.currentTarget.value })} />
            </label>
            <button type="button" class={css.ghost} onClick={() => patchQuiet({ timezone: Intl.DateTimeFormat().resolvedOptions().timeZone })}>{t('settings-notif-timezone-device')}</button>
          </div>
        </Show>
      </fieldset>

      {/* Rules */}
      <fieldset class={css.group} disabled={locked()}>
        <h3 class={css.subheading}>{t('settings-notif-rules-title')}</h3>
        <Show when={view().rules.length > 0}><p class={css.prose}>{t('settings-notif-background-rules')}</p></Show>
        <ul class={css.list} data-testid="notif-rule-list">
          <For each={view().rules}>
            {(rule) => (
              <li class={css.item}>
                <input
                  class={`${css.input} ${css.grow}`}
                  placeholder={t('settings-notif-rule-match-placeholder')}
                  aria-label={t('settings-notif-rule-match-placeholder')}
                  value={rule.match}
                  onInput={(e) => updateRule(rule.id, { match: e.currentTarget.value })}
                />
                <select
                  class={css.select}
                  aria-label={t('settings-notif-rule-action-label')}
                  value={rule.action}
                  onChange={(e) => updateRule(rule.id, { action: e.currentTarget.value as NotificationRule['action'] })}
                >
                  <option value="notify">{t('settings-notif-rule-notify')}</option>
                  <option value="mute">{t('settings-notif-rule-mute')}</option>
                </select>
                <button type="button" class={css.danger} onClick={() => removeRule(rule.id)}>
                  {t('settings-delete')}
                </button>
              </li>
            )}
          </For>
        </ul>
        <div class={css.actions}>
          <button type="button" class={css.ghost} onClick={addRule} data-testid="notif-add-rule">
            {t('settings-notif-rule-add')}
          </button>
        </div>
      </fieldset>

      {/* Save applies to every group above, so it sits in its own ruled-off row. */}
      <fieldset class={css.group} disabled={locked()}>
        <div class={css.actions}>
          <button type="button" class={css.button} disabled={busy()} onClick={() => void save()} data-testid="notif-save">
            {t('settings-save')}
          </button>
          <Show when={saved()}>
            <span class={css.success} role="status" data-testid="notif-saved">
              {t('settings-saved')}
            </span>
          </Show>
        </div>
      </fieldset>
      <Show when={error() !== ''}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>
    </section>
  );
}

export default Notifications;
