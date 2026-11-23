import {TimeZoneCombobox} from '../../components/TimeZoneCombobox.tsx';
import {SettingsSubtabs} from './SettingsSubtabs.tsx';
import {NotificationRules} from './NotificationRules.tsx';
import {useEntityNavigation} from './entityEditor.tsx';
import {NotificationControls,StorageRequestControl} from './NotificationControls.tsx';
import {ALERT_CONTROLS,alertSlug} from '../../api/notificationControls.ts';
import {toast} from '../../components/toast/toast.ts';
import {container} from './notificationSettings.css.ts';
import { preferenceAccount } from '../../api/accountPreferenceLifecycle.ts';
import { PersonalAccountBoundary } from './PersonalAccountBoundary.tsx';
// Notification rules + quiet hours use verified-owner personal_notification_rules.
// Rules match a
// substring on sender/mailbox/subject and either surface or mute a notification;
// quiet hours suppress all notifications within a local time window.

import { createEffect, createResource, createSignal, For, Show, onMount, onCleanup, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { SettingsService,SettingsError } from './service.ts';
import { useSetting } from './useSetting.ts';
import { LockNote } from './LockNote.tsx';
import { BrowserPermissions } from './BrowserPermissions.tsx';
import type { NotificationConfig, NotificationRule } from './types.ts';
import { ControlRow } from './layout/SettingRow.tsx';
import { SettingsCard } from './layout/SettingsCard.tsx';
import type { CardSpec } from './layout/rows.tsx';
import * as css from './styles.css.ts';

/** Search-index entries (static; the rows render once the configuration loads). */
export const NOTIFICATION_CARDS: readonly CardSpec[] = [
  ...(['incoming','events','toasts','loginSummary'] as const).map(group=>({id:`notif-card-${group}`,section:'notifications',title:`settings-alert-${group==='incoming'?'channels':group}-title`,rows:ALERT_CONTROLS.filter(row=>row[0].startsWith(`notifications.${group}.`)).map(row=>({id:`alert-${alertSlug(row[0])}`,label:`settings-alert-${alertSlug(row[0])}`}))})),
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

export function Notifications(props: NotificationsProps): JSX.Element {
 return <PersonalAccountBoundary>{()=> <OwnedNotifications {...props}/>}</PersonalAccountBoundary>;
}
function OwnedNotifications(props: NotificationsProps): JSX.Element {
  const owner=preferenceAccount(),navigation=useEntityNavigation('notifications');
  const rulesPage=()=>navigation.route().editor?.id==='notification-rules';
  const service = props.service ?? new SettingsService();
  onMount(() => void loadCatalog('settings'));
  onMount(() => void loadCatalog('settings-policy'));
  // An administrator can hold notifications on or off (`notifications.enabled`,
  // t32 settings model C); the server refuses a held change, so offer none.
  const enabledHold = useSetting('notifications.enabled', false);
  const askPermission = useSetting('notifications.askPermission', true);
  const viewEnabled = useSetting('notifications.view.enabled', true);
  const viewSidebar = useSetting('notifications.view.showInSidebar', true);

  const [loaded,{refetch}] = createResource<NotificationConfig>(() => service.notifications());
  // Working copy; hydrated from the resource once it settles (exactly once).
  const [config, setConfig] = createSignal<NotificationConfig>(DEFAULT_CONFIG);
  const [hydrated, setHydrated] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const setError=(message:string):void=>{if(message)toast.error(message,{id:'notification-settings-error',action:{label:t('common-retry'),run:()=>{if(hydrated())void save();else void refetch();}}});};
  const [saved, setSaved] = createSignal(false);
  const [unsaved,setUnsaved]=createSignal(false);
  let revision=0,saving=false,disposed=false,timer:ReturnType<typeof setTimeout>|undefined;
  onCleanup(()=>{clearTimeout(timer);if(unsaved()&&owner===preferenceAccount())void save();disposed=true;});
  onMount(()=>{const changed=(event:Event)=>{const value=(event as CustomEvent<unknown>).detail;if(value==='granted'||value==='denied'||value==='default')setPermission(value);};window.addEventListener('mw:notification-permission-changed',changed);onCleanup(()=>window.removeEventListener('mw:notification-permission-changed',changed));});
  const [permission, setPermission] = createSignal<NotificationPermission | 'unsupported'>(typeof Notification === 'undefined' ? 'unsupported' : Notification.permission);
  async function requestNotifications(): Promise<void> {
    if (!hydrated() || typeof Notification === 'undefined') return;
    try { const result=await Notification.requestPermission();if(owner!==preferenceAccount())return;setPermission(result);if(result==='granted')window.dispatchEvent(new Event('mw:notifications-changed')); }
    catch { if(owner===preferenceAccount())toast.error(t('settings-notif-permission-error')); }
  }


  createEffect(() => {
    if (loaded.error) { setError(t('settings-notif-error')); return; }
    const fetched = loaded();
    if (fetched !== undefined && !hydrated()) {
      setConfig(fetched);
      setHydrated(true);
    }
  });

  const locked = (): boolean => !hydrated();

  const view = (): NotificationConfig =>
    enabledHold.locked() ? { ...config(), enabled: enabledHold.value() } : config();

  function patch(next: Partial<NotificationConfig>): void {
    setConfig({ ...config(), ...next });
    setSaved(false);setUnsaved(true);revision++;
    clearTimeout(timer);timer=setTimeout(()=>void save(),350);
  }

  function patchQuiet(next: Partial<NotificationConfig['quietHours']>): void {
    patch({ quietHours: { ...config().quietHours, ...next } });
  }

  async function save(): Promise<void> {
    if(owner!==preferenceAccount()||saving||!unsaved()||!hydrated())return;
    clearTimeout(timer);saving=true;const sent=revision,payload=view();
    if(!disposed){setError('');setBusy(true);}
    try {
      await service.saveNotifications(payload);
      if(owner!==preferenceAccount())return;
      window.dispatchEvent(new Event('mw:notifications-changed'));
      if(!disposed&&sent===revision){setUnsaved(false);setSaved(true);}
    } catch(error) {
      if(!disposed&&owner===preferenceAccount())setError(error instanceof SettingsError?error.message:t('settings-notif-error'));
    } finally {
      saving=false;if(!disposed)setBusy(false);
      if(owner===preferenceAccount()&&revision>sent)void save();
    }
  }

  async function saveRules(rules:readonly NotificationRule[]):Promise<void>{
    if(owner!==preferenceAccount()||busy())throw new Error('Account changed');setBusy(true);
    try{await service.saveNotifications({...view(),rules});if(owner!==preferenceAccount())throw new Error('Account changed');setConfig(previous=>({...previous,rules}));window.dispatchEvent(new Event('mw:notifications-changed'));}finally{setBusy(false);}
  }
  return (
    <div class={container} data-testid="notifications-settings">
      <Show when={!rulesPage()} fallback={<NotificationRules rules={view().rules} save={saveRules} back={()=>void navigation.navigate({section:'notifications',card:'notif-card-rules'})}/>}>
      <SettingsSubtabs section="notifications" label="settings-notif-title" {...(navigation.shared?{navigation:navigation.shared}:{})} tabs={[
        {id:'mail',label:'settings-alert-tab-mail',cards:['notif-card-general','notif-card-device','notif-card-incoming','notif-card-folders','notif-card-loginSummary'],content:<>
      <SettingsCard id="notif-card-general" title={t('settings-notif-title')} description={t('settings-notif-intro')}>
        <For each={[
          {id:'notif-view-enabled',label:'settings-notif-view-enabled',handle:viewEnabled},
          {id:'notif-view-sidebar',label:'settings-notif-view-sidebar',handle:viewSidebar},
        ]}>{item=><ControlRow controlId={item.id} label={t(item.label)} help={t('settings-notif-view-help')} note={<><LockNote id={`${item.id}-lock`} locked={item.handle.locked()}/><Show when={item.handle.error()}><p role="alert">{item.handle.error()}</p></Show></>}>{()=>
          <input id={item.id} type="checkbox" checked={item.handle.value()} disabled={!item.handle.entry()||item.handle.locked()||item.handle.saving()||(item.id==='notif-view-sidebar'&&!viewEnabled.value())} onChange={event=>void item.handle.set(event.currentTarget.checked)}/>
        }</ControlRow>}</For>
        <fieldset class={`${css.bareFieldset} ${css.stack}`} disabled={locked()}>
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
        <fieldset class={`${css.bareFieldset} ${css.stack}`} disabled={locked()}>
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
              <button type="button" class={css.button} disabled={!hydrated()} onClick={() => void requestNotifications()}>{t('settings-notif-permission-request')}</button>
            </div>
          </Show>
        </fieldset>
      </SettingsCard>

<NotificationControls group="incoming"/><NotificationControls group="loginSummary"/></>},
        {id:'events',label:'settings-alert-tab-events',cards:['notif-card-events','notif-card-toasts'],content:<><NotificationControls group="events"/><NotificationControls group="toasts"/></>},
        {id:'browser',label:'settings-alert-tab-browser',cards:['notif-card-storage','notif-card-background','notif-card-storage-offer'],content:<>
      <SettingsCard id="notif-card-storage-offer" title={t('settings-permissions-storage-title')}><StorageRequestControl/></SettingsCard>
      <BrowserPermissions disabled={locked()||busy()||unsaved()} enabled={view().enabled} pushAllowed={config().pushAllowed === true} persistentStorageAllowed={config().persistentStorageAllowed === true} />

</>},
        {id:'quiet',label:'settings-alert-tab-quiet',cards:['notif-card-quiet','notif-card-rules'],content:<>
      <SettingsCard id="notif-card-quiet" title={t('settings-notif-quiet-title')}>
        <fieldset class={`${css.bareFieldset} ${css.stack}`} disabled={locked()}>
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
              <TimeZoneCombobox id="notif-quiet-timezone" hideLabel label={t('settings-notif-timezone')} value={view().quietHours.timezone??'UTC'} browserValue={Intl.DateTimeFormat().resolvedOptions().timeZone} onChange={value=>patchQuiet({timezone:value})}/>
            </>}</ControlRow>
          </Show>
        </fieldset>
      </SettingsCard>

      <SettingsCard id="notif-card-rules" title={t('settings-notif-rules-title')} description={t('settings-alert-rule-intro')}><button type="button" class={css.button} disabled={!hydrated()||busy()||unsaved()} onClick={()=>void navigation.navigate({section:'notifications',editor:{kind:'edit',id:'notification-rules'}})}>{t('settings-alert-manage-rules')}</button></SettingsCard>
</>},
      ]}/></Show>
      <Show when={busy()||saved()}><p class={css.meta} role="status" data-testid={busy()?'notif-saving':'notif-saved'}>{t(busy()?'common-saving':'settings-saved')}</p></Show>
    </div>
  );
}

export default Notifications;
