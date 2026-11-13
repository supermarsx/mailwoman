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
import { ControlRow } from './layout/SettingRow.tsx';
import { SettingsCard } from './layout/SettingsCard.tsx';
import * as kit from './layout/layout.css.ts';
import type { CardSpec } from './layout/rows.tsx';
import * as css from './styles.css.ts';

/** Search-index entries (static; the rows render once the configuration loads). */
export const NOTIFICATION_CARDS: readonly CardSpec[] = [
  { id: 'notif-card-general', section: 'notifications', title: 'settings-notif-title', help: 'settings-notif-intro', rows: [{ id: 'notif-enabled', label: 'settings-notif-enabled-label' }, { id: 'notif-view-enabled', label: 'settings-notif-view-enabled' }, { id: 'notif-view-sidebar', label: 'settings-notif-view-sidebar' }] },
  { id: 'notif-card-device', section: 'notifications', title: 'settings-notif-device-title', help: 'settings-notif-device-help', rows: [{ id: 'notif-ask-permission', label: 'settings-notif-ask-permission', help: 'settings-notif-ask-permission-help' }] },
  { id: 'notif-card-quiet', section: 'notifications', title: 'settings-notif-quiet-title', rows: [{ id: 'notif-quiet-enabled', label: 'settings-notif-quiet-enabled-label' }, { id: 'notif-quiet-start', label: 'settings-notif-quiet-start' }, { id: 'notif-quiet-end', label: 'settings-notif-quiet-end' }, { id: 'notif-quiet-timezone', label: 'settings-notif-timezone' }] },
  { id: 'notif-card-background', section: 'notifications', title: 'settings-permissions-background-title', help: 'settings-permissions-background-help', rows: [] },
  { id: 'notif-card-storage', section: 'notifications', title: 'settings-permissions-storage-title', help: 'settings-permissions-storage-help', rows: [] },
  { id: 'notif-card-rules', section: 'notifications', title: 'settings-notif-rules-title', rows: [] },
];

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
  const askPermission = useSetting('notifications.askPermission', true);
  const viewEnabled = useSetting('notifications.view.enabled', true);
  const viewSidebar = useSetting('notifications.view.showInSidebar', true);

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
    try { const result=await Notification.requestPermission();if(owner!==preferenceAccount())return;setPermission(result);if(result==='granted')window.dispatchEvent(new Event('mw:notifications-changed')); }
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
    <div data-testid="notifications-settings">
      <SettingsCard id="notif-card-general" title={t('settings-notif-title')} description={t('settings-notif-intro')}>
        <For each={[
          {id:'notif-view-enabled',label:'settings-notif-view-enabled',handle:viewEnabled},
          {id:'notif-view-sidebar',label:'settings-notif-view-sidebar',handle:viewSidebar},
        ]}>{item=><ControlRow controlId={item.id} label={t(item.label)} help={t('settings-notif-view-help')} note={<><LockNote id={`${item.id}-lock`} locked={item.handle.locked()}/><Show when={item.handle.error()}><p role="alert">{item.handle.error()}</p></Show></>}>{()=>
          <input id={item.id} type="checkbox" checked={item.handle.value()} disabled={!item.handle.entry()||item.handle.locked()||item.handle.saving()||(item.id==='notif-view-sidebar'&&!viewEnabled.value())} onChange={event=>void item.handle.set(event.currentTarget.checked)}/>
        }</ControlRow>}</For>
        <fieldset class={css.bareFieldset} disabled={locked()}>
          <ControlRow controlId="notif-enabled" label={t('settings-notif-enabled-label')} note={<LockNote id="settings-notif-enabled-lock" locked={enabledHold.locked()} />}>{() =>
            <input
              id="notif-enabled"
              type="checkbox"
              aria-describedby={enabledHold.locked() ? 'settings-notif-enabled-lock' : undefined}
              disabled={enabledHold.locked()}
              checked={view().enabled}
              onChange={(e) => patch({ enabled: e.currentTarget.checked })}
            />
          }</ControlRow>
        </fieldset>
      </SettingsCard>

      <SettingsCard id="notif-card-device" title={t('settings-notif-device-title')} description={t('settings-notif-device-help')}>
        <fieldset class={css.bareFieldset} disabled={locked()}>
          <ControlRow controlId="notif-ask-permission" label={t('settings-notif-ask-permission')} help={t('settings-notif-ask-permission-help')}
            note={<>
              <LockNote id="settings-notif-ask-lock" locked={askPermission.locked()}/>
              <Show when={askPermission.error()}><p role="alert">{askPermission.error()}</p></Show>
            </>}>{() =>
            <input id="notif-ask-permission" type="checkbox" checked={askPermission.value()} disabled={!askPermission.entry()||askPermission.locked()||askPermission.saving()} aria-describedby="notif-ask-permission-help settings-notif-ask-lock" onChange={event=>void askPermission.set(event.currentTarget.checked)}/>
          }</ControlRow>
          <p role="status">{t(`settings-notif-permission-${permission()}`)}</p>
          <Show when={permission() === 'default'}>
            <div class={css.actions}>
              <button type="button" class={css.button} disabled={!view().enabled || !hydrated()} onClick={() => void requestNotifications()}>{t('settings-notif-permission-request')}</button>
            </div>
          </Show>
        </fieldset>
      </SettingsCard>

      <BrowserPermissions disabled={locked()} enabled={config().enabled} pushAllowed={config().pushAllowed === true} persistentStorageAllowed={config().persistentStorageAllowed === true} />

      <SettingsCard id="notif-card-quiet" title={t('settings-notif-quiet-title')}>
        <fieldset class={css.bareFieldset} disabled={locked()}>
          <ControlRow controlId="notif-quiet-enabled" label={t('settings-notif-quiet-enabled-label')}>{() =>
            <input
              id="notif-quiet-enabled"
              type="checkbox"
              checked={view().quietHours.enabled}
              onChange={(e) => patchQuiet({ enabled: e.currentTarget.checked })}
            />
          }</ControlRow>
          <Show when={view().quietHours.enabled}>
            <ControlRow controlId="notif-quiet-start" label={t('settings-notif-quiet-start')}>{() =>
              <input id="notif-quiet-start" class={css.input} type="time" value={view().quietHours.start} onInput={(e) => patchQuiet({ start: e.currentTarget.value })} />
            }</ControlRow>
            <ControlRow controlId="notif-quiet-end" label={t('settings-notif-quiet-end')}>{() =>
              <input id="notif-quiet-end" class={css.input} type="time" value={view().quietHours.end} onInput={(e) => patchQuiet({ end: e.currentTarget.value })} />
            }</ControlRow>
            <ControlRow controlId="notif-quiet-timezone" label={t('settings-notif-timezone')}>{() => <>
              <input id="notif-quiet-timezone" class={css.input} value={view().quietHours.timezone ?? 'UTC'} onInput={event => patchQuiet({ timezone: event.currentTarget.value })} />
              <button type="button" class={kit.rowReset} onClick={() => patchQuiet({ timezone: Intl.DateTimeFormat().resolvedOptions().timeZone })}>{t('settings-notif-timezone-device')}</button>
            </>}</ControlRow>
          </Show>
        </fieldset>
      </SettingsCard>

      <SettingsCard id="notif-card-rules" title={t('settings-notif-rules-title')} wide>
        <fieldset class={css.bareFieldset} disabled={locked()}>
          <Show when={view().rules.length > 0}><p class={css.meta}>{t('settings-notif-background-rules')}</p></Show>
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
      </SettingsCard>

      {/* Save applies to every card above. */}
      <fieldset class={`${css.bareFieldset} ${css.actions}`} disabled={locked()}>
        <button type="button" class={css.button} disabled={busy()} onClick={() => void save()} data-testid="notif-save">
          {t('settings-save')}
        </button>
        <Show when={saved()}>
          <span class={css.success} role="status" data-testid="notif-saved">
            {t('settings-saved')}
          </span>
        </Show>
      </fieldset>
      <Show when={error() !== ''}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>
    </div>
  );
}

export default Notifications;
